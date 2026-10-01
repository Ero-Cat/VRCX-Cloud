use std::sync::LazyLock;

use regex::Regex;
use vrcx_0_contracts::telemetry::TelemetryErrorDetail;

const MAX_SUMMARY_LENGTH: usize = 500;
const MAX_TOKEN_LENGTH: usize = 64;

static URL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\bhttps?://[^\s'"`<>]+"#).unwrap());
static WINDOWS_PATH_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[A-Za-z]:\\[^\s'"`<>]+"#).unwrap());
static SLASH_PATH_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?:^|\s)/[^\s'"`<>]+(?:/[^\s'"`<>]+)+"#).unwrap());
static VRCHAT_ID_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:usr|wrld|avtr|grp|file|vol|inst|auth|rgn|prn)_[A-Za-z0-9-]+\b").unwrap()
});
static NOTIFICATION_ID_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bnot_[A-Za-z0-9-]+\b").unwrap());
static PROVIDER_ID_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:org|req|key|sk)[_-][A-Za-z0-9_-]{3,}\b").unwrap());
static UUID_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\b")
        .unwrap()
});
static LONG_HEX_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b[0-9a-f]{24,}\b").unwrap());
static LONG_TOKEN_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b[A-Za-z0-9_-]{48,}\b").unwrap());
static ISO_LINE_PREFIX_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?\s*")
        .unwrap()
});
static WHITESPACE_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
static SAFE_TOKEN_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^A-Za-z0-9_.:-]+").unwrap());
static SAFE_VERSION_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^A-Za-z0-9._+-]+").unwrap());

pub fn sanitize_error_summary(value: impl AsRef<str>) -> String {
    let value = value.as_ref();
    let value = ISO_LINE_PREFIX_PATTERN.replace_all(value, "");
    let value = URL_PATTERN.replace_all(&value, "<url>");
    let value = WINDOWS_PATH_PATTERN.replace_all(&value, "<path>");
    let value = SLASH_PATH_PATTERN.replace_all(&value, " <path>");
    let value = VRCHAT_ID_PATTERN.replace_all(&value, "<id>");
    let value = NOTIFICATION_ID_PATTERN.replace_all(&value, |captures: &regex::Captures<'_>| {
        if &captures[0] == "not_found" {
            "not_found"
        } else {
            "<id>"
        }
    });
    let value = PROVIDER_ID_PATTERN.replace_all(&value, "<id>");
    let value = UUID_PATTERN.replace_all(&value, "<uuid>");
    let value = LONG_HEX_PATTERN.replace_all(&value, "<hash>");
    let value = LONG_TOKEN_PATTERN.replace_all(&value, "<token>");
    let value = WHITESPACE_PATTERN.replace_all(&value, " ");
    truncate_chars(value.trim(), MAX_SUMMARY_LENGTH)
}

fn sanitize_error_token(value: Option<&str>) -> Option<String> {
    let value = value.unwrap_or_default().trim();
    let value = SAFE_TOKEN_PATTERN.replace_all(value, "_");
    let value = value.trim_matches('_');
    (!value.is_empty()).then(|| truncate_chars(value, MAX_TOKEN_LENGTH))
}

fn sanitize_app_version(value: Option<&str>) -> Option<String> {
    let value = value.unwrap_or_default().trim();
    let value = SAFE_VERSION_PATTERN.replace_all(value, "_");
    let value = value.trim_matches('_');
    (!value.is_empty()).then(|| truncate_chars(value, MAX_TOKEN_LENGTH))
}

pub fn build_error_detail(
    kind: &str,
    source: Option<&str>,
    code: Option<&str>,
    name: Option<&str>,
    summary: Option<&str>,
    app_version: Option<&str>,
) -> TelemetryErrorDetail {
    let source = sanitize_error_token(source);
    let code = sanitize_error_token(code);
    let name = sanitize_error_token(name);
    let summary = summary
        .map(sanitize_error_summary)
        .filter(|value| !value.is_empty());
    let app_version = sanitize_app_version(app_version);
    let stable_parts = [
        kind,
        source.as_deref().unwrap_or("-"),
        code.as_deref().unwrap_or("-"),
        name.as_deref().unwrap_or("-"),
        summary.as_deref().unwrap_or("-"),
    ];
    TelemetryErrorDetail {
        kind: kind.to_string(),
        signature: format!("{kind}:{}", hash_string(&stable_parts.join("|"))),
        source,
        code,
        name,
        summary,
        app_version,
        count: 1,
    }
}

fn truncate_chars(value: &str, max_len: usize) -> String {
    value.chars().take(max_len).collect()
}

fn hash_string(value: &str) -> String {
    let mut hash = 0x811c9dc5u32;
    for unit in value.encode_utf16() {
        hash ^= u32::from(unit);
        hash = hash.wrapping_mul(0x01000193);
    }
    format!("{hash:08x}")
}
