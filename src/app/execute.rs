use std::{fs::File, io::Read};

use crate::{cli::args::Execute, config::validate::validate_runtime_config_metadata, Rules};

#[derive(Debug)]
pub struct ConfigRequest {
    pub only_check: bool,
    pub check_permissions: bool,
    pub path: String,
}

impl ConfigRequest {
    pub fn from_execute(opts: &Execute) -> Self {
        match &opts.config_file {
            None => Self {
                only_check: false,
                check_permissions: true,
                path: String::from("/etc/doas.conf"),
            },
            Some(path) => Self {
                only_check: true,
                check_permissions: false,
                path: path.clone(),
            },
        }
    }
}

pub fn load_rules(request: &ConfigRequest) -> Result<Rules, String> {
    let mut file = File::open(&request.path).map_err(|err| {
        if request.check_permissions {
            format!("doas is not enabled, {}: {err}", request.path)
        } else {
            format!("could not open config file {}: {err}", request.path)
        }
    })?;

    if request.check_permissions {
        let metadata = file
            .metadata()
            .map_err(|err| format!("fstat(\"{}\"): {err}", request.path))?;
        validate_runtime_config_metadata(&request.path, &metadata)?;
    }

    let mut config = String::new();
    file.read_to_string(&mut config)
        .map_err(|err| format!("could not read config file {}: {err}", request.path))?;

    Rules::try_from(config.as_str()).map_err(|err| format!("Error parsing config: {err}"))
}
