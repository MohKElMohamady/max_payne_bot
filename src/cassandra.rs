use anyhow::Ok;
use rand::Rng;
use stargate_grpc::client::StargateClientBuilder;
use stargate_grpc::*;
use std::convert::TryInto;
use std::{env, str::FromStr};

#[derive(Debug)]
pub struct Quote {
    pub id : i64,
    pub text : String,
    pub game : String,
    pub part : String,
    pub chapter : String,
}
#[derive(Debug)]
pub struct Tweet {
    id: u64,
    text : String
}
#[derive(Debug)]
pub struct SuccessfulTweetStatus {
    tweet_id: String,
    tweet_text: String,
    tweeted_on_timestamp: u128,
}
#[derive(Debug)]
pub struct UnSuccessfulTweetStatus {
    pub status_code: i8,
    pub failure_reason: String,
    pub serialized_headers: String,
}
// Wrapper struct that abstracts the insertion and selection query statements from the max_payne struct
pub struct CassandraClient {
    stargate_client: StargateClient,
}

pub async fn init_db_and_get_cassandra_instance() -> anyhow::Result<CassandraClient> {
    println!("Starting to initialize the connection to the database");
    let datastax_remote_url: String = env::var("MAX_PAYNE_BOT_DATASTAX_REMOTE_URL")?;
    let datastax_token: String = env::var("MAX_PAYNE_BOT_DATASTAX_TOKEN")?;

    println!("Building up the client and connection to remote Cassandra instance");
    let stargate_client /* : StargateClient  */= StargateClientBuilder::new()
        .uri(datastax_remote_url)?
        .auth_token(AuthToken::from_str(&datastax_token)?)
        .tls(Some(client::default_tls_config()?))
        .connect()
        .await?;
    println!("Building up the client and connection to remote Cassandra instance was successfull");

    
    println!("Returning Result enum with Cassandra Client for Max Payne Struct");
    Ok(CassandraClient {
        stargate_client: (stargate_client),
    })
}
