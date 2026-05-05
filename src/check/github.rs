use async_trait::async_trait;
use crate::check::{Check, CheckCtx, CheckError, CheckOutcome};

// TODO: HttpCheck will have its own url
pub struct GitHubCheck;

#[async_trait]
impl Check for GitHubCheck {
    async fn check(&self, ctx: CheckCtx<'_>) -> Result<CheckOutcome, CheckError> {
        let response = ctx
            .http_client
            .get("https://www.githubstatus.com/api/v2/status.json")
            .send()
            .await?
            .error_for_status()?;
        let value = response.json::<serde_json::Value>().await?;
        let status = value.get("status").and_then(|s| s.get("indicator"));

        if status.is_none() {
            return Err(CheckError::ParseError);
        }

        let status = status.unwrap();

        if status != "none" {
            return Ok(CheckOutcome::Down);
        }

        Ok(CheckOutcome::Ok)
    }
}
