use super::timestamp::{configured_persist_mode, timestamp_clear, PersistMode};

pub fn can_deauth() -> bool {
    configured_persist_mode().is_enabled()
}

pub fn persist_mode() -> PersistMode {
    configured_persist_mode()
}

pub fn deauth() -> Result<(), String> {
    match configured_persist_mode() {
        PersistMode::Disabled => Ok(()),
        PersistMode::Enabled => timestamp_clear(),
    }
}
