use super::*;

#[test]
fn runtime_snapshot_distinguishes_unavailable_empty_and_wrong_visit() {
    let mut live = crate::RuntimeSnapshot::default();
    assert_eq!(
        player_list_runtime_snapshot(&live, "")
            .context
            .player_facts_known,
        Some(false)
    );
    live.ready = true;
    live.has_player_events = true;
    live.location = "wrld_current:1".into();
    let empty = player_list_runtime_snapshot(&live, "wrld_current:1");
    assert_eq!(empty.context.player_facts_known, Some(true));
    assert!(empty.players.is_empty());
    let wrong = player_list_runtime_snapshot(&live, "wrld_previous:2");
    assert_eq!(wrong.context.player_facts_known, Some(false));
    assert!(wrong.players.is_empty());
}
