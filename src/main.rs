pub mod cassandra;
pub mod max_payne;
pub mod parser;

use cassandra::initialise_database_connection;
use dotenv::dotenv;
use max_payne::spawn_max_payne;
use stargate_grpc::{client::StargateClientBuilder, *};
use std::str::FromStr;
use tokio::{time::sleep, task};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load enviroment variables from env
    dotenv().ok();

    return Ok(())
}
