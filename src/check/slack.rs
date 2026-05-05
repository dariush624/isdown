use crate::check::{Check, CheckCtx, CheckError, CheckOutcome};
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
        let status = value.get("status").ok_or(CheckError::ParseError)?;

        if status != "ok" {
            return Ok(CheckOutcome::Down);
        }

        Ok(CheckOutcome::Ok)
    }
}
