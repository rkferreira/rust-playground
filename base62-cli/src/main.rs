use clap::{Parser, Subcommand};
use anyhow::Result;
use hex;
use base62_generator::{encode, decode};

/// Base62 CLI
#[derive(Parser)]
#[command(name = "base62-cli")]
#[command(author = "")]
#[command(version = "0.1.0")]
#[command(about = "Encode/decode binary data using Base62")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode a hex string to Base62
    Encode {
        /// Hex string to encode
        #[arg(value_name = "HEX")]
        hex_input: String,
    },
    /// Decode a Base62 string to hex
    Decode {
        /// Base62 string to decode
        #[arg(value_name = "BASE62")]
        base62_input: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Encode { hex_input } => {
            let bytes = hex::decode(&hex_input)?;
            let encoded = encode(&bytes);
            println!("{}", encoded);
        }
        Commands::Decode { base62_input } => {
            let decoded = decode(&base62_input)?;
            println!("{}", hex::encode(decoded));
        }
    }
    Ok(())
}
