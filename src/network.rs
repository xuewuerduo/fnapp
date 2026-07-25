use std::net::Ipv4Addr;
use if_addrs::{get_if_addrs, IfAddr};

/// 获取本机 IPv4 地址和子网掩码（跳过 loopback 和 link-local）
pub fn get_local_network() -> Option<(Ipv4Addr, Ipv4Addr)> {
    let interfaces = get_if_addrs().ok()?;
    for iface in interfaces {
        if iface.is_loopback() {
            continue;
        }
        if let IfAddr::V4(v4) = iface.addr {
            if v4.is_link_local() {
                continue;
            }
            return Some((v4.ip, v4.netmask));
        }
    }
    None
}

/// 根据 IP 和子网掩码计算扫描范围（排除网络地址和广播地址）
pub fn get_scan_range(ip: Ipv4Addr, netmask: Ipv4Addr) -> Vec<Ipv4Addr> {
    let ip_u32 = u32::from(ip);
    let mask_u32 = u32::from(netmask);
    let network = ip_u32 & mask_u32;
    let broadcast = network | !mask_u32;

    let mut range = Vec::new();
    for i in (network + 1)..broadcast {
        range.push(Ipv4Addr::from(i));
    }
    range
}
