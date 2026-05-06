mod github;
mod slack;

use crate::check::github::GitHubCheck;
use crate::check::slack::SlackCheck;
use crate::registry::ProviderKind;
use async_trait::async_trait;
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
