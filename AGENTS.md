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
