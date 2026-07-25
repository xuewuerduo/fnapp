use std::net::UdpSocket;
use std::time::Duration;

use crate::network::{get_local_network, get_scan_range};
use crate::oui;

pub struct ScanResult {
    pub ip: String,
    pub mac: String,
    pub vendor: Option<String>,
    pub wol_support: bool,
}

const VIRTUAL_OUI: &[&str] = &[
    "005056", "000C29", "001C42", "0050B6", // VMware
    "080027",                               // VirtualBox
    "00155D", "0003FF",                     // Hyper-V
    "525400",                               // QEMU/KVM
    "00163E",                               // Xen
];

fn check_wol_support(mac: &str) -> bool {
    let prefix: String = mac
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect::<String>()
        .to_uppercase();
    if prefix.len() < 6 {
        return false;
    }
    !VIRTUAL_OUI.contains(&&prefix[..6])
}

fn vendor_for(mac: &str) -> Option<String> {
    oui::lookup_vendor_local(mac).map(|s| s.to_string())
}

fn send_udp_probes(ips: &[String]) {
    for ip in ips {
        if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
            let _ = sock.set_read_timeout(Some(Duration::from_millis(10)));
            let _ = sock.send_to(&[0u8], format!("{}:9", ip));
        }
    }
}

#[cfg(target_os = "linux")]
fn read_arp_pairs() -> Result<Vec<(String, String)>, String> {
    let arp_content = std::fs::read_to_string("/proc/net/arp")
        .map_err(|e| format!("读取 ARP 表失败: {}", e))?;

    let mut pairs = Vec::new();
    for (i, line) in arp_content.lines().enumerate() {
        if i == 0 {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 {
            continue;
        }
        let ip_addr = fields[0];
        let mac_addr = fields[3];

        if mac_addr == "00:00:00:00:00:00" || mac_addr == "FF:FF:FF:FF:FF:FF" {
            continue;
        }
        if mac_addr.contains(':') {
            pairs.push((ip_addr.to_string(), mac_addr.to_lowercase()));
        }
    }

    Ok(pairs)
}

#[cfg(target_os = "windows")]
fn read_arp_pairs() -> Result<Vec<(String, String)>, String> {
    use std::process::Command;

    let output = Command::new("arp")
        .arg("-a")
        .output()
        .map_err(|e| format!("执行 arp 命令失败: {}", e))?;

    let content = String::from_utf8_lossy(&output.stdout);

    let mut pairs = Vec::new();
    for line in content.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 2 {
            let ip_addr = fields[0];
            let mac_addr = fields[1];

            if mac_addr.contains('-') && mac_addr.len() == 17 {
                let mac_normalized = mac_addr.to_lowercase().replace('-', ":");
                if mac_normalized != "00:00:00:00:00:00"
                    && mac_normalized != "ff:ff:ff:ff:ff:ff"
                {
                    pairs.push((ip_addr.to_string(), mac_normalized));
                }
            }
        }
    }

    Ok(pairs)
}

fn read_arp_table() -> Result<Vec<ScanResult>, String> {
    let pairs = read_arp_pairs()?;
    Ok(pairs
        .into_iter()
        .map(|(ip, mac)| ScanResult {
            ip,
            mac: mac.clone(),
            vendor: vendor_for(&mac),
            wol_support: false,
        })
        .collect())
}

pub fn scan_network() -> Result<Vec<ScanResult>, String> {
    let (ip, netmask) = get_local_network()
        .ok_or("无法获取本机网络信息，请检查网络连接")?;

    let range = get_scan_range(ip, netmask);

    let handles: Vec<_> = range
        .iter()
        .map(|target_ip| {
            let ip = *target_ip;
            std::thread::spawn(move || {
                if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
                    let _ = sock.set_read_timeout(Some(Duration::from_millis(10)));
                    let _ = sock.send_to(&[0u8], format!("{}:9", ip));
                }
            })
        })
        .collect();

    for h in handles {
        let _ = h.join();
    }

    std::thread::sleep(Duration::from_millis(1500));

    let mut results = read_arp_table()?;

    for r in results.iter_mut() {
        r.wol_support = check_wol_support(&r.mac);
    }

    let mut seen = std::collections::HashSet::new();
    results.retain(|r| {
        if !r.wol_support {
            return false;
        }
        let octets: Vec<u8> = r.ip.split('.').filter_map(|s| s.parse().ok()).collect();
        if octets.len() != 4 {
            return false;
        }
        if octets[0] >= 224 || octets[0] == 127 {
            return false;
        }
        seen.insert(r.mac.clone())
    });

    Ok(results)
}

pub fn check_online_presence(devices: &[(String, String)]) -> Result<Vec<(String, bool)>, String> {
    let ips: Vec<String> = devices.iter().map(|(_, ip)| ip.clone()).collect();

    send_udp_probes(&ips);

    std::thread::sleep(Duration::from_millis(1500));

    let arp_entries = read_arp_pairs()?;

    Ok(devices
        .iter()
        .map(|(mac, ip)| {
            let found = arp_entries.iter().any(|(aip, amac)| aip == ip && amac == mac);
            (mac.clone(), found)
        })
        .collect())
}
