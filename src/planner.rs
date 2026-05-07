use crate::check::url::UrlCheck;
use crate::check::{Check, CheckCtx, CheckError, CheckOutcome};
use crate::target::Target;
use futures::future::join_all;

pub struct Planner {
    pub http_client: reqwest::Client,
}

impl Planner {
    pub fn plan(&self, targets: &[Target]) -> Vec<Box<dyn Check>> {
        let mut checks: Vec<Box<dyn Check>> = Vec::new();
        for target in targets.iter() {
            match target {
                Target::Provider(details) => {
                    checks.push(details.kind.check());
                }
                Target::Url(url) => checks.push(Box::new(UrlCheck::new(url.clone()))),
            }
        }

        checks
    }

    pub async fn run(&self, checks: &[Box<dyn Check>]) -> Vec<Result<CheckOutcome, CheckError>> {
        let ctx = CheckCtx {
            http_client: &self.http_client,
        };
        join_all(checks.iter().map(|check| check.check(ctx)))
            .await
            .into_iter()
            .flat_map(|result| match result {
                Ok(outcomes) => outcomes.into_iter().map(Ok).collect(),
                Err(e) => vec![Err(e)],
            })
            .collect()
    }
}
