use crate::{
    config::validate::validate_rules,
    policy::rule::{EnvDirective, Rule, RuleAction, RuleIdentity, RuleOpts, Rules},
};

#[derive(Clone, Debug)]
struct Token {
    text: String,
    pure: bool,
    line: usize,
}

impl TryFrom<&str> for Rules {
    type Error = String;

    fn try_from(config: &str) -> Result<Self, Self::Error> {
        let mut rules = Rules { rules: Vec::new() };

        for (line_index, raw_line) in config.lines().enumerate() {
            let line_no = line_index + 1;
            let tokens = tokenize_line(raw_line, line_no)?;
            if tokens.is_empty() {
                continue;
            }

            rules.push(parse_rule_line(&tokens)?);
        }

        validate_rules(&rules)?;
        Ok(rules)
    }
}

fn parse_rule_line(tokens: &[Token]) -> Result<Rule, String> {
    let line = tokens[0].line;
    let action_token = &tokens[0];
    let action = if action_token.pure {
        match action_token.text.as_str() {
            "permit" => RuleAction::Permit,
            "deny" => RuleAction::Deny,
            _ => return Err(syntax_error(line)),
        }
    } else {
        return Err(syntax_error(line));
    };

    let mut index = 1;
    let mut rule = blank_rule();
    rule.action = action.clone();

    while index < tokens.len() {
        let token = &tokens[index];
        if !token.pure || !matches!(
            token.text.as_str(),
            "nopass" | "nolog" | "persist" | "keepenv" | "setenv"
        ) {
            break;
        }

        if matches!(action, RuleAction::Deny) {
            return Err(syntax_error(line));
        }

        match token.text.as_str() {
            "nopass" => rule.options.nopass = true,
            "nolog" => rule.options.nolog = true,
            "persist" => rule.options.persist = true,
            "keepenv" => rule.options.keepenv = true,
            "setenv" => {
                if rule.options.setenv.is_some() {
                    return Err(format!("can't have two setenv sections at line {}", line));
                }
                let (env, next_index) = parse_setenv(tokens, index + 1)?;
                rule.options.setenv = Some(env);
                index = next_index;
                continue;
            }
            _ => (),
        }

        index += 1;
    }

    if rule.options.nopass && rule.options.persist {
        return Err(format!("can't combine nopass and persist at line {}", line));
    }

    let ident = tokens.get(index).ok_or_else(|| syntax_error(line))?;
    rule.identity = if let Some(group) = ident.text.strip_prefix(':') {
        RuleIdentity::Group(group.into())
    } else {
        RuleIdentity::User(ident.text.clone())
    };
    index += 1;

    while index < tokens.len() {
        match tokens[index].text.as_str() {
            "as" if tokens[index].pure => {
                index += 1;
                let target = tokens.get(index).ok_or_else(|| syntax_error(line))?;
                rule.target = Some(target.text.clone());
                index += 1;
            }
            "cmd" if tokens[index].pure => {
                index += 1;
                let command = tokens.get(index).ok_or_else(|| syntax_error(line))?;
                rule.command = Some(command.text.clone());
                index += 1;
            }
            "args" if tokens[index].pure => {
                index += 1;
                rule.args = Some(tokens[index..].iter().map(|token| token.text.clone()).collect());
                index = tokens.len();
            }
            _ => return Err(syntax_error(line)),
        }
    }

    Ok(rule)
}

fn parse_setenv(tokens: &[Token], mut index: usize) -> Result<(Vec<EnvDirective>, usize), String> {
    let line = tokens
        .get(index)
        .or_else(|| tokens.last())
        .map(|token| token.line)
        .unwrap_or(1);
    if tokens.get(index).map(|token| token.text.as_str()) != Some("{") {
        return Err(syntax_error(line));
    }
    index += 1;

    let mut env = Vec::new();
    while index < tokens.len() {
        let token = &tokens[index];
        if token.text == "}" {
            return Ok((env, index + 1));
        }

        if let Some(name) = token.text.strip_prefix('-') {
            env.push(EnvDirective::Remove(name.into()));
        } else if let Some((key, value)) = token.text.split_once('=') {
            env.push(EnvDirective::Set(key.into(), value.into()));
        } else {
            env.push(EnvDirective::Inherit(token.text.clone()));
        }
        index += 1;
    }

    Err(syntax_error(line))
}

fn tokenize_line(line: &str, line_no: usize) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut pure = true;
    let mut quote = false;
    let mut escape = false;
    let mut forced = false;

    let flush = |tokens: &mut Vec<Token>, current: &mut String, pure: &mut bool, forced: &mut bool| {
        if !current.is_empty() || *forced {
            tokens.push(Token {
                text: std::mem::take(current),
                pure: *pure,
                line: line_no,
            });
            *pure = true;
            *forced = false;
        }
    };

    for chr in line.chars() {
        if escape {
            current.push(chr);
            escape = false;
            continue;
        }

        match chr {
            '#' if !quote => break,
            '"' => {
                pure = false;
                quote = !quote;
                forced = true;
            }
            '\\' => {
                escape = true;
            }
            ' ' | '\t' if !quote => flush(&mut tokens, &mut current, &mut pure, &mut forced),
            '{' | '}' if !quote => {
                flush(&mut tokens, &mut current, &mut pure, &mut forced);
                tokens.push(Token {
                    text: chr.to_string(),
                    pure: true,
                    line: line_no,
                });
            }
            _ => current.push(chr),
        }
    }

    if escape {
        return Err(format!("unterminated escape at line {}", line_no));
    }
    if quote {
        return Err(format!("syntax error: unterminated quotes at line {}", line_no));
    }

    flush(&mut tokens, &mut current, &mut pure, &mut forced);
    Ok(tokens)
}

fn syntax_error(line: usize) -> String {
    format!("syntax error at line {}", line)
}

fn blank_rule() -> Rule {
    Rule {
        action: RuleAction::Deny,
        options: RuleOpts {
            nopass: false,
            nolog: false,
            persist: false,
            keepenv: false,
            setenv: None,
        },
        identity: RuleIdentity::User(String::new()),
        target: None,
        command: None,
        args: None,
    }
}
