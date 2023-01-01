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

