# Qualification policy

## Current checkpoint — 2026-11-09

The exact existing engine `6b68eb4e84aa41b713a148e6a456048f3c1168f9` passed [qualification42617390880](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42617390880): committed lock, Rust1.95 formatting, model/native adapter tests and strict Clippy on Ubuntu/macOS/Windows. This includes derived smooth-step routes/markers, shared hit geometry and edge selection/Delete. Earlier e5dc514 failed visibility compilation and was corrected before this qualified revision. These are engineering checks, not complete native/React Flow/production acceptance.

Completed preparation is [CC-01](https://github.com/avacadorun-dev/craft-canvas/issues/1) under [ROADMAP.md](ROADMAP.md). The74-row RF inventory,122props,66source examples, public catalogue and B01–B26 consumer boundary have independent source omission review; every behavior remains unaccepted until its own evidence. Exact preparation9da1c24 passed hosted qualification42626580148 on all3platforms, with independent preparation review in CC01_REVIEW.md. Issue1 is closed. CC-02 also completed at99db516; CC-03 is active. The preparation documents do not modify Rust source or adopt a new dependency. No package publication or production readiness is claimed. Older status paragraphs below are historical checkpoints.

CC-02 exact source `99db5166f287831c96da7ba6630a2bb4d620ee4e` passed [qualification42633082977](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42633082977): all104 tests, headless named-port example, fmt, strict Clippy and unchanged lock on all3platforms. [CC02_PROGRESS.md](CC02_PROGRESS.md) records failed attempts and complete final acceptance; [CC02_REVIEW.md](CC02_REVIEW.md) contains independent exact-content review. This establishes the model/codec boundary only; native named-port gestures and production acceptance remain pending.

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
