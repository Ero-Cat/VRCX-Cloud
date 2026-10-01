use futures_util::future::BoxFuture;

use std::{collections::HashSet, sync::Mutex, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use vrcx_0_application_core::{
    vrchat_api::{VrchatApiRequest, VrchatScope},
    Error, RemoteMutationGate, Result, RuntimeAuthScope, RuntimeAuthScopeSnapshot, RuntimeEventBus,
};
use vrcx_0_core::OwnerId;

use crate::remote::VrchatRequestPort;

use super::service::{GroupRemoteRequest, GroupRemoteRequests};
use super::types::{
    GroupMemberPatch, GroupMemberVisibility, VrchatGroupIdInput, VrchatGroupMemberPropsInput,
};

pub const GROUP_MEMBERSHIP_BATCH_MAX_TARGETS: usize = 250;
const GROUP_MEMBERSHIP_REMOTE_INTERVAL: Duration = Duration::from_millis(250);
const SCOPE_CHANGED_MESSAGE: &str = "Group membership batch authentication scope changed.";

#[derive(Default)]
pub struct GroupMembershipBatchCoordinator {
    active: Mutex<HashSet<String>>,
}

struct GroupMembershipBatchGuard<'a> {
    coordinator: &'a GroupMembershipBatchCoordinator,
    key: String,
}

impl GroupMembershipBatchCoordinator {
    fn try_begin(&self, owner_user_id: &OwnerId) -> Result<GroupMembershipBatchGuard<'_>> {
        let key = owner_user_id.to_string();
        let mut active = self.active.lock().map_err(|_| {
            Error::Custom("Group membership batch coordinator is unavailable.".into())
        })?;
        if !active.insert(key.clone()) {
            return Err(Error::Custom(
                "A group membership batch is already running for this account.".into(),
            ));
        }
        Ok(GroupMembershipBatchGuard {
            coordinator: self,
            key,
        })
    }
}

impl Drop for GroupMembershipBatchGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut active) = self.coordinator.active.lock() {
            active.remove(&self.key);
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, specta::Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GroupMembershipBatchAction {
    Leave,
    SetVisibility { visibility: GroupMemberVisibility },
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct GroupMembershipBatchInput {
    pub expected_owner_user_id: OwnerId,
    pub expected_endpoint: String,
    pub action: GroupMembershipBatchAction,
    #[serde(default)]
    pub group_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum GroupMembershipBatchItemState {
    Applied,
    Failed,
    NotAttempted,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct GroupMembershipBatchItemResult {
    pub group_id: String,
    pub state: GroupMembershipBatchItemState,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct GroupMembershipBatchResult {
    pub owner_user_id: OwnerId,
    pub endpoint: String,
    pub total: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub items: Vec<GroupMembershipBatchItemResult>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct GroupMembershipBatchProgress {
    pub owner_user_id: OwnerId,
    pub endpoint: String,
    pub completed: u32,
    pub total: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GroupMembershipRemoteOutcome {
    Applied,
    AppliedScopeChanged,
}

enum GroupMembershipOperation<'a> {
    Leave {
        group_id: &'a str,
    },
    SetVisibility {
        group_id: &'a str,
        user_id: &'a str,
        visibility: GroupMemberVisibility,
    },
}

trait GroupMembershipBatchActions: Send + Sync {
    fn execute<'a>(
        &'a self,
        operation: GroupMembershipOperation<'a>,
    ) -> BoxFuture<'a, Result<GroupMembershipRemoteOutcome>>;
    fn scope_matches(&self) -> bool;
    fn current_user_id(&self) -> &str;
    fn current_endpoint(&self) -> &str;
    fn report_progress(&self, _progress: GroupMembershipBatchProgress) {}
    fn wait_for_remote_slot<'a>(&'a self) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
}

pub struct VrchatGroupMembershipBatchActions<'a> {
    remote: &'a dyn VrchatRequestPort,
    remote_requests: &'a dyn GroupRemoteRequests,
    auth_scope: &'a RuntimeAuthScope,
    pub expected_scope: RuntimeAuthScopeSnapshot,
    event_bus: RuntimeEventBus,
    remote_mutation_gate: &'a RemoteMutationGate,
}

impl<'a> VrchatGroupMembershipBatchActions<'a> {
    pub fn new(
        remote: &'a dyn VrchatRequestPort,
        remote_requests: &'a dyn GroupRemoteRequests,
        auth_scope: &'a RuntimeAuthScope,
        expected_scope: RuntimeAuthScopeSnapshot,
        event_bus: RuntimeEventBus,
        remote_mutation_gate: &'a RemoteMutationGate,
    ) -> Self {
        Self {
            remote,
            remote_requests,
            auth_scope,
            expected_scope,
            event_bus,
            remote_mutation_gate,
        }
    }

    async fn execute_request(
        &self,
        mut request: VrchatApiRequest,
        action: &str,
    ) -> Result<GroupMembershipRemoteOutcome> {
        ensure_scope_matches(&self.auth_scope.snapshot(), &self.expected_scope)?;
        request.endpoint = Some(self.expected_scope.endpoint.clone());
        let response = self.remote.send(request, VrchatScope::Vrchat).await?;
        let fallback_payload = Value::String(response.data.clone());
        if !(200..300).contains(&response.status) {
            return Err(Error::Custom(response_error_message(
                &serde_json::from_str::<Value>(&response.data).unwrap_or(fallback_payload),
                response.status,
                action,
            )));
        }
        let payload = if response.data.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str::<Value>(&response.data).map_err(|error| {
                Error::Custom(format!("VRChat {action} returned invalid JSON: {error}"))
            })?
        };
        if payload.get("error").is_some() {
            return Err(Error::Custom(response_error_message(
                &payload,
                response.status,
                action,
            )));
        }
        if self
            .auth_scope
            .snapshot()
            .generation_matches(&self.expected_scope)
        {
            Ok(GroupMembershipRemoteOutcome::Applied)
        } else {
            Ok(GroupMembershipRemoteOutcome::AppliedScopeChanged)
        }
    }
}

impl GroupMembershipBatchActions for VrchatGroupMembershipBatchActions<'_> {
    fn execute<'a>(
        &'a self,
        operation: GroupMembershipOperation<'a>,
    ) -> BoxFuture<'a, Result<GroupMembershipRemoteOutcome>> {
        Box::pin(async move {
            let (built, action) = match operation {
                GroupMembershipOperation::Leave { group_id } => {
                    let built = self.remote_requests.build(GroupRemoteRequest::Leave(
                        VrchatGroupIdInput {
                            group_id: group_id.to_string(),
                        },
                    ))?;
                    (built, "group leave")
                }
                GroupMembershipOperation::SetVisibility {
                    group_id,
                    user_id,
                    visibility,
                } => {
                    let built = self
                        .remote_requests
                        .build(GroupRemoteRequest::SetMemberProps(
                            VrchatGroupMemberPropsInput {
                                group_id: group_id.to_string(),
                                user_id: user_id.to_string(),
                                params: GroupMemberPatch {
                                    visibility: Some(visibility),
                                    ..GroupMemberPatch::default()
                                },
                            },
                        ))?;
                    (built, "group member visibility update")
                }
            };
            self.execute_request(built.request, action).await
        })
    }

    fn scope_matches(&self) -> bool {
        self.auth_scope
            .snapshot()
            .generation_matches(&self.expected_scope)
    }

    fn current_user_id(&self) -> &str {
        &self.expected_scope.current_user_id
    }

    fn current_endpoint(&self) -> &str {
        &self.expected_scope.endpoint
    }

    fn report_progress(&self, progress: GroupMembershipBatchProgress) {
        self.event_bus.emit(progress);
    }

    fn wait_for_remote_slot<'a>(&'a self) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            self.remote_mutation_gate
                .wait(&self.expected_scope, GROUP_MEMBERSHIP_REMOTE_INTERVAL)
                .await;
        })
    }
}

pub async fn run_group_membership_batch(
    coordinator: &GroupMembershipBatchCoordinator,
    actions: &VrchatGroupMembershipBatchActions<'_>,
    input: GroupMembershipBatchInput,
) -> Result<GroupMembershipBatchResult> {
    if input.expected_owner_user_id.as_str().trim() != actions.expected_scope.current_user_id
        || input.expected_endpoint.trim() != actions.expected_scope.endpoint
    {
        return Err(Error::Custom(
            "Group membership batch is stale for the current auth scope.".into(),
        ));
    }
    let _guard = coordinator.try_begin(&OwnerId::new(
        actions.expected_scope.current_user_id.clone(),
    ))?;
    run_group_membership_batch_with_actions(actions, input).await
}

async fn run_group_membership_batch_with_actions(
    actions: &dyn GroupMembershipBatchActions,
    input: GroupMembershipBatchInput,
) -> Result<GroupMembershipBatchResult> {
    let prepared = prepare_input(input)?;
    let owner_user_id = actions.current_user_id().to_string();
    let endpoint = actions.current_endpoint().to_string();
    let mut items = prepared
        .group_ids
        .iter()
        .map(|group_id| not_attempted(group_id))
        .collect::<Vec<_>>();
    let mut stop_after = None;
    let total = items.len();
    let mut completed = 0;

    for (index, group_id) in prepared.group_ids.iter().enumerate() {
        if !actions.scope_matches() {
            stop_after = Some((index, SCOPE_CHANGED_MESSAGE.to_string()));
            break;
        }

        actions.wait_for_remote_slot().await;
        let operation = match prepared.action {
            GroupMembershipBatchAction::Leave => GroupMembershipOperation::Leave { group_id },
            GroupMembershipBatchAction::SetVisibility { visibility } => {
                GroupMembershipOperation::SetVisibility {
                    group_id,
                    user_id: actions.current_user_id(),
                    visibility,
                }
            }
        };
        match actions.execute(operation).await {
            Ok(outcome) => {
                items[index] = applied(group_id);
                if outcome == GroupMembershipRemoteOutcome::AppliedScopeChanged {
                    stop_after = Some((index + 1, SCOPE_CHANGED_MESSAGE.to_string()));
                }
            }
            Err(error) => {
                items[index] = failed(group_id, error.to_string());
            }
        }

        actions.report_progress(GroupMembershipBatchProgress {
            owner_user_id: OwnerId::new(owner_user_id.clone()),
            endpoint: endpoint.clone(),
            completed: crate::wire_count(index + 1),
            total: crate::wire_count(total),
        });
        completed = index + 1;
        if stop_after.is_some() {
            break;
        }
    }

    let scope_error = stop_after.map(|(start, message)| {
        for item in items.iter_mut().skip(start) {
            item.message = message.clone();
        }
        message
    });
    if completed < total {
        actions.report_progress(GroupMembershipBatchProgress {
            owner_user_id: OwnerId::new(owner_user_id.clone()),
            endpoint: endpoint.clone(),
            completed: crate::wire_count(total),
            total: crate::wire_count(total),
        });
    }
    Ok(summarize(
        OwnerId::new(owner_user_id),
        endpoint,
        items,
        scope_error,
    ))
}

struct PreparedGroupMembershipBatch {
    action: GroupMembershipBatchAction,
    group_ids: Vec<String>,
}

fn prepare_input(input: GroupMembershipBatchInput) -> Result<PreparedGroupMembershipBatch> {
    let mut seen = HashSet::new();
    let mut group_ids = Vec::new();
    for group_id in input.group_ids {
        let group_id = require_group_id(group_id)?;
        if seen.insert(group_id.clone()) {
            group_ids.push(group_id);
        }
    }
    if group_ids.is_empty() {
        return Err(Error::Custom(
            "Group membership batch requires at least one group.".into(),
        ));
    }
    if group_ids.len() > GROUP_MEMBERSHIP_BATCH_MAX_TARGETS {
        return Err(Error::Custom(format!(
            "Group membership batch cannot exceed {GROUP_MEMBERSHIP_BATCH_MAX_TARGETS} groups."
        )));
    }
    Ok(PreparedGroupMembershipBatch {
        action: input.action,
        group_ids,
    })
}

fn summarize(
    owner_user_id: OwnerId,
    endpoint: String,
    items: Vec<GroupMembershipBatchItemResult>,
    scope_error: Option<String>,
) -> GroupMembershipBatchResult {
    let succeeded = items
        .iter()
        .filter(|item| item.state == GroupMembershipBatchItemState::Applied)
        .count();
    let last_error = scope_error.or_else(|| {
        items
            .iter()
            .rev()
            .find(|item| !item.message.is_empty())
            .map(|item| item.message.clone())
    });
    GroupMembershipBatchResult {
        owner_user_id,
        endpoint,
        total: crate::wire_count(items.len()),
        succeeded: crate::wire_count(succeeded),
        failed: crate::wire_count(items.len() - succeeded),
        items,
        last_error,
    }
}

fn require_group_id(value: String) -> Result<String> {
    let value = value.trim().to_string();
    if value.starts_with("grp_") && value.len() > "grp_".len() {
        Ok(value)
    } else {
        Err(Error::Custom(
            "Group membership batch contains an invalid group id.".into(),
        ))
    }
}

fn applied(group_id: &str) -> GroupMembershipBatchItemResult {
    GroupMembershipBatchItemResult {
        group_id: group_id.to_string(),
        state: GroupMembershipBatchItemState::Applied,
        message: String::new(),
    }
}

fn failed(group_id: &str, message: String) -> GroupMembershipBatchItemResult {
    GroupMembershipBatchItemResult {
        group_id: group_id.to_string(),
        state: GroupMembershipBatchItemState::Failed,
        message,
    }
}

fn not_attempted(group_id: &str) -> GroupMembershipBatchItemResult {
    GroupMembershipBatchItemResult {
        group_id: group_id.to_string(),
        state: GroupMembershipBatchItemState::NotAttempted,
        message: String::new(),
    }
}

fn ensure_scope_matches(
    current: &RuntimeAuthScopeSnapshot,
    expected: &RuntimeAuthScopeSnapshot,
) -> Result<()> {
    crate::scope_gate::ensure_snapshot_scope_matches(current, expected, "Group membership batch")
}

fn response_error_message(payload: &Value, status: i32, action: &str) -> String {
    crate::scope_gate::response_error_message(payload, status, action)
}

#[cfg(test)]
mod tests;
