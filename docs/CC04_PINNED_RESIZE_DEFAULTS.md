# CC-04 pinned resize defaults — exact dependency addendum

Date: 2026-11-10. Read-only source audit resolving only ambiguity 1 in [the original sizing source audit](CC04_SIZING_SOURCE_CONTRACT.md) (SHA256 `218a32ea66c6085cd0604d29bb2ef9da2b55316cfd4edf513e4ecbd4bfdc1c51`). This establishes original dependency behavior from verified release bytes, not executed browser/native acceptance or approval of a new sizing API. No package installation/execution, builds, Cargo, GUI, source checkout modifications or private project-content inspection occurred. Only official npm metadata/archive downloads and local report artifacts were written. CC-04 remains the sole active engine issue; its separate takeover-cancellation correction is outside this audit.

## Exact release identity and primary sources

Original Studio source revision is `24ac9ef222700a6e3d618139b701e53abc61d6a5`; its unchanged `bun.lock:850–852` pins React 12.11.2 and system 0.0.79. Each downloaded archive's SHA-512 was verified against both that lockfile and official registry metadata **before extraction**. React's package manifest also depends exactly on system 0.0.79. Registry metadata points to `github.com/xyflow/xyflow`, directories `packages/react` and `packages/system`, but supplies no `gitHead`; no unverified Git commit/tag is asserted.

| Release | Official primary sources | Archive SHA256 |
| --- | --- | --- |
| `@xyflow/react@12.11.2` | [Version metadata](https://registry.npmjs.org/@xyflow%2freact/12.11.2), [published tarball](https://registry.npmjs.org/@xyflow/react/-/react-12.11.2.tgz) | `ddfb74037a3979ed98c8784034071f24040b5a6b7d560254fcb6c6676946da47` |
| `@xyflow/system@0.0.79` | [Version metadata](https://registry.npmjs.org/@xyflow%2fsystem/0.0.79), [published tarball](https://registry.npmjs.org/@xyflow/system/-/system-0.0.79.tgz) | `1b099ccb0022753fa8185647a9ad2dff9f02872ec6be7b40a052a89ba7d9e87a` |

Verified archive integrity:

- React: `sha512-eLAlDWJfWnQEhJwGMjlWdAXO9eYllKpliUmPQlAmOLxz6mExXuzMVDUKLMquixgkrtmMFFtug3jGKmYYld12cA==`
- System: `sha512-czLyOh91NF0hIzbNzwi8I6GlqG23BHh2435OddfI6uiaLH3xdrdygO93gqgH1Bv9mhy8XPFQJOBn1FTq4LvEWA==`

Extracted source references below:

- **R** = React archive member `package/dist/esm/index.js`, SHA256 `75838d1889dce44e6b9cbfd1d2e19cd8cfcf587099837b46af7245b78ee198e2`.
- **S** = system archive member `package/dist/esm/index.js`, SHA256 `42cc6461712c2b302bbc2b1890593af31415bc3a3e93d548c76f3d2b3e7268ea`.
- Registry responses and verification manifests are retained beside each `package.tgz`. Metadata SHA256: React `687745c393bd6ef3a2db9bd10abbb33538c99318f626ac883c22f71b8b1cfad9`; system `e772d02b1e00690216691555d0351200f083f81c1266ff81f64dd7e32f1059f6`.
- **B** = original Studio `src/widgets/canvas-board/ui/canvas-board.tsx`, revision above, SHA256 `be67e4c9d97a490df4053844e1cc317ce77d7098c8383299f5a3940300940d87`.

## Confirmed defaults and geometry

| Area | Exact pinned behavior and consequence |
| --- | --- |
| Controls | `NodeResizer` produces four corner handles (top-left, top-right, bottom-left, bottom-right) and four side controls (top, right, bottom, left): R4976–4980; S3115–3116. Its controls have the `nodrag` class (R4939). Studio's visible/selected editable wrapper therefore delegates to all eight controls, not one bottom-right handle. The standalone `NodeResizeControl` default position is bottom-right for a handle and right for a line; that standalone default does not reduce `NodeResizer`'s set (R4801–4805). |
| Free resize anchoring | Left changes x and width; top changes y and height; right/bottom change their dimension without moving the top-left position. Thus, with default origin, a side holds its opposite side and a corner holds its opposite corner. Side controls affect one dimension while free-aspect corner controls affect both. This follows the direction flags and final geometry equations, S3145–3155, S3188–3200, S3327–3333. |
| Aspect and Shift | `keepAspectRatio` defaults false (R4805, R4976). Neither the resizer wrapper nor XYResizer implements a Shift-key aspect toggle. Studio supplies no aspect prop (B325–343), so its original reachable card resizing is free-aspect. The optional dependency mode captures the starting measured width/height ratio (S3406–3410), couples dimensions and cross-checks constraints (S3243–3325). With origin zero, ratio-preserving right/bottom side drags hold the upper-left corner; left/top side drags hold the lower-right corner, rather than the orthogonal midpoint. This optional mode is dependency capability, not an original Studio image policy. |
| Origin | Store default is `[0,0]` (R176, R3269); a node-specific origin overrides the store during resize (S3453–3467). Size changes compensate stored x/y according to the origin (S3327–3333); origin coordinates represent a fraction of node width/height (S272–280). Studio supplies no override or parent hierarchy in its codec, so default top-left semantics apply. This is independent of its shape-only SVG rotation center. |
| Pointer/grid/rounding | Pointer coordinates are made pane-relative, transformed into flow units, then optionally snapped to nearest grid multiples (S640–651, S843–852). Both initial and current pointers are snapped; affected-axis displacement is then `Math.floor(current − initial)`, including when grid snapping is off (S3394–3410, S3195–3196). This preserves initial geometry residue rather than necessarily snapping final edges or dimensions to the grid. It also makes negative subunit displacement differ from rounding toward zero. Ratio derivation and clamping can produce fractional geometry despite floored input deltas. |
| Bounds/clamping | Defaults are min width/height 10 and max width/height `Number.MAX_VALUE` (R4976); Studio replaces only minima, with 180×120 ordinary cards, 240×160 groups/frames, 40×40 shapes. Size, applicable node/parent extents and constrained-child enclosure contribute the strongest per-axis clamp; aspect mode also propagates the other dimension's restrictions (S3188–3306). The resulting clamped displacement drives both position and size. Parent/child extents are captured at gesture start (S3413–3448), not rebuilt each frame. Studio has no parentId mapping, so those hierarchy branches do not establish group/frame containment behavior there. |
| Handle scale | `autoScale` defaults true. Handles use scale `max(1/zoom, 1)`; the calculation is not applied to line variants (R4796–4805). This prevents corner handles shrinking below their base screen size when zoomed out; it is not a constant-screen-size formula at zoom above one. |

The original board passes snap grid 20×20 and disables it through Alt-held state (B1190–1200, B2574–2575). React emits a position change when top/left resizing changes x/y (R4863–4869), so that **can enter** Studio's existing single-position alignment-guide pass (B2592–2627). That pass may adjust position independently of dimensions. Therefore, pointer-grid snapping does not establish that final opposite edges, final dimensions or final saved geometry always align to 20 units. The backend's 10,000 size maximum is not supplied to this resizer; the previously recorded UI/save-bound mismatch remains real source-level evidence.

## Lifecycle and cancellation limits

1. **Start:** XYResizer reads measured dimensions and stored position, snapshots pointer/ratio and applicable hierarchy constraints, then invokes `onResizeStart` (S3386–3450). The geometry baseline is the gesture start, not accumulated pointer deltas.
2. **Drag:** unchanged geometry returns immediately (S3472–3474). Accepted updates call `onResize` and internal `onChange`; React emits optional expansion/position changes, dimensions with `resizing: true`, and child position corrections (S3530–3536; R4833–4892). Top/left hierarchy resizing compensates child-relative positions to preserve their world placement (S3480–3493).
3. **End:** `onResizeEnd` and internal `onEnd` run only after at least one accepted resize frame (S3538–3544). React's `onEnd` emits final dimensions with `resizing: false` (R4894–4904). A start-only/fully vetoed gesture gets no end callback. Studio commits on that terminal false value (B2628–2638).
4. **No explicit rollback contract:** the inspected XYResizer exposes `update` and `destroy`, with start/drag/end handlers. It has no explicit Escape, focus-loss, pointer-loss or cancellation-restoration branch. `destroy()` removes `.drag` listeners from the control selection (S3548–3553); React cleanup calls it (R4923–4925). Those branches do not themselves restore starting geometry or emit a terminal change. Underlying d3 active-listener/touch-cancel behavior was not audited, so no broader browser cancellation guarantee is claimed.
5. **Optional veto caution:** internal `prevValues` and child positions are updated before `shouldResize` is called (S3475–3506, S3530). Returning false suppresses that frame's callbacks, not those internal assignments. Source implication, not an executed test: an accepted frame followed by a vetoed frame can leave the later computed values available to end callbacks, including React's final dimension emission. Studio does not pass this optional veto callback. This behavior must not be copied as a desired native rollback policy.

## Bounded conclusion

Ambiguity 1 is resolved for exact original package bytes: all eight controls, opposite anchoring, free aspect by default without Shift toggling, top-left default origin, grid-before-delta behavior, bounds precedence and terminal commit lifecycle are established. Existing Studio-specific media, rotation, group/frame and lock findings in the earlier report remain unchanged. Generic geometry/constraint/gesture mechanics belong in CraftCanvas; host per-kind minima, media presentation and authoring/persistence policy remain host-owned. Native cancellation must be specified explicitly; this dependency source does not supply a safe rollback guarantee.

A comparison with frozen React 12.12.1/system 0.0.84 was unnecessary to resolve the original-source ambiguity and was not performed. No live browser gesture, physical modifier/touch sequence, native acceptance or exhaustive React Flow parity is asserted. There is no remaining archive-access blocker. This report contains no manuscripts, project fixtures, credentials, media or screenshots and does not authorize any additional implementation scope.

Original independent local report SHA256: `3045fd4aa168e94c341df8d60ce71d72d4fd605b55993875a5cd91573793be99`. This repository copy replaces temporary local paths with archive-relative references.
