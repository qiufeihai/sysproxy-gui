use fltk::{
    app,
    button::Button,
    dialog::{FileDialogType, NativeFileChooser},
    enums::Font,
    frame::Frame,
    input::Input,
    menu::Choice,
    prelude::*,
    text::{TextBuffer, TextDisplay},
    window::Window,
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, fs, path::PathBuf, process::Command, rc::Rc};

#[derive(Default, Serialize, Deserialize, Clone)]
struct AppConfig {
    http_proxy: String,
    https_proxy: String,
    socks_proxy: String,
    bypass_list: String,
    proxy_mode: String,
    pac_url: String,
    import_export_path: String,
}

impl AppConfig {
    fn with_defaults() -> Self {
        Self {
            http_proxy: "127.0.0.1:7788".to_string(),
            https_proxy: "127.0.0.1:7788".to_string(),
            socks_proxy: "127.0.0.1:7777".to_string(),
            bypass_list: "localhost,127.0.0.0/8;10.0.0.0/8;172.16.0.0/12;192.168.0.0/16"
                .to_string(),
            proxy_mode: "manual".to_string(),
            pac_url: "".to_string(),
            import_export_path: "config-export.json".to_string(),
        }
    }
}

#[derive(Default)]
struct AppState {
    logs: Vec<String>,
}

impl AppState {
    fn log(&mut self, msg: &str) {
        self.logs.push(msg.to_string());
        if self.logs.len() > 200 {
            let keep_from = self.logs.len() - 200;
            self.logs.drain(0..keep_from);
        }
    }

    fn logs_text(&self) -> String {
        self.logs.join("\n")
    }
}

fn run_cmd(cmd: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("执行失败 {cmd}: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(format!(
            "{cmd} {:?} 失败: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

fn config_path() -> Result<PathBuf, String> {
    let base =
        dirs::config_dir().ok_or_else(|| "无法获取系统配置目录（config_dir）".to_string())?;
    Ok(base.join("sysproxy-gui").join("config.json"))
}

fn load_config() -> AppConfig {
    let defaults = AppConfig::with_defaults();
    let path = match config_path() {
        Ok(p) => p,
        Err(_) => return defaults,
    };
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return defaults,
    };
    match serde_json::from_str::<AppConfig>(&content) {
        Ok(mut cfg) => {
            if cfg.http_proxy.trim().is_empty() {
                cfg.http_proxy = defaults.http_proxy;
            }
            if cfg.https_proxy.trim().is_empty() {
                cfg.https_proxy = defaults.https_proxy;
            }
            if cfg.socks_proxy.trim().is_empty() {
                cfg.socks_proxy = defaults.socks_proxy;
            }
            if cfg.bypass_list.trim().is_empty() {
                cfg.bypass_list = defaults.bypass_list;
            }
            if cfg.proxy_mode.trim().is_empty() {
                cfg.proxy_mode = defaults.proxy_mode;
            }
            if cfg.pac_url.trim().is_empty() {
                cfg.pac_url = defaults.pac_url;
            }
            if cfg.proxy_mode != "manual" && cfg.proxy_mode != "pac" {
                cfg.proxy_mode = "manual".to_string();
            }
            if cfg.import_export_path.trim().is_empty() {
                cfg.import_export_path = defaults.import_export_path;
            }
            cfg
        }
        Err(_) => defaults,
    }
}

fn save_config(cfg: &AppConfig) -> Result<String, String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败 {}: {e}", parent.display()))?;
    }
    let data = serde_json::to_string_pretty(cfg).map_err(|e| format!("序列化配置失败: {e}"))?;
    fs::write(&path, data).map_err(|e| format!("写入配置失败 {}: {e}", path.display()))?;
    Ok(format!("配置已保存到 {}", path.display()))
}

fn export_config(path: &str, cfg: &AppConfig) -> Result<String, String> {
    let p = PathBuf::from(path);
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建导出目录失败 {}: {e}", parent.display()))?;
        }
    }
    let data = serde_json::to_string_pretty(cfg).map_err(|e| format!("序列化配置失败: {e}"))?;
    fs::write(&p, data).map_err(|e| format!("导出配置失败 {}: {e}", p.display()))?;
    Ok(format!("配置已导出到 {}", p.display()))
}

fn import_config(path: &str) -> Result<AppConfig, String> {
    let p = PathBuf::from(path);
    let content =
        fs::read_to_string(&p).map_err(|e| format!("读取导入文件失败 {}: {e}", p.display()))?;
    let mut cfg =
        serde_json::from_str::<AppConfig>(&content).map_err(|e| format!("解析配置失败: {e}"))?;
    let defaults = AppConfig::with_defaults();
    if cfg.http_proxy.trim().is_empty() {
        cfg.http_proxy = defaults.http_proxy;
    }
    if cfg.https_proxy.trim().is_empty() {
        cfg.https_proxy = defaults.https_proxy;
    }
    if cfg.socks_proxy.trim().is_empty() {
        cfg.socks_proxy = defaults.socks_proxy;
    }
    if cfg.bypass_list.trim().is_empty() {
        cfg.bypass_list = defaults.bypass_list;
    }
    if cfg.proxy_mode.trim().is_empty() {
        cfg.proxy_mode = defaults.proxy_mode;
    }
    if cfg.pac_url.trim().is_empty() {
        cfg.pac_url = defaults.pac_url;
    }
    if cfg.proxy_mode != "manual" && cfg.proxy_mode != "pac" {
        cfg.proxy_mode = "manual".to_string();
    }
    if cfg.import_export_path.trim().is_empty() {
        cfg.import_export_path = defaults.import_export_path;
    }
    Ok(cfg)
}

fn choose_path(dialog_type: FileDialogType, initial: &str) -> Result<String, String> {
    let mut chooser = NativeFileChooser::new(dialog_type);
    chooser.set_filter("*.json");
    if !initial.trim().is_empty() {
        chooser.set_preset_file(initial);
    }
    chooser.show();
    let file = chooser.filename();
    if file.as_os_str().is_empty() {
        return Err("已取消选择".to_string());
    }
    Ok(file.to_string_lossy().to_string())
}

fn parse_host_port(v: &str) -> Result<(String, String), String> {
    let idx = v
        .rfind(':')
        .ok_or_else(|| format!("地址格式错误: {v}，需要 host:port"))?;
    if idx == 0 || idx == v.len() - 1 {
        return Err(format!("地址格式错误: {v}，需要 host:port"));
    }
    let host = v[..idx].trim().to_string();
    let port = v[idx + 1..].trim().to_string();
    let p: u16 = port.parse().map_err(|_| format!("端口非法: {port}"))?;
    if p == 0 {
        return Err("端口不能为 0".to_string());
    }
    Ok((host, p.to_string()))
}

#[cfg(target_os = "macos")]
fn mac_services() -> Result<Vec<String>, String> {
    let out = run_cmd("networksetup", &["-listallnetworkservices"])?;
    let services = out
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !s.starts_with("An asterisk"))
        .map(|s| s.trim_start_matches('*').trim().to_string())
        .collect::<Vec<_>>();
    Ok(services)
}

#[cfg(target_os = "macos")]
fn platform_enable_proxy(
    http_proxy: &str,
    https_proxy: &str,
    socks_proxy: &str,
    _bypass_list: &str,
) -> Result<String, String> {
    let (http_host, http_port) = parse_host_port(http_proxy)?;
    let (https_host, https_port) = parse_host_port(https_proxy)?;
    let (socks_host, socks_port) = parse_host_port(socks_proxy)?;
    let services = mac_services()?;
    for svc in services {
        run_cmd("networksetup", &["-setwebproxy", &svc, &http_host, &http_port])?;
        run_cmd("networksetup", &["-setwebproxystate", &svc, "on"])?;
        run_cmd(
            "networksetup",
            &["-setsecurewebproxy", &svc, &https_host, &https_port],
        )?;
        run_cmd("networksetup", &["-setsecurewebproxystate", &svc, "on"])?;
        run_cmd(
            "networksetup",
            &["-setsocksfirewallproxy", &svc, &socks_host, &socks_port],
        )?;
        run_cmd("networksetup", &["-setsocksfirewallproxystate", &svc, "on"])?;
    }
    Ok("macOS 代理已启用".to_string())
}

#[cfg(target_os = "macos")]
fn platform_enable_pac(pac_url: &str) -> Result<String, String> {
    if pac_url.trim().is_empty() {
        return Err("PAC 地址不能为空".to_string());
    }
    let services = mac_services()?;
    for svc in services {
        run_cmd("networksetup", &["-setautoproxyurl", &svc, pac_url])?;
        run_cmd("networksetup", &["-setautoproxystate", &svc, "on"])?;
        run_cmd("networksetup", &["-setwebproxystate", &svc, "off"])?;
        run_cmd("networksetup", &["-setsecurewebproxystate", &svc, "off"])?;
        run_cmd("networksetup", &["-setsocksfirewallproxystate", &svc, "off"])?;
    }
    Ok("macOS PAC 规则代理已启用".to_string())
}

#[cfg(target_os = "macos")]
fn platform_disable_proxy() -> Result<String, String> {
    let services = mac_services()?;
    for svc in services {
        run_cmd("networksetup", &["-setwebproxystate", &svc, "off"])?;
        run_cmd("networksetup", &["-setsecurewebproxystate", &svc, "off"])?;
        run_cmd("networksetup", &["-setsocksfirewallproxystate", &svc, "off"])?;
        run_cmd("networksetup", &["-setautoproxystate", &svc, "off"])?;
    }
    Ok("macOS 代理已关闭".to_string())
}

#[cfg(target_os = "macos")]
fn platform_query_status() -> Result<String, String> {
    let mut all = String::new();
    for svc in mac_services()? {
        let web = run_cmd("networksetup", &["-getwebproxy", &svc])?;
        let secure = run_cmd("networksetup", &["-getsecurewebproxy", &svc])?;
        let socks = run_cmd("networksetup", &["-getsocksfirewallproxy", &svc])?;
        let auto = run_cmd("networksetup", &["-getautoproxyurl", &svc])?;
        all.push_str(&format!("[{svc}]\n{web}{secure}{socks}{auto}\n"));
    }
    Ok(all)
}

#[cfg(target_os = "windows")]
fn platform_enable_proxy(
    http_proxy: &str,
    https_proxy: &str,
    socks_proxy: &str,
    bypass_list: &str,
) -> Result<String, String> {
    let _ = parse_host_port(http_proxy)?;
    let _ = parse_host_port(https_proxy)?;
    let _ = parse_host_port(socks_proxy)?;
    let proxy_server = format!("http={http_proxy};https={https_proxy};socks={socks_proxy}");
    run_cmd(
        "reg",
        &[
            "add",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            "/v",
            "ProxyEnable",
            "/t",
            "REG_DWORD",
            "/d",
            "1",
            "/f",
        ],
    )?;
    run_cmd(
        "reg",
        &[
            "add",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            "/v",
            "ProxyServer",
            "/t",
            "REG_SZ",
            "/d",
            &proxy_server,
            "/f",
        ],
    )?;
    run_cmd(
        "reg",
        &[
            "add",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            "/v",
            "ProxyOverride",
            "/t",
            "REG_SZ",
            "/d",
            bypass_list,
            "/f",
        ],
    )?;
    Ok("Windows 代理已启用（WinINet）".to_string())
}

#[cfg(target_os = "windows")]
fn platform_enable_pac(pac_url: &str) -> Result<String, String> {
    if pac_url.trim().is_empty() {
        return Err("PAC 地址不能为空".to_string());
    }
    run_cmd(
        "reg",
        &[
            "add",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            "/v",
            "AutoConfigURL",
            "/t",
            "REG_SZ",
            "/d",
            pac_url,
            "/f",
        ],
    )?;
    run_cmd(
        "reg",
        &[
            "add",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            "/v",
            "ProxyEnable",
            "/t",
            "REG_DWORD",
            "/d",
            "0",
            "/f",
        ],
    )?;
    Ok("Windows PAC 规则代理已启用".to_string())
}

#[cfg(target_os = "windows")]
fn platform_disable_proxy() -> Result<String, String> {
    run_cmd(
        "reg",
        &[
            "add",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            "/v",
            "ProxyEnable",
            "/t",
            "REG_DWORD",
            "/d",
            "0",
            "/f",
        ],
    )?;
    let _ = run_cmd(
        "reg",
        &[
            "delete",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            "/v",
            "AutoConfigURL",
            "/f",
        ],
    );
    Ok("Windows 代理已关闭（WinINet）".to_string())
}

#[cfg(target_os = "windows")]
fn platform_query_status() -> Result<String, String> {
    run_cmd(
        "reg",
        &[
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
        ],
    )
}

#[cfg(target_os = "linux")]
fn platform_enable_proxy(
    http_proxy: &str,
    https_proxy: &str,
    socks_proxy: &str,
    bypass_list: &str,
) -> Result<String, String> {
    let (http_host, http_port) = parse_host_port(http_proxy)?;
    let (https_host, https_port) = parse_host_port(https_proxy)?;
    let (socks_host, socks_port) = parse_host_port(socks_proxy)?;

    run_cmd("gsettings", &["set", "org.gnome.system.proxy", "mode", "manual"])?;
    run_cmd(
        "gsettings",
        &["set", "org.gnome.system.proxy.http", "host", &format!("'{http_host}'")],
    )?;
    run_cmd(
        "gsettings",
        &["set", "org.gnome.system.proxy.http", "port", &http_port],
    )?;
    run_cmd(
        "gsettings",
        &[
            "set",
            "org.gnome.system.proxy.https",
            "host",
            &format!("'{https_host}'"),
        ],
    )?;
    run_cmd(
        "gsettings",
        &["set", "org.gnome.system.proxy.https", "port", &https_port],
    )?;
    run_cmd(
        "gsettings",
        &[
            "set",
            "org.gnome.system.proxy.socks",
            "host",
            &format!("'{socks_host}'"),
        ],
    )?;
    run_cmd(
        "gsettings",
        &["set", "org.gnome.system.proxy.socks", "port", &socks_port],
    )?;

    let bypass = bypass_list
        .split(';')
        .flat_map(|seg| seg.split(','))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("'{s}'"))
        .collect::<Vec<_>>()
        .join(", ");
    run_cmd(
        "gsettings",
        &[
            "set",
            "org.gnome.system.proxy",
            "ignore-hosts",
            &format!("[{bypass}]"),
        ],
    )?;
    Ok("Linux(GNOME) 代理已启用".to_string())
}

#[cfg(target_os = "linux")]
fn platform_enable_pac(pac_url: &str) -> Result<String, String> {
    if pac_url.trim().is_empty() {
        return Err("PAC 地址不能为空".to_string());
    }
    run_cmd("gsettings", &["set", "org.gnome.system.proxy", "mode", "auto"])?;
    run_cmd(
        "gsettings",
        &["set", "org.gnome.system.proxy", "autoconfig-url", pac_url],
    )?;
    Ok("Linux(GNOME) PAC 规则代理已启用".to_string())
}

#[cfg(target_os = "linux")]
fn platform_disable_proxy() -> Result<String, String> {
    run_cmd("gsettings", &["set", "org.gnome.system.proxy", "mode", "none"])?;
    Ok("Linux(GNOME) 代理已关闭".to_string())
}

#[cfg(target_os = "linux")]
fn platform_query_status() -> Result<String, String> {
    let mode = run_cmd("gsettings", &["get", "org.gnome.system.proxy", "mode"])?;
    let pac = run_cmd(
        "gsettings",
        &["get", "org.gnome.system.proxy", "autoconfig-url"],
    )?;
    let http_h = run_cmd("gsettings", &["get", "org.gnome.system.proxy.http", "host"])?;
    let http_p = run_cmd("gsettings", &["get", "org.gnome.system.proxy.http", "port"])?;
    let https_h = run_cmd("gsettings", &["get", "org.gnome.system.proxy.https", "host"])?;
    let https_p = run_cmd("gsettings", &["get", "org.gnome.system.proxy.https", "port"])?;
    let socks_h = run_cmd("gsettings", &["get", "org.gnome.system.proxy.socks", "host"])?;
    let socks_p = run_cmd("gsettings", &["get", "org.gnome.system.proxy.socks", "port"])?;
    Ok(format!(
        "mode: {mode}autoconfig-url: {pac}http.host: {http_h}http.port: {http_p}https.host: {https_h}https.port: {https_p}socks.host: {socks_h}socks.port: {socks_p}"
    ))
}

fn detect_proxy_enabled(status: &str) -> Option<bool> {
    #[cfg(target_os = "macos")]
    {
        if status.contains("Enabled: Yes") {
            return Some(true);
        }
        if status.contains("Enabled: No") {
            return Some(false);
        }
        return None;
    }
    #[cfg(target_os = "windows")]
    {
        if status.contains("ProxyEnable") && status.contains("0x1") {
            return Some(true);
        }
        if status.contains("AutoConfigURL") {
            return Some(true);
        }
        if status.contains("ProxyEnable") && status.contains("0x0") {
            return Some(false);
        }
        return None;
    }
    #[cfg(target_os = "linux")]
    {
        if status.contains("mode: 'manual'") {
            return Some(true);
        }
        if status.contains("mode: 'auto'") {
            return Some(true);
        }
        if status.contains("mode: 'none'") {
            return Some(false);
        }
        return None;
    }
}

fn proxy_state_text(status: &str) -> &'static str {
    match detect_proxy_enabled(status) {
        Some(true) => "开启中",
        Some(false) => "已关闭",
        None => "未知",
    }
}

fn main() {
    let app = app::App::default();
    let mut win = Window::new(100, 100, 940, 760, "System Proxy GUI");
    let cfg = load_config();

    let _title = Frame::new(20, 16, 900, 24, "通用系统代理工具");
    let _hint = Frame::new(
        20,
        42,
        900,
        20,
        "填写代理地址后，选择手动/PAC 模式，再点击开启/关闭/查看状态",
    );

    let _http_label = Frame::new(20, 80, 120, 30, "HTTP 代理");
    let mut http_input = Input::new(140, 80, 770, 30, "");
    http_input.set_value(&cfg.http_proxy);

    let _https_label = Frame::new(20, 120, 120, 30, "HTTPS 代理");
    let mut https_input = Input::new(140, 120, 770, 30, "");
    https_input.set_value(&cfg.https_proxy);

    let _socks_label = Frame::new(20, 160, 120, 30, "SOCKS 代理");
    let mut socks_input = Input::new(140, 160, 770, 30, "");
    socks_input.set_value(&cfg.socks_proxy);

    let _bypass_label = Frame::new(20, 200, 120, 30, "绕过列表");
    let mut bypass_input = Input::new(140, 200, 770, 30, "");
    bypass_input.set_value(&cfg.bypass_list);

    let _mode_label = Frame::new(20, 240, 120, 30, "代理模式");
    let mut mode_choice = Choice::new(140, 240, 160, 30, "");
    mode_choice.add_choice("手动|PAC");
    mode_choice.set_value(if cfg.proxy_mode == "pac" { 1 } else { 0 });

    let _pac_label = Frame::new(320, 240, 80, 30, "PAC 地址");
    let mut pac_input = Input::new(400, 240, 510, 30, "");
    pac_input.set_value(&cfg.pac_url);

    let _ie_path_label = Frame::new(20, 280, 120, 30, "导入导出路径");
    let mut ie_path_input = Input::new(140, 280, 560, 30, "");
    ie_path_input.set_value(&cfg.import_export_path);
    let mut pick_import_path_btn = Button::new(710, 280, 100, 30, "选导入文件");
    let mut pick_export_path_btn = Button::new(820, 280, 100, 30, "选导出文件");

    let mut enable_btn = Button::new(140, 340, 120, 34, "开启代理");
    let mut disable_btn = Button::new(270, 340, 120, 34, "关闭代理");
    let mut status_btn = Button::new(400, 340, 120, 34, "查看状态");
    let mut save_btn = Button::new(530, 340, 100, 34, "保存配置");
    let mut import_btn = Button::new(640, 340, 100, 34, "导入配置");
    let mut export_btn = Button::new(750, 340, 100, 34, "导出配置");

    let _status_title = Frame::new(20, 400, 900, 24, "当前状态");
    let mut status_display = TextDisplay::new(20, 426, 900, 130, "");
    status_display.set_text_font(Font::Courier);
    let mut status_buf = TextBuffer::default();
    status_display.set_buffer(status_buf.clone());
    status_buf.set_text("未查询");

    let _log_title = Frame::new(20, 570, 900, 24, "操作日志");
    let mut log_display = TextDisplay::new(20, 596, 900, 140, "");
    log_display.set_text_font(Font::Courier);
    let log_buf = TextBuffer::default();
    log_display.set_buffer(log_buf.clone());

    win.end();
    win.make_resizable(true);
    win.show();
    win.set_callback(|_| app::quit());

    let state = Rc::new(RefCell::new(AppState::default()));

    {
        let state = state.clone();
        let mut status_buf = status_buf.clone();
        let mut log_buf = log_buf.clone();
        let http_input = http_input.clone();
        let https_input = https_input.clone();
        let socks_input = socks_input.clone();
        let bypass_input = bypass_input.clone();
        let mode_choice = mode_choice.clone();
        let pac_input = pac_input.clone();
        enable_btn.set_callback(move |_| {
            let mut s = state.borrow_mut();
            s.log("开始启用系统代理...");
            let mode = if mode_choice.value() == 1 {
                "pac"
            } else {
                "manual"
            };
            let result = if mode == "pac" {
                platform_enable_pac(&pac_input.value())
            } else {
                platform_enable_proxy(
                    &http_input.value(),
                    &https_input.value(),
                    &socks_input.value(),
                    &bypass_input.value(),
                )
            };
            match result {
                Ok(msg) => {
                    s.log(&format!("启用成功: {msg}"));
                    if let Ok(status) = platform_query_status() {
                        status_buf.set_text(&status);
                        s.log(&format!(
                            "当前模式: {}，系统代理{}",
                            if mode == "pac" { "PAC" } else { "手动" },
                            proxy_state_text(&status)
                        ));
                    }
                }
                Err(err) => s.log(&format!("启用失败: {err}")),
            }
            log_buf.set_text(&s.logs_text());
        });
    }

    {
        let state = state.clone();
        let mut status_buf = status_buf.clone();
        let mut log_buf = log_buf.clone();
        disable_btn.set_callback(move |_| {
            let mut s = state.borrow_mut();
            s.log("开始关闭系统代理...");
            match platform_disable_proxy() {
                Ok(msg) => {
                    s.log(&format!("关闭成功: {msg}"));
                    if let Ok(status) = platform_query_status() {
                        status_buf.set_text(&status);
                    }
                }
                Err(err) => s.log(&format!("关闭失败: {err}")),
            }
            log_buf.set_text(&s.logs_text());
        });
    }

    {
        let state = state.clone();
        let mut status_buf = status_buf.clone();
        let mut log_buf = log_buf.clone();
        status_btn.set_callback(move |_| {
            let mut s = state.borrow_mut();
            match platform_query_status() {
                Ok(status) => {
                    status_buf.set_text(&status);
                    s.log(&format!("状态刷新成功：系统代理{}", proxy_state_text(&status)));
                }
                Err(err) => {
                    status_buf.set_text(&format!("状态查询失败: {err}"));
                    s.log(&format!("状态查询失败: {err}"));
                }
            }
            log_buf.set_text(&s.logs_text());
        });
    }

    {
        let state = state.clone();
        let mut log_buf = log_buf.clone();
        let http_input = http_input.clone();
        let https_input = https_input.clone();
        let socks_input = socks_input.clone();
        let bypass_input = bypass_input.clone();
        let mode_choice = mode_choice.clone();
        let pac_input = pac_input.clone();
        let ie_path_input = ie_path_input.clone();
        save_btn.set_callback(move |_| {
            let mut s = state.borrow_mut();
            let cfg = AppConfig {
                http_proxy: http_input.value(),
                https_proxy: https_input.value(),
                socks_proxy: socks_input.value(),
                bypass_list: bypass_input.value(),
                proxy_mode: if mode_choice.value() == 1 {
                    "pac".to_string()
                } else {
                    "manual".to_string()
                },
                pac_url: pac_input.value(),
                import_export_path: ie_path_input.value(),
            };
            match save_config(&cfg) {
                Ok(msg) => s.log(&msg),
                Err(err) => s.log(&format!("保存配置失败: {err}")),
            }
            log_buf.set_text(&s.logs_text());
        });
    }

    {
        let state = state.clone();
        let mut log_buf = log_buf.clone();
        let mut http_input = http_input.clone();
        let mut https_input = https_input.clone();
        let mut socks_input = socks_input.clone();
        let mut bypass_input = bypass_input.clone();
        let mut mode_choice = mode_choice.clone();
        let mut pac_input = pac_input.clone();
        let mut ie_path_input = ie_path_input.clone();
        import_btn.set_callback(move |_| {
            let mut s = state.borrow_mut();
            let path = ie_path_input.value();
            match import_config(&path) {
                Ok(cfg) => {
                    http_input.set_value(&cfg.http_proxy);
                    https_input.set_value(&cfg.https_proxy);
                    socks_input.set_value(&cfg.socks_proxy);
                    bypass_input.set_value(&cfg.bypass_list);
                    mode_choice.set_value(if cfg.proxy_mode == "pac" { 1 } else { 0 });
                    pac_input.set_value(&cfg.pac_url);
                    ie_path_input.set_value(&cfg.import_export_path);
                    s.log(&format!("配置已导入: {}", path));
                }
                Err(err) => s.log(&format!("导入配置失败: {err}")),
            }
            log_buf.set_text(&s.logs_text());
        });
    }

    {
        let state = state.clone();
        let mut log_buf = log_buf.clone();
        let mut ie_path_input = ie_path_input.clone();
        pick_import_path_btn.set_callback(move |_| {
            let mut s = state.borrow_mut();
            match choose_path(FileDialogType::BrowseFile, &ie_path_input.value()) {
                Ok(path) => {
                    ie_path_input.set_value(&path);
                    s.log(&format!("已选择导入文件: {path}"));
                }
                Err(err) => s.log(&format!("选择导入文件: {err}")),
            }
            log_buf.set_text(&s.logs_text());
        });
    }

    {
        let state = state.clone();
        let mut log_buf = log_buf.clone();
        let mut ie_path_input = ie_path_input.clone();
        pick_export_path_btn.set_callback(move |_| {
            let mut s = state.borrow_mut();
            match choose_path(FileDialogType::BrowseSaveFile, &ie_path_input.value()) {
                Ok(path) => {
                    ie_path_input.set_value(&path);
                    s.log(&format!("已选择导出文件: {path}"));
                }
                Err(err) => s.log(&format!("选择导出文件: {err}")),
            }
            log_buf.set_text(&s.logs_text());
        });
    }

    {
        let state = state.clone();
        let mut log_buf = log_buf.clone();
        let http_input = http_input.clone();
        let https_input = https_input.clone();
        let socks_input = socks_input.clone();
        let bypass_input = bypass_input.clone();
        let mode_choice = mode_choice.clone();
        let pac_input = pac_input.clone();
        let ie_path_input = ie_path_input.clone();
        export_btn.set_callback(move |_| {
            let mut s = state.borrow_mut();
            let cfg = AppConfig {
                http_proxy: http_input.value(),
                https_proxy: https_input.value(),
                socks_proxy: socks_input.value(),
                bypass_list: bypass_input.value(),
                proxy_mode: if mode_choice.value() == 1 {
                    "pac".to_string()
                } else {
                    "manual".to_string()
                },
                pac_url: pac_input.value(),
                import_export_path: ie_path_input.value(),
            };
            match export_config(&cfg.import_export_path, &cfg) {
                Ok(msg) => s.log(&msg),
                Err(err) => s.log(&format!("导出配置失败: {err}")),
            }
            log_buf.set_text(&s.logs_text());
        });
    }

    {
        let state = state.clone();
        let mut status_buf = status_buf.clone();
        let mut log_buf = log_buf.clone();
        let mut s = state.borrow_mut();
        if let Ok(path) = config_path() {
            s.log(&format!("配置文件路径: {}", path.display()));
        }
        match platform_query_status() {
            Ok(status) => {
                status_buf.set_text(&status);
                s.log("启动时状态查询成功");
            }
            Err(err) => s.log(&format!("启动时状态查询失败: {err}")),
        }
        log_buf.set_text(&s.logs_text());
    }

    while app.wait() {}
}
