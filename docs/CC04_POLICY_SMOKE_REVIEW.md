# CC-04 exact-candidate native compatibility smoke review

Independent reviewer: `/root/cc04_review`. Reviewed 2026-11-10. **Accepted only as the existing scripted connection/zoom/acknowledgement/Undo compatibility smoke. No blocking defect found within that scope.**

Exact source: `4ff564123a82483e6faf61ad12e0b77533b8c561`; Git tree: `33eed42aa7eb1b211d70e6a0e490c2f209b2596a`. Independently recomputed the Git commit object hash and matched the checkout, artifact manifest, run report, evidence source and compiled-source identity. The reviewed Rust source and locked dependencies matched that commit. Current Cargo.lock and validator source match that commit.

Evidence: qualification run `42749740099`, retained log the retained qualification log; the isolated downloaded bundle and capture directory. The log contains 164 passed and zero failed tests on each of macOS, Windows and Linux, formatting, strict Clippy, headless example/native build and unchanged-lock stages, with zero CI error markers. This review did not rerun Cargo.

Artifact verification: archive CRC validation passed; all five archived files match their unpacked counterparts byte-for-byte. The executable is Mach-O ARM64. `codesign --verify --deep --strict` reports valid on disk and satisfaction of the designated requirement. This is an ad-hoc engineering bundle, not notarization or production certification.

Independently reran only the read-only `validate()` function from the exact committed `scripts/capture-showcase.py`, without invoking its launcher. Validation passed: 29/29 declared steps, five valid RGBA PNGs at 2400 × 1790, matching readback frames and hashes, exact final canonical bytes, preserved legacy raw numeric lexeme, no final pending request, zero final Undo entries and two Redo entries. Final counters: four Begin, two Submitted, one NoOp, one Escape Cancel, two view transaction intents, three host submissions and two acknowledgements. The recorded process returned zero in 7.427 seconds; stderr is empty. No captured stage reports an engine error.

All five original PNGs were visually inspected. Baseline shows the three synthetic bodies and legacy route. Rejected shows the retained light-blue preview and explicit paused/recovery explanation. Connected shows the accepted main-to-input route. Reconnected shows the selected route reaching the alternative target. Complete shows only the restored legacy connection, enabled Redo and no pending notice. The graph, native controls and ports render consistently in light appearance. After zoom, the lower controls card is clipped at the viewport boundary; this is not evidence of a missing canonical record or full visibility/scroll acceptance.

Limits: input provenance is scripted RawInput in an actual native WGPU window. This does not accept new selectability/visibility/deletion flags natively, full CC-04, natural OS input, physical touchscreen behavior, Windows/Linux GUI behavior, host durability, WriterCraft integration or production readiness. All corresponding acceptance flags remain false. This review launched no application, performed no GUI interaction, changed no source and touched neither the user-controlled d2bdaec showcase nor WriterCraft preview. Its only write is this review report.

## Recomputed SHA-256 evidence

| Item | SHA-256 |
|---|---|
| Artifact manifest | `3601e7c898c3c69002e883ea571910bc06ac1c3defd043b7f71955efa61e5722` |
| Downloaded ZIP | `c802ff796e675e6d47200d511b4a508fd80a5586b18522f42348d8d11344daf3` |
| Executable | `096acf8b53e235ab1907ad35775c78d88bf3397ee9462d8b1af8ee1ca265ca2b` |
| Cargo.lock | `5a833ea74faaa3b70ecad110472537687a850ff0bb4fd72ac481f0d7447f44d0` |
| Validator source | `cacba8387dfca7c3a6c1cb12146cda75a64e6275575c3fe0383a1387afe22885` |
| Qualification log | `f5623cd32c3c80f787230be5f5f53d0dda1883a27abfc25bba8a798f356cdcac` |
| run.json | `a2cf8283f26a8afda0b8a96b5d63d0ca8061ef4c0e771d48d8cd7c125a3aafea` |
| capture/evidence.json | `bdf72746c4c761583cf76fc247282d840e788e2fbc1ed1f636ca2c540207ee10` |
| baseline.png | `d5460e28fb8a0bb1423b6da6ee9a4f933cc7aefec9b8e790a09c723fe16a85dc` |
| rejected.png | `e054c8fe3330a7f64c08c4cd777ce63eb71fab1489a83a786dd1260edaa005b1` |
| connected.png | `c8b93a21fedbbaf63609c9a04455e61677b1e98ceb5d1874ff7ee3d01545cb5c` |
| reconnected.png | `5cb6409b51219d422d1930da35dcd0e49948b933b55833d4aed2b8984acbd5d3` |
| complete.png | `48dcda051866075b543aa488d1e95832f004ad29f9521573191fbd62522543bc` |

Original independent local report SHA256: `540da476fea430adeab33bab4c0b82dab09344f31a05330f2f9f9fbc4fa3cdc3`. This repository copy replaces local paths and states the code-match boundary without implying subsequent documentation was already committed.
