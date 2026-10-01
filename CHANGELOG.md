# Changelog

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
