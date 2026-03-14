#[derive(Clone, Debug)]
pub enum RuleAction {
    Permit,
    Deny,
}

#[derive(Clone, Debug)]
pub enum EnvDirective {
    Inherit(String),
    Set(String, String),
    Remove(String),
}

#[derive(Clone, Debug)]
pub struct RuleOpts {
    pub nopass: bool,
    pub nolog: bool,
    pub persist: bool,
    pub keepenv: bool,
    pub setenv: Option<Vec<EnvDirective>>,
}

#[derive(Clone, Debug)]
pub enum RuleIdentity {
    User(String),
    Group(String),
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub action: RuleAction,
    pub options: RuleOpts,
    pub identity: RuleIdentity,
    pub target: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Rules {
    pub rules: Vec<Rule>,
}

impl Rules {
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter()
    }

    pub fn push(&mut self, rule: Rule) {
        self.rules.push(rule);
    }
}
