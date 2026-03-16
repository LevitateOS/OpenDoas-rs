use super::rule::RuleIdentity;

fn parse_uid_spec(spec: &str) -> Option<u32> {
    pwd_grp::getpwnam(spec)
        .ok()
        .flatten()
        .map(|passwd| passwd.uid.into())
        .or_else(|| spec.parse::<u32>().ok())
}

pub fn matches_identity<G: AsRef<str>>(
    identity: &RuleIdentity,
    user: &str,
    uid: u32,
    groups: &[G],
    gids: &[u32],
) -> bool {
    match identity {
        RuleIdentity::User(rule_user) => rule_user == user || rule_user == &uid.to_string(),
        RuleIdentity::Group(group) => {
            groups.iter().any(|value| value.as_ref() == group)
                || group
                    .parse::<u32>()
                    .ok()
                    .is_some_and(|value| gids.contains(&value))
        }
    }
}

pub fn matches_target(target_spec: &str, target_uid: u32) -> bool {
    parse_uid_spec(target_spec) == Some(target_uid)
}

#[cfg(test)]
mod tests {
    use super::matches_identity;
    use crate::policy::rule::RuleIdentity;

    #[test]
    fn numeric_group_rule_matches_without_resolved_names() {
        let identity = RuleIdentity::Group(String::from("4242"));

        assert!(matches_identity(
            &identity,
            "alice",
            1000,
            &[] as &[&str],
            &[4242]
        ));
    }
}
