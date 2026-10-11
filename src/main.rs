use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const PLUGIN_ID: &str = "remotehand.notes";
const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");
const API_VERSION: u32 = 1;
const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;
const MAX_NOTES_DOCUMENT_BYTES: usize = 15 * 1024 * 1024;
const CATALOG_KEY: &str = "meta/catalog";
const SETTINGS_KEY: &str = "settings-v1";
const REQUIRED_PERMISSIONS: &[&str] = &[
    "vault.documents.read",
    "vault.documents.write",
    "vault.blobs.read",
    "vault.blobs.write",
    "vault.access.request",
    "storage.plugin",
    "files.open",
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PluginHello<'a> {
    #[serde(rename = "type")]
    message_type: &'static str,
    plugin_id: &'a str,
    api_version: u32,
    plugin_version: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostReady {
    #[serde(rename = "type")]
    message_type: String,
    plugin_id: String,
    api_version: u32,
    #[serde(default)]
    granted_permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostRequest {
    #[serde(rename = "type")]
    message_type: String,
    request_id: u64,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PluginResponse {
    #[serde(rename = "type")]
    message_type: &'static str,
    request_id: u64,
    ok: bool,
    result: Value,
    error: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PluginHostRequest<'a> {
    #[serde(rename = "type")]
    message_type: &'static str,
    request_id: u64,
    method: &'a str,
    params: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostResponse {
    #[serde(rename = "type")]
    message_type: String,
    request_id: u64,
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    result: Value,
    #[serde(default)]
    error: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveNoteParams {
    note: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogSetParams {
    catalog: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteNoteParams {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsSetParams {
    settings: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttachmentAddParams {
    id: String,
    existing_total_bytes: u64,
    existing_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttachmentBlobParams {
    blob_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttachmentAddPathParams {
    id: String,
    path: String,
    existing_total_bytes: u64,
    existing_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttachmentOpenParams {
    blob_key: String,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttachmentAddBytesParams {
    id: String,
    name: String,
    media_type: String,
    data: String,
    existing_total_bytes: u64,
    existing_count: u64,
}

fn write_frame<W: Write, T: Serialize>(writer: &mut W, value: &T) -> Result<(), String> {
    let payload = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    if payload.is_empty() || payload.len() > MAX_FRAME_BYTES {
        return Err("frame exceeds Notes Plugin API limit".to_string());
    }
    writer
        .write_all(&(payload.len() as u32).to_le_bytes())
        .map_err(|error| error.to_string())?;
    writer.write_all(&payload).map_err(|error| error.to_string())?;
    writer.flush().map_err(|error| error.to_string())
}

fn read_frame<R: Read>(reader: &mut R) -> Result<Option<Vec<u8>>, String> {
    let mut length = [0u8; 4];
    match reader.read_exact(&mut length) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error.to_string()),
    }

    let length = u32::from_le_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(format!("invalid frame length: {length}"));
    }

    let mut payload = vec![0u8; length];
    reader
        .read_exact(&mut payload)
        .map_err(|error| error.to_string())?;
    Ok(Some(payload))
}

fn call_host<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    next_request_id: &mut u64,
    method: &str,
    params: Value,
) -> Result<Value, String> {
    *next_request_id = next_request_id.wrapping_add(1).max(1);
    let request_id = *next_request_id;

    write_frame(
        writer,
        &PluginHostRequest {
            message_type: "plugin.request",
            request_id,
            method,
            params,
        },
    )?;

    let payload = read_frame(reader)?
        .ok_or_else(|| "host closed stream while awaiting host.response".to_string())?;
    let response: HostResponse = serde_json::from_slice(&payload)
        .map_err(|error| format!("invalid host.response: {error}"))?;

    if response.message_type != "host.response" {
        return Err(format!(
            "unexpected Plugin API v1 message type while awaiting host.response: {}",
            response.message_type
        ));
    }
    if response.request_id != request_id {
        return Err(format!(
            "host.response requestId mismatch: expected {request_id}, got {}",
            response.request_id
        ));
    }

    if response.ok {
        Ok(response.result)
    } else if response.error.trim().is_empty() {
        Err(format!("host API method {method} failed"))
    } else {
        Err(response.error)
    }
}

fn validate_note_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 128 {
        return Err("note id is invalid".to_string());
    }
    if !id
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("note id contains unsupported characters".to_string());
    }
    Ok(())
}

fn note_id(note: &Value) -> Result<&str, String> {
    let id = note
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "note id is required".to_string())?;
    validate_note_id(id)?;
    Ok(id)
}

fn note_key(id: &str) -> String {
    format!("note/{id}")
}

fn ensure_document_size(value: &Value, label: &str) -> Result<String, String> {
    let serialized = serde_json::to_string(value).map_err(|error| error.to_string())?;
    if serialized.len() > MAX_NOTES_DOCUMENT_BYTES {
        return Err(format!("{label} exceeds the current RemoteHand Vault document limit"));
    }
    Ok(serialized)
}

fn empty_catalog() -> Value {
    json!({
        "version": 1,
        "noteIds": [],
        "manualOrder": {
            "main": [],
            "archive": [],
            "trash": []
        },
        "pinnedOrder": []
    })
}

fn normalize_catalog(mut catalog: Value) -> Value {
    if !catalog.is_object() {
        return empty_catalog();
    }
    let object = catalog.as_object_mut().expect("checked object");
    object.insert("version".to_string(), json!(1));
    if !object.get("noteIds").is_some_and(Value::is_array) {
        object.insert("noteIds".to_string(), json!([]));
    }
    if !object.get("manualOrder").is_some_and(Value::is_object) {
        object.insert(
            "manualOrder".to_string(),
            json!({"main": [], "archive": [], "trash": []}),
        );
    }
    if !object.get("pinnedOrder").is_some_and(Value::is_array) {
        object.insert("pinnedOrder".to_string(), json!([]));
    }
    catalog
}

fn request_vault_access(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
) -> Result<(), String> {
    host_call(
        "vault.request_access",
        json!({ "reason": "Open your private Notes." }),
    )?;
    Ok(())
}

fn get_document(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
    key: &str,
) -> Result<Option<String>, String> {
    let result = host_call("vault.document.get", json!({ "key": key }))?;
    match result.get("value") {
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(Value::Null) | None => Ok(None),
        _ => Err("host returned an invalid Vault document response".to_string()),
    }
}

fn set_document(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
    key: &str,
    value: String,
) -> Result<(), String> {
    host_call(
        "vault.document.set",
        json!({ "key": key, "value": value }),
    )?;
    Ok(())
}

fn delete_document(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
    key: &str,
) -> Result<bool, String> {
    let result = host_call("vault.document.delete", json!({ "key": key }))?;
    Ok(result
        .get("deleted")
        .and_then(Value::as_bool)
        .unwrap_or(false))
}

fn load_catalog(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
) -> Result<Value, String> {
    let Some(raw) = get_document(host_call, CATALOG_KEY)? else {
        return Ok(empty_catalog());
    };
    let parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("Notes catalog is invalid: {error}"))?;
    Ok(normalize_catalog(parsed))
}

fn save_catalog(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
    catalog: &Value,
) -> Result<(), String> {
    let normalized = normalize_catalog(catalog.clone());
    let serialized = ensure_document_size(&normalized, "Notes catalog")?;
    set_document(host_call, CATALOG_KEY, serialized)
}

fn catalog_note_ids(catalog: &Value) -> Vec<String> {
    catalog
        .get("noteIds")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .filter(|id| validate_note_id(id).is_ok())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn set_catalog_note_ids(catalog: &mut Value, ids: Vec<String>) {
    if let Some(object) = catalog.as_object_mut() {
        object.insert("noteIds".to_string(), json!(ids));
    }
}

fn remove_id_from_array(value: &mut Value, id: &str) {
    if let Some(array) = value.as_array_mut() {
        array.retain(|item| item.as_str() != Some(id));
    }
}

fn remove_id_from_catalog(catalog: &mut Value, id: &str) {
    if let Some(object) = catalog.as_object_mut() {
        if let Some(note_ids) = object.get_mut("noteIds") {
            remove_id_from_array(note_ids, id);
        }
        if let Some(pinned) = object.get_mut("pinnedOrder") {
            remove_id_from_array(pinned, id);
        }
        if let Some(orders) = object.get_mut("manualOrder").and_then(Value::as_object_mut) {
            for order in orders.values_mut() {
                remove_id_from_array(order, id);
            }
        }
    }
}

fn is_locked(note: &Value) -> bool {
    note.get("locked")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn location(note: &Value) -> &str {
    note.get("location")
        .and_then(Value::as_str)
        .unwrap_or("main")
}

fn comparable_note(note: &Value) -> Value {
    let mut clone = note.clone();
    if let Some(object) = clone.as_object_mut() {
        object.remove("locked");
        object.remove("location");
        object.remove("updatedAt");
    }
    clone
}

fn locked_transition_allowed(previous: &Value, next: &Value) -> bool {
    if !is_locked(previous) {
        return true;
    }
    if comparable_note(previous) != comparable_note(next) {
        return false;
    }

    let previous_location = location(previous);
    let next_location = location(next);
    let safe_location = previous_location == next_location
        || (matches!(previous_location, "archive" | "trash") && next_location == "main");
    if !safe_location {
        return false;
    }

    let next_locked = next
        .get("locked")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    next_locked || !next_locked
}

fn parse_note_document(raw: &str) -> Result<Value, String> {
    let note: Value =
        serde_json::from_str(raw).map_err(|error| format!("invalid note document: {error}"))?;
    note_id(&note)?;
    Ok(note)
}

fn load_note(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
    id: &str,
) -> Result<Option<Value>, String> {
    validate_note_id(id)?;
    let Some(raw) = get_document(host_call, &note_key(id))? else {
        return Ok(None);
    };
    Ok(Some(parse_note_document(&raw)?))
}

fn save_note(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
    note: Value,
) -> Result<Value, String> {
    request_vault_access(host_call)?;
    let id = note_id(&note)?.to_string();

    if let Some(previous) = load_note(host_call, &id)? {
        if !locked_transition_allowed(&previous, &note) {
            return Err("locked note cannot be modified".to_string());
        }
    }

    let serialized = ensure_document_size(&note, "Note")?;
    set_document(host_call, &note_key(&id), serialized)?;

    let mut catalog = load_catalog(host_call)?;
    let mut ids = catalog_note_ids(&catalog);
    if !ids.iter().any(|existing| existing == &id) {
        ids.push(id.clone());
        set_catalog_note_ids(&mut catalog, ids);
        save_catalog(host_call, &catalog)?;
    }

    Ok(json!({ "saved": true, "id": id }))
}

fn load_all_notes(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
) -> Result<Value, String> {
    request_vault_access(host_call)?;
    let catalog = load_catalog(host_call)?;
    let ids = catalog_note_ids(&catalog);
    let mut notes = Vec::new();
    let mut warnings = Vec::new();

    for id in ids {
        match load_note(host_call, &id) {
            Ok(Some(note)) => notes.push(note),
            Ok(None) => warnings.push(format!("Missing note document: {id}")),
            Err(error) => warnings.push(format!("{id}: {error}")),
        }
    }

    let settings = get_settings(host_call)?;
    Ok(json!({
        "catalog": catalog,
        "notes": notes,
        "settings": settings,
        "warnings": warnings
    }))
}

fn get_settings(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
) -> Result<Value, String> {
    let result = host_call("storage.get", json!({ "key": SETTINGS_KEY }))?;
    Ok(result.get("value").cloned().unwrap_or(Value::Null))
}

fn default_settings() -> Value {
    json!({
        "autosaveDelayMs": 650,
        "maxFileMiB": 10,
        "maxImageMiB": 8,
        "maxTotalAttachmentMiB": 50,
        "maxAttachments": 10,
        "imageCompression": true,
        "imageQuality": 85,
        "defaultSort": "updated",
        "cardDensity": "comfortable",
        "hideCompletedByDefault": false,
        "confirmPermanentDelete": true,
        "confirmEmptyTrash": true
    })
}

fn sanitize_settings(value: Value) -> Value {
    let defaults = default_settings();
    let mut merged = defaults.as_object().cloned().unwrap_or_default();
    if let Some(input) = value.as_object() {
        for (key, value) in input {
            if merged.contains_key(key) {
                merged.insert(key.clone(), value.clone());
            }
        }
    }
    Value::Object(merged)
}

fn set_settings(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
    settings: Value,
) -> Result<Value, String> {
    let settings = sanitize_settings(settings);
    host_call(
        "storage.set",
        json!({ "key": SETTINGS_KEY, "value": settings }),
    )?;
    Ok(json!({ "stored": true, "settings": settings }))
}


fn attachment_blob_keys(note: &Value) -> Vec<String> {
    note.get("attachments")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("blobKey").and_then(Value::as_str))
                .filter(|key| key.starts_with("attachment/"))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn settings_u64(settings: &Value, key: &str, default: u64) -> u64 {
    settings.get(key).and_then(Value::as_u64).unwrap_or(default)
}

fn attachment_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "md" | "markdown" => "text/markdown",
        "json" => "application/json",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

fn is_image_mime(value: &str) -> bool {
    value.starts_with("image/")
}

fn limit_bytes(mib: u64) -> Option<u64> {
    if mib == 0 {
        None
    } else {
        mib.checked_mul(1024 * 1024)
    }
}

fn validate_attachment_limits(
    settings: &Value,
    size: u64,
    mime_type: &str,
    existing_total_bytes: u64,
    existing_count: u64,
) -> Result<(), String> {
    let max_count = settings_u64(settings, "maxAttachments", 10);
    if max_count != 0 && existing_count >= max_count {
        return Err(format!("note already has the configured maximum of {max_count} attachments"));
    }

    let per_file_mib = if is_image_mime(mime_type) {
        settings_u64(settings, "maxImageMiB", 8)
    } else {
        settings_u64(settings, "maxFileMiB", 10)
    };
    if let Some(limit) = limit_bytes(per_file_mib) {
        if size > limit {
            return Err(format!(
                "attachment exceeds the configured {} MiB {} limit",
                per_file_mib,
                if is_image_mime(mime_type) { "image" } else { "file" }
            ));
        }
    }

    let total_mib = settings_u64(settings, "maxTotalAttachmentMiB", 50);
    if let Some(limit) = limit_bytes(total_mib) {
        let total = existing_total_bytes
            .checked_add(size)
            .ok_or_else(|| "attachment total size overflowed".to_string())?;
        if total > limit {
            return Err(format!(
                "attachments exceed the configured {total_mib} MiB per-note total"
            ));
        }
    }
    Ok(())
}

fn upload_attachment_from_path(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
    id: String,
    path: PathBuf,
    existing_total_bytes: u64,
    existing_count: u64,
) -> Result<Value, String> {
    validate_note_id(&id)?;
    request_vault_access(host_call)?;

    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| format!("cannot inspect attachment: {error}"))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err("attachment must be a regular file".to_string());
    }

    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "attachment filename is invalid".to_string())?
        .to_string();
    let size = metadata.len();
    let mime_type = attachment_mime_type(&path).to_string();
    let settings = sanitize_settings(get_settings(host_call)?);
    validate_attachment_limits(
        &settings,
        size,
        &mime_type,
        existing_total_bytes,
        existing_count,
    )?;

    let blob_key = format!("attachment/{id}");
    let begin = host_call(
        "vault.blob.begin",
        json!({ "key": blob_key, "size": size }),
    )?;
    let upload_id = begin
        .get("uploadId")
        .and_then(Value::as_str)
        .ok_or_else(|| "host did not return a Vault blob upload id".to_string())?
        .to_string();
    let chunk_size = begin
        .get("chunkSize")
        .and_then(Value::as_u64)
        .ok_or_else(|| "host did not return a Vault blob chunk size".to_string())?
        as usize;
    if chunk_size == 0 || chunk_size > 4 * 1024 * 1024 {
        return Err("host returned an invalid Vault blob chunk size".to_string());
    }

    let upload_result = (|| -> Result<(), String> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|error| format!("cannot read attachment: {error}"))?;
        let mut index = 0u32;
        let mut remaining = size;
        while remaining > 0 {
            let length = remaining.min(chunk_size as u64) as usize;
            let mut bytes = vec![0u8; length];
            file.read_exact(&mut bytes)
                .map_err(|error| format!("cannot read attachment: {error}"))?;
            let data = STANDARD_NO_PAD.encode(bytes);
            host_call(
                "vault.blob.write",
                json!({
                    "key": blob_key,
                    "uploadId": upload_id,
                    "index": index,
                    "data": data
                }),
            )?;
            remaining -= length as u64;
            index += 1;
        }
        host_call(
            "vault.blob.commit",
            json!({ "key": blob_key, "uploadId": upload_id }),
        )?;
        Ok(())
    })();

    if let Err(error) = upload_result {
        let _ = host_call(
            "vault.blob.abort",
            json!({ "key": blob_key, "uploadId": upload_id }),
        );
        return Err(error);
    }

    Ok(json!({
        "opened": true,
        "attachment": {
            "id": id,
            "name": name,
            "mediaType": mime_type,
            "size": size,
            "blobKey": blob_key
        }
    }))
}

fn upload_attachment(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
    params: AttachmentAddParams,
) -> Result<Value, String> {
    let opened = host_call(
        "file.open",
        json!({ "filters": [{ "name": "Files", "extensions": ["*"] }] }),
    )?;
    if opened.get("opened") == Some(&Value::Bool(false)) {
        return Ok(json!({ "opened": false }));
    }
    let path = opened
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| "host file.open did not return a path".to_string())?;
    upload_attachment_from_path(
        host_call,
        params.id,
        PathBuf::from(path),
        params.existing_total_bytes,
        params.existing_count,
    )
}

fn upload_attachment_bytes(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
    params: AttachmentAddBytesParams,
) -> Result<Value, String> {
    validate_note_id(&params.id)?;
    if params.name.trim().is_empty() || params.name.len() > 255 {
        return Err("attachment name is invalid".to_string());
    }
    if params.media_type.trim().is_empty() || params.media_type.len() > 128 {
        return Err("attachment media type is invalid".to_string());
    }
    if params.data.len() > 12 * 1024 * 1024 {
        return Err("clipboard attachment exceeds the inline transfer limit".to_string());
    }
    let bytes = STANDARD_NO_PAD
        .decode(params.data)
        .map_err(|_| "clipboard attachment encoding is invalid".to_string())?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(
            "clipboard attachment exceeds the 8 MiB inline transfer limit; save it as a file and attach it instead"
                .to_string(),
        );
    }

    request_vault_access(host_call)?;
    let settings = sanitize_settings(get_settings(host_call)?);
    validate_attachment_limits(
        &settings,
        bytes.len() as u64,
        &params.media_type,
        params.existing_total_bytes,
        params.existing_count,
    )?;

    let blob_key = format!("attachment/{}", params.id);
    let begin = host_call(
        "vault.blob.begin",
        json!({ "key": blob_key, "size": bytes.len() as u64 }),
    )?;
    let upload_id = begin
        .get("uploadId")
        .and_then(Value::as_str)
        .ok_or_else(|| "host did not return a Vault blob upload id".to_string())?
        .to_string();
    let chunk_size = begin
        .get("chunkSize")
        .and_then(Value::as_u64)
        .ok_or_else(|| "host did not return a Vault blob chunk size".to_string())?
        as usize;
    if chunk_size == 0 || chunk_size > 4 * 1024 * 1024 {
        return Err("host returned an invalid Vault blob chunk size".to_string());
    }

    let upload_result = (|| -> Result<(), String> {
        for (index, chunk) in bytes.chunks(chunk_size).enumerate() {
            host_call(
                "vault.blob.write",
                json!({
                    "key": blob_key,
                    "uploadId": upload_id,
                    "index": index as u32,
                    "data": STANDARD_NO_PAD.encode(chunk)
                }),
            )?;
        }
        host_call(
            "vault.blob.commit",
            json!({ "key": blob_key, "uploadId": upload_id }),
        )?;
        Ok(())
    })();

    if let Err(error) = upload_result {
        let _ = host_call(
            "vault.blob.abort",
            json!({ "key": blob_key, "uploadId": upload_id }),
        );
        return Err(error);
    }

    Ok(json!({
        "attachment": {
            "id": params.id,
            "name": params.name,
            "mediaType": params.media_type,
            "size": bytes.len() as u64,
            "blobKey": blob_key
        }
    }))
}

fn delete_attachment_blob(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
    blob_key: &str,
) -> Result<bool, String> {
    if !blob_key.starts_with("attachment/") {
        return Err("attachment blob key is invalid".to_string());
    }
    request_vault_access(host_call)?;
    let result = host_call("vault.blob.delete", json!({ "key": blob_key }))?;
    Ok(result
        .get("deleted")
        .and_then(Value::as_bool)
        .unwrap_or(false))
}

fn safe_temp_name(name: &str) -> String {
    let base = Path::new(name)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("attachment");
    let cleaned: String = base
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | ' ') {
                ch
            } else {
                '_'
            }
        })
        .take(160)
        .collect();
    if cleaned.trim().is_empty() {
        "attachment".to_string()
    } else {
        cleaned
    }
}

fn open_system_file(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("rundll32.exe");
        command.arg("url.dll,FileProtocolHandler").arg(path);
        command
    };
    #[cfg(target_os = "linux")]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(path);
        command
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(path);
        command
    };
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    return Err("opening attachments is unsupported on this platform".to_string());

    command.spawn().map_err(|error| error.to_string())?;
    Ok(())
}

fn open_attachment(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
    params: AttachmentOpenParams,
) -> Result<Value, String> {
    if !params.blob_key.starts_with("attachment/") {
        return Err("attachment blob key is invalid".to_string());
    }
    request_vault_access(host_call)?;
    let info = host_call("vault.blob.info", json!({ "key": params.blob_key }))?;
    if info.get("exists") != Some(&Value::Bool(true)) {
        return Err("attachment blob was not found".to_string());
    }
    let chunks = info
        .get("chunks")
        .and_then(Value::as_u64)
        .ok_or_else(|| "host returned invalid Vault blob info".to_string())?
        as u32;

    let temp_root = std::env::temp_dir().join("RemoteHand").join("Notes");
    fs::create_dir_all(&temp_root).map_err(|error| error.to_string())?;
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp_path = temp_root.join(format!(
        "{}-{}-{}",
        std::process::id(),
        unique,
        safe_temp_name(&params.name)
    ));

    let result = (|| -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp_path)
            .map_err(|error| error.to_string())?;
        for index in 0..chunks {
            let response = host_call(
                "vault.blob.read",
                json!({ "key": params.blob_key, "index": index }),
            )?;
            let data = response
                .get("data")
                .and_then(Value::as_str)
                .ok_or_else(|| "host returned invalid Vault blob chunk data".to_string())?;
            let bytes = STANDARD_NO_PAD
                .decode(data)
                .map_err(|_| "host returned invalid Vault blob chunk encoding".to_string())?;
            file.write_all(&bytes).map_err(|error| error.to_string())?;
        }
        file.flush().map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        open_system_file(&temp_path)?;
        Ok(())
    })();

    if let Err(error) = result {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }

    let cleanup = temp_path.clone();
    thread::spawn(move || {
        for _ in 0..30 {
            thread::sleep(Duration::from_secs(60));
            match fs::remove_file(&cleanup) {
                Ok(()) => return,
                Err(error) if error.kind() == ErrorKind::NotFound => return,
                Err(_) => {}
            }
        }
    });

    Ok(json!({ "opened": true }))
}

fn permanent_delete(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
    id: &str,
) -> Result<Value, String> {
    request_vault_access(host_call)?;
    validate_note_id(id)?;
    let note = load_note(host_call, id)?;
    if let Some(note) = note.as_ref() {
        if is_locked(note) {
            return Err("locked note cannot be permanently deleted".to_string());
        }
        if location(note) != "trash" {
            return Err("permanent delete is only allowed from Trash".to_string());
        }
    }
    let blob_keys = note.as_ref().map(attachment_blob_keys).unwrap_or_default();

    let deleted = delete_document(host_call, &note_key(id))?;
    let mut catalog = load_catalog(host_call)?;
    remove_id_from_catalog(&mut catalog, id);
    save_catalog(host_call, &catalog)?;

    let mut orphaned_attachments = 0usize;
    for blob_key in blob_keys {
        if delete_attachment_blob(host_call, &blob_key).is_err() {
            orphaned_attachments += 1;
        }
    }

    Ok(json!({
        "deleted": deleted,
        "id": id,
        "orphanedAttachments": orphaned_attachments
    }))
}

fn empty_trash(
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
) -> Result<Value, String> {
    request_vault_access(host_call)?;
    let mut catalog = load_catalog(host_call)?;
    let ids = catalog_note_ids(&catalog);
    let mut deleted = 0usize;
    let mut kept_locked = 0usize;
    let mut kept_other = 0usize;

    for id in ids {
        let Some(note) = load_note(host_call, &id)? else {
            remove_id_from_catalog(&mut catalog, &id);
            continue;
        };
        if location(&note) != "trash" {
            continue;
        }
        if is_locked(&note) {
            kept_locked += 1;
            continue;
        }
        let blob_keys = attachment_blob_keys(&note);
        if delete_document(host_call, &note_key(&id))? {
            deleted += 1;
        }
        remove_id_from_catalog(&mut catalog, &id);
        for blob_key in blob_keys {
            let _ = delete_attachment_blob(host_call, &blob_key);
        }
    }

    if let Some(ids) = catalog.get("noteIds").and_then(Value::as_array) {
        kept_other = ids.len();
    }
    save_catalog(host_call, &catalog)?;

    Ok(json!({
        "deleted": deleted,
        "keptLocked": kept_locked,
        "remainingNotes": kept_other
    }))
}

fn validate_catalog(value: &Value) -> Result<(), String> {
    if !value.is_object() {
        return Err("catalog must be an object".to_string());
    }
    let ids = catalog_note_ids(value);
    let unique: BTreeSet<_> = ids.iter().collect();
    if unique.len() != ids.len() {
        return Err("catalog contains duplicate note ids".to_string());
    }
    ensure_document_size(value, "Notes catalog")?;
    Ok(())
}

fn dispatch(
    request: &HostRequest,
    host_call: &mut dyn FnMut(&str, Value) -> Result<Value, String>,
) -> Result<Value, String> {
    match request.method.as_str() {
        "runtime.ping" => Ok(json!({
            "pluginId": PLUGIN_ID,
            "pluginVersion": PLUGIN_VERSION,
            "apiVersion": API_VERSION,
            "status": "ok"
        })),
        "notes.load" => {
            let mut result = load_all_notes(host_call)?;
            if result.get("settings").is_some_and(Value::is_null) {
                if let Some(object) = result.as_object_mut() {
                    object.insert("settings".to_string(), default_settings());
                }
            } else if let Some(settings) = result.get("settings").cloned() {
                if let Some(object) = result.as_object_mut() {
                    object.insert("settings".to_string(), sanitize_settings(settings));
                }
            }
            Ok(result)
        }
        "notes.request_access" => {
            request_vault_access(host_call)?;
            Ok(json!({ "unlocked": true }))
        }
        "notes.save" => {
            let params: SaveNoteParams = serde_json::from_value(request.params.clone())
                .map_err(|error| format!("invalid notes.save request: {error}"))?;
            save_note(host_call, params.note)
        }
        "notes.catalog.set" => {
            request_vault_access(host_call)?;
            let params: CatalogSetParams = serde_json::from_value(request.params.clone())
                .map_err(|error| format!("invalid notes.catalog.set request: {error}"))?;
            validate_catalog(&params.catalog)?;
            save_catalog(host_call, &params.catalog)?;
            Ok(json!({ "stored": true }))
        }
        "notes.delete" => {
            let params: DeleteNoteParams = serde_json::from_value(request.params.clone())
                .map_err(|error| format!("invalid notes.delete request: {error}"))?;
            permanent_delete(host_call, &params.id)
        }
        "notes.empty_trash" => empty_trash(host_call),
        "attachment.add" => {
            let params: AttachmentAddParams = serde_json::from_value(request.params.clone())
                .map_err(|error| format!("invalid attachment.add request: {error}"))?;
            upload_attachment(host_call, params)
        }
        "attachment.add_path" => {
            let params: AttachmentAddPathParams = serde_json::from_value(request.params.clone())
                .map_err(|error| format!("invalid attachment.add_path request: {error}"))?;
            upload_attachment_from_path(
                host_call,
                params.id,
                PathBuf::from(params.path),
                params.existing_total_bytes,
                params.existing_count,
            )
        }
        "attachment.delete" => {
            let params: AttachmentBlobParams = serde_json::from_value(request.params.clone())
                .map_err(|error| format!("invalid attachment.delete request: {error}"))?;
            let deleted = delete_attachment_blob(host_call, &params.blob_key)?;
            Ok(json!({ "deleted": deleted }))
        }
        "attachment.add_bytes" => {
            let params: AttachmentAddBytesParams = serde_json::from_value(request.params.clone())
                .map_err(|error| format!("invalid attachment.add_bytes request: {error}"))?;
            upload_attachment_bytes(host_call, params)
        }
        "attachment.open" => {
            let params: AttachmentOpenParams = serde_json::from_value(request.params.clone())
                .map_err(|error| format!("invalid attachment.open request: {error}"))?;
            open_attachment(host_call, params)
        }
        "attachment.preview" => {
            let params: AttachmentBlobParams = serde_json::from_value(request.params.clone())
                .map_err(|error| format!("invalid attachment.preview request: {error}"))?;
            if !params.blob_key.starts_with("attachment/") {
                return Err("attachment blob key is invalid".to_string());
            }
            request_vault_access(host_call)?;
            let info = host_call("vault.blob.info", json!({ "key": params.blob_key }))?;
            if info.get("exists") != Some(&Value::Bool(true)) {
                return Err("attachment blob was not found".to_string());
            }
            let size = info.get("size").and_then(Value::as_u64).unwrap_or(u64::MAX);
            if size > 8 * 1024 * 1024 {
                return Err("attachment is too large for inline preview".to_string());
            }
            let chunks = info
                .get("chunks")
                .and_then(Value::as_u64)
                .ok_or_else(|| "host returned invalid Vault blob info".to_string())?
                as u32;
            let mut bytes = Vec::with_capacity(size as usize);
            for index in 0..chunks {
                let response = host_call(
                    "vault.blob.read",
                    json!({ "key": params.blob_key, "index": index }),
                )?;
                let data = response
                    .get("data")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "host returned invalid Vault blob chunk data".to_string())?;
                bytes.extend_from_slice(
                    &STANDARD_NO_PAD
                        .decode(data)
                        .map_err(|_| "host returned invalid Vault blob chunk encoding".to_string())?,
                );
            }
            Ok(json!({ "data": STANDARD_NO_PAD.encode(bytes) }))
        }
        "settings.get" => {
            let value = get_settings(host_call)?;
            Ok(if value.is_null() {
                default_settings()
            } else {
                sanitize_settings(value)
            })
        }
        "settings.set" => {
            let params: SettingsSetParams = serde_json::from_value(request.params.clone())
                .map_err(|error| format!("invalid settings.set request: {error}"))?;
            set_settings(host_call, params.settings)
        }
        method => Err(format!("unsupported plugin method: {method}")),
    }
}


fn cleanup_attachment_temp_files() {
    let root = std::env::temp_dir().join("RemoteHand").join("Notes");
    let Ok(entries) = fs::read_dir(&root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let _ = fs::remove_file(path);
        }
    }
}

fn run() -> Result<(), String> {
    cleanup_attachment_temp_files();
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    write_frame(
        &mut stdout,
        &PluginHello {
            message_type: "plugin.hello",
            plugin_id: PLUGIN_ID,
            api_version: API_VERSION,
            plugin_version: PLUGIN_VERSION,
        },
    )?;

    let payload = read_frame(&mut stdin)?
        .ok_or_else(|| "host closed stream before host.ready".to_string())?;
    let ready: HostReady =
        serde_json::from_slice(&payload).map_err(|error| format!("invalid host.ready: {error}"))?;

    if ready.message_type != "host.ready"
        || ready.plugin_id != PLUGIN_ID
        || ready.api_version != API_VERSION
    {
        return Err("RemoteHand rejected Plugin API v1 startup contract".to_string());
    }

    for permission in REQUIRED_PERMISSIONS {
        if !ready
            .granted_permissions
            .iter()
            .any(|granted| granted == permission)
        {
            return Err(format!("RemoteHand did not grant required permission: {permission}"));
        }
    }

    let mut next_host_request_id = 0u64;

    while let Some(payload) = read_frame(&mut stdin)? {
        let request: HostRequest = serde_json::from_slice(&payload)
            .map_err(|error| format!("invalid host request: {error}"))?;

        if request.message_type != "host.request" {
            return Err(format!(
                "unexpected Plugin API v1 message type: {}",
                request.message_type
            ));
        }

        let mut host_call = |method: &str, params: Value| {
            call_host(
                &mut stdin,
                &mut stdout,
                &mut next_host_request_id,
                method,
                params,
            )
        };

        let (ok, result, error) = match dispatch(&request, &mut host_call) {
            Ok(result) => (true, result, String::new()),
            Err(error) => (false, Value::Null, error),
        };

        write_frame(
            &mut stdout,
            &PluginResponse {
                message_type: "plugin.response",
                request_id: request.request_id,
                ok,
                result,
                error,
            },
        )?;
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("RemoteHand Notes plugin failed: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(id: &str, locked: bool, location: &str, title: &str) -> Value {
        json!({
            "version": 1,
            "id": id,
            "type": "note",
            "title": title,
            "body": "",
            "checklist": [],
            "labels": [],
            "color": "yellow",
            "pinned": false,
            "locked": locked,
            "location": location,
            "createdAt": "2026-10-01T00:00:00Z",
            "updatedAt": "2026-10-01T00:00:00Z",
            "attachments": []
        })
    }

    #[test]
    fn runtime_version_matches_manifest() {
        let manifest: Value =
            serde_json::from_str(include_str!("../plugin.json")).expect("parse plugin.json");
        assert_eq!(manifest["version"], PLUGIN_VERSION);
        assert_eq!(manifest["id"], PLUGIN_ID);
        assert_eq!(manifest["apiVersion"], API_VERSION);
    }

    #[test]
    fn validates_note_ids() {
        assert!(validate_note_id("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(validate_note_id("../escape").is_err());
        assert!(validate_note_id("has/slash").is_err());
        assert!(validate_note_id("").is_err());
    }

    #[test]
    fn locked_note_allows_unlock_only() {
        let previous = note("one", true, "main", "A");
        let mut next = previous.clone();
        next["locked"] = Value::Bool(false);
        assert!(locked_transition_allowed(&previous, &next));

        let mut edited = previous.clone();
        edited["title"] = Value::String("Changed".to_string());
        assert!(!locked_transition_allowed(&previous, &edited));
    }

    #[test]
    fn locked_archive_and_trash_allow_restore_to_main() {
        for source in ["archive", "trash"] {
            let previous = note("one", true, source, "A");
            let mut next = previous.clone();
            next["location"] = Value::String("main".to_string());
            assert!(locked_transition_allowed(&previous, &next));
        }
    }

    #[test]
    fn locked_main_note_cannot_move_to_archive_or_trash() {
        for target in ["archive", "trash"] {
            let previous = note("one", true, "main", "A");
            let mut next = previous.clone();
            next["location"] = Value::String(target.to_string());
            assert!(!locked_transition_allowed(&previous, &next));
        }
    }

    #[test]
    fn catalog_removes_ids_everywhere() {
        let mut catalog = json!({
            "version": 1,
            "noteIds": ["one", "two"],
            "manualOrder": {
                "main": ["one", "two"],
                "archive": ["one"],
                "trash": []
            },
            "pinnedOrder": ["one"]
        });
        remove_id_from_catalog(&mut catalog, "one");
        assert_eq!(catalog["noteIds"], json!(["two"]));
        assert_eq!(catalog["manualOrder"]["main"], json!(["two"]));
        assert_eq!(catalog["manualOrder"]["archive"], json!([]));
        assert_eq!(catalog["pinnedOrder"], json!([]));
    }

    #[test]
    fn settings_ignore_unknown_fields() {
        let settings = sanitize_settings(json!({
            "autosaveDelayMs": 900,
            "secretNoteTitle": "do not persist"
        }));
        assert_eq!(settings["autosaveDelayMs"], 900);
        assert!(settings.get("secretNoteTitle").is_none());
    }
}
