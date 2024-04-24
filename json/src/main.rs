use std::fs;
use serde_json::{Result, Value};
use clap::Parser;

/// VERY VERY VERY simple Json validator
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the file to read
    #[arg(short, long)]
    name: String,
}

fn main() {
    let args = Args::parse();
    let file_name = args.name;
    let file = fs::read_to_string(file_name).expect("Unable to read file");
    let json_value: Value = serde_json::from_str(&file).expect("Unable to parse JSON");
    println!("JSON parsed successfully");
}
