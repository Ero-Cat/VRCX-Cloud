//! Public API: credential-free, read-only endpoints for external
//! consumers (home-page embeds, bots, status widgets).
//!
//! Responses are minimal projections — never cookies, credentials, or
//! private notes. Rate-limited per process.
//!
//! The profile card merges three read-only sources:
//! - the realtime current-user snapshot (presence, avatar, verification),
//! - a TTL-cached `GET profile/{id}?asSelf` fetch — the only VRChat
//!   route that still serves `bio`/`bioLinks` since `auth/user`
//!   dropped them,
//! - the local friend log (friend count).

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Map, Value};
use vrcx_0_runtime_host_server::ServerRuntimeHostState;

use super::WebContext;

/// Simple token-bucket-ish limiter shared by all public endpoints.
fn rate_limited() -> bool {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static WINDOW_START: AtomicU64 = AtomicU64::new(0);
    static COUNT: AtomicU64 = AtomicU64::new(0);
    const LIMIT: u64 = 240;
    const WINDOW_SECS: u64 = 60;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let start = WINDOW_START.load(Ordering::Relaxed);
    if now.saturating_sub(start) >= WINDOW_SECS {
        WINDOW_START.store(now, Ordering::Relaxed);
        COUNT.store(1, Ordering::Relaxed);
        return false;
    }
    COUNT.fetch_add(1, Ordering::Relaxed) >= LIMIT
}

fn json_response(body: Value) -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
        ],
        Json(body),
    )
        .into_response()
}

/// Fields mirrored from VRChat's `profile/{id}` route onto the public
/// card. That route is the only one still serving `bio`/`bioLinks`.
/// `statusFirstTime` is deliberately excluded: the profile route returns
/// a boolean there while `auth/user` returns the timestamp we want.
const RICH_PROFILE_FIELDS: &[&str] = &[
    "bio",
    "bioLinks",
    "pronouns",
    "profilePicOverride",
    "userIcon",
    "iconUrl",
    "bannerUrl",
    "bannerCustomUrl",
    "bannerType",
    "badges",
    "country",
    "dateJoined",
];

/// Serve the rich-profile projection from a per-process cache so public
/// polling cannot amplify upstream requests: at most one `profile/{id}`
/// fetch per TTL, with a shorter retry backoff after failures.
const RICH_PROFILE_TTL: Duration = Duration::from_secs(5 * 60);
const RICH_PROFILE_RETRY: Duration = Duration::from_secs(60);

struct RichProfileCache {
    user_id: String,
    fetched_at_ms: u64,
    attempted_at_ms: u64,
    fields: Map<String, Value>,
}

static RICH_PROFILE_CACHE: std::sync::Mutex<Option<RichProfileCache>> = std::sync::Mutex::new(None);
/// Single-flight guard: while a fetch is in flight, other requests serve
/// the cached view instead of piling up behind the upstream call.
static RICH_PROFILE_FETCHING: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn unix_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

async fn cached_rich_profile(state: &ServerRuntimeHostState, user_id: &str) -> Map<String, Value> {
    use std::sync::atomic::Ordering;
    let ttl_ms = RICH_PROFILE_TTL.as_millis() as u64;
    let retry_ms = RICH_PROFILE_RETRY.as_millis() as u64;
    let cached_view = |slot: &Option<RichProfileCache>| -> Option<Map<String, Value>> {
        slot.as_ref()
            .filter(|entry| entry.user_id == user_id)
            .filter(|entry| {
                let now = unix_ms();
                now.saturating_sub(entry.fetched_at_ms) < ttl_ms
                    || now.saturating_sub(entry.attempted_at_ms) < retry_ms
            })
            .map(|entry| entry.fields.clone())
    };
    {
        let guard = RICH_PROFILE_CACHE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(fields) = cached_view(&guard) {
            return fields;
        }
    }
    if RICH_PROFILE_FETCHING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        // Someone else is already fetching; serve the cached view.
        return RICH_PROFILE_CACHE
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .filter(|entry| entry.user_id == user_id)
            .map(|entry| entry.fields.clone())
            .unwrap_or_default();
    }
    let response = state
        .vrchat_remote()
        .user_profile(user_id.to_string(), true)
        .await;
    let mut fields = Map::new();
    if let Ok(response) = response {
        if (200..300).contains(&response.status) {
            if let Ok(profile) = serde_json::from_str::<Value>(&response.data) {
                if profile.get("id").and_then(Value::as_str) == Some(user_id) {
                    for key in RICH_PROFILE_FIELDS {
                        if let Some(value) = profile.get(key).filter(|v| !v.is_null()) {
                            fields.insert((*key).to_string(), value.clone());
                        }
                    }
                }
            }
        }
    }
    let now = unix_ms();
    let succeeded = !fields.is_empty();
    let mut guard = RICH_PROFILE_CACHE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let previous = guard.take().filter(|entry| entry.user_id == user_id);
    let mut entry = previous.unwrap_or(RichProfileCache {
        user_id: user_id.to_string(),
        fetched_at_ms: 0,
        attempted_at_ms: 0,
        fields: Map::new(),
    });
    entry.user_id = user_id.to_string();
    entry.attempted_at_ms = now;
    if succeeded {
        entry.fetched_at_ms = now;
        entry.fields = fields;
    }
    // on failure keep the previous (possibly stale) fields as a
    // best-effort view for this user
    let view = entry.fields.clone();
    *guard = Some(entry);
    RICH_PROFILE_FETCHING.store(false, Ordering::Release);
    view
}

/// `auth/user` returns snake_case keys for a few fields (`date_joined`,
/// `last_platform`, `statusFirstTime`); accept both spellings so the
/// card survives either response shape.
fn pick_alias(user: &Value, keys: &[&str]) -> Value {
    for key in keys {
        if let Some(value) = user.get(key).filter(|v| !v.is_null()) {
            return value.clone();
        }
    }
    Value::Null
}

/// Build the public profile card from the merged sources. Pure so the
/// field mapping stays unit-testable.
fn build_profile_card(
    snapshot: &Value,
    rich: &Map<String, Value>,
    friend_count: Option<usize>,
) -> Value {
    let pick = |key: &str| pick_alias(snapshot, &[key]);
    let pick_named = |keys: &[&str]| pick_alias(snapshot, keys);
    // Rich-profile values win for profile fields the realtime snapshot no
    // longer receives (bio and friends); the snapshot stays authoritative
    // for presence (status, avatars) because it updates in real time.
    let rich_or = |key: &str, fallback: Value| {
        rich.get(key)
            .filter(|v| !v.is_null())
            .cloned()
            .unwrap_or(fallback)
    };
    let language_tags = match snapshot.get("languageTags") {
        Some(Value::Array(tags)) => json!(tags),
        _ => snapshot
            .get("userLanguageCode")
            .and_then(Value::as_str)
            .filter(|code| !code.is_empty())
            .map(|code| json!([code]))
            .unwrap_or(Value::Null),
    };
    let friends_count = match friend_count {
        Some(count) => json!(count),
        None => snapshot
            .get("friends")
            .and_then(Value::as_array)
            .map(|friends| json!(friends.len()))
            .unwrap_or(Value::Null),
    };
    json!({
        "id": pick("id"),
        "displayName": pick("displayName"),
        "status": pick("status"),
        "statusDescription": pick("statusDescription"),
        "bio": rich_or("bio", pick("bio")),
        "bioLinks": rich_or("bioLinks", pick("bioLinks")),
        "pronouns": rich_or("pronouns", pick("pronouns")),
        "profilePicOverride": rich_or("profilePicOverride", pick("profilePicOverride")),
        "userIcon": rich_or("userIcon", pick("userIcon")),
        "iconUrl": rich_or("iconUrl", pick("iconUrl")),
        "bannerUrl": rich_or("bannerUrl", pick("bannerUrl")),
        "bannerType": rich_or("bannerType", pick("bannerType")),
        "badges": rich_or("badges", pick("badges")),
        "currentAvatarImageUrl": pick("currentAvatarImageUrl"),
        "currentAvatarThumbnailImageUrl": pick("currentAvatarThumbnailImageUrl"),
        "currentAvatarTags": pick("currentAvatarTags"),
        "fallbackAvatar": pick("fallbackAvatar"),
        "lastPlatform": pick_named(&["lastPlatform", "last_platform"]),
        "dateJoined": rich_or("dateJoined", pick_named(&["dateJoined", "date_joined"])),
        "country": rich_or("country", pick("country")),
        "languageTags": language_tags,
        "pronounsHistory": pick("pronounsHistory"),
        "pastDisplayNames": pick("pastDisplayNames"),
        "friendsCount": friends_count,
        "acceptedPrivacyVersion": pick("acceptedPrivacyVersion"),
        "twoFactorAuthEnabled": pick("twoFactorAuthEnabled"),
        "emailVerified": pick("emailVerified"),
        "statusFirstTimeAt": pick_named(&["statusFirstTimeAt", "statusFirstTime"]),
        "state": pick("state"),
        "tags": pick("tags"),
    })
}

/// `GET /api/public/profile` — the server account's public profile card.
pub async fn profile(State(ctx): State<Arc<WebContext>>) -> Response {
    if rate_limited() {
        return (StatusCode::TOO_MANY_REQUESTS, "rate limited").into_response();
    }
    let scope = ctx.state.auth_scope_snapshot();
    if !scope.active || scope.current_user_id.is_empty() {
        return json_response(json!({
            "ok": false,
            "error": "no_authenticated_user",
            "message": "The server has no active VRChat session."
        }));
    }

    // The freshest rich profile is the realtime current-user snapshot.
    // Freshest rich profile: the authenticated session's current-user
    // snapshot maintained by the realtime runtime.
    let session = ctx
        .state
        .runtime()
        .authenticated_session_projection()
        .session;
    let user = match session {
        Some(ref snapshot) if snapshot.current_user_snapshot.as_value().is_object() => {
            snapshot.current_user_snapshot.as_value().clone()
        }
        _ => {
            return json_response(json!({
                "ok": false,
                "error": "profile_not_hydrated",
                "message": "Profile snapshot not hydrated yet; retry shortly."
            }))
        }
    };

    let rich = cached_rich_profile(ctx.state.as_ref(), &scope.current_user_id).await;
    let friend_count = ctx
        .state
        .local_data()
        .friend_log_current_list(scope.current_user_id.clone())
        .ok()
        .map(|rows| rows.len());
    let profile = build_profile_card(&user, &rich, friend_count);
    json_response(json!({
        "ok": true,
        "profile": profile,
        "generatedAt": chrono::Utc::now().to_rfc3339()
    }))
}

/// `GET /api/public/mutual-friends?userId=usr_...&limit=100` — friends
/// the given user shares with the server account (the "circle" view).
pub async fn mutual_friends(
    State(ctx): State<Arc<WebContext>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    if rate_limited() {
        return (StatusCode::TOO_MANY_REQUESTS, "rate limited").into_response();
    }
    let Some(user_id) = params.get("userId").cloned() else {
        return json_response(json!({
            "ok": false,
            "error": "missing_user_id",
            "message": "Query parameter `userId` is required (e.g. usr_xxx)."
        }));
    };
    if !user_id.starts_with("usr_") || user_id.len() > 64 {
        return json_response(json!({
            "ok": false,
            "error": "invalid_user_id"
        }));
    }
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(100)
        .clamp(1, 500);

    let scope = ctx.state.auth_scope_snapshot();
    if !scope.active {
        return json_response(json!({
            "ok": false,
            "error": "no_authenticated_user"
        }));
    }

    // Mutual graph rows are synced from the desktop; fall back to the
    // live realtime friend snapshot when present.
    let persisted = ctx
        .state
        .local_data()
        .user_mutual_friends_list(vrcx_0_application::social::UserMutualFriendsListInput {
            user_id: user_id.clone(),
        })
        .await;

    let friends = match persisted {
        Ok(output) if !output.rows.is_empty() => output
            .rows
            .iter()
            .take(limit)
            .map(|row| {
                let obj = row.as_value();
                json!({
                    "id": obj.get("id").or_else(|| obj.get("userId")),
                    "displayName": obj.get("displayName").or_else(|| obj.get("display_name")),
                    "thumbnailUrl": obj
                        .get("currentAvatarThumbnailImageUrl")
                        .or_else(|| obj.get("thumbnail_url")),
                    "profilePicOverride": obj.get("profilePicOverride"),
                    "status": obj.get("status"),
                    "lastPlatform": obj.get("lastPlatform")
                })
            })
            .collect::<Vec<_>>(),
        _ => {
            // Fallback: intersect realtime friends with the target's
            // friend list is not available without a graph fetch; return
            // a hint instead of empty success.
            return json_response(json!({
                "ok": true,
                "userId": user_id,
                "friends": [],
                "persisted": false,
                "message": "No synced mutual graph for this user yet; open the mutual graph page once to build it."
            }));
        }
    };

    json_response(json!({
        "ok": true,
        "userId": user_id,
        "persisted": true,
        "count": friends.len(),
        "friends": friends,
        "generatedAt": chrono::Utc::now().to_rfc3339()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shape VRChat's `auth/user` returns today: snake_case dates, no
    /// `bio`/`bioLinks`/`profilePicOverride`, language as a code.
    fn current_auth_user() -> Value {
        json!({
            "id": "usr_self",
            "displayName": "DokiDoki",
            "status": "active",
            "statusDescription": "vibing",
            "pronouns": "they",
            "currentAvatarImageUrl": "https://avatar.example/full.png",
            "currentAvatarThumbnailImageUrl": "https://avatar.example/thumb.png",
            "currentAvatarTags": ["author_tag_x"],
            "fallbackAvatar": "avtr_fallback",
            "date_joined": "2019-04-01",
            "last_platform": "standalone",
            "statusFirstTime": "2026-01-01T00:00:00.000Z",
            "userLanguageCode": "en",
            "friends": ["usr_a", "usr_b", "usr_c"],
            "emailVerified": true,
            "twoFactorAuthEnabled": false,
            "acceptedPrivacyVersion": 2,
            "tags": ["system_trust_trusted"],
        })
    }

    fn rich_profile() -> Map<String, Value> {
        let mut fields = Map::new();
        fields.insert("bio".into(), json!("hello from the profile route"));
        fields.insert("bioLinks".into(), json!(["https://links.example/me"]));
        fields.insert(
            "profilePicOverride".into(),
            json!("https://pic.example/override.png"),
        );
        fields.insert("badges".into(), json!([{ "id": "bdg_1" }]));
        fields
    }

    #[test]
    fn card_fills_bio_from_the_rich_profile_route() {
        let card = build_profile_card(&current_auth_user(), &rich_profile(), Some(3));
        assert_eq!(card["bio"], json!("hello from the profile route"));
        assert_eq!(card["bioLinks"], json!(["https://links.example/me"]));
        assert_eq!(
            card["profilePicOverride"],
            json!("https://pic.example/override.png")
        );
        assert_eq!(card["badges"], json!([{ "id": "bdg_1" }]));
    }

    #[test]
    fn card_maps_snake_case_auth_user_keys() {
        let card = build_profile_card(&current_auth_user(), &Map::new(), Some(3));
        assert_eq!(card["dateJoined"], json!("2019-04-01"));
        assert_eq!(card["lastPlatform"], json!("standalone"));
        assert_eq!(card["statusFirstTimeAt"], json!("2026-01-01T00:00:00.000Z"));
    }

    #[test]
    fn card_derives_language_tags_from_the_language_code() {
        let card = build_profile_card(&current_auth_user(), &Map::new(), Some(3));
        assert_eq!(card["languageTags"], json!(["en"]));
        let mut legacy = current_auth_user();
        legacy["languageTags"] = json!(["eng", "jpn"]);
        let card = build_profile_card(&legacy, &Map::new(), None);
        assert_eq!(card["languageTags"], json!(["eng", "jpn"]));
    }

    #[test]
    fn friend_count_prefers_the_local_friend_log() {
        let card = build_profile_card(&current_auth_user(), &Map::new(), Some(42));
        assert_eq!(card["friendsCount"], json!(42));
    }

    #[test]
    fn friend_count_falls_back_to_the_snapshot_friend_list() {
        let card = build_profile_card(&current_auth_user(), &Map::new(), None);
        assert_eq!(card["friendsCount"], json!(3));
    }

    #[test]
    fn presence_fields_stay_authoritative_from_the_snapshot() {
        let mut rich = rich_profile();
        rich.insert("status".into(), json!("offline"));
        let card = build_profile_card(&current_auth_user(), &rich, Some(3));
        assert_eq!(card["status"], json!("active"));
        assert_eq!(
            card["currentAvatarImageUrl"],
            json!("https://avatar.example/full.png")
        );
    }
}
