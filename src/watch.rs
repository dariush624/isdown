use crate::check::{CheckError, CheckOutcome};
use crate::planner::Planner;
use crate::target;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::time::Duration;
use tokio::time::Instant;

pub struct WatchEvent {
    pub outcomes: Vec<Result<CheckOutcome, CheckError>>,
    pub elapsed: Duration,
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

    pub fn watch(&self, http_timeout: u64) -> Receiver<WatchEvent> {
        let (sender, receiver) = tokio::sync::mpsc::channel(64);

        let targets = self.targets.clone();
        let interval = self.interval;
        let duration = self.duration.unwrap_or(Duration::from_secs(u64::MAX));

        tokio::spawn(async move {
            let now = Instant::now();

            let http_client = reqwest::Client::builder()
                .timeout(Duration::from_secs(http_timeout))
                .build();

            if http_client.is_err() {
                return;
            }

            let planner = Planner {
                http_client: http_client.unwrap(),
            };

            let checks = planner.plan(&targets);

            while now.elapsed() < duration {
                let results = planner.run(&checks).await;
                if sender
                    .send(WatchEvent {
                        outcomes: results,
                        elapsed: now.elapsed(),
                    })
                    .await
                    .is_err()
                {
                    break;
                }
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        break;
                    },
                    _ = sender.closed() => {
                        break;
                    },
                    _ = tokio::time::sleep(interval) => {}
                }
            }
        });

        receiver
    }
}
