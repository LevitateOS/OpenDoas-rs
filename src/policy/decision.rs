use super::rule::{Rule, RuleAction, RuleOpts};

#[derive(Clone, Debug)]
pub enum Decision {
    Deny,
    Permit(RuleOpts),
}

impl Decision {
    pub fn from_rule(rule: Option<&Rule>) -> Self {
        match rule {
            Some(rule) => match rule.action {
                RuleAction::Permit => Self::Permit(rule.options.clone()),
                RuleAction::Deny => Self::Deny,
            },
            None => Self::Deny,
        }
    }

    pub fn check_output(&self) -> String {
        match self {
            Self::Deny => "deny\n".into(),
            Self::Permit(rule_opts) => {
                format!("permit{}\n", if rule_opts.nopass { " nopass" } else { "" })
            }
        }
    }

    pub fn check_exit_code(&self) -> i32 {
        match self {
            Self::Deny => 1,
            Self::Permit(_) => 0,
        }
    }

    pub fn permit_opts(self) -> Option<RuleOpts> {
        match self {
            Self::Deny => None,
            Self::Permit(rule_opts) => Some(rule_opts),
        }
    }
}
