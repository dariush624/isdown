use async_trait::async_trait;
use thiserror::Error;
use crate::registry::ProviderKind;

#[derive(Copy, Clone)]
pub struct CheckCtx<'a> {
    pub(crate) http_client: &'a reqwest::Client,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum CheckOutcome {
    Ok,
    Down,
}

#[async_trait]
pub trait Check {
    async fn check(&self, ctx: CheckCtx<'_>) -> Result<CheckOutcome, CheckError>;
}

pub struct GitHubCheck;

#[async_trait]
impl Check for GitHubCheck {
    async fn check(&self, ctx: CheckCtx<'_>) -> Result<CheckOutcome, CheckError> {
        let response = ctx.http_client.get("https://www.githubstatus.com/api/v2/status.json").send().await?.error_for_status()?;
        let value = response.json::<serde_json::Value>().await?;
        let status = value.get("status").and_then(|s| s.get("indicator"));
        
        if status.is_none() {
            return Err(CheckError::ParseError)
        }
        
        let status = status.unwrap();
        
        if status != "none" {
            return Ok(CheckOutcome::Down)
        }
        
        Ok(CheckOutcome::Ok)
    }
}

impl ProviderKind {
    pub fn check(&self) -> Box<dyn Check> {
        match self {
            ProviderKind::GitHub => Box::new(GitHubCheck),
        }
    }
}


#[derive(Error, Debug)]
pub enum CheckError {
    HttpError(#[from] reqwest::Error),
    JsonError(#[from] serde_json::Error),
    ParseError,
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
