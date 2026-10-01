use vrcx_0_core::social_circles::friend_circles;

fn ids(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn edges(values: &[(&str, &str)]) -> Vec<(String, String)> {
    values
        .iter()
        .map(|(left, right)| ((*left).to_string(), (*right).to_string()))
        .collect()
}

#[test]
fn friend_circles_groups_chain_and_clique_components() {
    let circles = friend_circles(
        &ids(&["usr_c", "usr_a", "usr_b", "usr_d", "usr_e", "usr_f"]),
        &edges(&[
            ("usr_a", "usr_b"),
            ("usr_b", "usr_c"),
            ("usr_d", "usr_e"),
            ("usr_e", "usr_d"),
            ("usr_f", "usr_outside"),
        ]),
    );

    assert_eq!(circles.len(), 2);
    assert_eq!(circles[0].members, ids(&["usr_a", "usr_b", "usr_c"]));
    assert_eq!(circles[1].members, ids(&["usr_d", "usr_e"]));
}

#[test]
fn friend_circles_omits_isolated_and_unknown_members() {
    let circles = friend_circles(
        &ids(&["usr_a", "usr_b", "usr_c"]),
        &edges(&[("usr_a", "usr_unknown")]),
    );

    assert!(circles.is_empty());
}

#[test]
fn friend_circles_orders_equal_size_components_by_first_member() {
    let circles = friend_circles(
        &ids(&["usr_d", "usr_c", "usr_b", "usr_a"]),
        &edges(&[("usr_c", "usr_d"), ("usr_a", "usr_b")]),
    );

    assert_eq!(circles.len(), 2);
    assert_eq!(circles[0].members, ids(&["usr_a", "usr_b"]));
    assert_eq!(circles[1].members, ids(&["usr_c", "usr_d"]));
}
