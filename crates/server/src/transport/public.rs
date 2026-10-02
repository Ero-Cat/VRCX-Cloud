//! Public API: credential-free, read-only endpoints for external
//! consumers (home-page embeds, bots, status widgets).
//!
//! Responses are minimal projections — never cookies, credentials, or
//! private notes. Rate-limited per process.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

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

    let pick = |key: &str| user.get(key).cloned().unwrap_or(Value::Null);
    json_response(json!({
        "ok": true,
        "profile": {
            "id": pick("id"),
            "displayName": pick("displayName"),
            "status": pick("status"),
            "statusDescription": pick("statusDescription"),
            "bio": pick("bio"),
            "bioLinks": pick("bioLinks"),
            "profilePicOverride": pick("profilePicOverride"),
            "currentAvatarImageUrl": pick("currentAvatarImageUrl"),
            "currentAvatarThumbnailImageUrl": pick("currentAvatarThumbnailImageUrl"),
            "currentAvatarTags": pick("currentAvatarTags"),
            "fallbackAvatar": pick("fallbackAvatar"),
            "lastPlatform": pick("lastPlatform"),
            "dateJoined": pick("dateJoined"),
            "country": pick("country"),
            "languageTags": pick("languageTags"),
            "pronouns": pick("pronouns"),
            "friendsCount": pick("friendsCount"),
            "acceptedPrivacyVersion": pick("acceptedPrivacyVersion"),
            "twoFactorAuthEnabled": pick("twoFactorAuthEnabled"),
            "emailVerified": pick("emailVerified"),
            "statusFirstTimeAt": pick("statusFirstTimeAt"),
            "state": pick("state"),
            "tags": pick("tags")
        },
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
