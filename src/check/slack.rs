use crate::check::{CheckError, CheckStatus, ProviderCheck};
use serde_json::Value;

pub struct SlackCheck;

impl ProviderCheck for SlackCheck {
    fn provider(&self) -> &'static str {
        "Slack"
    }

    fn url(&self) -> &'static str {
        "https://slack-status.com/api/v2.0.0/current"
    }

    fn parse_status(&self, value: &Value) -> Result<String, CheckError> {
        value
            .get("status")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string())
            .ok_or(CheckError::ParseError)
    }

    fn map_status(&self, status: String) -> CheckStatus {
        match status.as_str() {
            "ok" => CheckStatus::Up,
            _ => CheckStatus::Down,
        }
    }

    fn causes(&self, value: &Value) -> Vec<String> {
        value
            .get("active_incidents")
            .and_then(|i| i.as_array())
            .map(|incidents| {
                incidents
                    .iter()
                    .filter_map(|i| {
                        let title = i.get("title")?.as_str()?;
                        let status = i.get("status")?.as_str()?;
                        Some(format!("{} ({})", title, status))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}
