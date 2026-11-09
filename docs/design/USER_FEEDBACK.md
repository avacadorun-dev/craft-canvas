# Canvas design feedback

## CV-01 — separate designer-authored WriterCraft board direction

2026-11-09. The user asked to see the actual native macOS Canvas and requested designer-authored visual references to guide implementation. The user clarified that the previous design references covered the writing editor only; there was no separate concept for the board inside WriterCraft. A technical comparison with React Flow is not that missing visual design.

Prepare a dedicated board concept using the agreed native editor style as context. Cover the full board composition, cards, connections, selection, object actions and interaction states. Distinguish primary external references, the designer's proposed adaptation and actual compiled implementation. Do not describe a concept as an accepted design or a working build.

The independent designer owns the proposal and visual criteria. Root integrates only applicable behavior within the current ordered milestone. CC-03 reviews connection affordances and states against this direction; later milestones retain responsibility for their complete component scope. The downstream app agent still owns application changes and waits for the qualified engine handoff. The existing app preview is preserved.

## CV-02 — retained conflict looks like missing direct manipulation

2026-11-09. In the actual `e68bbdda298dd5a4401912f3ae45b7a61c64c344` native showcase, the user reached a held connection conflict through the testing controls and asked whether only buttons were supported, including whether node dragging, deletion and mouse connection creation were unavailable. The screenshot showed `Some(Conflict)` and disabled board interaction, but no clear recovery explanation.

The guarded pending state is intentional; its discoverability is inadequate. Explain visibly that a pending/conflicting host decision temporarily blocks graph input and provide a direct explicit discard/recovery action in the synthetic host. The host must reject/cancel its held write before dismissing the pending request; recovery must not alter canonical data or erase an unconfirmed real persistence operation. Preserve direct mouse creation/reconnection, node movement and selected node/edge Delete/Backspace. Keep embedded text input ownership.

The board design and independent native review must include blocked and recovered states as well as normal manipulation. Engineering failure-injection controls must be identified as testing controls, not presented as the intended product workflow. Do not restart the user's currently explored window to install the correction.

## CV-03 — rejected native demonstration

2026-11-09. After trying the native showcase, the user rejected it as nonworking and reported being unable to accomplish anything in it, then instructed continued library development. This supersedes any implication that successful hosted checks or the root's limited pointer observations established usable native acceptance. CC-03 remains open.

The default demonstration must expose an understandable ordinary graph workflow with immediate in-memory acknowledgement, direct manipulation and Undo/Redo. Put deliberate failure-injection controls behind a clearly identified testing disclosure. Keep pending/conflict explanation and explicit safe recovery reachable even when that disclosure is closed. Do not ask the user to navigate a test harness to prove core functionality.

Complete and independently verify one coherent native graph journey through creation, movement, edge selection/deletion, either-end reconnection, cancellation, failure recovery and embedded input before another candidate handoff. Preserve the rejected evidence and the explored window. The separately prepared board design remains a proposal, not native acceptance.

## CV-04 — board reference must represent the full original workspace

2026-11-09. Three privately supplied screenshots demonstrate that the first designer proposal represents a substantially narrower board than the original Studio. Visible examples include mixed text/document cards with types and statuses, labeled relations, spatial groups and nested frames, image/PDF/audio/video surfaces, URL and other-board references, shapes/arrows/freehand marks, minimap and compact tool/context panels. The screenshots are visual scope evidence, not proof of hidden interactions or native implementation; no private image, media or project text is published.

Retain all B01–B26 requirements in [the consumer contract](../WRITERCRAFT_CONSUMER.md). These are existing migration obligations, not newly selected enhancements. Revise the design using synthetic research/moodboard and story-planning examples with representative density, heterogeneous content and direct creation/edit/resize/delete entry points. Show how compact tools and contextual properties expose the scope without a permanently expanded command list. Cover media-control input ownership, image display sizing, frame/group distinction, nested-board navigation/return and connection labels; static media placeholders do not demonstrate playback or decoding.

Independent design review must check both representative composition and reachability of the remaining workflows: views/layout preview, schema editing, navigation/search, diagnostics/playthrough, presentation/timer, board lifecycle and editable/export output. Anything not yet designed remains explicitly pending. The revised reference is a proposal until reviewed; it cannot accept library implementation or WriterCraft parity. CC-03 remains the only active implementation issue, with the ordered milestones unchanged. The [scope allocation](../WRITERCRAFT_CONSUMER.md#visual-scope-correction--cv-04) keeps engine primitives and host-owned workflows required at their respective gates.
