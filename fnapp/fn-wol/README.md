# fn-wol — 飞牛 WOL 远程唤醒工具

轻量级局域网远程唤醒应用，专为飞牛 OS 原生应用市场设计。

## 安装

从 [Releases](https://github.com/xuewuerduo/fnapp/releases) 下载 `.fpk` 包，在飞牛 OS 应用中心手动安装。

## 功能特性

- **一键扫描局域网设备** — 自动获取网段，扫描在线设备 IP + MAC
- **手动添加设备** — 适合关机但支持 WOL 的设备，MAC 格式自动兼容
- **一键唤醒** — 发送魔术包远程开机
- **设备列表管理** — 备注编辑、删除，MAC 自动去重
- **数据持久化** — JSON 存储，重启不丢失
- **极致轻量** — 静态编译约 1.5MB，零系统依赖，长期稳定运行
- **跨平台** — 支持 ARM64 / x86_64 Linux

## 使用说明

安装后在飞牛桌面打开 fn-wol，进入 Web 界面：

1. 点击 **扫描设备** → 自动列出在线设备
2. 点击设备名称 → 填写备注（电脑、NAS、笔记本等）
3. 点击 **唤醒** → 远程开机
4. 不需要的设备可删除

手动添加适合不在线但支持 WOL 的设备，MAC 地址支持 `aa:bb:cc:dd:ee:ff`、`aa-bb-cc-dd-ee-ff`、`aabbccddeeff` 格式。

## 首次安装

安装向导会提示输入服务端口（默认 10101），安装后可在应用设置中修改。

## 自行编译

```bash
# 安装 Rust + zigbuild
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install cargo-zigbuild

# 编译（ARM64）
cargo zigbuild --release --target aarch64-unknown-linux-musl

# 编译（x86_64）
cargo zigbuild --release --target x86_64-unknown-linux-musl
```

## 技术参数

| 项目 | 参数 |
|------|------|
| 端口 | 可配置（默认 10101） |
| 架构 | ARM64 / x86_64 |
| 编译 | musl 静态链接 |
| 体积 | ~1.5MB |
| 框架 | Rust + tiny_http |
| 运行权限 | 普通用户（无需 root） |
