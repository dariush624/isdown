mod check;
mod planner;
mod registry;
mod target;

use crate::planner::Planner;
use crate::target::Target;
use clap::{Parser, Subcommand};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "isdown")]
#[command(about = "Detect downtime on any service", long_about = "")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Check {
        #[arg(required = true)]
        targets: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    let planner = Planner {
        // TODO: configure timeout
        http_client: reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap(),
    };
    let cli = Cli::parse();
    match cli.command {
        Commands::Check { targets } => {
            let checks = planner.plan(
                &targets
                    .iter()
                    .map(|target| Target::parse(target).unwrap())
                    .collect::<Vec<_>>(),
            );
            let outcomes = planner.run(&checks).await;
            for outcome in outcomes.iter() {
                println!("{:?}", outcome);
            }
        }
    }
}
