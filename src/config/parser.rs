use crate::{
    config::validate::validate_rules,
    policy::rule::{EnvDirective, Rule, RuleAction, RuleIdentity, RuleOpts, Rules},
};

const MAX_TOKEN_LEN: usize = 1023;

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

        for tokens in tokenize_config(config)? {
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
        if !token.pure
            || !matches!(
                token.text.as_str(),
                "nopass" | "nolog" | "persist" | "keepenv" | "setenv"
            )
        {
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

    if keyword_at(tokens, index, "as") {
        index += 1;
        let target = tokens.get(index).ok_or_else(|| syntax_error(line))?;
        rule.target = Some(target.text.clone());
        index += 1;
    }

    if keyword_at(tokens, index, "cmd") {
        index += 1;
        let command = tokens.get(index).ok_or_else(|| syntax_error(line))?;
        rule.command = Some(command.text.clone());
        index += 1;

        if keyword_at(tokens, index, "args") {
            index += 1;
            rule.args = Some(
                tokens[index..]
                    .iter()
                    .map(|token| token.text.clone())
                    .collect(),
            );
            index = tokens.len();
        }
    }

    if index != tokens.len() {
        return Err(syntax_error(line));
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

fn tokenize_config(config: &str) -> Result<Vec<Vec<Token>>, String> {
    let mut lines = Vec::new();
    let mut line_tokens = Vec::new();
    let mut current = String::new();
    let mut pure = true;
    let mut quote = false;
    let mut escape = false;
    let mut forced = false;
    let mut comment = false;
    let mut logical_line = 1;
    let mut physical_line = 1;

    let flush = |tokens: &mut Vec<Token>,
                 current: &mut String,
                 pure: &mut bool,
                 forced: &mut bool,
                 line_no: usize| {
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

    for chr in config.chars() {
        if comment {
            if chr == '\n' {
                if !line_tokens.is_empty() {
                    lines.push(std::mem::take(&mut line_tokens));
                }
                comment = false;
                physical_line += 1;
                logical_line = physical_line;
            }
            continue;
        }

        if escape {
            match chr {
                '\0' => return Err(format!("unallowed character NUL at line {}", physical_line)),
                '\n' => {
                    pure = false;
                    escape = false;
                    physical_line += 1;
                }
                _ => {
                    push_token_char(&mut current, chr, logical_line)?;
                    escape = false;
                }
            }
            continue;
        }

        match chr {
            '\0' => return Err(format!("unallowed character NUL at line {}", physical_line)),
            '\n' => {
                if quote {
                    return Err(format!(
                        "syntax error: unterminated quotes at line {}",
                        physical_line
                    ));
                }
                flush(
                    &mut line_tokens,
                    &mut current,
                    &mut pure,
                    &mut forced,
                    logical_line,
                );
                if !line_tokens.is_empty() {
                    lines.push(std::mem::take(&mut line_tokens));
                }
                physical_line += 1;
                logical_line = physical_line;
            }
            '#' if !quote => {
                flush(
                    &mut line_tokens,
                    &mut current,
                    &mut pure,
                    &mut forced,
                    logical_line,
                );
                comment = true;
            }
            '"' => {
                pure = false;
                quote = !quote;
                forced = true;
            }
            '\\' => {
                escape = true;
            }
            ' ' | '\t' if !quote => flush(
                &mut line_tokens,
                &mut current,
                &mut pure,
                &mut forced,
                logical_line,
            ),
            '{' | '}' if !quote => {
                flush(
                    &mut line_tokens,
                    &mut current,
                    &mut pure,
                    &mut forced,
                    logical_line,
                );
                line_tokens.push(Token {
                    text: chr.to_string(),
                    pure: true,
                    line: logical_line,
                });
            }
            _ => push_token_char(&mut current, chr, logical_line)?,
        }
    }

    if escape {
        return Err(format!("unterminated escape at line {}", physical_line));
    }
    if quote {
        return Err(format!(
            "syntax error: unterminated quotes at line {}",
            physical_line
        ));
    }

    flush(
        &mut line_tokens,
        &mut current,
        &mut pure,
        &mut forced,
        logical_line,
    );
    if !line_tokens.is_empty() {
        lines.push(line_tokens);
    }

    Ok(lines)
}

fn syntax_error(line: usize) -> String {
    format!("syntax error at line {}", line)
}

fn keyword_at(tokens: &[Token], index: usize, keyword: &str) -> bool {
    tokens
        .get(index)
        .is_some_and(|token| token.pure && token.text == keyword)
}

fn push_token_char(current: &mut String, chr: char, line_no: usize) -> Result<(), String> {
    if current.len() + chr.len_utf8() > MAX_TOKEN_LEN {
        return Err(format!("too long line at line {}", line_no));
    }

    current.push(chr);
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::rule::RuleIdentity;

    fn parse_rules(config: &str) -> Result<Rules, String> {
        Rules::try_from(config)
    }

    #[test]
    fn rejects_args_without_cmd() {
        assert!(parse_rules("permit nopass vince args -u\n").is_err());
    }

    #[test]
    fn rejects_cmd_before_as() {
        assert!(parse_rules("permit nopass vince cmd /usr/bin/id as root\n").is_err());
    }

    #[test]
    fn rejects_repeated_cmd_clause() {
        assert!(parse_rules("permit nopass vince cmd /bin/echo cmd /usr/bin/id\n").is_err());
    }

    #[test]
    fn supports_backslash_newline_continuation() {
        let rules = parse_rules("permit nopass alice as root cmd /usr/bin/pri\\\nntf args hello\n")
            .expect("expected config to parse");
        let rule = &rules.rules[0];

        assert!(matches!(&rule.identity, RuleIdentity::User(name) if name == "alice"));
        assert_eq!(rule.target.as_deref(), Some("root"));
        assert_eq!(rule.command.as_deref(), Some("/usr/bin/printf"));
        assert_eq!(rule.args.as_ref(), Some(&vec![String::from("hello")]));
    }

    #[test]
    fn rejects_nul_bytes_in_words() {
        assert!(parse_rules("permit nopass v\0ince as root cmd /usr/bin/id\n").is_err());
    }

    #[test]
    fn rejects_overlong_tokens() {
        let config = format!("permit nopass alice cmd /usr/bin/{}\n", "x".repeat(1100));
        let err = parse_rules(&config).expect_err("expected overlong token to fail");

        assert!(err.contains("too long line"));
    }

    #[test]
    fn continuation_inside_keyword_disables_keyword_parsing() {
        assert!(parse_rules("per\\\nmit nopass alice\n").is_err());
    }
}
