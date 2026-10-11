# Changelog

## 0.1.3 - 2026-10-12 (pre-release candidate)

### Added
- Inline checklist toggles on post-it cards, with completed items grouped at the bottom while preserving custom order within each group.
- Expandable checklist previews with `+N more` and `Show less` controls.
- Drag-and-drop checklist item reordering in Edit Notes using a dedicated drag handle.

### Validation
- Pending Papa's install and manual acceptance testing. This is a pre-release candidate; no v0.1.3 release or tag has been published.

## 0.1.2 - 2026-10-01

### Added
- Auto-hiding Notes header so the card board can use the full panel height, with a subtle top-edge reveal chevron and animated drop-down behavior.
- Double-clicking empty board space creates a new plain note while preserving normal card/control interactions.

### Changed
- Card attachment previews now adapt to available card width and show as many image/file tiles as fit before using a `+N` overflow indicator.
- Lightweight Markdown heading rendering is more forgiving and accepts headings with or without a space after the `#` markers.

### Fixed
- The board Sort dropdown now persists the selected sort preference through Notes settings and restores it on reload/restart.
- Markdown preview handling was hardened so supported lightweight Markdown renders correctly after editing.

## 0.1.1 - 2026-10-01

### Added
- Responsive Notes toolbar that keeps creation, search, view tabs, sort, status, and settings on one line when space allows and adapts cleanly at narrower widths.
- Editor keyboard flow from Title → Body → Labels/tags, including reverse Shift+Tab navigation for plain notes.
- Transient empty-draft behavior: newly-created notes/checklists are discarded on close when they contain no meaningful content.
- Sticky-note sidebar icon aligned with the Notes card visual identity.
- Card attachment previews: image thumbnails, document/file tiles, mixed attachment rows, and `+N` overflow indication when more than three attachments are present.

### Changed
- Refined sticky-note cards with a clipped folded corner and a stronger top-left → bottom-right paper gradient.
- Card attachment indicators now use visible attachment previews instead of the redundant bottom paperclip count.
- Multiple image/file attachments are represented by their actual attachment types rather than treating all remaining items as additional images.

## 0.1.0 - 2026-10-01

### Added
- Initial RemoteHand Notes plugin foundation.
- Vault-backed private note and checklist documents.
- Sticky-note board with pinning, colors, labels, search, archive, Trash, locking, and manual card ordering.
- Lightweight Markdown rendering.
- Autosave and plugin settings.
- Encrypted file/image attachment persistence through the generic RemoteHand Vault blob API.
- Attachment picker, drag-and-drop attachment paths, image previews up to 8 MiB, attachment reordering, open-with-system-app, and encrypted blob cleanup on note deletion.

### Fixed
- Vault auto-lock redaction now avoids a re-entrant Notes reload while the original host-owned unlock request is still in flight.
- Permanent-delete and Empty Trash confirmations use Notes-owned UI instead of browser modal dialogs, which are intentionally unavailable in RemoteHand's sandboxed plugin panels.
- Image previews are loaded sequentially and rendered as bounded data URLs, avoiding concurrent nested Vault reads that could previously desynchronize the plugin runtime.
- Note cards now use solid selected colors and more paper-like sticky-note styling, with dark ink contrast and subtle paper depth.

### Notes
- Attachment open materializes a temporary plaintext copy under the OS temp directory only for the external system application, schedules cleanup retries, and removes stale Notes temp files on the next plugin start.
- Image compression settings remain reserved for the follow-up compression-policy implementation; current attachment storage preserves original file bytes.

### Validation
- Manual acceptance completed on Windows for the full v0.1.0 scope, including Vault lifecycle behavior and encrypted attachment workflows.
