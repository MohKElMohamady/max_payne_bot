use anyhow::Ok;
use rand::Rng;
use stargate_grpc::client::StargateClientBuilder;
use stargate_grpc::*;
use std::convert::TryInto;
use std::{env, str::FromStr};
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
#[derive(Debug)]
pub struct Quote {
    pub id : i64,
    pub text : String,
    pub game : String,
    pub part : String,
    pub chapter : String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Tweet {
    pub id: i64,
    pub text : String
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessfulTweetStatus {
    pub tweet_id: String,
    pub tweet_text: String,
    pub tweeted_on_timestamp: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UnSuccessfulTweetStatus {
    pub status_code: i64,
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
    let stargate_client : StargateClient = StargateClientBuilder::new()
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

impl CassandraClient {
    pub async fn create_tables_if_not_exists(&mut self) -> anyhow::Result<()> {
        /* All the ddl will be done in main keyspace */
        let query_builder = stargate_grpc::Query::builder().keyspace("main");

        /* The quotes by id table will be the main table that we will be fetching the random quoets from  */
        let quotes_by_id_table_ddl = String::from("CREATE TABLE IF NOT EXISTS quotes_by_id ")
            + "(id int PRIMARY KEY, quote text, game text, part text, chapter text);";
        let create_quotes_by_id_table_query = query_builder
            .clone()
            .query(quotes_by_id_table_ddl.as_str())
            .build();
        self.stargate_client
            .execute_query(create_quotes_by_id_table_query)
            .await?;
        println!("Successfully created the table quotes_by_id");

        /* An auxillary table will be present to save the quotes in partitions according to their games */
        let quotes_by_id_table_ddl = String::from("CREATE TABLE IF NOT EXISTS quotes_by_game ")
            + "(game text, quote text, part text, chapter text, PRIMARY KEY (game, quote));";
        let create_quotes_by_game_table_query = query_builder
            .clone()
            .query(quotes_by_id_table_ddl.as_str())
            .build();
        self.stargate_client
            .execute_query(create_quotes_by_game_table_query)
            .await?;
        println!("Successfully created the table quotes_by_game");

        /* The table should contain the tweet_id, text and timestamp when the tweet was created */
        let successfull_tweet_logs_ddl =
            String::from("CREATE TABLE IF NOT EXISTS successfull_tweets_by_id ")
                + "(tweet_id bigint PRIMARY KEY, tweet_text text, tweeted_on_timestamp bigint);";
        let create_successfull_tweet_logs = query_builder
            .clone()
            .query(successfull_tweet_logs_ddl.as_str())
            .build();
        self.stargate_client
            .execute_query(create_successfull_tweet_logs)
            .await?;
        println!("Successfully created the table successfull_tweets_by_id");

        /* The table should contain the timestamp of creating the tweet as id, the text of the tweet, the status code, the failure reason and headers */
        let unsuccessfull_tweet_logs_ddl = String::from("CREATE TABLE IF NOT EXISTS unsuccessfull_tweets_by_id ") 
                                                                + "(tweet_attempted_at timeuuid PRIMARY KEY, status_code int, failure_reason text, serialized_headers text);";
        let create_unsuccessfull_tweet_logs = query_builder
            .clone()
            .query(unsuccessfull_tweet_logs_ddl.as_str())
            .build();
        self.stargate_client
            .execute_query(create_unsuccessfull_tweet_logs)
            .await?;
        println!("Successfully created the table unsuccessfull_tweets_by_id");

        Ok(())
    }

    pub async fn save_quote_by_id(&mut self, quote: &Quote) -> anyhow::Result<()> {
        println!("Saving quote by id {:?}", quote);
        let q = Query::builder()
            .keyspace("main")
            .query("INSERT INTO quotes_by_id (id, chapter, game, part, quote) VALUES (:id, :chapter, :game, :part, :quote)")
            .bind_name("id", quote.id)
            .bind_name("chapter", quote.chapter.as_str())
            .bind_name("game", quote.game.as_str())
            .bind_name("part", quote.part.as_str())
            .bind_name("quote", quote.text.as_str())
            .build();
        self.stargate_client.execute_query(q).await?;

        Ok(())
    }

    pub async fn save_quote_by_game(&mut self, quote: &Quote) -> anyhow::Result<()> {
        println!("Saving quote by game {:?}", quote);
        let q = Query::builder()
            .keyspace("main")
            .query("INSERT INTO quotes_by_game (game, quote, chapter, part) VALUES (:game, :quote, :chapter, :part)")
            .bind_name("chapter", quote.chapter.as_str())
            .bind_name("part", quote.part.as_str())
            .bind_name("game", quote.game.as_str())
            .bind_name("quote", quote.text.as_str())
            .build();
        self.stargate_client.execute_query(q).await?;

        Ok(())
    }

    /*
     * Analyitics of interacting with Twitter API:
     * Everytime a tweet is successfully sent, the tweet's status and when it was tweeted will be saved in an auxillarly table to monitor the tweets
     */
    /* The parameter SuccessfulTweetStatus is not a reference rather ownership because it will be immediately droped after saving it in the database, hence it makes no sense to pass it as a reference */
    pub async fn save_suc_tweet_logs(&mut self, successfull_tweet_status : &Tweet) -> anyhow::Result<()> {
        let q = Query::builder()
            .keyspace("main")
            .query("INSERT INTO successfull_tweets_by_id (tweet_id , tweet_text , tweeted_on_timestamp) VALUES ( :id, :tweet_text, :tweeted_on_timestamp);")
            .bind_name("id", successfull_tweet_status.id)
            .bind_name("tweet_text", successfull_tweet_status.text.as_str())
            .bind_name("tweeted_on_timestamp",Value::bigint(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_nanos() as i64))
            .build();
        self.stargate_client.execute_query(q).await.unwrap();
        Ok(())
    }

    /*
     * Similarly, When something goes wrong, the failure will be logged and saved in the database
     */
    /* The parameter UnSuccessfulTweetStatus is not a reference rather ownership because it will be immediately droped after saving it in the database, hence it makes no sense to pass it as a reference */
    
    pub async fn save_unsuc_tweet_attempt_log(&mut self, unsuccessfull_tweet_status : &UnSuccessfulTweetStatus) -> anyhow::Result<()> {
        let some_timestamp = uuid::Timestamp::now(uuid::NoContext);
        uuid::Uuid::new_v1(some_timestamp, &[2,0,0,1,07,25]);
        let q = Query::builder()
            .keyspace("main")
            .query("INSERT INTO unsuccessfull_tweets_by_id ( tweet_attempted_at , failure_reason , status_code , serialized_headers ) VALUES ( now(), :failure_reason, :status_code , :serialized_headers);")
            .bind_name("failure_reason", unsuccessfull_tweet_status.failure_reason.as_str())
            .bind_name("status_code",unsuccessfull_tweet_status.status_code)
            .bind_name("serialized_headers", unsuccessfull_tweet_status.serialized_headers.as_str())
            .build();
        self.stargate_client.execute_query(q).await.unwrap();
        Ok(())
    }

    pub async fn fetch_random_quote(&mut self) -> anyhow::Result<Quote> {
        let random_id = rand::thread_rng().gen_range(0..=255);
        println!("Fetching the random quote with id {:?}", random_id);
        let q = Query::builder()
            .keyspace("main")
            .query("SELECT quote FROM quotes_by_id WHERE id = :id")
            .bind_name("id", random_id)
            .build();
        let fetched_quote : ResultSet  = self.stargate_client.execute_query(q).await?.try_into()?;
        let mut text_of_quote : String = String::new();
        for row in fetched_quote.rows { 
            let (quote,) : (String,) = row.try_into()?;
            // TODO: Check for the length of the quote and make sure if its length is less that Twitter's tweet character limit
            text_of_quote = quote.clone();
        }
        println!("Successfully fetched the quote with text {:?}", text_of_quote);
        return Ok(Quote{id : 5 ,text : String::from(text_of_quote), game : String::from(""), part : String::from(""), chapter : String::from("")})
    }

}
