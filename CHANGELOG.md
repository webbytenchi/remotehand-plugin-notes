# Changelog

## 0.1.0 - Unreleased

### Added
- Initial RemoteHand Notes plugin foundation.
- Vault-backed private note and checklist documents.
- Sticky-note board with pinning, colors, labels, search, archive, Trash, locking, and manual card ordering.
- Lightweight Markdown rendering.
- Autosave and plugin settings.
- Attachment UI placeholder pending the generic RemoteHand Vault blob API.

### Fixed
- Vault auto-lock redaction now avoids a re-entrant Notes reload while the original host-owned unlock request is still in flight.
- Permanent-delete and Empty Trash confirmations use Notes-owned UI instead of browser modal dialogs, which are intentionally unavailable in RemoteHand's sandboxed plugin panels.
