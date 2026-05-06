use crate::check::{CheckError, CheckStatus, ProviderCheck};
use serde_json::Value;

pub struct GitHubCheck;

impl ProviderCheck for GitHubCheck {
    fn provider(&self) -> &'static str {
        "GitHub"
    }

    fn url(&self) -> &'static str {
        "https://www.githubstatus.com/api/v2/summary.json"
    }

    fn parse_status(&self, value: &Value) -> Result<String, CheckError> {
        value
            .get("status")
            .and_then(|s| s.get("indicator"))
            .and_then(|i| i.as_str())
            .map(|s| s.to_string())
            .ok_or(CheckError::ParseError)
    }

    fn map_status(&self, status: String) -> CheckStatus {
        match status.as_str() {
            "none" => CheckStatus::Up,
            "minor" => CheckStatus::Degraded,
            _ => CheckStatus::Down,
        }
    }

    fn causes(&self, value: &Value) -> Vec<String> {
        value
            .get("incidents")
            .and_then(|i| i.as_array())
            .map(|incidents| {
                incidents
                    .iter()
                    .filter_map(|i| {
                        let name = i.get("name")?.as_str()?;
                        let status = i.get("status")?.as_str()?;
                        Some(format!("{} ({})", name, status))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}
