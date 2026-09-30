# RemoteHand Notes — Vault and Storage Integration Contract

Status: **Design baseline derived from RemoteHand v1.3.0**

Reviewed host baseline:

- RemoteHand tag: `v1.3.0`
- Release commit: `bcff5cf290f0aa5c5601521dd221723c055f2ab8`
- Plugin API version: `1`

This document records what Notes can use from the existing RemoteHand host today and what generic core capability is still required before attachment support is complete.

## 1. Canonical plugin identity

RemoteHand Notes should use:

`remotehand.notes`

This follows the established RemoteHand plugin-ID convention used by plugins such as `remotehand.editor`, `remotehand.toolbox`, and `remotehand.target-monitor`.

The host derives Notes Vault references from caller identity. Notes supplies only relative keys.

Example:

Plugin key:

`notes/019abc.../record`

Host-owned full Vault reference:

`plugin:remotehand.notes/notes/019abc.../record`

Notes must never construct or submit a cross-plugin or `core:` Vault reference.

## 2. Required v0.1.0 manifest permissions

For private Notes data:

- `vault.documents.read`
- `vault.documents.write`
- `vault.access.request`

For non-private plugin preferences only, Notes may also use:

- `storage.plugin`

Notes does **not** need the deprecated `credentials.use` / `credentials.store` aliases.

Notes does not need `vault.secrets.*` for the initial design because note content is structured private data rather than credential material.

Attachment support will require a future generic Vault blob permission/API if that capability is added to core.

## 3. Existing host-owned Vault access flow

When Notes needs private data and Vault is locked, call:

`vault.request_access`

Example params:

~~~json
{
  "reason": "Open your private Notes."
}
~~~

RemoteHand owns the create/unlock UI.

Notes:

- does not collect the Vault passphrase;
- does not submit the passphrase;
- does not receive the passphrase;
- does not receive the derived/master key.

The request completes only when Vault is available, the user cancels, or the host request times out.

If the Vault is already unlocked, access succeeds without another prompt.

## 4. Existing Vault document API

Notes may use these existing Plugin API v1 methods:

- `vault.document.get`
- `vault.document.set`
- `vault.document.delete`

Required permissions:

- read: `vault.documents.read`
- set/delete: `vault.documents.write`

### Key rules

Caller-relative Vault document keys:

- 1-256 characters
- ASCII letters/digits plus `/`, `.`, `-`, `_`
- no empty path segments
- no `.` or `..` path segments

Core derives:

`plugin:<plugin-id>/<key>`

Therefore Notes' effective namespace is automatically:

`plugin:remotehand.notes/...`

### Document values

The current API stores a string value.

The host currently enforces a maximum document value of **16 MiB**.

A missing document returns `value: null`.

## 5. Proposed Notes private-document layout

Use multiple Vault documents rather than one monolithic notebook.

Suggested logical keys:

~~~text
meta/schema
board/main
board/archive
board/trash
note/<note-id>
~~~

Examples:

~~~text
meta/schema
board/main
note/019abcde-...
note/019abcef-...
~~~

### meta/schema

Small versioned private document for Notes data-model versioning.

Example:

~~~json
{
  "version": 1
}
~~~

### note/<note-id>

One encrypted structured document per note/checklist.

Suggested shape:

~~~json
{
  "version": 1,
  "id": "019abcde-...",
  "type": "note",
  "title": "Example",
  "body": "Lightweight **Markdown**.",
  "checklist": [],
  "labels": ["work"],
  "color": "yellow",
  "pinned": true,
  "locked": false,
  "location": "main",
  "createdAt": "2026-10-01T03:00:00+08:00",
  "updatedAt": "2026-10-01T03:10:00+08:00",
  "attachments": [
    {
      "id": "019abd...",
      "name": "example.pdf",
      "mediaType": "application/pdf",
      "size": 123456,
      "blobKey": "attachment/019abd..."
    }
  ]
}
~~~

For checklist notes, `type` is `checklist`, body use may be minimal/optional, and checklist items are structured records.

Everything in this document is private Vault-backed data.

### board/main, board/archive, board/trash

These documents may hold board-level ordering/state that should not be duplicated into every note.

Suggested example:

~~~json
{
  "version": 1,
  "manualOrder": ["note-id-1", "note-id-2"],
  "pinnedOrder": ["note-id-3"]
}
~~~

Automatic sort selection is a user preference and may live in plugin settings; manual order itself is private board metadata and should remain Vault-backed.

## 6. Non-private plugin settings

`storage.plugin` is acceptable only for settings that do not reveal private note content.

Examples:

- autosave delay
- attachment default limits
- image-resize/compression preference
- default board density
- default sort mode
- confirm-permanent-delete setting
- confirm-empty-trash setting

Do not put the following in plaintext plugin storage:

- note titles
- note bodies
- checklist text
- labels/tags
- attachment names belonging to notes
- note snippets
- search index/content cache
- private board order if it exposes note identity/content relationships
- other user-authored private note metadata

If a setting later becomes privacy-sensitive, move it into Vault-backed Notes data.

## 7. Search model

Because the current Vault API does not provide list/search operations over documents, Notes should build its searchable in-memory model after Vault access by loading its own known note records.

Do not maintain a plaintext disk search index.

The in-memory search index may contain decrypted data only while Notes/Vault access is active and should be discarded when the plugin session ends or the application/Vault lifecycle makes it invalid.

The exact efficient enumeration strategy must be implemented by Notes-owned encrypted index/catalog data because `vault.document.*` addresses documents by key and does not expose a plugin namespace listing API.

A practical design is an encrypted catalog document such as:

`meta/catalog`

containing note IDs only or the minimum encrypted metadata needed to enumerate Notes records.

## 8. Atomicity model for note saves

A single `vault.document.set` replaces one Notes document atomically at the Vault persistence layer as part of the host's encrypted Vault save.

However, a logical operation spanning multiple Notes documents is not currently a host transaction.

Therefore Notes should minimize multi-document invariants.

Preferred pattern:

1. Save the note document first.
2. Update board/catalog documents second.
3. On load, reconcile harmless inconsistencies where possible.
4. Never delete attachment payloads before the note no longer references them.
5. Treat cleanup of unreferenced attachment blobs as recoverable garbage collection rather than risking data loss.

Do not claim cross-document transactional atomicity unless RemoteHand later provides it.

## 9. Existing technical limits relevant to Notes

RemoteHand v1.3.0 currently has these relevant bounds:

- Plugin IPC frame maximum: **16 MiB**
- Vault document value maximum: **16 MiB**
- Encrypted Vault file maximum: **128 MiB**
- Plugin-scoped plaintext storage entry maximum: **16 MiB**

These are host technical constraints, not Notes product-policy limits.

They are compatible with normal structured note documents but are not suitable for the agreed attachment model where users control attachment limits.

## 10. Attachment gap in RemoteHand v1.3.0

The agreed Notes design requires:

- arbitrary files and images;
- user-configurable attachment size/count defaults;
- no arbitrary Notes-level hard ceiling;
- attachment bytes protected by RemoteHand Vault;
- separate attachment blobs rather than inline Markdown or giant note documents.

The current `vault.document.*` API is **not** the correct transport/storage mechanism for this.

Reasons:

1. Document values are capped at 16 MiB.
2. IPC itself has a 16 MiB frame bound, so base64 encoding a near-limit binary payload is already unsuitable.
3. Base64 adds avoidable expansion and memory copies.
4. The current whole-Vault file is capped at 128 MiB.
5. Re-encrypting/replacing the whole Vault container for every large attachment operation would scale poorly.
6. User-configurable attachment limits must not be silently reduced to the current document limit.

Therefore attachment implementation is **blocked on a generic core Vault blob capability**.

Do not work around this by:

- storing plaintext attachment sidecars;
- placing attachment bytes in `storage.plugin`;
- inventing Notes-owned encryption;
- asking core for a raw Vault key;
- base64-packing large attachments into ordinary Vault documents.

## 11. Proposed generic Vault blob capability

This should be a RemoteHand core feature reusable by Notes and future plugins.

Conceptual permissions:

- `vault.blobs.read`
- `vault.blobs.write`

Conceptual operations:

- begin blob write
- stream/chunk blob data
- commit blob write
- abort blob write
- begin blob read
- stream/chunk blob data
- delete blob
- optionally query blob metadata

Exact method names are not locked by this Notes spec; they should be designed in RemoteHand core.

### Required properties

A future blob API should:

- derive namespace from authenticated plugin identity;
- never expose master/session encryption keys;
- encrypt attachment payloads in core;
- support payloads larger than one IPC frame by chunking/streaming;
- avoid loading an entire large attachment into memory when unnecessary;
- use bounded chunks;
- authenticate blob metadata and bytes;
- use atomic commit semantics so interrupted writes do not replace a valid existing blob;
- clean up abandoned temporary uploads safely;
- support deterministic delete;
- keep logs free of attachment content;
- allow the user-facing Notes settings to choose operational limits without Notes inventing arbitrary hard caps;
- impose only genuine implementation/format/resource limits required by core;
- support future blob-format versioning.

### Physical storage direction

Large blobs should not live inside the current monolithic `vault.enc` document.

A likely generic direction is separately encrypted core-owned blob files under Vault-managed storage, with authenticated headers and caller-scoped identifiers.

The exact file format, key derivation/domain separation, backup behavior, and recovery semantics belong in RemoteHand core design/security review and must not be invented ad hoc by Notes.

## 12. Attachment reference lifecycle

Once the generic blob API exists:

1. Notes creates a stable attachment ID.
2. Notes writes the attachment through the core blob API.
3. Core returns/confirms the caller-relative blob key.
4. Notes writes the attachment reference and private metadata into the owning note document.
5. On attachment removal, Notes first saves the note without that reference, then deletes the blob.
6. Permanent note deletion deletes its referenced blobs after the note is removed from active private metadata.
7. Startup/maintenance may garbage-collect unreferenced Notes blobs after a conservative safety policy.

This ordering prefers temporary orphaned encrypted blobs over dangling note references to missing data.

## 13. Lock-state behavior

The Notes note-level `locked` property is unrelated to Vault cryptographic lock state.

### Vault locked

- no decrypted Notes content may be shown;
- Notes requests host-owned Vault access;
- title/body/labels/previews/attachment names remain unavailable until Vault access succeeds.

### Note locked

- content remains visible because Vault is already available;
- note is read-only/protected from destructive changes;
- restore from Archive/Trash remains allowed;
- Trash emptying skips locked items.

Do not conflate these two meanings of "locked" in code or UI.

## 14. Error handling

Notes should distinguish at least:

- Vault unavailable/locked
- user cancelled Vault access
- Vault access timed out
- document missing
- invalid/corrupt Notes document
- Notes schema version unsupported
- attachment/blob operation failure
- ordinary plugin/storage failure

Do not automatically replace invalid private data with an empty note and overwrite the original.

## 15. First implementation boundary

Notes implementation may proceed for all **structured note functionality** against the existing v1.3.0 API:

- host-owned Vault access
- encrypted catalog
- encrypted note/checklist documents
- board ordering
- labels
- Markdown
- pin/archive/trash/lock state
- autosave
- search after unlock
- non-private plugin Settings

Attachment UI/model can be scaffolded, but persistence of actual file/image bytes should wait for the generic core Vault blob capability.

## 16. Core work triggered by Notes

Before full Notes v0.1.0 acceptance, RemoteHand needs one generic extension:

**Vault encrypted blob storage/streaming for plugin-owned binary data.**

This is a legitimate core capability surfaced by a real plugin consumer, not a Notes-specific exception.

Because RemoteHand `v1.3.0` is already tagged and its tag must not move, any implemented core change must occur after that commit/tag under the normal RemoteHand version/release workflow.

The currently prepared v1.3.0 GitHub Release remains intentionally unpublished.


## 17. Vault status host event required by Notes

During implementation, Notes exposed one additional generic host-integration requirement beyond encrypted blobs:

- plugin panels need immediate notification when RemoteHand Vault locks or unlocks.

RemoteHand now has a focused integration branch:

`emma/notes-vault-status-event`

That branch forwards the existing core `remotehand-vault-status-changed` event into plugin panels as:

~~~javascript
window.remotehand.onEvent("vault.status", (status) => {
  // status.exists
  // status.unlocked
});
~~~

Notes uses this notification to immediately clear decrypted UI state when `status.unlocked` becomes false. The redaction path clears pending autosave state, note arrays, board DOM, open editor state, search text, and visible private fields without attempting a Vault write after lock.

This is a generic RemoteHand plugin capability and is not Notes-specific policy.

The existing RemoteHand `v1.3.0` tag remains unchanged. This host-event addition lives after that tagged checkpoint and must follow normal RemoteHand validation before merge/release.


### Re-entrant unlock request rule

The first manual acceptance pass exposed an important IPC rule for Notes.

When a Notes call such as `notes.load` invokes `vault.request_access`, the plugin runtime is synchronously waiting for the corresponding host response. RemoteHand may emit `vault.status { unlocked: true }` as part of that same successful unlock.

Notes must **not** start a second `notes.load` merely because that unlocked notification arrived. Doing so can place a new `host.request` on the framed plugin stream while the plugin is waiting for the previous `host.response`, desynchronizing the synchronous Plugin API v1 request flow.

Accepted behavior:

- react immediately to `unlocked: false` by redacting decrypted UI;
- do not issue a new host request solely from `unlocked: true`;
- let the original `vault.request_access` caller resume and finish loading;
- guard Notes loading against concurrent duplicate calls.

The first acceptance failure presented as `failed to read plugin frame length: failed to fill whole buffer` after the plugin process exited from the desynchronized request flow.

### Sandboxed confirmation rule

RemoteHand plugin panels intentionally run without browser modal-dialog permission. Notes therefore must not rely on native `alert()`, `confirm()`, or `prompt()` for product flows.

Destructive confirmations such as permanent delete and Empty Trash use Notes-owned modal UI inside the panel.


## 18. Blob implementation checkpoint

The generic blob capability described above is now implemented for validation on RemoteHand branch `emma/vault-blob-api`.

Current host surface:

- `vault.blobs.read`
- `vault.blobs.write`
- `vault.blob.begin`
- `vault.blob.write`
- `vault.blob.commit`
- `vault.blob.abort`
- `vault.blob.info`
- `vault.blob.read`
- `vault.blob.delete`

Core uses 1 MiB authenticated chunks, a 4 GiB implementation bound per blob, opaque keyed filenames, domain-separated per-blob encryption keys, and XChaCha20-Poly1305. Blob creation is staged and immutable: a committed key must be deleted before reuse.

Notes attachment integration now includes picker upload, native file-drop path upload, metadata references in note documents, attachment reordering, inline image previews up to 8 MiB, open-with-system-app, and blob cleanup after attachment/note deletion.

Opening a general attachment for an external system application necessarily materializes a plaintext copy in the operating-system temp directory. Notes gives the file a randomized temporary path, schedules cleanup retries, and clears stale Notes temp files on the next plugin start. This temporary external-view copy is not used as persistent Notes storage.

The image compression/resize setting remains reserved until its exact transformation policy is finalized. The current attachment path preserves original file bytes.
