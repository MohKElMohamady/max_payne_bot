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
                                                                + "(tweet_attempted_at timeuuid PRIMARY KEY, text_of_tweet text, status_code int, failure_reason text, serialized_headers text);";
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

}
