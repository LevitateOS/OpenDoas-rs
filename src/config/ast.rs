use crate::policy::rule::{Rule, RuleAction, RuleIdentity, RuleOpts, Rules};

pub type ConfigRule = Rule;
pub type ConfigAction = RuleAction;
pub type ConfigIdentity = RuleIdentity;
pub type ConfigRuleOpts = RuleOpts;

#[derive(Clone, Debug)]
pub struct ConfigFile {
    pub rules: Rules,
}

impl ConfigFile {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

impl From<Rules> for ConfigFile {
    fn from(rules: Rules) -> Self {
        Self::new(rules)
    }
}

impl AsRef<Rules> for ConfigFile {
    fn as_ref(&self) -> &Rules {
        &self.rules
    }
}
