use crate::aoc::{EventYear, MemberId};
use crate::leaders::LeaderboardOrder;
use config::{Config, ConfigError, File, Value};
use std::convert::TryInto;

pub struct AppSettings {
    pub leaderboard_name: String,
    pub leaderboard_ids: Vec<String>,
    pub leaderboard_default_order: LeaderboardOrder,
    pub leaderboard_update_sec: u64,
    pub exclude_members: Vec<MemberId>,
    pub session_cookie: String,
    pub latest_event_year: Option<EventYear>,
}

impl AppSettings {
    pub fn load_from_file(filename: &str) -> Result<Self, ConfigError> {
        let mut settings = Config::default();

        // Set default values
        settings.set_default("leaderboard_update_sec", 15 * 60)?;
        settings.set_default("leaderboard_default_order", "local_score")?;
        settings.set_default("exclude_members", Vec::<Value>::new())?;

        // Load settings from file
        settings.merge(File::with_name(filename))?;

        // Required settings
        let leaderboard_name = settings.get_str("leaderboard_name")?;
        let leaderboard_ids = settings
            .get_array("leaderboard_ids")?
            .into_iter()
            .map(|v| v.into_str())
            .collect::<Result<Vec<_>, _>>()?;

        // TODO: load session cookie from different file?
        let session_cookie = settings.get_str("session_cookie")?;

        // Optional settings
        let leaderboard_default_order =
            settings.get("leaderboard_default_order").map_err(|_| {
                ConfigError::Message(
                    "invalid leaderboard_default_order".to_string(),
                )
            })?;
        let leaderboard_update_sec = settings
            .get_int("leaderboard_update_sec")?
            .try_into()
            .map_err(|_| {
                ConfigError::Message(
                    "leaderboard_update_sec must not be negative".to_string(),
                )
            })?;
        let exclude_members = settings
            .get_array("exclude_members")?
            .into_iter()
            .map(|v| v.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        let latest_event_year =
            settings.get_int("latest_event_year").map_or_else(
                |err| match err {
                    ConfigError::NotFound(_) => Ok(None),
                    _ => Err(err),
                },
                |value| {
                    value
                        .try_into()
                        .map_err(|err| {
                            ConfigError::Message(format!(
                                "invalid latest_event_year: {}",
                                err
                            ))
                        })
                        .map(Some)
                },
            )?;

        Ok(Self {
            leaderboard_name,
            leaderboard_ids,
            leaderboard_default_order,
            leaderboard_update_sec,
            exclude_members,
            session_cookie,
            latest_event_year,
        })
    }
}
