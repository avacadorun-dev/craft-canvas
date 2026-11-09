# Restore visible connector interaction for WriterCraft #17

The existing adapter paints every stored connector as a straight unmarked line, cannot select edges directly, and ignores Delete when only an edge is selected. Imported marker/side data and prepared model commands therefore lack reachable native behavior.

Restore source smooth-step geometry and arrow ends, share the actual derived path with hit testing and host label anchors, select a line without consuming a node hit, and emit guarded edge-only deletion. Correct the source's hidden same-card route with a visible outside-card loop. Keep canonical data, host schemas, persistence and endpoint-authoring UI unchanged. Qualification and application acceptance remain separate.
