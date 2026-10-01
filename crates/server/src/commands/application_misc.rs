//! Miscellaneous application commands: AI assistant + LLM endpoints,
//! translation, profile backup/restore, VRChat status, MCP server control,
//! privacy lock, proxy settings, overlay/notification activity filters,
//! runtime job records and external integration API queries.
//!
//! Ported from the desktop `src-tauri/src/commands/application/*` and
//! `src-tauri/src/commands/integrations/external_api` wrappers; command
//! names and camelCase argument keys are identical so the shared frontend
//! call surface works unchanged over HTTP.
//!
//! Desktop-only commands intentionally skipped here (no facade method on
//! `ServerRuntimeHostState`, or player-machine shell surface):
//! - `app__telemetry_record_event` / `app__telemetry_submit_feedback`
//!   (server builds keep telemetry permanently disabled).
//! - `app__integration_api_*` (desktop runtime-host integration API
//!   server; the web server does not expose it).
//! - `app__ancillary_runtime_snapshot_get`,
//!   `app__notification_do_not_disturb_mode_set`,
//!   `app__runtime_discord_reconcile_request` (desktop lifecycle surface).
//! - `app__registry_backup_*` (Windows registry + native file dialogs).
//! - `app__vr_overlay_*`, `background_image`, `background_mode`,
//!   `community_theme`, `deep_link`, `desktop_notification` (shell).
//! - `app__profile_restore_request` keeps its name but no longer restarts
//!   the process: the server returns the validation outcome and the
//!   restart flag is surfaced by the host layer instead.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::Deserialize;
use vrcx_0_application::discovery::{
    complete_translation, OpenAiTranslationPort, OpenAiTranslationRequest,
    TranslationCompletionError, TranslationDispatch, TranslationResult, TranslationTranslateInput,
};
use vrcx_0_application::profile::ProfileBackupSettings;
use vrcx_0_application_activity::notification::{
    NotificationActivityFiltersSetInput, OverlayActivityPreferenceFilters,
};
use vrcx_0_application_activity::overlay_activity_type_definitions;
use vrcx_0_assistant::{
    AssistantController, AssistantRuntimeSelection, AssistantRuntimeStatus,
    LlmEndpointDetectModelsInput, LlmEndpointDetectModelsResult, LlmEndpointDto,
    LlmEndpointUpsertInput, LlmTranslateInput, PlaybookMode, SendResult, Session, SessionSummary,
};
use vrcx_0_mcp::{McpCaller, McpServerController};
use vrcx_0_runtime_host_server::{
    test_proxy_connectivity, RuntimeJobRecordInput, ServerRuntimeHostState,
};

use crate::transport::error::ApiError;
use crate::transport::invoke::CommandRegistry;

use super::{ok, run_blocking};

fn arg<T: serde::de::DeserializeOwned>(args: &serde_json::Value, key: &str) -> Result<T, ApiError> {
    let value = args.get(key).cloned().unwrap_or(serde_json::Value::Null);
    serde_json::from_value(value)
        .map_err(|error| ApiError::BadRequest(format!("invalid `{key}` argument: {error}")))
}

fn assistant_error(error: vrcx_0_assistant::AssistantError) -> ApiError {
    ApiError::Message(error.to_string())
}

fn mcp_error(error: vrcx_0_mcp::McpError) -> ApiError {
    ApiError::Message(error.to_string())
}

// The desktop cached the assistant controller (and its MCP controller) on
// `AppState`; the server state facade exposes only the constructors, so the
// built controllers live here as process-lifetime singletons. The
// self-hosted server serves a single profile at a time.
static ASSISTANT: tokio::sync::OnceCell<AssistantController> = tokio::sync::OnceCell::const_new();
static MCP_SERVER: std::sync::OnceLock<McpServerController> = std::sync::OnceLock::new();

async fn assistant(
    state: &Arc<ServerRuntimeHostState>,
) -> Result<&'static AssistantController, ApiError> {
    ASSISTANT
        .get_or_try_init(|| async {
            AssistantController::new(state.assistant_controller_deps()).await
        })
        .await
        .map_err(|error| ApiError::Message(format!("assistant runtime init failed: {error}")))
}

fn mcp_server(state: &ServerRuntimeHostState) -> &'static McpServerController {
    MCP_SERVER
        .get_or_init(|| McpServerController::new(state.mcp_runtime(McpCaller::ExternalServer)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProxySettingsTestInput {
    #[serde(default)]
    proxy: String,
}

/// Serializable mirror of the desktop proxy test result; the core type is
/// not `Serialize` because the desktop wrapper owned the binding shape.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ProxySettingsTestResult {
    normalized_proxy: Option<String>,
    status: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalApiAvatarSearchInput {
    #[serde(default)]
    url: String,
    #[serde(default)]
    vrcx_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalApiYoutubeVideoInput {
    #[serde(default)]
    video_id: String,
    #[serde(default)]
    api_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalApiUrlInput {
    #[serde(default)]
    url: String,
    #[serde(default)]
    headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalApiImageInput {
    #[serde(default)]
    url: String,
}

/// Routes OpenAI-mode translation through the assistant LLM endpoints, the
/// same way the desktop's `TauriOpenAiTranslationPort` did.
struct ServerOpenAiTranslationPort {
    state: Arc<ServerRuntimeHostState>,
}

impl OpenAiTranslationPort for ServerOpenAiTranslationPort {
    type Error = ApiError;

    fn resolve_default_endpoint_id(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<String, ApiError>> + Send + '_>> {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            assistant(&state)
                .await?
                .endpoint_list()
                .map_err(assistant_error)?;
            state
                .resolved_openai_translation_endpoint_id()
                .map_err(ApiError::from)
        })
    }

    fn translate(
        &self,
        request: OpenAiTranslationRequest,
    ) -> Pin<Box<dyn Future<Output = Result<String, ApiError>> + Send + '_>> {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            assistant(&state)
                .await?
                .translate(LlmTranslateInput {
                    endpoint_id: request.endpoint_id,
                    model: request.model,
                    text: request.text,
                    target_lang: request.target_language,
                    prompt: request.prompt,
                    reasoning_effort: request.reasoning_effort,
                })
                .await
                .map_err(assistant_error)
        })
    }
}

pub fn register(registry: &mut CommandRegistry) {
    // ----- assistant.rs -----
    registry.register(
        "app__assistant_send_message",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let session_id: Option<String> = arg(&args, "sessionId")?;
            let text: String = arg(&args, "text")?;
            let locale: Option<String> = arg(&args, "locale")?;
            let result: SendResult = assistant(&state)
                .await?
                .send_message(session_id, text, locale)
                .await
                .map_err(assistant_error)?;
            ok(result)
        },
    );
    registry.register(
        "app__assistant_cancel",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let session_id: String = arg(&args, "sessionId")?;
            assistant(&state).await?.cancel(&session_id);
            ok(serde_json::json!(null))
        },
    );
    registry.register(
        "app__assistant_list_sessions",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            let sessions: Vec<SessionSummary> = assistant(&state).await?.list_sessions();
            ok(sessions)
        },
    );
    registry.register(
        "app__assistant_get_session",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let session_id: String = arg(&args, "sessionId")?;
            let session: Option<Session> = assistant(&state)
                .await?
                .get_session(&session_id)
                .map_err(assistant_error)?;
            ok(session)
        },
    );
    registry.register(
        "app__assistant_new_session",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            let session: Session = assistant(&state).await?.new_session();
            ok(session)
        },
    );
    registry.register(
        "app__assistant_delete_session",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let session_id: String = arg(&args, "sessionId")?;
            assistant(&state).await?.delete_session(&session_id);
            ok(serde_json::json!(null))
        },
    );
    registry.register(
        "app__assistant_set_panel_open",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let session_id: String = arg(&args, "sessionId")?;
            let open: bool = arg(&args, "open")?;
            assistant(&state)
                .await?
                .set_entity_panel_open(&session_id, open);
            ok(serde_json::json!(null))
        },
    );
    registry.register(
        "app__assistant_runtime_status",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            let status: AssistantRuntimeStatus = assistant(&state)
                .await?
                .runtime_status()
                .map_err(assistant_error)?;
            ok(status)
        },
    );
    registry.register(
        "app__assistant_set_session_runtime",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let session_id: String = arg(&args, "sessionId")?;
            let endpoint_id: Option<String> = arg(&args, "endpointId")?;
            let model: Option<String> = arg(&args, "model")?;
            let allow_writes: bool = arg(&args, "allowWrites")?;
            let playbook_mode: PlaybookMode = arg(&args, "playbookMode")?;
            let session: Session = assistant(&state)
                .await?
                .set_session_runtime(
                    &session_id,
                    endpoint_id,
                    model
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string),
                    allow_writes,
                    playbook_mode,
                )
                .map_err(assistant_error)?;
            ok(session)
        },
    );
    registry.register(
        "app__assistant_set_default_runtime",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let endpoint_id: Option<String> = arg(&args, "endpointId")?;
            let model: Option<String> = arg(&args, "model")?;
            let allow_writes: bool = arg(&args, "allowWrites")?;
            let playbook_mode: PlaybookMode = arg(&args, "playbookMode")?;
            let selection: AssistantRuntimeSelection = assistant(&state)
                .await?
                .set_default_runtime(
                    endpoint_id,
                    model
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string),
                    allow_writes,
                    playbook_mode,
                )
                .map_err(assistant_error)?;
            ok(selection)
        },
    );

    // ----- llm_endpoint.rs -----
    registry.register(
        "app__llm_endpoint_follow_custom_proxy",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            let enabled: bool = assistant(&state)
                .await?
                .follow_custom_proxy()
                .map_err(assistant_error)?;
            ok(enabled)
        },
    );
    registry.register(
        "app__llm_endpoint_set_follow_custom_proxy",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let enabled: bool = arg(&args, "enabled")?;
            let enabled: bool = assistant(&state)
                .await?
                .set_follow_custom_proxy(enabled)
                .map_err(assistant_error)?;
            ok(enabled)
        },
    );
    registry.register(
        "app__llm_endpoint_list",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            let endpoints: Vec<LlmEndpointDto> = assistant(&state)
                .await?
                .endpoint_list()
                .map_err(assistant_error)?;
            ok(endpoints)
        },
    );
    registry.register(
        "app__llm_endpoint_upsert",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: LlmEndpointUpsertInput = arg(&args, "input")?;
            let endpoint: LlmEndpointDto = assistant(&state)
                .await?
                .endpoint_upsert(input)
                .map_err(assistant_error)?;
            ok(endpoint)
        },
    );
    registry.register(
        "app__llm_endpoint_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let id: String = arg(&args, "id")?;
            assistant(&state)
                .await?
                .endpoint_delete(&id)
                .map_err(assistant_error)?;
            ok(serde_json::json!(null))
        },
    );
    registry.register(
        "app__llm_endpoint_detect_models",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: LlmEndpointDetectModelsInput = arg(&args, "input")?;
            let result: LlmEndpointDetectModelsResult = assistant(&state)
                .await?
                .endpoint_detect_models(input)
                .await
                .map_err(assistant_error)?;
            ok(result)
        },
    );
    registry.register(
        "app__assistant_reasoning_effort",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            let effort: String = assistant(&state)
                .await?
                .assistant_reasoning_effort()
                .map_err(assistant_error)?;
            ok(effort)
        },
    );
    registry.register(
        "app__assistant_set_reasoning_effort",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let effort: String = arg(&args, "effort")?;
            let effort: String = assistant(&state)
                .await?
                .set_assistant_reasoning_effort(&effort)
                .map_err(assistant_error)?;
            ok(effort)
        },
    );

    // ----- translation.rs -----
    registry.register(
        "app__translation_translate",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: TranslationTranslateInput = arg(&args, "input")?;
            let dispatch: TranslationDispatch = state.translate_dispatch(input).await?;
            let result: TranslationResult = complete_translation(
                dispatch,
                &ServerOpenAiTranslationPort {
                    state: Arc::clone(&state),
                },
            )
            .await
            .map_err(|error| match error {
                TranslationCompletionError::Application(error) => ApiError::from(error),
                TranslationCompletionError::Port(error) => error,
            })?;
            ok(result)
        },
    );

    // ----- profile_backup.rs -----
    registry.register(
        "app__profile_backup_get_settings",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("profile backup settings", move || {
                Ok(state.profile_backup().settings())
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__profile_backup_set_settings",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let settings: ProfileBackupSettings = arg(&args, "settings")?;
            run_blocking("profile backup settings", move || {
                state.profile_backup().set_settings(settings)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__profile_backup_run_manual",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let target_path: String = arg(&args, "targetPath")?;
            run_blocking("manual profile backup", move || {
                state.profile_backup().run_manual(target_path)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__profile_backup_retry_delivery",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("profile backup retry", move || {
                Ok(state.profile_backup().retry_delivery())
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__profile_backup_discard_pending",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("profile backup discard", move || {
                Ok(state.profile_backup().discard_pending())
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__profile_backup_dismiss_error",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("profile backup dismiss error", move || {
                Ok(state.profile_backup().dismiss_error())
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__profile_backup_current_status",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("profile backup status", move || {
                Ok(state.profile_backup_status())
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__profile_restore_validate",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let path: String = arg(&args, "path")?;
            run_blocking("profile restore validation", move || {
                state.profile_backup().validate_restore(path)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__profile_restore_request",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let expected_sha256: String = arg(&args, "expectedSha256")?;
            // No process restart on the server: the web host surfaces the
            // staged restore and the frontend reloads once it is applied.
            let request = run_blocking("profile restore request", move || {
                Ok(state.profile_backup().request_restore(expected_sha256))
            })
            .await?;
            ok(request.outcome)
        },
    );
    registry.register(
        "app__profile_restore_discard_staged",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("profile restore discard", move || {
                state.profile_backup().discard_staged_restore()
            })
            .await?;
            ok(serde_json::json!(null))
        },
    );
    registry.register(
        "app__profile_restore_take_last_result",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("profile restore result", move || {
                state.profile_backup().take_last_restore_result()
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__profile_restore_rollback_state",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("profile rollback state", move || {
                state.profile_backup().restore_rollback_state()
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__profile_restore_clear_rollback",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("profile rollback cleanup", move || {
                Ok(state.profile_backup().clear_restore_rollback())
            })
            .await
            .and_then(ok)
        },
    );

    // ----- vrc_status.rs -----
    registry.register(
        "app__vrc_status_get",
        |state: Arc<ServerRuntimeHostState>, _args| async move { ok(state.vrc_status_snapshot()) },
    );
    registry.register(
        "app__vrc_status_refresh",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.refresh_vrc_status().await?)
        },
    );

    // ----- mcp_server.rs -----
    registry.register(
        "app__mcp_server_status",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(mcp_server(&state).status().await.map_err(mcp_error)?)
        },
    );
    registry.register(
        "app__mcp_server_set_enabled",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let enabled: bool = arg(&args, "enabled")?;
            ok(mcp_server(&state)
                .set_enabled(enabled)
                .await
                .map_err(mcp_error)?)
        },
    );
    registry.register(
        "app__mcp_server_set_allow_vrchat_writes",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let enabled: bool = arg(&args, "enabled")?;
            ok(mcp_server(&state)
                .set_allow_vrchat_writes(enabled)
                .await
                .map_err(mcp_error)?)
        },
    );
    registry.register(
        "app__mcp_server_set_allow_lan_connections",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let enabled: bool = arg(&args, "enabled")?;
            ok(mcp_server(&state)
                .set_allow_lan_connections(enabled)
                .await
                .map_err(mcp_error)?)
        },
    );
    registry.register(
        "app__mcp_server_set_port",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let port: u16 = arg(&args, "port")?;
            ok(mcp_server(&state).set_port(port).await.map_err(mcp_error)?)
        },
    );
    registry.register(
        "app__mcp_server_rotate_token",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(mcp_server(&state).rotate_token().await.map_err(mcp_error)?)
        },
    );

    // ----- privacy_lock.rs (portable: pure password gating state) -----
    registry.register(
        "app__privacy_lock_setup_request_take",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("privacy lock setup request", move || {
                Ok(state.privacy_lock().take_setup_request())
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__privacy_lock_engage",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("privacy lock engage", move || {
                state
                    .privacy_lock()
                    .engage()
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__privacy_lock_unlock",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let password: String = arg(&args, "password")?;
            run_blocking("privacy lock unlock", move || {
                state
                    .privacy_lock()
                    .unlock(&password)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__privacy_lock_password_set",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let password: String = arg(&args, "password")?;
            run_blocking("privacy lock password set", move || {
                state
                    .privacy_lock()
                    .set_password(&password)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__privacy_lock_password_change",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let current_password: String = arg(&args, "currentPassword")?;
            let new_password: String = arg(&args, "newPassword")?;
            run_blocking("privacy lock password change", move || {
                state
                    .privacy_lock()
                    .change_password(&current_password, &new_password)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__privacy_lock_password_clear",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let account_password: String = arg(&args, "accountPassword")?;
            run_blocking("privacy lock password clear", move || {
                state
                    .privacy_lock()
                    .clear_with_account_password(&account_password)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );

    // ----- proxy.rs -----
    registry.register(
        "app__proxy_settings_test",
        |_state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ProxySettingsTestInput = arg(&args, "input")?;
            let result = test_proxy_connectivity(&input.proxy, env!("CARGO_PKG_VERSION")).await?;
            ok(ProxySettingsTestResult {
                normalized_proxy: result.normalized_proxy,
                status: result.status,
            })
        },
    );

    // ----- overlay_activity.rs -----
    registry.register(
        "app__overlay_activity_definitions_get",
        |_state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(overlay_activity_type_definitions())
        },
    );
    registry.register(
        "app__overlay_activity_filters_set",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let filters: OverlayActivityPreferenceFilters = arg(&args, "filters")?;
            run_blocking("overlay activity filters set", move || {
                state.set_overlay_activity_filters(filters)
            })
            .await?;
            ok(serde_json::json!(null))
        },
    );
    registry.register(
        "app__notification_activity_filters_set",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: NotificationActivityFiltersSetInput = arg(&args, "input")?;
            run_blocking("notification activity filters set", move || {
                state.set_notification_activity_filters(input)
            })
            .await?;
            ok(serde_json::json!(null))
        },
    );

    // ----- lifecycle.rs (portable subset) -----
    registry.register(
        "app__runtime_group_instances_refresh",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            state.refresh_runtime_group_instances().await;
            ok(serde_json::json!(null))
        },
    );
    registry.register(
        "app__runtime_background_job_record",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: RuntimeJobRecordInput = arg(&args, "input")?;
            state.record_runtime_job(input);
            ok(serde_json::json!(null))
        },
    );

    // ----- integrations/external_api -----
    registry.register(
        "app__external_api_avatar_search_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ExternalApiAvatarSearchInput = arg(&args, "input")?;
            ok(state
                .external_api()
                .avatar_search(input.url, input.vrcx_id)
                .await?)
        },
    );
    registry.register(
        "app__external_api_youtube_video_metadata_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ExternalApiYoutubeVideoInput = arg(&args, "input")?;
            ok(state
                .external_api()
                .youtube_video_metadata(input.video_id, input.api_key)
                .await?)
        },
    );
    registry.register(
        "app__external_api_github_releases_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ExternalApiUrlInput = arg(&args, "input")?;
            ok(state
                .external_api()
                .github_releases(input.url, input.headers)
                .await?)
        },
    );
    registry.register(
        "app__external_api_github_contributors_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ExternalApiUrlInput = arg(&args, "input")?;
            ok(state
                .external_api()
                .github_contributors(input.url, input.headers)
                .await?)
        },
    );
    registry.register(
        "app__external_api_image_data_url_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ExternalApiImageInput = arg(&args, "input")?;
            ok(state.external_api().image_data_url(input.url).await?)
        },
    );
}
