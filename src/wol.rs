use std::net::UdpSocket;

/// 构造 WOL 魔术包: 6 字节 0xFF + 16 次 MAC 地址
fn build_magic_packet(mac_bytes: &[u8; 6]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(102);
    packet.extend_from_slice(&[0xFF; 6]);
    for _ in 0..16 {
        packet.extend_from_slice(mac_bytes);
    }
    packet
}

/// 发送 WOL 魔术包到广播地址 (255.255.255.255:9)
pub fn send_wol(mac_bytes: &[u8; 6]) -> std::io::Result<()> {
    let packet = build_magic_packet(mac_bytes);
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.send_to(&packet, "255.255.255.255:9")?;
    Ok(())
}
