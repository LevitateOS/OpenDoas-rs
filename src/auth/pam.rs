use std::ffi::{CStr, CString};

use nix;
use pam_client::{Context, ConversationHandler, ErrorCode, Flag};
use pwd_grp::Passwd;
use rpassword::prompt_password;

use crate::platform::tty::current_tty_name;

pub struct Converser<'a> {
    pub username: &'a str,
}

impl ConversationHandler for Converser<'_> {
    fn prompt_echo_on(&mut self, _msg: &CStr) -> Result<CString, ErrorCode> {
        CString::new(String::from(self.username)).map_err(|_| ErrorCode::CONV_ERR)
    }

    fn prompt_echo_off(&mut self, msg: &CStr) -> Result<CString, ErrorCode> {
        let hostname = nix::unistd::gethostname().expect("Failed to get hostname");
        let hostname = hostname.into_string().expect("Hostname is not valid UTF-8");
        let msg = msg.to_string_lossy();
        let prompt = if msg == "Password:" || msg == "Password: " {
            format!("\rdoas ({}@{}) password: ", &self.username, &hostname)
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
    ) -> Result<(), &'static str> {
        let converser = Converser {
            username: &source_passwd.name,
        };
        let mut context = Context::new("doas", None, converser).expect("Failed to initialize PAM");

        context
            .set_ruser(Some(&source_passwd.name))
            .map_err(|_| "Authentication failed")?;

        if let Some(tty) = current_tty_name() {
            context
                .set_tty(Some(&tty))
                .map_err(|_| "Authentication failed")?;
        }

        context
            .authenticate(Flag::NONE)
            .map_err(|_| "Authentication failed")?;

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
}
