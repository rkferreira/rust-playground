use anyhow::{bail, Context, Result};
use clap::Parser;
use std::process::Command;

use crate::build_ebpf;

#[derive(Debug, Parser)]
pub struct Options {
    /// Build and run in release mode
    #[clap(long)]
    pub release: bool,
    /// Network interface to attach to (default: lo or eth0)
    #[clap(short, long, default_value = "lo")]
    pub iface: String,
    /// Extra arguments passed to the hello-ebpf binary
    #[clap(last = true)]
    pub run_args: Vec<String>,
}

pub fn run(opts: Options) -> Result<()> {
    // Step 1: Build the eBPF bytecode
    build_ebpf::build_ebpf(build_ebpf::Options {
        release: opts.release,
    })?;

    // Step 2: Build and run the userspace loader
    let mut args = vec!["run", "--package", "hello-ebpf"];
    if opts.release {
        args.push("--release");
    }
    args.push("--");
    args.push("--iface");
    args.push(&opts.iface);

    for arg in &opts.run_args {
        args.push(arg);
    }

    let status = Command::new("cargo")
        .args(&args)
        .status()
        .context("Failed to run userspace loader")?;

    if !status.success() {
        bail!("hello-ebpf exited with error");
    }

    Ok(())
}
