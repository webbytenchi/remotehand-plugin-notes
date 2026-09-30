# RemoteHand Notes

Private, Vault-backed sticky-note and checklist plugin for RemoteHand.

## Status

**v0.1.0 foundation is under active development on `emma/notes-v0.1.0-foundation`.**

The current foundation includes:

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
- attachment UI/storage model placeholder

Actual attachment bytes are deliberately **not** persisted yet. Full attachment persistence requires the generic RemoteHand Vault encrypted-blob capability described in [docs/VAULT-INTEGRATION.md](docs/VAULT-INTEGRATION.md).

See [docs/V0.1.0-SPEC.md](docs/V0.1.0-SPEC.md) for the approved product design.

## Core direction

- Sticky-note card board
- Plain notes and checklist notes
- Lightweight Markdown
- Vault-encrypted note content and private metadata
- Labels/tags rather than folders
- Pinning, colors, search, archive, Trash, and note locking
- File and image attachments after the core Vault blob API exists
- Autosave
- Manual card rearranging plus automatic sort modes

## Security boundary

RemoteHand core owns Vault cryptography, unlock state, passphrase handling, key handling, permission enforcement, and namespace enforcement.

Notes uses the caller-scoped Vault APIs and never receives the Vault passphrase or master/session key.

Plaintext `storage.plugin` is used only for non-private user preferences. Note titles, bodies, checklist text, labels, ordering/private state, and attachment metadata stay in Vault-backed private documents.

## Build on Windows

~~~powershell
cd C:\Users\Ringo\remotehand-plugin-notes
git switch emma/notes-v0.1.0-foundation
git pull --ff-only

cargo test
.\scripts\build-windows.ps1
~~~

The build script produces an installable folder under `dist\remotehand-notes-windows-x86_64` and a validation ZIP with SHA256.

## Documentation

- [v0.1.0 design specification](docs/V0.1.0-SPEC.md)
- [Vault/storage integration contract](docs/VAULT-INTEGRATION.md)
