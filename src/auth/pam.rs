use std::ffi::{CStr, CString};

use pam_client::{Context, ConversationHandler, ErrorCode, Flag};
use pwd_grp::Passwd;
use rpassword::prompt_password;

use crate::{auth::prompt::password_prompt, platform::tty::current_tty_name};

pub struct Converser<'a> {
    pub username: &'a str,
}

impl ConversationHandler for Converser<'_> {
    fn prompt_echo_on(&mut self, _msg: &CStr) -> Result<CString, ErrorCode> {
        CString::new(String::from(self.username)).map_err(|_| ErrorCode::CONV_ERR)
    }

    fn prompt_echo_off(&mut self, msg: &CStr) -> Result<CString, ErrorCode> {
        let msg = msg.to_string_lossy();
        let prompt = if msg == "Password:" || msg == "Password: " {
            password_prompt(self.username)
        } else {
            msg.into_owned()
        };
        let password = prompt_password(prompt).map_err(|_| ErrorCode::CONV_ERR)?;

        CString::new(password).map_err(|_| ErrorCode::CONV_ERR)
    }

    fn text_info(&mut self, msg: &CStr) {
        let msg = String::from_utf8_lossy(msg.to_bytes()).to_string();
        eprintln!("{msg}");
    }

    fn error_msg(&mut self, msg: &CStr) {
        let msg = String::from_utf8_lossy(msg.to_bytes()).to_string();
        eprintln!("{msg}");
    }
}

pub struct Transaction<'a> {
    pub context: Option<Context<Converser<'a>>>,
}

impl<'a> Transaction<'a> {
    pub fn new() -> Self {
        Self { context: None }
    }

    pub fn begin<'s>(
        &'s mut self,
        source_passwd: &'a Passwd,
        target_passwd: &'a Passwd,
        require_auth: bool,
    ) -> Result<(), &'static str> {
        let converser = Converser {
            username: &source_passwd.name,
        };
        let mut context =
            Context::new("doas", None, converser).map_err(|_| "Authentication failed")?;

        context
            .set_ruser(Some(&source_passwd.name))
            .map_err(|_| "Authentication failed")?;

        if let Some(tty) = current_tty_name() {
            context
                .set_tty(Some(&tty))
                .map_err(|_| "Authentication failed")?;
        }

        if require_auth {
            context
                .authenticate(Flag::NONE)
                .map_err(|_| "Authentication failed")?;
        }

        if let Err(err) = context.acct_mgmt(Flag::NONE) {
            let code = err.code();
            if code == ErrorCode::NEW_AUTHTOK_REQD {
                context
                    .chauthtok(Flag::CHANGE_EXPIRED_AUTHTOK)
                    .map_err(|_| "Authentication failed")?;
            } else {
                return Err("Authentication failed");
            }
        }

        context
            .set_user(Some(&target_passwd.name))
            .map_err(|_| "Authentication failed")?;
        context
            .reinitialize_credentials(Flag::NONE)
            .map_err(|_| "Authentication failed")?;
        self.context = Some(context);

        Ok(())
    }

    pub fn into_context(mut self) -> Result<Context<Converser<'a>>, &'static str> {
        self.context.take().ok_or("Authentication failed")
    }
}

#[cfg(test)]
mod tests {
    use super::Transaction;

    #[test]
    fn missing_context_is_error_not_panic() {
        let transaction = Transaction::new();

        assert_eq!(
            transaction.into_context().err(),
            Some("Authentication failed")
        );
    }
}
