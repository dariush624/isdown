use reqwest::Response;
use crate::check::{Check, CheckCtx, CheckError, CheckOutcome};

pub struct UrlCheck {
    url: String,
}

impl UrlCheck {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Check for UrlCheck {
    async fn check(&self, ctx: CheckCtx) -> Result<Vec<CheckOutcome>, CheckError> {
        let response = ctx.http_client.get(&self.url).send().await?;
        
        match response { 
            
        }
    }   
}