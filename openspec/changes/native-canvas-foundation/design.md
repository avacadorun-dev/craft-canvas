# Foundation design

Use exact original document bytes plus preserved raw node/edge envelopes. Known edits merge only intended fields and revalidate the entire candidate before producing a private prepared change. A unique session epoch and monotonic change revision make stale intent and Undo refusal explicit.

The native adapter keeps only camera/selection/gesture/edit drafts. It emits a transaction at gesture commit and discards preview state on Escape, focus loss or changed base. Host callbacks paint authorized resources and allocate IDs; the engine has no ambient I/O authority.

Source-app extension encoding lives in the host dialect. Standard group semantics and unknown preservation are generic; domain schemas, resource checks, revisions and assistant Apply remain host-owned.
