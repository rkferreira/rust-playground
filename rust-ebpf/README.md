# Hello World eBPF in Rust (Aya)

> **Disclaimer**: This project and its documentation were generated with the assistance of **Gemini** (Google DeepMind).

---

A comprehensive, end-to-end "Hello World" eBPF project written in pure Rust using the [Aya](https://aya-rs.dev/) framework.

This repository demonstrates how to write, compile, load, and test an eBPF program with an **XDP (eXpress Data Path)** network hook and stream structured logs directly from the Linux kernel to userspace.

---

## Table of Contents

- [Overview & Architecture](#overview--architecture)
- [Project Structure](#project-structure)
- [Deep Dive into the Code](#deep-dive-into-the-code)
  - [1. Kernel-Space eBPF (`hello-ebpf-ebpf`)](#1-kernel-space-ebpf-hello-ebpf-ebpf)
  - [2. Userspace Loader (`hello-ebpf`)](#2-userspace-loader-hello-ebpf)
  - [3. Shared Data Types (`hello-ebpf-common`)](#3-shared-data-types-hello-ebpf-common)
  - [4. Build Automation (`xtask`)](#4-build-automation-xtask)
- [Prerequisites (Native Linux)](#prerequisites-native-linux)
- [Testing Environments](#testing-environments)
  - [Option A: Testing with Podman (Container)](#option-a-testing-with-podman-container)
  - [Option B: Testing on Native Linux](#option-b-testing-on-native-linux)
  - [Option C: Testing with QEMU (Virtual Machine)](#option-c-testing-with-qemu-virtual-machine)
  - [Option D: Testing with Lima (macOS VM)](#option-d-testing-with-lima-macos-vm)
- [Key Concepts & Gotchas](#key-concepts--gotchas)

---

## Overview & Architecture

**eBPF (Extended Berkeley Packet Filter)** allows executing sandboxed, verified code within the Linux kernel space without modifying the kernel source or loading out-of-tree kernel modules.

In this project, we use **Aya**, a pure-Rust eBPF framework that eliminates the need for C toolchains (such as `clang` / `libbpf` headers) by compiling Rust code directly to the `bpfel-unknown-none` target architecture.

```
                    +-------------------------------------------------------+
                    |                      Linux Kernel                     |
                    |                                                       |
 [Incoming Packet] ---> [ Network Driver / XDP Hook ]                      |
                                   |                                        |
                                   v                                        |
                    +------------------------------+                        |
                    | hello-ebpf-ebpf (Kernel BPF) |                        |
                    | - Inspects packet length     |                        |
                    | - aya_log_ebpf::info!(...)   |                        |
                    | - Returns XDP_PASS           |                        |
                    +--------------+---------------+                        |
                                   | (eBPF Ring Buffer / Perf Event Array)  |
                    +--------------v----------------------------------------+
                                   |
                                   v
                    +------------------------------+
                    |    hello-ebpf (Userspace)    |
                    | - Ebpf::load(...)            |
                    | - EbpfLogger::init(...)      |
                    | - Prints logs to stdout      |
                    +------------------------------+
```

---

## Project Structure

```
rust-ebpf/
├── Cargo.toml                  # Workspace definition
├── .cargo/config.toml          # Cargo alias & eBPF target compiler flags
├── rust-toolchain.toml         # Pins nightly channel + rust-src component
├── Containerfile               # Podman/Docker image definition
├── hello-ebpf/                 # Userspace application (loads & logs)
│   ├── Cargo.toml
│   └── src/main.rs
├── hello-ebpf-ebpf/            # Kernel eBPF program (no_std, XDP hook)
│   ├── Cargo.toml
│   └── src/main.rs
├── hello-ebpf-common/          # Shared structs (repr(C)) between kernel & user
│   ├── Cargo.toml
│   └── src/lib.rs
└── xtask/                      # Build tasks (cargo xtask build-ebpf / run)
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── build_ebpf.rs
        └── run.rs
```

---

## Deep Dive into the Code

### 1. Kernel-Space eBPF (`hello-ebpf-ebpf`)

File: [`hello-ebpf-ebpf/src/main.rs`](file:///Users/rodrigoke/git/rust-playground/rust-ebpf/hello-ebpf-ebpf/src/main.rs)

The eBPF program runs inside kernel space with strict sandboxing and verifier rules:

```rust
#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action,
    macros::xdp,
    programs::XdpContext,
};
use aya_log_ebpf::info;

#[xdp]
pub fn hello_ebpf(ctx: XdpContext) -> u32 {
    match try_hello_ebpf(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_hello_ebpf(ctx: XdpContext) -> Result<u32, ()> {
    let len = ctx.data_end() - ctx.data();
    info!(&ctx, "Hello, World from eBPF! Received packet (len: {} bytes)", len);
    Ok(xdp_action::XDP_PASS)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
```

#### Key Elements:
- `#![no_std]` & `#![no_main]`: The kernel environment has no standard library (`std`) and no conventional `main()` entrypoint.
- `#[xdp]`: Macro provided by `aya-ebpf` designating this function as an eXpress Data Path program entrypoint.
- `ctx: XdpContext`: Contains direct memory pointers to the packet payload:
  - `ctx.data()`: Pointer to the start of the packet header/data.
  - `ctx.data_end()`: Pointer to the end of the packet data.
- `aya_log_ebpf::info!`: Serializes the formatted string into an eBPF perf event / ring buffer that the userspace loader listens to.
- `xdp_action::XDP_PASS`: Returns `XDP_PASS` to let the packet continue up the normal Linux network stack.
- `panic_handler`: Required for `no_std` binaries; uses `unreachable_unchecked()` because panics cannot safely occur in the kernel.

---

### 2. Userspace Loader (`hello-ebpf`)

File: [`hello-ebpf/src/main.rs`](file:///Users/rodrigoke/git/rust-playground/rust-ebpf/hello-ebpf/src/main.rs)

The userspace program is an asynchronous Tokio binary responsible for loading the bytecode into the kernel and managing its lifecycle:

```rust
use aya::programs::{Xdp, XdpFlags};
use aya::{include_bytes_aligned, Ebpf};
use aya_log::EbpfLogger;
use clap::Parser;
use log::{info, warn};
use tokio::signal;

#[derive(Debug, Parser)]
struct Opt {
    #[arg(short, long, default_value = "lo")]
    iface: String,
    #[arg(long)]
    skb_mode: bool,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    // Default log level to 'info' so events are logged without needing RUST_LOG=info
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Bump RLIMIT_MEMLOCK to allow loading BPF programs into kernel
    #[cfg(target_os = "linux")]
    {
        let rlim = libc::rlimit {
            rlim_cur: libc::RLIM_INFINITY,
            rlim_max: libc::RLIM_INFINITY,
        };
        unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    }

    // Embed the compiled eBPF bytecode into the binary at compile time
    #[cfg(debug_assertions)]
    let mut bpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/hello-ebpf-ebpf"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/hello-ebpf-ebpf"
    ))?;

    // Connect aya-log logger to stream kernel events
    if let Err(e) = EbpfLogger::init(&mut bpf) {
        warn!("Failed to initialize eBPF logger: {}", e);
    }

    // Retrieve and attach the XDP program to the network interface
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
```

#### Key Elements:
- `RLIMIT_MEMLOCK`: Older Linux kernels lock BPF map/program memory under `RLIMIT_MEMLOCK`. Increasing this limit prevents `EPERM` when loading eBPF bytecode.
- `include_bytes_aligned!`: Embeds the compiled eBPF ELF binary into the userspace binary at compile time with correct memory alignment.
- `EbpfLogger::init`: Polls the kernel-side log ring buffer and converts records into standard Rust `log` crate events.
- `XdpFlags::SKB_MODE` Fallback: Loopback (`lo`) and virtual container interfaces (`veth`) do not implement native driver XDP, so generic SKB mode (`SKB_MODE`) is used.
- `signal::ctrl_c()`: Awaits `SIGINT` and automatically detaches the program when the process exits.

---

### 3. Shared Data Types (`hello-ebpf-common`)

File: [`hello-ebpf-common/src/lib.rs`](file:///Users/rodrigoke/git/rust-playground/rust-ebpf/hello-ebpf-common/src/lib.rs)

Contains structures shared across the kernel-userspace boundary:

```rust
#![no_std]

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PacketLog {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub protocol: u16,
    pub length: u32,
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for PacketLog {}
```

- `#[repr(C)]`: Guarantees predictable C-compatible memory layout and struct alignment across architectures.
- `aya::Pod`: Marks the struct as Plain Old Data, enabling zero-copy reads from BPF maps.

---

### 4. Build Automation (`xtask`)

Files: [`xtask/src/`](file:///Users/rodrigoke/git/rust-playground/rust-ebpf/xtask/src/)

The `xtask` pattern provides standardized build and run tasks:
- `cargo xtask build-ebpf`: Compiles the eBPF crate with `cargo build --package hello-ebpf-ebpf --target bpfel-unknown-none -Z build-std=core`.
- `cargo xtask run`: First triggers `build-ebpf`, then launches the userspace loader.

---

## Prerequisites (Native Linux)

### 1. Install Nightly Rust & `rust-src`
```bash
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
```

### 2. Install `bpf-linker`
Using `cargo-binstall` (recommended to avoid building LLVM from source):
```bash
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
cargo binstall -y bpf-linker
```

---

## Testing Environments

### Option A: Testing with Podman (Container)

Podman allows you to run and test eBPF without installing Linux toolchains directly on your host.

#### 1. (macOS only) Start Podman Machine:
```bash
podman machine init --cpus 2 --memory 4096
podman machine start
```

#### 2. Build the Container Image:
```bash
podman build -t hello-ebpf -f Containerfile .
```

#### 3. Run with eBPF Privileges (Terminal 1):
```bash
podman run --name hello-ebpf --rm -it --privileged --net=host hello-ebpf
```

You will see:
```text
[INFO  hello_ebpf] eBPF program 'hello_ebpf' loaded and attached to lo
[INFO  hello_ebpf] Waiting for incoming packets... Press Ctrl-C to terminate.
```

#### 4. Trigger Packets (Terminal 2):
```bash
podman exec -it hello-ebpf ping -c 3 127.0.0.1
```

Terminal 1 will print:
```text
[INFO  hello_ebpf_ebpf] Hello, World from eBPF! Received packet (len: 84 bytes)
[INFO  hello_ebpf_ebpf] Hello, World from eBPF! Received packet (len: 84 bytes)
[INFO  hello_ebpf_ebpf] Hello, World from eBPF! Received packet (len: 84 bytes)
```

---

### Option B: Testing on Native Linux

On a Linux machine with root access:

```bash
# 1. Build the eBPF bytecode
cargo xtask build-ebpf --release

# 2. Run the loader on loopback (lo)
sudo $(which cargo) xtask run --release -- --iface lo
```

In another terminal:
```bash
ping -c 3 127.0.0.1
```

---

### Option C: Testing with QEMU (Virtual Machine)

Test inside a full virtualized Linux kernel on QEMU:

#### 1. Download an Ubuntu Cloud Image:
```bash
# macOS Apple Silicon (ARM64):
curl -LO https://cloud-images.ubuntu.com/noble/current/noble-server-cloudimg-arm64.img

# x86_64:
# curl -LO https://cloud-images.ubuntu.com/noble/current/noble-server-cloudimg-amd64.img
```

#### 2. Resize Image:
```bash
qemu-img resize noble-server-cloudimg-arm64.img 15G
```

#### 3. Launch QEMU with 9p Host Directory Sharing:
```bash
qemu-system-aarch64 \
  -M virt,accel=hvf \
  -cpu host \
  -m 4G \
  -smp 2 \
  -bios /opt/homebrew/share/qemu/edk2-aarch64-code.fd \
  -drive if=virtio,file=noble-server-cloudimg-arm64.img,format=qcow2 \
  -netdev user,id=net0,hostfwd=tcp::2222-:22 \
  -device virtio-net-pci,netdev=net0 \
  -virtfs local,path=$(pwd),mount_tag=ebpf_share,security_model=none,id=ebpf_share \
  -nographic
```

#### 4. Inside the VM:
```bash
sudo mkdir -p /mnt/ebpf
sudo mount -t 9p -o trans=virtio ebpf_share /mnt/ebpf
cd /mnt/ebpf
sudo cargo xtask run -- --iface lo
```

---

### Option D: Testing with Lima (macOS VM)

[Lima](https://lima-vm.io/) provides automatic directory mounting and networking on macOS:

```bash
limactl start template://ubuntu
limactl shell ubuntu

# Inside Lima VM:
cd /Users/yourusername/path/to/rust-ebpf
sudo cargo xtask run -- --iface lo
```

---

## Key Concepts & Gotchas

1. **XDP Driver Mode vs Generic Mode (`SKB_MODE`)**:
   - Native XDP runs at the network card driver level before memory allocation.
   - Virtual interfaces (`lo`, `veth`, Docker networks) require generic `SKB_MODE`, which runs slightly later in the kernel network stack.
2. **eBPF Verifier**:
   - All eBPF programs must pass the Linux kernel verifier. Loops must be bounded, memory access must be checked, and programs must be guaranteed to terminate.
3. **`RLIMIT_MEMLOCK`**:
   - Linux limits locked memory per process. Modern kernels (5.11+) use cgroups for eBPF accounting, but older kernels require `setrlimit(RLIMIT_MEMLOCK, ...)` in userspace.
