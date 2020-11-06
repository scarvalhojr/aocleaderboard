use config::{Config, ConfigError, File};
use std::convert::TryInto;

pub struct AppSettings {
    pub leaderboard_name: String,
    pub leaderboard_ids: Vec<String>,
    pub leaderboard_update_sec: u64,
    pub session_cookie: String,
}

impl AppSettings {
    pub fn load_from_file(filename: &str) -> Result<Self, ConfigError> {
        let mut settings = Config::default();

        // Set default values
        settings.set_default("leaderboard_update_sec", 15 * 60)?;

        // Load settings from file
        settings.merge(File::with_name(filename))?;

        // Required settings
        let leaderboard_name = settings.get_str("leaderboard_name")?;
        let leaderboard_ids = settings
            .get_array("leaderboard_ids")?
            .into_iter()
            .map(|v| v.into_str())
            .collect::<Result<Vec<_>, _>>()?;
        let session_cookie = settings.get_str("session_cookie")?;

        // Optional overrides
        let leaderboard_update_sec = settings
            .get_int("leaderboard_update_sec")?
            .try_into()
            .map_err(|_| {
                ConfigError::Message(
                    "leaderboard_update_sec must not be negative".to_string(),
                )
            })?;

        // TODO: add support to filter users out

        Ok(Self {
            leaderboard_name,
            leaderboard_ids,
            leaderboard_update_sec,
            session_cookie,
        })
    }
}
