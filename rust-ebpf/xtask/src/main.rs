mod build_ebpf;
mod run;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "Build and run tasks for hello-ebpf")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Build the eBPF kernel program
    BuildEbpf(build_ebpf::Options),
    /// Build eBPF program and run the userspace loader
    Run(run::Options),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::BuildEbpf(opts) => build_ebpf::build_ebpf(opts),
        Commands::Run(opts) => run::run(opts),
    }
}
