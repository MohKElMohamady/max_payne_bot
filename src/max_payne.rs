use crate::{
    cassandra::{
        init_db_and_get_cassandra_instance, CassandraClient, Quote, Tweet, UnSuccessfulTweetStatus,
    },
};
use reqwest_oauth1::Secrets;
use serde_json::Value;
use std::env;
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

impl MaxPayneBot {
    pub async fn tweet_a_quote(&mut self) -> anyhow::Result<()> {
        let secrets: Secrets = Secrets::new(&self._665.consumer_key, &self._665.consumer_secret)
            .token(&self._665.token, &self._665.token_secret);

        // If we want to enter line breaks in JSON resposne bodys, new lines have to be preceeded with \\n.
        // TODO: Figure out how to escape double quotes in JSON request bodies.

        let fetched_quote = self.cassandra_client.fetch_random_quote().await?;

        println!(
            "The quote that is about to be tweeted is : {:?}",
            fetched_quote.text
        );

        let request_body = format!("{{\"text\": \"{}\"}}", fetched_quote.text);

        let reponse_from_twitter = self
            .http_client
            .post("https://api.twitter.com/2/tweets")
            .header("Content-Type", "application/json")
            .sign(secrets)
            .body(request_body)
            .send()
            .await;
        match reponse_from_twitter {
            Ok(suc_response) => {
                
                if suc_response.status().is_success() {
                    println!("Request was successfull and the quote got tweeted");
                    let tweet_response_body = suc_response.text().await?;
                    let tweet: Tweet = parse_tweet_from_response(tweet_response_body).await?;
                    self.log_successfull_tweet(&tweet).await?;
                } else {
                    println!("Request was successfull and but the quote was not tweeted");
                    let unsuccessfull_twitter_status : UnSuccessfulTweetStatus = parse_unsucessful_request(TwitterResponse::Response(suc_response)).await?;
                    self.log_unsuccessfull_tweet(&unsuccessfull_twitter_status).await?;    
                }
            }
            Err(_unsuc_response) => {
                // self.cassandra_client.save_unsuc_tweet_attempt_log().await?;
            }
        }
        Ok(())
    }

    pub async fn create_tables_if_not_exists(&mut self) -> anyhow::Result<()> {
        self.cassandra_client.create_tables_if_not_exists().await?;
        Ok(())
    }

    pub async fn save_quote(&mut self, parsed_quote: &Quote) -> anyhow::Result<()> {
        self.cassandra_client
            .save_quote_by_id(parsed_quote)
            .await?;
        self.cassandra_client
            .save_quote_by_game(parsed_quote)
            .await?;
        Ok(())
    }

    pub async fn log_successfull_tweet(&mut self, tweet: &Tweet) -> anyhow::Result<()> {
        println!(
            "Logging the successfully tweeted quote with id {:?} and the text {:?}",
            tweet.id, tweet.text
        );
        self.cassandra_client.save_suc_tweet_logs(tweet).await?;
        Ok(())
    }

    pub async fn log_unsuccessfull_tweet(&mut self, unsuccessfull_tweet_status: &UnSuccessfulTweetStatus) -> anyhow::Result<()> {
        println!(
            "Logging the failed tweet attempt having the reason {:?} and with status code {:?} and headers {:?}",
            unsuccessfull_tweet_status.failure_reason, unsuccessfull_tweet_status.status_code, unsuccessfull_tweet_status.serialized_headers
        );
        self.cassandra_client.save_unsuc_tweet_attempt_log(unsuccessfull_tweet_status).await.unwrap();
        Ok(())
    }
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
