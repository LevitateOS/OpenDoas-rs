const AUTH_MODE: &'static str = "plain";
const SAFE_PATH: &'static str = "/bin:/sbin:/usr/bin:/usr/sbin:/usr/local/bin:/usr/local/sbin";
const TIMESTAMP_MODE: &'static str = "off";

fn main() {
	let mut var;
	let feature_auth_none = std::env::var_os("CARGO_FEATURE_AUTH_NONE").is_some();
	let feature_auth_plain = std::env::var_os("CARGO_FEATURE_AUTH_PLAIN").is_some();
	let feature_auth_pam = std::env::var_os("CARGO_FEATURE_AUTH_PAM").is_some();
	let enabled_features = [
		feature_auth_none,
		feature_auth_plain,
		feature_auth_pam,
	].into_iter().filter(|x| *x).count();
	if enabled_features > 1 {
		panic!("Enable only one of auth-none, auth-plain, or auth-pam");
	}
	
	println!("cargo::rerun-if-env-changed=SAFE_PATH");
	var = option_env!("SAFE_PATH").unwrap_or(SAFE_PATH);
	println!("cargo::rustc-env=SAFE_PATH={var}");
	
	println!("cargo::rerun-if-env-changed=DEFAULT_CONF_PATH");
	var = option_env!("DEFAULT_CONF_PATH").unwrap_or("/dev/null");
	println!("cargo::rustc-env=DEFAULT_CONF_PATH={var}");
	
	println!("cargo::rerun-if-env-changed=AUTH_MODE");
	let default_auth_mode = if feature_auth_pam {
		"pam"
	} else if feature_auth_plain {
		"plain"
	} else {
		AUTH_MODE
	};
	var = option_env!("AUTH_MODE").unwrap_or(default_auth_mode);
	match var {
		"none" => {
			if enabled_features != 0 && !feature_auth_none {
				panic!("AUTH_MODE=none requires auth-none or no auth feature");
			}
		},
		"pam" => {
			if !feature_auth_pam {
				panic!("AUTH_MODE=pam requires cargo feature auth-pam");
			}
		},
		"plain" => {
			if !feature_auth_plain {
				panic!("AUTH_MODE=plain requires cargo feature auth-plain");
			}
		},
		_ => panic!("AUTH_MODE is set to an invalid value!"),
	}
	println!("cargo::rustc-cfg=auth=\"{var}\"");
	println!(r#"cargo::rustc-check-cfg=cfg(auth, values("none", "pam", "plain"))"#);

	println!("cargo::rerun-if-env-changed=OPENDOAS_RS_TIMESTAMP");
	var = option_env!("OPENDOAS_RS_TIMESTAMP").unwrap_or(TIMESTAMP_MODE);
	match var {
		"off" | "on" => {}
		_ => panic!("OPENDOAS_RS_TIMESTAMP is set to an invalid value!"),
	}
	println!("cargo::rustc-env=OPENDOAS_RS_TIMESTAMP_MODE={var}");
}
