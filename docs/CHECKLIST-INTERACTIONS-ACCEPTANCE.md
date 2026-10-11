# Checklist interaction enhancement — acceptance plan

Branch: `emma/checklist-postit-interactions`

## Post-it view
- [ ] Toggle an unchecked checklist item without opening Edit Notes.
- [ ] Toggle a completed item back to incomplete without opening Edit Notes.
- [ ] Confirm each toggle persists after refreshing/reloading the board.
- [ ] Confirm unchecked items appear first and checked items last.
- [ ] Confirm the original relative order is preserved within both groups.
- [ ] Confirm locked notes and notes outside the main Notes view cannot be toggled.
- [ ] Click `+N more` to reveal all items and `Show less` to collapse the preview.
- [ ] Confirm toggling items works while expanded and does not open the editor.
- [ ] Confirm a normal click elsewhere on the card still opens Edit Notes.

## Edit Notes view
- [ ] Drag an item using its drag handle to a new position.
- [ ] Confirm the saved custom sequence is shown in Edit Notes, regardless of completion state.
- [ ] Confirm post-it view respects that custom sequence within unchecked/checked groups.
- [ ] Check and uncheck items, then confirm their saved custom sequence is retained.
- [ ] Confirm adding, renaming, removing, and hiding completed items still work.

## Persistence and compatibility
- [ ] Confirm edits persist after closing/reopening a note and restarting RemoteHand.
- [ ] Confirm existing checklists with no explicit order metadata remain usable (the existing array order is the custom sequence).
- [ ] Confirm ordinary note cards and card-level drag-and-drop are unchanged.
