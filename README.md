# System Proxy

一个通用的系统代理配置工具。

## 功能

- 填写 `HTTP / HTTPS / SOCKS` 代理地址（`host:port`）
- 支持 `手动 / PAC` 两种代理模式切换
- 可填写 PAC 地址（URL）
- 填写绕过列表（如 `localhost,127.0.0.0/8;10.0.0.0/8`）
- 开启代理
- 关闭代理
- 查看代理状态
- 内置操作日志展示
- 配置持久化（保存后自动写入系统配置目录）
- 配置导入/导出（JSON）
- 导入/导出文件选择器（无需手输路径）

## 技术选型

- Rust
- `fltk-rs`（跨平台桌面）

## 运行

```bash
cd sysproxy-gui
cargo run
```

提示：

- 可通过 `导入导出路径` + `导入配置/导出配置` 在设备间迁移配置
- 可通过 `选导入文件/选导出文件` 调起系统文件对话框快速选择 JSON 文件
- PAC 模式下可按规则控制“走代理 / 直连”
- 关闭窗口会直接退出应用进程

## 打包

```bash
cd sysproxy-gui
cargo build --release
```

生成文件：

- macOS/Linux: `target/release/sysproxy-gui`
- Windows: `target/release/sysproxy-gui.exe`

## 平台实现说明

- macOS: 使用 `networksetup` 设置 `web/secureweb/socks` 代理
- Windows: 使用 `reg` 写入 WinINet 代理键值
- Linux(GNOME): 使用 `gsettings` 设置系统代理

配置文件路径：

- macOS: `~/Library/Application Support/sysproxy-gui/config.json`
- Linux: `~/.config/sysproxy-gui/config.json`
- Windows: `%APPDATA%\sysproxy-gui\config.json`

## 注意

- macOS 某些网络服务设置可能需要权限
- Windows 修改注册表后，部分程序可能需要重启才完全生效
- Linux 目前按 GNOME 路径实现，非 GNOME 桌面环境可能不生效
