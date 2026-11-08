# Qualification policy

All fixtures are synthetic. Hosted CI must retain exact source, compiler, lock and feature metadata. Locked workspace tests, strict Clippy, rustfmt and the supported MSRV must pass before a revision is acknowledged to WriterCraft.

Model proof does not establish native gestures, media playback, application ownership, IME, accessibility or independent design acceptance. The adapter requires synthetic native egui event paths plus actual compiled host journeys. No private reference archive or owner screenshot belongs in CI.

Current status: source scaffolding; no Cargo qualification, native interaction acceptance or release is claimed.

## Foundation checkpoint

Source3910a16/c356b1e began hosted qualification42483952407. The model library compiled on all three targets; integration test compilation was correctly blocked by missing test-crate documentation. That source is not qualified. The documentation is corrected in the next adapter checkpoint without weakening any test.

The current adapter supplies native camera/grid/culling, click/multiselect/marquee, move/resize previews, typed commit/cancel and clipboard/navigation intents, and a host-mounted native Markdown source-draft component. Its plain example body is deliberately inert text, not a claim of GFM/media rendering. Connector hit/edit/handles, minimap, touch/pressure drawing, shape/frame controls, real media/GFM hosts, complete keyboard/accessibility/IME/native-OS/performance/host acceptance remain pending. No WriterCraft issue is closed by this checkpoint.

## First exact green foundation

Exact source `78f6469bc978d597e603c8de9c7f526e04af2f0b`, hosted run [42487443225](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42487443225), passed committed-lock verification, all26 semantic/native-egui-event tests, rustfmt and strict Clippy on Linux/macOS/Windows with Rust1.97. This proves the foundation contracts only; full Canvas/app/native-OS acceptance remains pending.

The consumer's baseline is Rust1.95, matching egui0.36.2's declared MSRV. The next checkpoint lowers the manifest and hosted exact compiler to1.95.0; the earlier1.97 green does not substitute for that actual qualification.

## Alpha.3 handoff after release-scope change

The owner moved all unfinished Canvas work to WriterCraft alpha.3. Exact generic foundation `6e3a54329a149dfb31af75d2e807abd99cfd0512` passed Rust1.95 locked tests/fmt/strictClippy on all3targets in run [42490395810](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42490395810), including the source-history-capacity and5000-node movement regressions (28 total contracts). Its lock remains unchanged from the reviewed baseline.

The next generic-only checkpoint adds connector model geometry, exact endpoint/label commands, four synthetic regressions and upstream MIT attribution. It is an alpha.3 preparation checkpoint, unqualified until its exact hosted run completes. It adds no WriterCraft integration/pin or release scope. Native connector controls/minimap/drawing/layout/GFM/media and complete Canvas acceptance remain unfinished.


## Alpha.3 source-compatibility correction

Hosted run42491125972 qualified exact connector-model source63845f9 on all3targets at Rust1.95 with an unchanged lock. Consumer audit then found the generic parser imposed a shared1024-character limit on file and URL references, while the original source accepts link URLs up to2048UTF-8bytes. The next checkpoint separates those existing source bounds and adds long-link move/Undo, absent-extension and CJK byte/character regressions. Its hosted qualification remains pending until exact-source CI completes; this is not native application acceptance.
