use crate::{
    cassandra::{
        init_db_and_get_cassandra_instance, CassandraClient, Quote, SuccessfulTweetStatus, Tweet, UnSuccessfulTweetStatus,
    },
    parser,
};
use base64::encode;
use oauth1_request::signature_method::hmac_sha1::HmacSha1;
use reqwest::multipart;
use reqwest::*;
use reqwest_oauth1::{Client, OAuthClientProvider, Secrets, Signer};
use serde_json::Value;
use std::{borrow::Borrow, env};
pub struct MaxPayneBot {
    cassandra_client: CassandraClient,
    // http_client: reqwest_oauth1::Client<Signer<'a, Secrets<'a>, HmacSha1>,
    http_client: reqwest_oauth1::Client<()>,
    _665: _665,
}

pub struct _665 {
    consumer_key: String,
    consumer_secret: String,
    token: String,
    token_secret: String,
}

#[derive(Debug)]
pub enum TwitterResponse {
    Response(reqwest::Response),
    FailedResponse(reqwest_oauth1::Error)
}

pub async fn spawn_max_payne() -> anyhow::Result<MaxPayneBot> {
    let cassandra_client = init_db_and_get_cassandra_instance().await?;
    let http_client = reqwest_oauth1::Client::new();
    let _665 = _665 {
        consumer_key: env::var("OAUTH_ONE_API_KEY")?,
        consumer_secret: env::var("OAUTH_ONE_API_KEY_SECRET")?,
        token: env::var("OAUTH_ONE_ACCESS_TOKEN")?,
        token_secret: env::var("OAUTH_ONE_ACCESS_TOKEN_SECRET")?,
    };
    Ok(MaxPayneBot {
        cassandra_client,
        http_client,
        _665,
    })
}

