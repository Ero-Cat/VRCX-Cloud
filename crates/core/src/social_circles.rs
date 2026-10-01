use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FriendCircle {
    pub members: Vec<String>,
}

pub fn friend_circles(friend_ids: &[String], edges: &[(String, String)]) -> Vec<FriendCircle> {
    let mut ids = friend_ids
        .iter()
        .filter(|id| !id.trim().is_empty())
        .map(String::as_str)
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    if ids.len() < 2 {
        return Vec::new();
    }

    let index_by_id = ids
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index))
        .collect::<HashMap<_, _>>();
    let mut union_find = UnionFind::new(ids.len());
    for (left, right) in edges {
        let left = left.trim();
        let right = right.trim();
        if left == right {
            continue;
        }
        let (Some(&left_index), Some(&right_index)) =
            (index_by_id.get(left), index_by_id.get(right))
        else {
            continue;
        };
        union_find.union(left_index, right_index);
    }

    let mut components: HashMap<usize, Vec<String>> = HashMap::new();
    for id in ids {
        let index = index_by_id[id];
        components
            .entry(union_find.find(index))
            .or_default()
            .push(id.to_string());
    }
    let mut circles = components
        .into_values()
        .filter(|members| members.len() >= 2)
        .map(|mut members| {
            members.sort();
            FriendCircle { members }
        })
        .collect::<Vec<_>>();
    circles.sort_by(|left, right| {
        right
            .members
            .len()
            .cmp(&left.members.len())
            .then_with(|| left.members[0].cmp(&right.members[0]))
    });
    circles
}

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
        }
    }

    fn find(&mut self, index: usize) -> usize {
        if self.parent[index] != index {
            self.parent[index] = self.find(self.parent[index]);
        }
        self.parent[index]
    }

    fn union(&mut self, left: usize, right: usize) {
        let left_root = self.find(left);
        let right_root = self.find(right);
        if left_root == right_root {
            return;
        }
        if self.rank[left_root] < self.rank[right_root] {
            self.parent[left_root] = right_root;
        } else if self.rank[left_root] > self.rank[right_root] {
            self.parent[right_root] = left_root;
        } else {
            self.parent[right_root] = left_root;
            self.rank[left_root] += 1;
        }
    }
}
