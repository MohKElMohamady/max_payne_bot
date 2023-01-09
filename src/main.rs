pub mod cassandra;
pub mod max_payne;
pub mod parser;
use dotenv::dotenv;
use max_payne::spawn_max_payne;
use std::env;
use tokio::{time::sleep, task};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load enviroment variables from env
    dotenv().ok();

    task::spawn(async {
        let mut max_payne = spawn_max_payne().await.unwrap();

        max_payne.create_tables_if_not_exists().await.unwrap();

        parser::parse_quotes(&mut max_payne).await.unwrap();

        // As part of immediate testing, if the bot can tweet or not, an immediate quote will be tweeted right after starting the application
       max_payne.tweet_a_quote().await.unwrap(); 

        // loop {
        //     let tweet_timer : u64 = env::var("TWEET_TIMER").unwrap().parse::<u64>().unwrap();
        //     sleep(tokio::time::Duration::from_secs(tweet_timer)).await;
        //     max_payne.tweet_a_quote().await.unwrap();
        // }
        for _ in 0..i128::MAX {
            let tweet_timer : u64 = env::var("TWEET_TIMER").unwrap().parse::<u64>().unwrap();
            sleep(tokio::time::Duration::from_secs(tweet_timer)).await;
            max_payne.tweet_a_quote().await.unwrap();
        }
    }).await?;
    
    return Ok(())
}
