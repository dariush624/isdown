use crate::check::{Check, CheckError, CheckOutcome};
use crate::planner::Planner;
use crate::target;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Duration;
use tokio::time::Instant;

pub struct WatchEvent {
    pub outcomes: Vec<Result<CheckOutcome, CheckError>>,
}

pub struct Watcher {
    targets: Arc<Vec<target::Target>>,
    interval: Duration,
    duration: Option<Duration>,
}

impl Watcher {
    pub fn new(
        targets: Vec<target::Target>,
        interval: Duration,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            targets: Arc::new(targets),
            interval,
            duration,
        }
    }

    pub async fn watch(&self) -> Receiver<WatchEvent> {
        let (sender, receiver) = tokio::sync::mpsc::channel(64);

        let targets = self.targets.clone();
        let interval = self.interval.clone();
        let duration = self
            .duration
            .clone()
            .unwrap_or(Duration::from_secs(u64::MAX));

        tokio::spawn(async move {
            let now = Instant::now();

            let planner = Planner {
                http_client: reqwest::Client::new(), // TODO: make configurable
            };

            let checks = planner.plan(&targets);

            while now.elapsed() < duration {
                let results = planner.run(&checks).await;

                sender.send(WatchEvent { outcomes: results }).await.unwrap(); // TODO: handle error
                tokio::time::sleep(interval).await;
            }
        });

        receiver
    }
}
