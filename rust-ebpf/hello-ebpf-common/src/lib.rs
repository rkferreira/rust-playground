#![no_std]

/// Shared packet metadata or event structure that can be transferred between
/// the eBPF kernel program and userspace via perf event arrays / ring buffers / maps.
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
