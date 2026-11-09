#!/usr/bin/env python3
"""Run one isolated synthetic native showcase and validate its actual readback evidence."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import sys
import time
import zlib


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def png_size(path: Path) -> tuple[int, int]:
    if not 0 < path.stat().st_size <= 32 * 1024 * 1024:
        raise ValueError("Missing or oversized native PNG")
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("Native readback is not PNG")
    offset, width, height = 8, 0, 0
    compressed = bytearray()
    ended = False
    while offset + 12 <= len(data):
        size = struct.unpack_from(">I", data, offset)[0]
        kind = data[offset + 4 : offset + 8]
        end = offset + size + 12
        if end > len(data):
            raise ValueError("Truncated PNG chunk")
        payload = data[offset + 8 : end - 4]
        crc = struct.unpack_from(">I", data, end - 4)[0]
        if zlib.crc32(kind + payload) != crc:
            raise ValueError("Invalid PNG checksum")
        if kind == b"IHDR":
            width, height, depth, color, compression, filtering, interlace = struct.unpack(">IIBBBBB", payload)
            if not (0 < width <= 4096 and 0 < height <= 4096):
                raise ValueError("Native PNG dimensions exceed the capture bound")
            if (depth, color, compression, filtering, interlace) != (8, 6, 0, 0, 0):
                raise ValueError("Expected native noninterlaced RGBA8 readback")
        elif kind == b"IDAT":
            compressed.extend(payload)
        elif kind == b"IEND":
            ended = size == 0 and end == len(data)
            break
        offset = end
    if not ended or not width or not compressed:
        raise ValueError("Incomplete native PNG")
    expected = (width * 4 + 1) * height
    decoder = zlib.decompressobj()
    pixels = decoder.decompress(compressed, expected + 1)
    if not decoder.eof or decoder.unused_data or len(pixels) != expected:
        raise ValueError("Invalid native PNG pixel data")
    if any(pixels[row * (width * 4 + 1)] > 4 for row in range(height)):
        raise ValueError("Invalid native PNG row filter")
    return width, height


def validate(directory: Path, source_sha: str) -> dict:
    sidecar = directory / "evidence.json"
    if not 0 < sidecar.stat().st_size <= 4 * 1024 * 1024:
        raise ValueError("Missing or oversized native evidence sidecar")
    evidence = json.loads(sidecar.read_text())
    if evidence.get("schema_version") != 1 or evidence.get("scenario") != "connections":
        raise ValueError("Unexpected native capture schema/scenario")
    if evidence.get("source_sha") != source_sha or evidence.get("compiled_source_sha") != source_sha:
        raise ValueError("Native capture does not match the compiled source identity")
    if evidence.get("status") != "passed" or evidence.get("failure") is not None:
        raise ValueError("Native journey failed: " + str(evidence.get("failure")))
    for flag in ("os_input_acceptance", "physical_touchscreen_acceptance", "production_acceptance"):
        if evidence.get(flag) is not False:
            raise ValueError("Scripted evidence cannot claim " + flag)
    if evidence.get("completed_steps") != 29 or evidence.get("total_steps") != 29:
        raise ValueError("Native journey did not complete its declared steps")
    final = evidence["final"]
    if final.get("canonical_json") != evidence.get("initial_canonical_json"):
        raise ValueError("Undo/no-op/cancellation did not restore exact canonical bytes")
    if final.get("pending") or final.get("history", {}).get("undo") != 0:
        raise ValueError("Native journey retained an unexpected pending edit/history entry")
    expected_counts = {"begin": 4, "submitted": 2, "no_op": 1, "cancelled": 1, "escape": 1,
                       "invalid": 0, "empty": 0, "view_transaction_intents": 2,
                       "host_submissions": 3, "host_acknowledgements": 2}
    if any(final.get("counters", {}).get(key) != value for key, value in expected_counts.items()):
        raise ValueError("Native lifecycle/acknowledgement counts did not match")
    captures = evidence.get("captures", [])
    if [capture.get("name") for capture in captures] != ["baseline", "rejected", "connected", "reconnected", "complete"]:
        raise ValueError("Actual readback coverage is incomplete")
    zoom_checks = [check["observed_state"] for check in evidence.get("checks", [])
                   if check.get("check") == "Zoomed"]
    if len(zoom_checks) != 1:
        raise ValueError("The injected native zoom observation is missing")
    zoomed = zoom_checks[0]
    baseline = captures[0]["observed_state"]
    before_zoom = baseline.get("geometry", {}).get("camera", {}).get("zoom")
    after_zoom = zoomed.get("geometry", {}).get("camera", {}).get("zoom")
    if (before_zoom != 1.0 or after_zoom != 1.125
            or zoomed.get("canonical_json") != evidence["initial_canonical_json"]
            or zoomed.get("revision") != 0 or zoomed.get("history") != {"undo": 0, "redo": 0}
            or zoomed.get("pending") is not False or zoomed.get("has_gesture") is not False
            or zoomed.get("counters", {}).get("view_transaction_intents") != 0):
        raise ValueError("Injected zoom did not change only the camera before graph authoring")
    for target in ("port:a:source:main", "port:b:target:input"):
        before = baseline.get("targets", {}).get(target)
        after = zoomed.get("targets", {}).get(target)
        if not isinstance(before, list) or not isinstance(after, list) or before == after:
            raise ValueError("Painted port positions did not update after injected zoom")
    files = []
    legacy_raw = captures[0]["observed_state"].get("legacy_raw")
    if not isinstance(legacy_raw, str) or '1.23000' not in legacy_raw:
        raise ValueError("The legacy preservation fixture was not observed")
    for capture in captures:
        filename = capture["name"] + ".png"
        if capture.get("file") != filename:
            raise ValueError("Unexpected capture filename")
        path = directory / filename
        width, height = png_size(path)
        if [width, height] != [capture.get("width"), capture.get("height")]:
            raise ValueError("PNG dimensions differ from the observed readback")
        if not 0 < capture.get("requested_frame", 0) <= capture.get("readback_frame", 0):
            raise ValueError("Native readback frame identity is missing")
        state = capture["observed_state"]
        if capture["name"] != "baseline" and state.get("geometry", {}).get("camera", {}).get("zoom") != after_zoom:
            raise ValueError("The connection readback did not retain the observed zoom")
        if state.get("legacy_raw") != legacy_raw or state.get("node_count") != 3:
            raise ValueError("An unrelated synthetic record changed during the native journey")
        name = capture["name"]
        if name in ("baseline", "rejected", "complete"):
            if state.get("canonical_json") != evidence["initial_canonical_json"] or state.get("history", {}).get("undo") != 0:
                raise ValueError("Canonical data changed in a retained/cancelled state")
            if state.get("pending") is not (name == "rejected"):
                raise ValueError("The native pending state does not match the captured stage")
        else:
            edge = state.get("new_edge") or {}
            expected_port = "input" if name == "connected" else "alternative"
            expected_undo = 1 if name == "connected" else 2
            if (edge.get("id") != "showcase-edge-1" or edge.get("from") != {"node": "a", "port": "main", "kind": "source"}
                    or edge.get("to") != {"node": "b", "port": expected_port, "kind": "target"}
                    or state.get("edge_count") != 2 or state.get("history", {}).get("undo") != expected_undo
                    or state.get("pending") is not False):
                raise ValueError("The captured named connection or reconnection did not match")
        files.append({"file": filename, "sha256": sha256(path), "width": width, "height": height})
    return {"sidecar_sha256": sha256(sidecar), "captures": files}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--executable", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path, help="New evidence directory; never replaced")
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--timeout", type=float, default=65.0)
    args = parser.parse_args()
    if not re.fullmatch(r"[0-9a-f]{40}", args.source_sha) or not 1 <= args.timeout <= 90:
        parser.error("A full source SHA and a timeout between 1 and 90 seconds are required")
    executable = args.executable.resolve(strict=True)
    if not executable.is_file():
        parser.error("--executable must identify the exact hosted binary")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = {"schema_version": 1, "source_sha": args.source_sha,
              "executable_sha256": sha256(executable), "status": "running",
              "os_input_acceptance": False, "physical_touchscreen_acceptance": False,
              "production_acceptance": False}
    started = time.monotonic()
    try:
        command = [str(executable), "--capture-dir", str(output / "capture"),
                   "--source-sha", args.source_sha, "--scenario", "connections"]
        with (output / "stdout.log").open("xb") as stdout, (output / "stderr.log").open("xb") as stderr:
            child = subprocess.Popen(command, cwd=output, env=os.environ.copy(), stdout=stdout, stderr=stderr)
            report["owned_child_pid"] = child.pid
            try:
                report["returncode"] = child.wait(timeout=args.timeout)
            except subprocess.TimeoutExpired:
                report["timed_out"] = True
                child.terminate()
                try:
                    child.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait(timeout=3)
                raise RuntimeError("The owned showcase child exceeded its capture deadline") from None
        if report["returncode"] != 0:
            raise RuntimeError("The showcase executable failed; inspect its retained stderr")
        report.update(validate(output / "capture", args.source_sha))
        report["status"] = "passed"
    except (OSError, ValueError, KeyError, TypeError, AttributeError, RuntimeError, struct.error, zlib.error) as error:
        report["status"] = "failed"
        report["error"] = str(error)
    finally:
        report["elapsed_seconds"] = round(time.monotonic() - started, 3)
        with (output / "run.json").open("x") as stream:
            json.dump(report, stream, indent=2)
            stream.write("\n")
    print(json.dumps({key: report[key] for key in ("status", "source_sha", "elapsed_seconds")}))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    sys.exit(main())
