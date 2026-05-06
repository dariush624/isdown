use crate::check::{Check, CheckCtx, CheckError, CheckOutcome, CheckStatus};
use async_trait::async_trait;

pub struct GitHubCheck;

#[async_trait]
impl Check for GitHubCheck {
    async fn check(&self, ctx: CheckCtx<'_>) -> Result<CheckOutcome, CheckError> {
        let response = ctx
            .http_client
            .get("https://www.githubstatus.com/api/v2/summary.json")
            .send()
            .await?
            .error_for_status()?;
        let value = response.json::<serde_json::Value>().await?;
        let indicator = value
            .get("status")
            .and_then(|s| s.get("indicator"))
            .and_then(|i| i.as_str())
            .ok_or(CheckError::ParseError)?;

        let status = match indicator {
            "none" => CheckStatus::Up,
            "minor" => CheckStatus::Degraded,
            _ => CheckStatus::Down,
        };

        let causes = value
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
            .unwrap_or_default();

        Ok(CheckOutcome { provider: "GitHub", status, causes })
    }
}
