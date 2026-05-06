mod github;
mod slack;

use crate::check::github::GitHubCheck;
use crate::check::slack::SlackCheck;
use crate::registry::ProviderKind;
use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;

#[derive(Copy, Clone)]
pub struct CheckCtx<'a> {
    pub(crate) http_client: &'a reqwest::Client,
}

#[derive(PartialEq, Debug, Clone)]
pub struct CheckOutcome {
    pub provider: &'static str,
    pub status: CheckStatus,
    pub causes: Vec<String>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum CheckStatus {
    Up,
    Degraded,
    Down,
}

#[async_trait]
pub trait Check {
    async fn check(&self, ctx: CheckCtx<'_>) -> Result<CheckOutcome, CheckError>;
}

pub trait ProviderCheck: Send + Sync {
    fn provider(&self) -> &'static str;
    fn url(&self) -> &'static str;
    fn parse_status(&self, value: &Value) -> Result<String, CheckError>;
    fn map_status(&self, status: String) -> CheckStatus;
    fn causes(&self, value: &Value) -> Vec<String>;
}

#[async_trait]
impl<T: ProviderCheck> Check for T {
    async fn check(&self, ctx: CheckCtx<'_>) -> Result<CheckOutcome, CheckError> {
        let result = ctx
            .http_client
            .get(self.url())
            .send()
            .await?
            .error_for_status()?;
        let value = result.json::<Value>().await?;
        let status_string = self.parse_status(&value)?;

        let provider = self.provider();
        let status = self.map_status(status_string);
        let causes = self.causes(&value);

        Ok(CheckOutcome {
            provider,
            status,
            causes,
        })
    }
}

impl ProviderKind {
    pub fn check(&self) -> Box<dyn Check> {
        match self {
            ProviderKind::GitHub => Box::new(GitHubCheck),
            ProviderKind::Slack => Box::new(SlackCheck),
        }
    }
}

#[derive(Error, Debug)]
pub enum CheckError {
    #[error("Connection error")]
    HttpError(#[from] reqwest::Error),
    #[error("Invalid json")]
    JsonError(#[from] serde_json::Error),
    #[error("Parse error")]
    ParseError,
}
