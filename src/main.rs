use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use structopt::*;

#[derive(Clone, Debug, StructOpt)]
#[structopt(name = "ts")]
struct Options {
    /// Location of text to tweetify
    #[structopt(short = "i", long, parse(from_str))]
    input_path: Option<PathBuf>,

    /// The maximum length of a tweet, in characters
    #[structopt(short = "l", long)]
    max_tweet_length: Option<usize>,

    #[structopt()]
    string: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::from_args();

    let input = if let Some(input_location) = options.input_path {
        fs::read_to_string(input_location)?
    } else {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    };

    let max_tweet_length = if let Some(chars) = options.max_tweet_length {
        chars
    } else {
        280
    };

    let splits = tweet_split::split_text(&input, max_tweet_length);
    for split in splits {
        println!("{}", split.replace('\n', "\\n").replace("'", "\\'").replace("\"", "\\\""));
    }

    Ok(())
}
