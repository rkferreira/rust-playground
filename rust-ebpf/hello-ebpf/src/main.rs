use aya::programs::{Xdp, XdpFlags};
use aya::{include_bytes_aligned, Ebpf};
use aya_log::EbpfLogger;
use clap::Parser;
use log::{info, warn};
use tokio::signal;

#[derive(Debug, Parser)]
#[command(author, version, about = "Hello World eBPF loader", long_about = None)]
struct Opt {
    #[arg(short, long, default_value = "lo")]
    iface: String,

    /// Force SKB (Generic XDP) mode
    #[arg(long)]
    skb_mode: bool,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    // Default log level to 'info' so logs show up without requiring RUST_LOG=info
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Bump RLIMIT_MEMLOCK to allow loading BPF programs into kernel
    #[cfg(target_os = "linux")]
    {
        let rlim = libc::rlimit {
            rlim_cur: libc::RLIM_INFINITY,
            rlim_max: libc::RLIM_INFINITY,
        };
        let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
        if ret != 0 {
            warn!("Failed to increase rlimit RLIMIT_MEMLOCK: {}", ret);
        }
    }

    // Load compiled eBPF bytecode
    #[cfg(debug_assertions)]
    let mut bpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/hello-ebpf-ebpf"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/hello-ebpf-ebpf"
    ))?;

    if let Err(e) = EbpfLogger::init(&mut bpf) {
        warn!("Failed to initialize eBPF logger: {}", e);
    }

    let program: &mut Xdp = bpf.program_mut("hello_ebpf").unwrap().try_into()?;
    program.load()?;

    let attach_flags = if opt.skb_mode {
        XdpFlags::SKB_MODE
    } else {
        XdpFlags::default()
    };

    if let Err(e) = program.attach(&opt.iface, attach_flags) {
        if !opt.skb_mode {
            warn!("Failed to attach XDP with default flags: {}. Falling back to SKB_MODE...", e);
            program.attach(&opt.iface, XdpFlags::SKB_MODE)?;
        } else {
            return Err(e.into());
        }
    }

    info!("eBPF program 'hello_ebpf' loaded and attached to {}", opt.iface);
    info!("Waiting for incoming packets... Press Ctrl-C to terminate.");

    signal::ctrl_c().await?;
    info!("Exiting and detaching eBPF program...");

    Ok(())
}
