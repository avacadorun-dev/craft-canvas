# CraftCanvas contributor agreements

- Use `feature/`, `fix/`, `chore/` or `release/` branches. Never use `codex/`.
- Read `docs/PRD.md`, `docs/architecture.md` and the active OpenSpec change before behavior changes.
- Keep application schemas, ownership, storage, network, clipboard and media decoding in the host. File/URL/resource strings are inert data.
- Keep one canonical document. The native adapter emits guarded intents; it does not save or mutate a second authority.
- Preserve exact no-edit bytes, unknown envelopes, numeric precision, stable IDs and array order. Reject duplicate JSON keys before interpretation.
- Never deserialize proof/change authority. Prepare validates an exact session base; apply and compensating Undo refuse stale state.
- Bound input, output, work, geometry and retained history. Escape/focus loss cancels an uncommitted gesture. A gesture produces one atomic action.
- Use English for repository documentation and comments. Keep user text verbatim.
- Keep public fixtures synthetic. Do not upload private WriterCraft projects/media/screenshots or local application state.
- No local Cargo build/fetch/test in this task. Resolve and qualify only on GitHub. Local direct rustfmt and static inspection are allowed.
- Commit focused source and exact evidence. Do not claim native/application acceptance from compilation, generated images or model tests.
- No releases or product issue closure without the coordinating agent's acceptance workflow.

## Current priority — 2026-11-09

Focus on standalone production readiness and full functional React Flow parity, then qualified WriterCraft integration for original Studio Canvas parity. Follow `docs/ROADMAP.md` strictly, closing one fully verified issue before starting the next. Read `docs/PRODUCTION_PLAN.md`, `docs/REACT_FLOW_PARITY.md` and `docs/WRITERCRAFT_CONSUMER.md`. Preserve current work; do not restart. A separately coordinated agent may deliver non-Canvas WriterCraft alpha.3 in parallel. Do not edit its app behavior; it must wait for an exact qualified engine handoff before Canvas integration. One bounded engine milestone at a time, with independent review; root owns integration, publication and hosted dispatch. Do not wait for upstream maintenance or freeze a narrow four-side API before the full named-port/interaction contract is reviewed. Scope inventories are not exhaustive test matrices or evidence of completion.
