use std::net::UdpSocket;
use std::time::Duration;

use crate::network::{get_local_network, get_scan_range};
use crate::oui;

pub struct ScanResult {
    pub ip: String,
    pub mac: String,
    pub vendor: Option<String>,
}

fn vendor_for(mac: &str) -> Option<String> {
    oui::lookup_vendor_local(mac).map(|s| s.to_string())
}

/// 扫描局域网在线设备
/// 1. 获取本机网段
/// 2. 并发发送 UDP 包触发 ARP 解析
/// 3. 读取系统 ARP 表获取 IP-MAC 映射
pub fn scan_network() -> Result<Vec<ScanResult>, String> {
    let (ip, netmask) = get_local_network()
        .ok_or("无法获取本机网络信息，请检查网络连接")?;

    let range = get_scan_range(ip, netmask);

    // 并发发送 UDP 包，触发内核 ARP 解析
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

    // 等待 ARP 表更新
    std::thread::sleep(Duration::from_millis(1500));

    let mut results = read_arp_table()?;

    // 过滤多播/回环地址，基于 MAC 去重
    let mut seen = std::collections::HashSet::new();
    results.retain(|r| {
        let octets: Vec<u8> = r.ip.split('.').filter_map(|s| s.parse().ok()).collect();
        if octets.len() != 4 {
            return false;
        }
        // 跳过多播 (224+) 和回环 (127)
        if octets[0] >= 224 || octets[0] == 127 {
            return false;
        }
        seen.insert(r.mac.clone())
    });

    Ok(results)
}

#[cfg(target_os = "linux")]
fn read_arp_table() -> Result<Vec<ScanResult>, String> {
    let arp_content = std::fs::read_to_string("/proc/net/arp")
        .map_err(|e| format!("读取 ARP 表失败: {}", e))?;

    let mut results = Vec::new();
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
            let mac = mac_addr.to_lowercase();
            results.push(ScanResult {
                ip: ip_addr.to_string(),
                mac: mac.clone(),
                vendor: vendor_for(&mac),
            });
        }
    }

    Ok(results)
}

#[cfg(target_os = "windows")]
fn read_arp_table() -> Result<Vec<ScanResult>, String> {
    use std::process::Command;

    let output = Command::new("arp")
        .arg("-a")
        .output()
        .map_err(|e| format!("执行 arp 命令失败: {}", e))?;

    let content = String::from_utf8_lossy(&output.stdout);

    let mut results = Vec::new();
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
                    results.push(ScanResult {
                        ip: ip_addr.to_string(),
                        mac: mac_normalized.clone(),
                        vendor: vendor_for(&mac_normalized),
                    });
                }
            }
        }
    }

    Ok(results)
}
