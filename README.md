# RemoteHand Notes

Private, Vault-backed sticky-note and checklist plugin for RemoteHand.

## Status

**v0.1.2 is the current published Notes release. The `emma/checklist-postit-interactions` branch prepares v0.1.3 with inline checklist toggles, expandable previews, and drag-to-reorder editing; it remains pending manual acceptance and is not yet released.**

v0.1.0 foundation includes:

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

### v0.1.2 board and interaction polish

v0.1.2 adds the next accepted Notes usability pass:

- auto-hiding Notes header with a subtle top-edge reveal chevron so cards can use more of the panel
- double-click empty board space to create a new plain note
- persistent Sort dropdown preference
- more forgiving lightweight Markdown heading rendering
- responsive card attachment previews that show as many image/file tiles as fit before using `+N`

#### v0.1.3 checklist interaction candidate

- Toggle checklist items directly on post-it cards without opening Edit Notes.
- Display unchecked items first and checked items last while preserving the user's saved order within each group.
- Expand beyond the five-item preview with `+N more`, then collapse with `Show less`.
- Reorder checklist items in Edit Notes using a dedicated drag handle; the saved sequence is retained independently of completion state.

Acceptance checklist: [docs/CHECKLIST-INTERACTIONS-ACCEPTANCE.md](docs/CHECKLIST-INTERACTIONS-ACCEPTANCE.md). These changes are pending manual testing and are not yet a published release.

### v0.1.1 UI/UX polish

v0.1.1 established the first accepted post-v0.1.0 UI/UX baseline:

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

The current card UI uses the selected note color as its base with a subtle top-left → bottom-right paper gradient, a clipped folded corner, dark ink contrast, and lightweight attachment previews. Image attachments can appear as card thumbnails, non-image files as document tiles, and mixed attachment sets now show as many tiles as fit in the available card width, with `+N` only when additional attachments do not fit.

New empty note/checklist drafts are transient: if the editor is closed before the user adds meaningful content (title, body/checklist text, labels, or attachments), the empty draft is discarded instead of leaving an empty card.

## Documentation

- [v0.1.0 design specification](docs/V0.1.0-SPEC.md)
- [v0.1.0 acceptance record](docs/V0.1.0-ACCEPTANCE.md)
- [v0.1.1 acceptance record](docs/V0.1.1-ACCEPTANCE.md)
- [v0.1.2 acceptance record](docs/V0.1.2-ACCEPTANCE.md)
- [Vault/storage integration contract](docs/VAULT-INTEGRATION.md)
