mod aws;
mod slack;
mod statuspageio;

use crate::check::aws::AwsCheck;
use crate::check::slack::SlackCheck;
use crate::check::statuspageio::{
    AtlassianCheck, CircleCICheck, CloudflareCheck, DatadogCheck, DiscordCheck, GitHubCheck,
    LinearCheck, NetlifyCheck, NpmCheck, OpenAICheck, VercelCheck,
};
use crate::registry::ProviderKind;
use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Copy, Clone)]
pub struct CheckCtx<'a> {
    pub(crate) http_client: &'a reqwest::Client,
}

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct CheckOutcome {
    pub provider: String,
    pub status: CheckStatus,
    pub causes: Vec<String>,
}

#[derive(PartialEq, Debug, Clone, Serialize)]
pub enum CheckStatus {
    Up,
    Degraded,
    Down,
}

#[async_trait]
pub trait Check {
    async fn check(&self, ctx: CheckCtx<'_>) -> Result<Vec<CheckOutcome>, CheckError>;
}

pub trait ProviderCheck: Send + Sync {
    fn provider(&self) -> &'static str;
    fn url(&self) -> &str;
    fn parse_status(&self, value: &Value) -> Result<String, CheckError>;
    fn map_status(&self, status: String) -> CheckStatus;
    fn causes(&self, value: &Value) -> Vec<String>;
}

#[async_trait]
impl<T: ProviderCheck> Check for T {
    async fn check(&self, ctx: CheckCtx<'_>) -> Result<Vec<CheckOutcome>, CheckError> {
        let result = ctx
            .http_client
            .get(self.url())
            .send()
            .await?
            .error_for_status()?;
        let value = result.json::<Value>().await?;
        let status_string = self.parse_status(&value)?;

        let provider = self.provider().to_string();
        let status = self.map_status(status_string);
        let causes = self.causes(&value);

        Ok(vec![CheckOutcome {
            provider,
            status,
            causes,
        }])
    }
}

impl ProviderKind {
    pub fn check(&self) -> Box<dyn Check> {
        match self {
            ProviderKind::GitHub => Box::new(GitHubCheck),
            ProviderKind::Slack => Box::new(SlackCheck),
            ProviderKind::Atlassian => Box::new(AtlassianCheck),
            ProviderKind::CircleCI => Box::new(CircleCICheck),
            ProviderKind::Cloudflare => Box::new(CloudflareCheck),
            ProviderKind::Datadog => Box::new(DatadogCheck),
            ProviderKind::Discord => Box::new(DiscordCheck),
            ProviderKind::Linear => Box::new(LinearCheck),
            ProviderKind::Netlify => Box::new(NetlifyCheck),
            ProviderKind::Npm => Box::new(NpmCheck),
            ProviderKind::OpenAI => Box::new(OpenAICheck),
            ProviderKind::Vercel => Box::new(VercelCheck),
            ProviderKind::Aws => Box::new(AwsCheck),
        }
    }
}

#[derive(Error, Debug)]
pub enum CheckError {
    #[error("Connection error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Invalid json: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Parse error")]
    ParseError,
}
