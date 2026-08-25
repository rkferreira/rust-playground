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
