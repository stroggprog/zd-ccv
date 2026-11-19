//! SillyTavern character card parser.
//!
//! Deliberately free of GUI dependencies so it can be unit tested against
//! the fixture set in `samples/`. Parses the card JSON embedded in PNG
//! `tEXt`/`zTXt`/`iTXt` chunks (keyword `chara` for V1/V2, `ccv3` for V3).
//! Only PNG files are loaded; other formats are ignored by the viewer.

use base64::{engine::general_purpose::STANDARD, Engine};
use flate2::read::ZlibDecoder;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Where a card payload was embedded in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadKind {
    /// PNG tEXt chunk with keyword `chara` (V1/V2).
    Chara,
    /// PNG tEXt chunk with keyword `ccv3` (V3).
    Ccv3,
}

impl PayloadKind {
    pub fn keyword(self) -> &'static str {
        match self {
            PayloadKind::Chara => "chara",
            PayloadKind::Ccv3 => "ccv3",
        }
    }
}

/// Card specification version detected from the payload structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecVersion {
    V1,
    V2,
    V3,
    Unknown,
}

impl SpecVersion {
    pub fn label(self) -> &'static str {
        match self {
            SpecVersion::V1 => "Character Card V1",
            SpecVersion::V2 => "Character Card V2",
            SpecVersion::V3 => "Character Card V3",
            SpecVersion::Unknown => "Unknown card format",
        }
    }
}

/// One decoded card payload.
#[derive(Debug, Clone)]
pub struct CardPayload {
    pub kind: PayloadKind,
    pub spec: SpecVersion,
    /// The field object (`data` for V2/V3, the whole object for V1).
    pub data: serde_json::Value,
    /// The full decoded JSON as embedded in the file.
    pub raw: serde_json::Value,
    /// 0-based index of this chunk among chunks with the same keyword.
    pub chunk_index: usize,
    /// Total number of chunks with this keyword in the file.
    pub chunk_total: usize,
}

impl CardPayload {
    /// A human-readable label identifying the source chunk, e.g.
    /// `chara chunk 2 of 2 (Character Card V2)`.
    pub fn label(&self) -> String {
        let position = if self.chunk_total > 1 {
            format!(" {} of {}", self.chunk_index + 1, self.chunk_total)
        } else {
            String::new()
        };
        format!(
            "{}{} ({})",
            self.kind.keyword(),
            position,
            self.spec.label()
        )
    }
}

/// A card file loaded from disk: display payloads and non-fatal problems.
#[derive(Debug, Clone)]
pub struct CardFile {
    pub path: PathBuf,
    pub warnings: Vec<String>,
    /// Every successfully decoded card payload, ordered the way
    /// SillyTavern resolves them: `ccv3` chunks before `chara` chunks,
    /// and within a keyword in file order. The first entry is the one
    /// SillyTavern would use; viewers may offer the others for
    /// selection.
    pub payloads: Vec<CardPayload>,
}

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

/// Recognized card chunk kinds by (case-insensitive) keyword.
fn kind_for_keyword(keyword: &str) -> Option<PayloadKind> {
    if keyword.eq_ignore_ascii_case("chara") {
        Some(PayloadKind::Chara)
    } else if keyword.eq_ignore_ascii_case("ccv3") {
        Some(PayloadKind::Ccv3)
    } else {
        None
    }
}

/// Load a card from a PNG file. Never fails on malformed card data:
/// problems are collected as warnings.
pub fn load(path: &Path) -> CardFile {
    let mut warnings = Vec::new();
    let payloads = load_png_file(path, &mut warnings);
    CardFile {
        path: path.to_path_buf(),
        warnings,
        payloads,
    }
}

fn load_png_file(path: &Path, warnings: &mut Vec<String>) -> Vec<CardPayload> {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            warnings.push(format!("could not read file: {e}"));
            return Vec::new();
        }
    };
    let chunks = walk_text_chunks(&bytes, warnings);
    if chunks.is_empty() {
        warnings
            .push("image contains no character card data (no chara or ccv3 text chunks)".into());
        return Vec::new();
    }

    let mut payloads = Vec::new();
    for kind in [PayloadKind::Ccv3, PayloadKind::Chara] {
        let matching: Vec<&String> = chunks
            .iter()
            .filter(|(k, _)| *k == kind)
            .map(|(_, text)| text)
            .collect();
        if matching.is_empty() {
            continue;
        }
        // Keep every chunk: tools may add a second `chara` chunk with new
        // data, so the viewer offers a selection instead of discarding.
        for (i, text) in matching.iter().enumerate() {
            match decode_payload(text) {
                Ok(value) => payloads.push(classify(kind, value, i, matching.len())),
                Err(e) => warnings.push(format!(
                    "could not decode `{}` chunk {} of {}: {e}",
                    kind.keyword(),
                    i + 1,
                    matching.len()
                )),
            }
        }
    }
    payloads
}

/// Decode a base64 text payload into JSON.
fn decode_payload(text: &str) -> Result<serde_json::Value, String> {
    let trimmed = text.trim();
    let json_bytes = STANDARD
        .decode(trimmed)
        .map_err(|e| format!("base64 error: {e}"))?;
    let json_text =
        String::from_utf8(json_bytes).map_err(|e| format!("payload is not valid UTF-8: {e}"))?;
    serde_json::from_str(&json_text).map_err(|e| format!("JSON error: {e}"))
}

/// Determine the spec version of a decoded payload and extract its data.
fn classify(
    kind: PayloadKind,
    value: serde_json::Value,
    chunk_index: usize,
    chunk_total: usize,
) -> CardPayload {
    let spec = match value.get("spec").and_then(|s| s.as_str()) {
        Some("chara_card_v2") => SpecVersion::V2,
        Some("chara_card_v3") => SpecVersion::V3,
        None => {
            if value.is_object() && value.get("name").is_some() {
                SpecVersion::V1
            } else {
                SpecVersion::Unknown
            }
        }
        Some(_) => SpecVersion::Unknown,
    };
    let data = match spec {
        SpecVersion::V1 => value.clone(),
        SpecVersion::V2 | SpecVersion::V3 => value
            .get("data")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        SpecVersion::Unknown => serde_json::Value::Null,
    };
    CardPayload {
        kind,
        spec,
        data,
        raw: value,
        chunk_index,
        chunk_total,
    }
}

/// Walk every chunk of a PNG file and collect card-relevant text chunks.
///
/// Card data is stored *after* the image data, so the whole file is
/// scanned. CRC values are not verified. Truncated or malformed files
/// yield whatever was collected up to that point.
fn walk_text_chunks(data: &[u8], warnings: &mut Vec<String>) -> Vec<(PayloadKind, String)> {
    if !data.starts_with(&PNG_SIGNATURE) {
        warnings.push("file is not a valid PNG".into());
        return Vec::new();
    }
    let mut chunks = Vec::new();
    let mut i = PNG_SIGNATURE.len();
    while i + 8 <= data.len() {
        let len = u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
        let typ = &data[i + 4..i + 8];
        let body_start = i + 8;
        if body_start + len + 4 > data.len() {
            warnings.push("truncated chunk in PNG stream".into());
            break;
        }
        let body = &data[body_start..body_start + len];
        if let Some(kind) = text_chunk_keyword(typ, body).and_then(|kw| kind_for_keyword(&kw)) {
            let text = text_chunk_content(typ, body, warnings);
            chunks.push((kind, text));
        }
        if typ == b"IEND" {
            break;
        }
        i = body_start + len + 4; // skip body and CRC
    }
    chunks
}

/// The keyword of a text-type chunk, if it is one.
fn text_chunk_keyword(typ: &[u8], body: &[u8]) -> Option<String> {
    if !matches!(typ, b"tEXt" | b"zTXt" | b"iTXt") {
        return None;
    }
    let nul = body.iter().position(|&b| b == 0)?;
    Some(String::from_utf8_lossy(&body[..nul]).into_owned())
}

/// The (possibly decompressed) text content of a text-type chunk.
fn text_chunk_content(typ: &[u8], body: &[u8], warnings: &mut Vec<String>) -> String {
    let nul = match body.iter().position(|&b| b == 0) {
        Some(n) => n,
        None => return String::new(),
    };
    let rest = &body[nul + 1..];
    match typ {
        b"tEXt" => String::from_utf8_lossy(rest).into_owned(),
        b"zTXt" => {
            if rest.first() != Some(&0) {
                warnings.push("zTXt chunk uses an unsupported compression method".into());
                return String::new();
            }
            inflate(&rest[1..], warnings)
        }
        b"iTXt" => {
            // keyword NUL comp_flag comp_method lang NUL translated_keyword NUL text
            let compressed = rest.first() == Some(&1);
            let mut parts = rest[2.min(rest.len())..].splitn(3, |&b| b == 0);
            let _lang = parts.next();
            let text = parts.next().and_then(|_| parts.next());
            match text {
                Some(text) if compressed => inflate(text, warnings),
                Some(text) => String::from_utf8_lossy(text).into_owned(),
                None => {
                    warnings.push("malformed iTXt chunk".into());
                    String::new()
                }
            }
        }
        _ => String::new(),
    }
}

fn inflate(bytes: &[u8], warnings: &mut Vec<String>) -> String {
    let mut out = Vec::new();
    match ZlibDecoder::new(bytes).read_to_end(&mut out) {
        Ok(_) => String::from_utf8_lossy(&out).into_owned(),
        Err(e) => {
            warnings.push(format!("could not decompress text chunk: {e}"));
            String::new()
        }
    }
}

// ---------------------------------------------------------------------------
// Field model for the UI
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// Absent or empty.
    Empty,
    /// Plain multiline text.
    Text(String),
    /// Example message list split on `<START>` markers.
    Dialogue(Vec<String>),
    /// A list of strings (greetings, tags, ...).
    Strings(Vec<String>),
    /// Embedded lorebook (`character_book`).
    Lorebook(serde_json::Value),
    /// Structured content shown as formatted JSON.
    Json(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub label: &'static str,
    pub value: FieldValue,
}

/// The card fields in display order. Fields that do not apply to `spec`
/// are skipped entirely; applicable but absent fields become `Empty`.
pub fn fields(data: &serde_json::Value, spec: SpecVersion) -> Vec<Field> {
    let mut out: Vec<Field> = Vec::new();
    let mut push = |label: &'static str, value: FieldValue| {
        out.push(Field { label, value });
    };

    let str_field = |key: &str| -> FieldValue {
        match data.get(key).and_then(|v| v.as_str()) {
            Some(s) if !s.trim().is_empty() => FieldValue::Text(s.to_string()),
            _ => FieldValue::Empty,
        }
    };

    push("Name", str_field("name"));
    if spec == SpecVersion::V3 {
        push("Nickname", str_field("nickname"));
    }
    push("Description", str_field("description"));
    push("Personality", str_field("personality"));
    push("Scenario", str_field("scenario"));
    push("First Message", str_field("first_mes"));

    let greetings = string_array(data, "alternate_greetings");
    if !greetings.is_empty() {
        push("Alternate Greetings", FieldValue::Strings(greetings));
    }

    push(
        "Example Messages",
        match data.get("mes_example").and_then(|v| v.as_str()) {
            Some(s) if !s.trim().is_empty() => FieldValue::Dialogue(split_dialogue(s)),
            _ => FieldValue::Empty,
        },
    );

    let group = string_array(data, "group_only_greetings");
    if spec == SpecVersion::V3 && !group.is_empty() {
        push("Group-Only Greetings", FieldValue::Strings(group));
    }

    push("Creator Notes", str_field("creator_notes"));
    if spec == SpecVersion::V3 {
        push(
            "Creator Notes (Multilingual)",
            json_field(data, "creator_notes_multilingual"),
        );
    }
    push("System Prompt", str_field("system_prompt"));
    push(
        "Post-History Instructions",
        str_field("post_history_instructions"),
    );

    let tags = string_array(data, "tags");
    if !tags.is_empty() {
        push("Tags", FieldValue::Strings(tags));
    }

    push("Creator", str_field("creator"));
    push("Character Version", str_field("character_version"));

    if spec == SpecVersion::V3 {
        let source = string_array(data, "source");
        if !source.is_empty() {
            push("Source", FieldValue::Strings(source));
        }
        push("Creation Date", unix_date_field(data, "creation_date"));
        push(
            "Modification Date",
            unix_date_field(data, "modification_date"),
        );
        if data.get("assets").is_some() {
            push("Assets", json_field(data, "assets"));
        }
    }

    if let Some(book) = data.get("character_book") {
        if !book.is_null() {
            push("Lorebook", FieldValue::Lorebook(book.clone()));
        }
    }

    if let Some(ext) = data.get("extensions") {
        if !ext.is_null() && ext.as_object().is_none_or(|o| !o.is_empty()) {
            push("Extensions", FieldValue::Json(ext.clone()));
        }
    }

    out
}

fn string_array(data: &serde_json::Value, key: &str) -> Vec<String> {
    data.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .filter(|s| !s.trim().is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn json_field(data: &serde_json::Value, key: &str) -> FieldValue {
    match data.get(key) {
        Some(v) if !v.is_null() && v != &serde_json::Value::Object(Default::default()) => {
            FieldValue::Json(v.clone())
        }
        Some(v) if v.is_array() && v.as_array().is_some_and(|a| !a.is_empty()) => {
            FieldValue::Json(v.clone())
        }
        _ => FieldValue::Empty,
    }
}

/// Split `mes_example` on `<START>` markers into per-conversation blocks.
fn split_dialogue(text: &str) -> Vec<String> {
    let parts: Vec<String> = text
        .split("<START>")
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        vec![text.trim().to_string()]
    } else {
        parts
    }
}

fn unix_date_field(data: &serde_json::Value, key: &str) -> FieldValue {
    match data.get(key).and_then(|v| v.as_i64()) {
        Some(n) if n > 0 => FieldValue::Text(format!("{} (unix {})", format_unix(n), n)),
        _ => FieldValue::Empty,
    }
}

/// Convert unix seconds to a `YYYY-MM-DD` UTC string (civil algorithm).
fn format_unix(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

/// Whether the payload embeds a usable lorebook. Many real-world cards
/// store `"character_book": null`; only an object counts as available.
pub fn has_lorebook(payload: &CardPayload) -> bool {
    payload
        .data
        .get("character_book")
        .is_some_and(|b| b.is_object())
}

/// Target path for character/all exports: the source file with a `.json`
/// extension.
pub fn json_sibling(path: &Path) -> PathBuf {
    path.with_extension("json")
}

/// The currently displayed character as a standalone card JSON document,
/// with any embedded lorebook removed.
pub fn export_character(payload: &CardPayload) -> String {
    let mut raw = payload.raw.clone();
    match payload.spec {
        SpecVersion::V2 | SpecVersion::V3 => {
            if let Some(data) = raw.get_mut("data").and_then(|v| v.as_object_mut()) {
                data.remove("character_book");
            }
        }
        _ => {
            if let Some(obj) = raw.as_object_mut() {
                obj.remove("character_book");
            }
        }
    }
    to_pretty(&raw)
}

/// The embedded lorebook as a standalone JSON document, if present.
pub fn export_lorebook(payload: &CardPayload) -> Option<String> {
    payload
        .data
        .get("character_book")
        .filter(|b| b.is_object())
        .map(to_pretty)
}

/// The name to use for the lorebook export file.
pub fn lorebook_file_name(payload: &CardPayload) -> Option<String> {
    payload
        .data
        .get("character_book")
        .and_then(|b| b.get("name"))
        .and_then(|n| n.as_str())
        .filter(|n| !n.trim().is_empty())
        .map(sanitize_file_name)
}

/// The full embedded payload, including any lorebook.
pub fn export_all(payload: &CardPayload) -> String {
    to_pretty(&payload.raw)
}

fn to_pretty(value: &serde_json::Value) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
    text.push('\n');
    text
}

/// Make a string safe to use as a file name: path separators and other
/// characters that are illegal on Windows are replaced, control
/// characters stripped, and the result is length-capped.
pub fn sanitize_file_name(name: &str) -> String {
    let mut cleaned: String = name
        .trim()
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
                || (c as u32) < 0x20
            {
                '_'
            } else {
                c
            }
        })
        .collect();
    if cleaned.len() > 120 {
        cleaned = cleaned.chars().take(120).collect();
    }
    let cleaned = cleaned
        .trim_matches(|c: char| c == '.' || c == ' ')
        .to_string();
    if cleaned.is_empty() {
        "lorebook".into()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Build a chunk with a (dummy) CRC; the walker ignores CRCs.
    fn chunk(typ: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(body.len() as u32).to_be_bytes());
        out.extend_from_slice(typ);
        out.extend_from_slice(body);
        out.extend_from_slice(&[0, 0, 0, 0]); // CRC placeholder
        out
    }

    fn text_chunk(keyword: &str, text: &str) -> Vec<u8> {
        let mut body = keyword.as_bytes().to_vec();
        body.push(0);
        body.extend_from_slice(text.as_bytes());
        chunk(b"tEXt", &body)
    }

    fn png_with(chunks: &[Vec<u8>]) -> Vec<u8> {
        let mut out = PNG_SIGNATURE.to_vec();
        let ihdr: Vec<u8> = vec![0u8; 13];
        out.extend_from_slice(&chunk(b"IHDR", &ihdr));
        for c in chunks {
            out.extend_from_slice(c);
        }
        let idat = chunk(b"IDAT", b"fake-image-data");
        out.extend_from_slice(&idat);
        out.extend_from_slice(&chunk(b"IEND", b""));
        out
    }

    fn card_json(name: &str, spec: Option<&str>) -> String {
        let inner = format!(r#"{{"name":"{name}","description":"desc"}}"#);
        match spec {
            Some(s) => {
                let data: serde_json::Value = serde_json::from_str(&inner).unwrap();
                serde_json::json!({"spec": s, "spec_version": "2.0", "data": data}).to_string()
            }
            None => inner,
        }
    }

    fn b64(s: &str) -> String {
        STANDARD.encode(s.as_bytes())
    }

    fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "zd-ccv-test-{}-{}-{name}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(unique);
        fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn parses_single_chara_chunk() {
        let file = png_with(&[text_chunk(
            "chara",
            &b64(&card_json("Alice", Some("chara_card_v2"))),
        )]);
        let path = temp_file("single.png", &file);
        let card = load(&path);
        assert_eq!(card.payloads.len(), 1);
        assert_eq!(card.payloads[0].spec, SpecVersion::V2);
        assert_eq!(card.payloads[0].kind, PayloadKind::Chara);
        let name = fields(&card.payloads[0].data, SpecVersion::V2)
            .into_iter()
            .find(|f| f.label == "Name")
            .unwrap();
        assert_eq!(name.value, FieldValue::Text("Alice".into()));
        fs::remove_file(path).ok();
    }

    #[test]
    fn duplicate_chunks_are_all_kept_first_is_preferred() {
        // SillyTavern reads the *first* `chara` chunk, and other tools may
        // add a second one; the viewer keeps every chunk for selection.
        let file = png_with(&[
            text_chunk("chara", &b64(&card_json("First", Some("chara_card_v2")))),
            text_chunk("chara", &b64(&card_json("Second", Some("chara_card_v2")))),
        ]);
        let path = temp_file("dup.png", &file);
        let card = load(&path);
        assert_eq!(card.payloads.len(), 2);
        assert!(card.warnings.is_empty(), "warnings: {:?}", card.warnings);
        assert!(card.payloads[0].raw.to_string().contains("First"));
        assert_eq!(card.payloads[0].chunk_index, 0);
        assert_eq!(card.payloads[1].chunk_index, 1);
        assert_eq!(card.payloads[0].chunk_total, 2);
        assert!(card.payloads[0].label().contains("1 of 2"));
        fs::remove_file(path).ok();
    }

    #[test]
    fn one_undecodable_duplicate_still_keeps_the_rest() {
        let file = png_with(&[
            text_chunk("chara", "not*base64!!"),
            text_chunk("chara", &b64(&card_json("Ok", Some("chara_card_v2")))),
        ]);
        let path = temp_file("dupbad.png", &file);
        let card = load(&path);
        assert_eq!(card.payloads.len(), 1);
        assert!(card
            .warnings
            .iter()
            .any(|w| w.contains("chunk 1 of 2") && w.contains("chara")));
        fs::remove_file(path).ok();
    }

    #[test]
    fn ccv3_takes_precedence_over_chara() {
        let file = png_with(&[
            text_chunk("chara", &b64(&card_json("V2Name", Some("chara_card_v2")))),
            text_chunk("ccv3", &b64(&card_json("V3Name", Some("chara_card_v3")))),
        ]);
        let path = temp_file("precedence.png", &file);
        let card = load(&path);
        assert_eq!(card.payloads[0].kind, PayloadKind::Ccv3);
        assert_eq!(card.payloads[0].spec, SpecVersion::V3);
        fs::remove_file(path).ok();
    }

    #[test]
    fn v1_flat_json_is_normalized() {
        let file = png_with(&[text_chunk("chara", &b64(&card_json("FlatV1", None)))]);
        let path = temp_file("v1.png", &file);
        let card = load(&path);
        assert_eq!(card.payloads[0].spec, SpecVersion::V1);
        fs::remove_file(path).ok();
    }

    #[test]
    fn undecodable_chunk_warns_without_panicking() {
        let file = png_with(&[text_chunk("chara", "not*base64!!")]);
        let path = temp_file("bad.png", &file);
        let card = load(&path);
        assert!(card.payloads.is_empty());
        assert!(card.warnings.iter().any(|w| w.contains("chara")));
        fs::remove_file(path).ok();
    }

    #[test]
    fn missing_card_data_warns() {
        let file = png_with(&[]);
        let path = temp_file("plain.png", &file);
        let card = load(&path);
        assert!(card.payloads.is_empty());
        assert!(card
            .warnings
            .iter()
            .any(|w| w.contains("no character card data")));
        fs::remove_file(path).ok();
    }

    #[test]
    fn ztxt_payload_is_inflated() {
        let json = b64(&card_json("Zed", Some("chara_card_v2")));
        let mut enc = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        enc.write_all(json.as_bytes()).unwrap();
        let compressed = enc.finish().unwrap();
        let mut body = b"chara".to_vec();
        body.push(0);
        body.push(0); // compression method: deflate
        body.extend_from_slice(&compressed);
        let file = png_with(&[chunk(b"zTXt", &body)]);
        let path = temp_file("ztxt.png", &file);
        let card = load(&path);
        assert_eq!(card.payloads.len(), 1);
        assert_eq!(card.payloads[0].spec, SpecVersion::V2);
        fs::remove_file(path).ok();
    }

    #[test]
    fn chunks_after_idat_are_found() {
        // Real cards store the payload right before IEND, after IDAT.
        let mut out = PNG_SIGNATURE.to_vec();
        out.extend_from_slice(&chunk(b"IHDR", &[0u8; 13]));
        out.extend_from_slice(&chunk(b"IDAT", b"fake-image-data"));
        out.extend_from_slice(&text_chunk(
            "chara",
            &b64(&card_json("LateChunk", Some("chara_card_v2"))),
        ));
        out.extend_from_slice(&chunk(b"IEND", b""));
        let path = temp_file("late.png", &out);
        let card = load(&path);
        assert_eq!(card.payloads.len(), 1);
        fs::remove_file(path).ok();
    }

    #[test]
    fn unix_date_formatting() {
        assert_eq!(format_unix(0), "1970-01-01");
        assert_eq!(format_unix(1_700_000_000), "2023-11-14");
    }

    #[test]
    fn export_character_strips_embedded_lorebook() {
        let raw = serde_json::json!({
            "spec": "chara_card_v2",
            "spec_version": "2.0",
            "data": {
                "name": "Alice",
                "description": "d",
                "character_book": {"name": "Book of Alice", "entries": []}
            }
        });
        let payload = classify(PayloadKind::Chara, raw, 0, 1);
        let out: serde_json::Value = serde_json::from_str(&export_character(&payload)).unwrap();
        assert!(out["data"].get("character_book").is_none());
        assert_eq!(out["data"]["name"], "Alice");

        let all: serde_json::Value = serde_json::from_str(&export_all(&payload)).unwrap();
        assert!(all["data"]["character_book"].is_object());

        let book = export_lorebook(&payload).unwrap();
        assert!(book.contains("Book of Alice"));
        assert_eq!(
            lorebook_file_name(&payload).as_deref(),
            Some("Book of Alice")
        );
    }

    #[test]
    fn export_handles_card_without_lorebook() {
        let raw = serde_json::json!({
            "spec": "chara_card_v2",
            "spec_version": "2.0",
            "data": {"name": "Solo", "description": "d"}
        });
        let payload = classify(PayloadKind::Chara, raw, 0, 1);
        assert!(export_lorebook(&payload).is_none());
        assert!(lorebook_file_name(&payload).is_none());
        assert!(!has_lorebook(&payload), "absent book is no book");

        let raw_null = serde_json::json!({
            "spec": "chara_card_v2",
            "spec_version": "2.0",
            "data": {"name": "Nullish", "character_book": null}
        });
        let null_payload = classify(PayloadKind::Chara, raw_null, 0, 1);
        assert!(!has_lorebook(&null_payload), "a null book is no book");
        assert!(
            export_lorebook(&null_payload).is_none(),
            "null book exports nothing"
        );
        let out: serde_json::Value = serde_json::from_str(&export_character(&payload)).unwrap();
        assert_eq!(out["data"]["name"], "Solo");
    }

    #[test]
    fn sanitize_file_name_replaces_bad_characters() {
        let name = sanitize_file_name("world/lore:book*?\"<>|");
        for bad in ['/', '\\', ':', '*', '?', '"', '<', '>', '|'] {
            assert!(!name.contains(bad), "{name} still contains {bad}");
        }
        assert_eq!(sanitize_file_name("  spaced. "), "spaced");
        assert_eq!(sanitize_file_name("."), "lorebook");
        assert_eq!(sanitize_file_name(""), "lorebook");
    }

    #[test]
    fn json_sibling_replaces_extension() {
        assert_eq!(
            json_sibling(std::path::Path::new("/x/card.png")),
            PathBuf::from("/x/card.json")
        );
    }

    fn samples_dir() -> Option<PathBuf> {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples");
        dir.is_dir().then_some(dir)
    }

    #[test]
    fn all_samples_parse() {
        let Some(dir) = samples_dir() else {
            eprintln!("samples/ not present, skipping");
            return;
        };
        let mut count = 0;
        let entries: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("png"))
            })
            .collect();
        for path in entries {
            let card = load(&path);
            assert!(
                !card.payloads.is_empty(),
                "{}: expected card payloads, warnings: {:?}",
                path.display(),
                card.warnings
            );
            let payload = &card.payloads[0];
            let name = fields(&payload.data, payload.spec)
                .into_iter()
                .find(|f| f.label == "Name")
                .expect("name field");
            assert_ne!(
                name.value,
                FieldValue::Empty,
                "{}: empty name",
                path.display()
            );
            count += 1;
        }
        assert!(count > 0, "no samples found");
    }

    #[test]
    fn samples_have_duplicate_chara_chunks() {
        // The real-world fixture set is known to carry two identical or
        // conflicting chara chunks in every PNG; the walker must surface
        // them all for the duplicate/conflict warning to fire.
        let Some(dir) = samples_dir() else {
            eprintln!("samples/ not present, skipping");
            return;
        };
        let path = dir.join("main_alice-89fca9a6_spec_v2.png");
        if !path.is_file() {
            return;
        }
        let mut warnings = Vec::new();
        let bytes = fs::read(&path).unwrap();
        let chunks = walk_text_chunks(&bytes, &mut warnings);
        let chara = chunks
            .iter()
            .filter(|(k, _)| *k == PayloadKind::Chara)
            .count();
        assert_eq!(chara, 2, "expected two chara chunks in the Alice fixture");
    }
}
