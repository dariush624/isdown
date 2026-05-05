mod target;
mod registry;
mod planner;
mod check;

use clap::{Parser, Subcommand};
use crate::planner::Planner;
use crate::target::Target;

#[derive(Parser, Debug)]
#[command(name="isdown")]
#[command(about="Detect downtime on any service", long_about="")]
struct Cli {
    #[command(subcommand)]
    command: Commands, 
}

#[derive(Subcommand, Debug)]
enum Commands {
    Check {
        #[arg(required = true)]
        targets: Vec<String>
    }
}

#[tokio::main]
async fn main() {
    let planner = Planner {
        http_client: reqwest::Client::new(),
    };
    let cli = Cli::parse();
    match cli.command {
        Commands::Check { targets } => {
            let checks = planner.plan(&targets.iter().map(|target| Target::parse(target).unwrap()).collect::<Vec<_>>()).unwrap();
            let outcomes = planner.run(&checks).await.unwrap();
            for outcome in outcomes.iter() {
                println!("{:?}", outcome);
            }
        }
    }
}
