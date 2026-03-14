use super::{
    decision::Decision,
    identity::{matches_identity, matches_target},
    rule::{Rule, RuleOpts, Rules},
};

impl Rules {
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
        let matched = self
            .iter()
            .filter(|rule| rule_matches(rule, user, uid, groups, gids, cmd, args, target_uid))
            .last();

        Decision::from_rule(matched)
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
        self.decide(user, uid, groups, gids, cmd, args, target_uid)
            .permit_opts()
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
