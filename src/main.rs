mod entity;
mod prepare;
mod rand;
mod run;
mod statistics;

use clap::{Parser, Subcommand};
use sea_orm::Database;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: SubCommandArgs,
    #[clap(short = 'u', long)]
    db_url: String,
    #[clap(short = 's', long, default_value = "1024")]
    txn_size: u32,
    #[clap(short = 'c', long, default_value = "4")]
    concurrent: u32,
}

#[derive(Subcommand, Debug)]
enum SubCommandArgs {
    Prepare {
        #[clap(long)]
        commodity_count: u32,
        #[clap(long)]
        consumer_count: u32,
    },
    Run {
        #[clap(long)]
        commodity_count: u32,
        #[clap(long)]
        consumer_count: u32,
        #[clap(long)]
        downgrade: bool,
        #[clap(long)]
        rate_limit: u32,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let db = Database::connect(&args.db_url)
        .await
        .expect("Failed to connect to database");
    match args.command {
        SubCommandArgs::Prepare { .. } => {
            prepare::execute(&db, &args)
                .await
                .expect("Failed to prepare data");
        }
        SubCommandArgs::Run { .. } => {
            run::execute(&db, &args).await.expect("Failed to run");
        }
    }
}
