# Dependency policy

Initial exact direct pins match the reviewed WriterCraft graph: serde1.0.229, serde_json1.0.154 (`raw_value`, `arbitrary_precision`), thiserror2.0.21 and egui0.36.2. These packages keep their upstream MIT/Apache licenses. New CraftCanvas code is MIT; no upstream application source is copied by the foundation.

No UUID, hash, async runtime, graph engine, WebView or network dependency is added. IDs and resource capabilities come from the host. Drawing and layout providers require a separate source/license/MSRV/feature/bounds qualification. Candidate research is not dependency approval.

Hosted metadata produces a lock artifact with source identity. Review and commit the exact minimal lock, then rerun locked tests/lints. Never pin an unqualified branch or mutate a lock by broad update to hide a failure.

CC-03 adds a `publish = false` standalone showcase workspace member with exact eframe0.36.2 and image0.25.10 pins. Eframe enables only wgpu/accesskit/default_fonts/x11/wayland; image enables PNG only. These are synthetic qualification-host dependencies, not new dependencies of the model or adapter crates. Existing egui/serde pins and Rust1.95 remain unchanged. Hosted lock resolution and review are required before candidate qualification; no local Cargo resolution is permitted.

First hosted resolution (source3910a16, run42483818599):81 packages, exact lockSHA256 `54a64840a06c1947f4df8b7978b094e087943b42af07a254e7e63bf7a9530a29`. Direct pins match the approved baseline. Native egui's existing transitive font package declares `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0`; self_cell permits Apache-2.0, foldhash is Zlib, unicode-ident includes Unicode-3.0 and zerocopy permits BSD-2-Clause/Apache/MIT. Choose permissive upstream alternatives and retain their notices in any bundled host distribution. No GPL-only dependency was introduced. All declared transitive MSRVs fit the1.97 supported compiler. Metadata resolution is not test or native acceptance.
