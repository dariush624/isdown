use std::fmt::Display;
use thiserror::Error;
use futures::future::try_join_all;
use crate::check::{Check, CheckCtx, CheckError, CheckOutcome};
use crate::target::Target;

pub struct Planner {
    pub(crate) http_client: reqwest::Client,   
}

impl Planner {
    pub fn plan(&self, targets: &[Target]) -> Result<Vec<Box<dyn Check>>, PlanError> {
        let mut checks: Vec<Box<dyn Check>> = Vec::new();
        for target in targets.iter() {
            match target {
                Target::Provider(details) => {
                    checks.push(details.kind.check());
                }
            }
        }

        Ok(checks)
    }
    
    pub async fn run(&self, checks: &[Box<dyn Check>]) -> Result<Vec<CheckOutcome>, PlanError> {
        let ctx = CheckCtx {
            http_client: &self.http_client,
        };
        let outcomes = try_join_all(checks.iter().map( |check| check.check(ctx))).await?;
        Ok(outcomes)
    }
}

#[derive(Error, Debug)]
pub enum PlanError {
    HttpError(#[from] reqwest::Error),
    CheckError(#[from] CheckError),   
}

impl Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}