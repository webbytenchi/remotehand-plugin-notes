# RemoteHand Notes

Private, Vault-backed sticky-note and checklist plugin for RemoteHand.

## Status

**v0.1.0 is complete and manually accepted on Windows. Current `main` also includes post-v0.1.0 UI/UX polish that has not yet been tagged as a new release.**

v0.1.0 includes:

- RemoteHand Plugin API v1 runtime/handshake
- scoped RemoteHand Vault document access
- encrypted Notes catalog + one encrypted Vault document per note/checklist
- host-owned Vault create/unlock request flow
- sticky-note card board
- plain notes + structured checklists
- lightweight Markdown rendering/editing
- colors, labels/tags, pinning, search, Archive, Trash, and note edit-lock
- drag-and-drop card ordering with Manual / Updated / Created / Title sort modes
- autosave
- Notes-specific settings
- Trash protection for locked notes
- encrypted file/image attachment storage through RemoteHand Vault blobs
- attachment picker, drag-and-drop, reorder, image preview, open, and cleanup lifecycle

### Current mainline polish

Current `main` additionally includes post-v0.1.0 UI/UX changes that are not part of the frozen v0.1.0 tag:

- responsive single-line Notes toolbar with adaptive wrapping
- editor keyboard flow from Title → Body → Labels/tags
- empty newly-created notes/checklists are discarded when closed without meaningful content
- sticky-note sidebar icon and refined folded-corner/diagonal paper-gradient card styling
- card-level attachment previews with image thumbnails, document tiles, mixed attachment rows, and overflow counts

Attachment bytes are persisted only through RemoteHand's encrypted Vault blob API. Normal note metadata stores opaque attachment references; attachment payloads are never placed in plaintext plugin storage.

See [docs/V0.1.0-SPEC.md](docs/V0.1.0-SPEC.md) for the approved product design.

## Core direction

- Sticky-note card board
- Plain notes and checklist notes
- Lightweight Markdown
- Vault-encrypted note content and private metadata
- Labels/tags rather than folders
- Pinning, colors, search, archive, Trash, and note locking
- File and image attachments backed by the core Vault blob API
- Autosave
- Manual card rearranging plus automatic sort modes

## Security boundary

RemoteHand core owns Vault cryptography, unlock state, passphrase handling, key handling, permission enforcement, and namespace enforcement.

Notes uses the caller-scoped Vault APIs and never receives the Vault passphrase or master/session key.

Plaintext `storage.plugin` is used only for non-private user preferences. Note titles, bodies, checklist text, labels, ordering/private state, and attachment metadata stay in Vault-backed private documents.

## Build on Windows

~~~powershell
cd C:\Users\Ringo\remotehand-plugin-notes
git switch main
git pull --ff-only

cargo test
.\scripts\build-windows.ps1
~~~

The build script produces an installable folder under `dist\remotehand-notes-windows-x86_64` and a validation ZIP with SHA256.

## Validation status

The v0.1.0 implementation was manually accepted with RemoteHand's Vault blob-capable host build. Acceptance covered note/checklist behavior, persistence, Vault lock/unlock redaction, file and image attachments, PNG/WEBP previews, system-open behavior, drag-and-drop, clipboard image paste, attachment reordering/removal, locked-note attachment restrictions, restart persistence, and Trash/permanent-delete cleanup.

The current card UI uses the selected note color as its base with a subtle top-left → bottom-right paper gradient, a clipped folded corner, dark ink contrast, and lightweight attachment previews. Image attachments can appear as card thumbnails, non-image files as document tiles, and mixed attachment sets show up to three visible items with a `+N` overflow indicator.

New empty note/checklist drafts are transient: if the editor is closed before the user adds meaningful content (title, body/checklist text, labels, or attachments), the empty draft is discarded instead of leaving an empty card.

## Documentation

- [v0.1.0 design specification](docs/V0.1.0-SPEC.md)
- [Vault/storage integration contract](docs/VAULT-INTEGRATION.md)
