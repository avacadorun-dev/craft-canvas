#!/usr/bin/env python3
"""Package only the freshly built synthetic engineering showcase on macOS ARM64."""
import argparse
import hashlib
import json
import pathlib
import platform
import plistlib
import shutil
import subprocess


def command(*args):
    return subprocess.check_output(args, text=True).strip()


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", required=True)
    parser.add_argument("--output", required=True, type=pathlib.Path)
    args = parser.parse_args()
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        raise SystemExit("The native showcase artifact requires a macOS ARM64 runner.")
    source = command("git", "rev-parse", "HEAD")
    if args.source != source:
        raise SystemExit("Requested source does not match the checkout.")
    binary = pathlib.Path("target/debug/craft-canvas-showcase")
    if command("lipo", "-archs", str(binary)) != "arm64":
        raise SystemExit("Showcase executable is not exactly ARM64.")
    args.output.mkdir(parents=True, exist_ok=False)
    title = f"CraftCanvas Showcase {source[:7]}"
    bundle = args.output / f"{title}.app"
    executable = bundle / "Contents/MacOS/craft-canvas-showcase"
    executable.parent.mkdir(parents=True)
    shutil.copy2(binary, executable)
    resources = bundle / "Contents/Resources"
    resources.mkdir()
    shutil.copy2("LICENSE", resources / "LICENSE")
    shutil.copytree("LICENSES", resources / "LICENSES")
    with (bundle / "Contents/Info.plist").open("xb") as stream:
        plistlib.dump({
            "CFBundleName": title,
            "CFBundleDisplayName": title,
            "CFBundleIdentifier": "dev.craftcanvas.showcase",
            "CFBundleExecutable": executable.name,
            "CFBundlePackageType": "APPL",
            "CFBundleShortVersionString": "0.1.0",
            "CFBundleVersion": "1",
            "NSHighResolutionCapable": True,
            "LSMinimumSystemVersion": "14.0",
        }, stream)
    subprocess.run(["codesign", "--force", "--deep", "--sign", "-", str(bundle)], check=True)
    subprocess.run(["codesign", "--verify", "--strict", "--deep", str(bundle)], check=True)
    archive = args.output / f"craft-canvas-showcase-{source[:7]}-macos-arm64.zip"
    subprocess.run(["ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", str(bundle), str(archive)], check=True)
    metadata = {
        "source": source,
        "compiler": command("rustc", "-vV"),
        "host": platform.platform(),
        "architecture": "arm64",
        "profile": "dev; debug information disabled; not capacity qualification",
        "lock_sha256": digest(pathlib.Path("Cargo.lock")),
        "executable_sha256": digest(executable),
        "archive_sha256": digest(archive),
        "bundle": bundle.name,
        "archive": archive.name,
        "signing": "ad-hoc verified; engineering artifact, not a notarized release",
    }
    with (args.output / "manifest.json").open("x") as stream:
        json.dump(metadata, stream, indent=2)
        stream.write("\n")
    # The zip is the artifact; avoid uploading the same executable a second time.
    shutil.rmtree(bundle)
    print(json.dumps(metadata, indent=2))


if __name__ == "__main__":
    main()
