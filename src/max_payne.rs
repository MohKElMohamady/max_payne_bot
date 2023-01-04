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

pub async fn parse_tweet_from_response(response_body: String) -> anyhow::Result<Tweet> {
    // Parse the string from json body
    let tweet_response_json_blob: Value = serde_json::from_str(&response_body)?;
    println!("{:?}", &tweet_response_json_blob);
    // Take out the id and the text of the tweet
    // https://stackoverflow.com/questions/27043268/convert-a-string-to-int
    let id: i64 = tweet_response_json_blob["data"]["id"]
        .to_string()
        .replace("\\", "")
        .replace("\"", "")
        .parse::<i64>()?;
    let text: String = tweet_response_json_blob["data"]["text"].to_string();

    println!(
        "Successfully parsed the quote with the id {:?} and the text is {:?}",
        id, text
    );

    return Ok(Tweet { id, text});
}

pub async fn parse_unsucessful_request(response : TwitterResponse) -> anyhow::Result<UnSuccessfulTweetStatus> {
    match response {
        TwitterResponse::Response(r) => {
            
            let mut serialized_headers : String = String::from("");
            for header in r.headers().clone().iter() {
                serialized_headers.push_str(format!("{}:{:?} \n", header.0, header.1).as_str())
            }
          
            let tweet_response_json_blob: Value = serde_json::from_str(r.text().await.unwrap().as_str())?;
            let failure_reason : String = tweet_response_json_blob["title"].to_string().replace("\\", "").replace("\"", "");
            let status_code  = tweet_response_json_blob["status"].as_i64().unwrap();
            println!("Parsed the unsucessfull response from tweet");
            Ok(UnSuccessfulTweetStatus{failure_reason : failure_reason, serialized_headers : serialized_headers, status_code : status_code})
        }
        TwitterResponse::FailedResponse(f) => {
            println!("Failed request {:?}", f);
            Ok(UnSuccessfulTweetStatus { status_code: 404, failure_reason: String::from("Error"), serialized_headers: String::from("Error")})
        }
    }
}
