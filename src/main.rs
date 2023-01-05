pub mod cassandra;
pub mod max_payne;
pub mod parser;
use dotenv::dotenv;
use max_payne::spawn_max_payne;
use stargate_grpc::*;
use tokio::{time::sleep, task};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load enviroment variables from env
    dotenv().ok();

    task::spawn(async {
        let mut max_payne = spawn_max_payne().await.unwrap();

        max_payne.create_tables_if_not_exists().await.unwrap();

        parser::parse_quotes(&mut max_payne).await.unwrap();

        loop {
            sleep(tokio::time::Duration::from_secs(3600 * 3)).await;
            max_payne.tweet_a_quote().await.unwrap();
        }
    }).await?;
    
    return Ok(())
}
