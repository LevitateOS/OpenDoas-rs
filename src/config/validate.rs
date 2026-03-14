use std::{fs::Metadata, os::unix::fs::MetadataExt};

use crate::policy::rule::{Rule, RuleIdentity, Rules};

pub fn validate_rules(rules: &Rules) -> Result<(), String> {
    for rule in rules.iter() {
        validate_rule(rule)?;
    }

    Ok(())
}

pub fn validate_runtime_config_metadata(path: &str, metadata: &Metadata) -> Result<(), String> {
    if (metadata.mode() & 0o022) != 0 {
        return Err(format!("{path} is writable by group or other"));
    }
    if metadata.uid() != 0 {
        return Err(format!("{path} is not owned by root"));
    }

    Ok(())
}

fn validate_rule(rule: &Rule) -> Result<(), String> {
    match &rule.identity {
        RuleIdentity::User(name) | RuleIdentity::Group(name) if name.is_empty() => {
            return Err(String::from("Rule identity cannot be empty"));
        }
        _ => (),
    }

    if rule.options.nopass && rule.options.persist {
        return Err(String::from("can't combine nopass and persist"));
    }

    Ok(())
}
