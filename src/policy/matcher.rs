use super::{
    decision::Decision,
    identity::{matches_identity, matches_target},
    rule::{Rule, RuleOpts, Rules},
};

impl Rules {
    pub fn matched_rule<'a, G: AsRef<str>, A: AsRef<str>>(
        &'a self,
        user: &str,
        uid: u32,
        groups: &'a [G],
        gids: &[u32],
        cmd: &str,
        args: &'a [A],
        target_uid: u32,
    ) -> Option<&'a Rule> {
        self.iter()
            .filter(|rule| rule_matches(rule, user, uid, groups, gids, cmd, args, target_uid))
            .last()
    }

    pub fn decide<G: AsRef<str>, A: AsRef<str>>(
        &self,
        user: &str,
        uid: u32,
        groups: &[G],
        gids: &[u32],
        cmd: &str,
        args: &[A],
        target_uid: u32,
    ) -> Decision {
        Decision::from_rule(self.matched_rule(user, uid, groups, gids, cmd, args, target_uid))
    }

    pub fn r#match<'a, G: AsRef<str>, A: AsRef<str>>(
        &self,
        user: &str,
        uid: u32,
        groups: &'a [G],
        gids: &[u32],
        cmd: &str,
        args: &'a [A],
        target_uid: u32,
    ) -> Option<RuleOpts> {
        self.matched_rule(user, uid, groups, gids, cmd, args, target_uid)
            .and_then(|rule| match rule.action {
                super::rule::RuleAction::Permit => Some(rule.options.clone()),
                super::rule::RuleAction::Deny => None,
            })
    }
}

fn rule_matches<G: AsRef<str>, A: AsRef<str>>(
    rule: &Rule,
    user: &str,
    uid: u32,
    groups: &[G],
    gids: &[u32],
    cmd: &str,
    args: &[A],
    target_uid: u32,
) -> bool {
    if !matches_identity(&rule.identity, user, uid, groups, gids) {
        return false;
    }
    if let Some(rule_cmd) = &rule.command {
        if rule_cmd != cmd {
            return false;
        }
    }
    if let Some(rule_args) = &rule.args {
        if rule_args.len() != args.len() {
            return false;
        }
        if !rule_args
            .iter()
            .zip(args)
            .all(|(left, right)| left == right.as_ref())
        {
            return false;
        }
    }
    if let Some(rule_target) = &rule.target {
        if !matches_target(rule_target, target_uid) {
            return false;
        }
    }

    true
}
