use crate::check::{Check, CheckCtx, CheckError, CheckOutcome};
use crate::target::Target;
use futures::future::join_all;

pub struct Planner {
    pub(crate) http_client: reqwest::Client,
}

impl Planner {
    pub fn plan(&self, targets: &[Target]) -> Vec<Box<dyn Check>> {
        let mut checks: Vec<Box<dyn Check>> = Vec::new();
        for target in targets.iter() {
            match target {
                Target::Provider(details) => {
                    checks.push(details.kind.check());
                }
            }
        }

        checks
    }

    pub async fn run(&self, checks: &[Box<dyn Check>]) -> Vec<Result<CheckOutcome, CheckError>> {
        let ctx = CheckCtx {
            http_client: &self.http_client,
        };
        join_all(checks.iter().map(|check| check.check(ctx))).await
    }
}
