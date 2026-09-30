# RemoteHand Notes

Private, Vault-backed sticky-note and checklist plugin for RemoteHand.

## Status

**Design/specification phase. No implementation yet.**

The initial target is **v0.1.0**.

RemoteHand Notes is designed as a lightweight sticky-note board rather than a document editor. Notes and checklists are private by design and use RemoteHand Vault for encrypted storage.

See [docs/V0.1.0-SPEC.md](docs/V0.1.0-SPEC.md) for the approved v0.1.0 design.

## Core direction

- Sticky-note card board
- Plain notes and checklist notes
- Lightweight Markdown
- Vault-encrypted note content and private metadata
- Labels/tags rather than folders
- Pinning, colors, search, archive, Trash, and note locking
- File and image attachments
- Autosave
- Manual card rearranging plus automatic sort modes

## Security boundary

RemoteHand core owns Vault cryptography, unlock state, and key handling. Notes must use the scoped Vault API and must not implement a separate encryption system or receive the Vault passphrase/master key.

## Repository

This repository intentionally begins with documentation only. Implementation starts after the v0.1.0 storage/API contract is reviewed against the current RemoteHand Vault capabilities.
