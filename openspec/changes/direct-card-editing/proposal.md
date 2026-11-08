# Direct native card editing

## Why

The original source Canvas mounts a multiline input inside a text card on double-click. A separate Markdown modal as the primary editor breaks that workflow and was observed in the native host.

## What changes

Add a native in-card TextEdit projection with exact zoom/clip/input ownership and IME-safe Done/Cancel outcomes. Preserve its private widget identity, local text selection/clipboard/Undo and later typing while an exact submitted source waits for host durability. Acknowledgement rebases only the exact submitted source in the same canonical session.

## Scope

The host owns debounced checkpoint/save, retries, project navigation and recovery. This library widget is not durable autosave or full GFM rendering acceptance. Native OS/IME and independent UX checks remain required.
