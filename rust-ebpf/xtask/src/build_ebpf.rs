use anyhow::{bail, Context, Result};
use clap::Parser;
use std::process::Command;

#[derive(Debug, Parser)]
pub struct Options {
    /// Set the release flag when compiling eBPF bytecode
    #[clap(long)]
    pub release: bool,
}

pub fn build_ebpf(opts: Options) -> Result<()> {
    let mut args = vec![
        "build",
        "--package",
        "hello-ebpf-ebpf",
        "--target",
        "bpfel-unknown-none",
        "-Z",
        "build-std=core",
    ];

    if opts.release {
        args.push("--release");
    }

    let status = Command::new("cargo")
        .args(&args)
        .status()
        .context("Failed to run cargo build for eBPF")?;

    if !status.success() {
        bail!("Failed to compile eBPF program");
    }

    Ok(())
}
