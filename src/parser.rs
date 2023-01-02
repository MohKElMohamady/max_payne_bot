use crate::max_payne;
use crate::cassandra::Quote;
use std::error::Error;
use std::env;

pub async fn parse_quotes(max_payne : &mut max_payne::MaxPayneBot) -> anyhow::Result<Vec<Quote>, Box<dyn Error>> {
    
    let mut quotes : Vec<Quote> = Vec::new();
    let mut no_quotes : i64 = 0;

    // Quotes are only saved once, once the bot is deployed in the cloud, it will not need to save the quotes again at start up.
    let is_data_saved: String = env::var("ARE_QUOTES_SAVED")?;
    if is_data_saved.eq("TRUE") {
        return Ok(quotes);
    }

    
    let mut parser = csv::ReaderBuilder::new()
    .delimiter(b';')
    .has_headers(true)
    .from_path("max_payne_quotes.csv")?;
    println!(" Sucessfully created the parser, now the csv file will be parsed to save the quotes");

    for result in parser.records() {

        let record = result?;

        // Retrieve the text, game, part and chapter
        let text = record.get(0).unwrap();
        let game = record.get(1).unwrap();
        let part = record.get(2).unwrap();
        let chapter = record.get(3).unwrap();
        
        // Create a quote out the parsed data and add it to the list
        let parsed_quote = Quote{id : no_quotes, text: String::from(text), game: String::from(game), part: String::from(part), chapter: String::from(chapter)};
        println!("{:?}", parsed_quote);
        max_payne.save_quote(&parsed_quote).await?;
        quotes.push(parsed_quote);
        // Increment the total counter of the quotes
        no_quotes = no_quotes + 1;
    
    }

    println!("Finished parsing the data from the csv file and all data now lies in the databse with total {} quotes", no_quotes);

    Ok(quotes)
}