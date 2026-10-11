# RemoteHand Notes v0.1.3 — Checklist Interactions Acceptance Record

Date: 2026-10-12

## Result

Papa manually tested the v0.1.3 checklist interaction candidate on Windows and confirmed that the checklist interactions work as intended.

## Accepted behavior

### Post-it view
- [x] Toggle an unchecked checklist item without opening Edit Notes.
- [x] Toggle a completed item back to incomplete without opening Edit Notes.
- [x] Checklist status changes persist.
- [x] Unchecked items appear first and checked items last.
- [x] Relative order is preserved within the unchecked and checked groups.
- [x] Locked notes and notes outside the main Notes view are not toggleable.
- [x] Click `+N more` to reveal all items and `Show less` to collapse the preview.
- [x] Toggling items while expanded works without opening the editor.
- [x] Normal clicks elsewhere on the card still open Edit Notes.

### Edit Notes view
- [x] Drag an item using its drag handle to a new position.
- [x] The saved custom sequence is shown in Edit Notes, regardless of completion state.
- [x] Post-it view respects the custom sequence within unchecked/checked groups.
- [x] Checking and unchecking items retains the saved custom sequence.
- [x] Adding, renaming, removing, and hiding completed items continue to work.

### Persistence and compatibility
- [x] Edits persist after closing/reopening a note and reloading the board.
- [x] Existing checklists without explicit order metadata remain usable; their array order is the custom sequence.
- [x] Ordinary note cards and card-level drag-and-drop remain unchanged.

## Implementation and build validation

- Checklist rendering groups a display copy by completion state; it does not mutate the saved custom sequence.
- Checklist item reordering uses a dedicated drag handle in Edit Notes.
- Runtime plugin version is derived from Cargo package metadata rather than a separately hard-coded string.
- Windows build script now stops on Rust test or release-build failure instead of continuing to package a failed build.
- Rust tests: **7 passed, 0 failed**.
- Optimized Windows build and installable package/ZIP creation succeeded.
- Windows ZIP SHA-256: `FE3E54FC7D33346DB0C840DEB53FF9B6123597F7FE606491AB6A3F106DA04E4D`.

## Compatibility

- RemoteHand Plugin API remains v1.
- Existing Vault-backed note/checklist data format is preserved.
- No note-data migration is required.

## Release state

Accepted for the v0.1.3 release. The annotated tag and GitHub Release should point to the final, documented main commit after the feature changes are merged.
