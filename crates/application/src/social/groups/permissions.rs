use std::collections::HashMap;

use serde_json::Value;
use vrcx_0_core::json::scalar_text_array;
use vrcx_0_core::GroupPermission;

pub(super) fn parse_permission_map(value: &Value) -> HashMap<String, Vec<GroupPermission>> {
    value
        .as_object()
        .map(|object| {
            object
                .iter()
                .map(|(group_id, permissions)| {
                    (group_id.clone(), parse_permissions(Some(permissions)))
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn permissions_for_group(
    group: &Value,
    permission_map: &HashMap<String, Vec<GroupPermission>>,
    group_id: &str,
) -> Vec<GroupPermission> {
    if let Some(permissions) = permission_map.get(group_id) {
        return permissions.clone();
    }
    group
        .as_object()
        .and_then(|object| object.get("myMember"))
        .and_then(Value::as_object)
        .map(|member| parse_permissions(member.get("permissions")))
        .unwrap_or_default()
}

fn parse_permissions(value: Option<&Value>) -> Vec<GroupPermission> {
    scalar_text_array(value)
        .into_iter()
        .map(GroupPermission::from)
        .collect()
}

pub(super) fn has_permission(
    permissions: &[GroupPermission],
    permission: &GroupPermission,
) -> bool {
    permissions
        .iter()
        .any(|value| value == &GroupPermission::All || value == permission)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn permissions_for_group_prefers_permission_map_over_my_member() {
        let permission_map = parse_permission_map(&json!({ "grp_1": ["group-bans-manage"] }));
        let group = json!({
            "id": "grp_1",
            "myMember": { "permissions": ["group-members-remove"] }
        });

        assert_eq!(
            permissions_for_group(&group, &permission_map, "grp_1"),
            vec![GroupPermission::BansManage]
        );
    }

    #[test]
    fn permissions_for_group_falls_back_to_my_member_when_missing_from_map() {
        let permission_map = HashMap::new();
        let group = json!({
            "id": "grp_1",
            "myMember": { "permissions": ["group-members-remove"] }
        });

        assert_eq!(
            permissions_for_group(&group, &permission_map, "grp_1"),
            vec![GroupPermission::MembersRemove]
        );
    }

    #[test]
    fn permissions_for_group_returns_empty_when_both_sources_are_missing() {
        let group = json!({ "id": "grp_1" });
        assert!(permissions_for_group(&group, &HashMap::new(), "grp_1").is_empty());
    }

    #[test]
    fn has_permission_matches_wildcard_and_exact_values() {
        assert!(has_permission(
            &[GroupPermission::All],
            &GroupPermission::BansManage
        ));
        assert!(has_permission(
            &[GroupPermission::BansManage],
            &GroupPermission::BansManage
        ));
        assert!(!has_permission(
            &[GroupPermission::InvitesManage],
            &GroupPermission::BansManage
        ));
    }
}
