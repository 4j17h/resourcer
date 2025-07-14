use clap::{Parser};
use std::fs;
use url::Url;
use sourcedumper_core::find_sourcemap_urls;

#[derive(Parser)]
#[command(version, about = "SourceDumper CLI")]
struct Cli {
    /// List discovered sourcemap URLs without downloading
    #[arg(long)]
    list_urls: bool,

    /// Input JavaScript file to analyze
    #[arg(long, value_name = "FILE")] 
    input: Option<String>,

    /// Print output as JSON array
    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.list_urls {
        let file = cli.input.expect("--input <FILE> required with --list-urls");
        let js = fs::read_to_string(&file).expect("unable to read file");
        let base = Url::from_file_path(&file).unwrap();
        let urls = find_sourcemap_urls(&base, &js);
        if cli.json {
            let json = serde_json::to_string_pretty(&urls).unwrap();
            println!("{}", json);
        } else {
            for u in urls {
                println!("{}", u);
            }
        }
        return;
    }

    println!("No action specified. Use --help for options.");
}
