use std::io::{self, BufRead};
use std::fs::File;
use std::io::Read;
use clap::Parser;

/// wc similar
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The number of bytes in each input file is written to the standard output.
    #[arg(short, long)]
    count: bool,

    /// file to open
    file_name: Option<String>,
}


fn main() -> io::Result<()> {
    let args = Args::parse();
    
    if args.count {
        let file = File::open(args.file_name.unwrap())?;
        let mut reader = io::BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        //println!("{:?}", buffer);
        println!("{:?}", buffer.len());
    } else {
        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle.read_line(&mut buffer)?;
        println!("{:?}", buffer.len());
    }
    Ok(())
}
