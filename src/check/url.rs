use crate::check::{Check, CheckCtx, CheckError, CheckOutcome, CheckStatus};
use async_trait::async_trait;

pub struct UrlCheck {
    url: String,
}

impl UrlCheck {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

#[async_trait]
impl Check for UrlCheck {
    async fn check(&self, ctx: CheckCtx<'_>) -> Result<Vec<CheckOutcome>, CheckError> {
        let response = ctx.http_client.get(&self.url).send().await?;

        match response.status() {
            reqwest::StatusCode::OK => Ok(vec![CheckOutcome {
                provider: self.url.clone(),
                status: CheckStatus::Up,
                causes: vec![],
            }]),
            _ => Ok(vec![CheckOutcome {
                provider: self.url.clone(),
                status: CheckStatus::Down,
                causes: vec![],
            }]),
        }
    }
}
