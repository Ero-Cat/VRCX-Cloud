use super::runtime_state::player_key;

pub(super) fn resolve_leave_key(
    candidates: &[(String, String)],
    user_id: &str,
    display_name: &str,
) -> Option<String> {
    let key = player_key(user_id, display_name);
    if candidates.iter().any(|(candidate, _)| candidate == &key) {
        return Some(key);
    }
    let display_name = display_name.trim();
    if display_name.is_empty() {
        return None;
    }
    let mut matches = candidates
        .iter()
        .filter(|(_, name)| name.trim().eq_ignore_ascii_case(display_name));
    let (key, _) = matches.next()?;
    matches.next().is_none().then(|| key.clone())
}
