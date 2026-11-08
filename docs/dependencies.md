# Dependency policy

Initial exact direct pins match the reviewed WriterCraft graph: serde1.0.229, serde_json1.0.154 (`raw_value`, `arbitrary_precision`), thiserror2.0.21 and egui0.36.2. These packages keep their upstream MIT/Apache licenses. New CraftCanvas code is MIT; no upstream application source is copied by the foundation.

No UUID, hash, async runtime, graph engine, WebView or network dependency is added. IDs and resource capabilities come from the host. Drawing and layout providers require a separate source/license/MSRV/feature/bounds qualification. Candidate research is not dependency approval.

Hosted metadata produces a lock artifact with source identity. Review and commit the exact minimal lock, then rerun locked tests/lints. Never pin an unqualified branch or mutate a lock by broad update to hide a failure.
