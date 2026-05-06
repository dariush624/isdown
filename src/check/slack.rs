use crate::check::{Check, CheckCtx, CheckError, CheckOutcome, CheckStatus};
use async_trait::async_trait;

pub struct SlackCheck;

#[async_trait]
impl Check for SlackCheck {
    async fn check(&self, ctx: CheckCtx<'_>) -> Result<CheckOutcome, CheckError> {
        let response = ctx
            .http_client
            .get("https://slack-status.com/api/v2.0.0/current")
            .send()
            .await?
            .error_for_status()?;
        let value = response.json::<serde_json::Value>().await?;
        let status_str = value
            .get("status")
            .and_then(|s| s.as_str())
            .ok_or(CheckError::ParseError)?;

        let status = match status_str {
            "ok" => CheckStatus::Up,
            _ => CheckStatus::Down,
        };

        let causes = value
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
            .unwrap_or_default();

        Ok(CheckOutcome { provider: "Slack", status, causes })
    }
}
