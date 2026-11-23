#![allow(deprecated)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{DateTime, Local, Utc};
mod i18n;
use i18n::Lang;
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde_json::Value;
use std::env;
use std::ffi::c_void;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration as StdDuration;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    GetStockObject, UpdateWindow, COLOR_WINDOW, DEFAULT_GUI_FONT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_PROGRESS_CLASS, INITCOMMONCONTROLSEX, PBM_SETPOS,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, LoadCursorW,
    PostQuitMessage, RegisterClassW, SendMessageW, ShowWindow, TranslateMessage, BS_GROUPBOX,
    BS_PUSHBUTTON, CW_USEDEFAULT, IDC_ARROW, MSG, SW_SHOW, WM_COMMAND, WM_DESTROY, WM_SETFONT,
    WNDCLASSW, WS_CAPTION, WS_CHILD, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_VISIBLE,
};
use winit::event::{Event, StartCause};
use winit::event_loop::{EventLoop, EventLoopProxy};

const USAGE_URL: &str = "https://opencode.ai/inference/go/v1/usage";
const REFRESH_SECS: u64 = 60;
const WINDOW_CLASS: &str = "OpenCodeGoUsageWindow";

static WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug)]
struct Meter {
    percent: i32,
    status: String,
    resets_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug)]
struct UsageState {
    rolling: Option<Meter>,
    weekly: Option<Meter>,
    monthly: Option<Meter>,
    error: Option<String>,
    updated: Option<DateTime<Local>>,
}

impl Default for UsageState {
    fn default() -> Self {
        Self {
            rolling: None,
            weekly: None,
            monthly: None,
            error: Some("正在获取用量…".to_string()),
            updated: None,
        }
    }
}

enum UserEvent {
    Usage(UsageState),
    Tray(TrayIconEvent),
    Menu(MenuEvent),
}

fn credential_from_database() -> Result<Option<String>, String> {
    let home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_default();
    let db = Path::new(&home).join(".local/share/opencode/opencode.db");
    if !db.exists() {
        return Ok(None);
    }

    let uri = format!("file:{}?mode=ro", db.to_string_lossy().replace('\\', "/"));
    let connection = Connection::open_with_flags(
        uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|e| format!("无法打开本地凭据库：{}", e))?;

    let stored: Option<String> = connection
        .query_row(
            "SELECT value FROM credential
             WHERE integration_id = 'opencode-go' AND active = 1
             ORDER BY time_updated DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("读取本地凭据失败：{}", e))?;

    let Some(stored) = stored else {
        return Ok(None);
    };
    let value: Value =
        serde_json::from_str(&stored).map_err(|e| format!("本地凭据格式错误：{}", e))?;
    Ok(value
        .get("key")
        .and_then(Value::as_str)
        .map(|key| key.trim().to_string()))
}

fn resolve_token() -> Result<String, String> {
    for name in ["OPENCODE_API_KEY", "OPENCODE_GO_API_KEY"] {
        if let Ok(value) = env::var(name) {
            let value = value.trim();
            if !value.is_empty() {
                return Ok(value.to_string());
            }
        }
    }
    if let Some(key) = credential_from_database()? {
        if !key.is_empty() {
            return Ok(key);
        }
    }
    Err("未找到有效凭据：请在 OpenCode GUI 中登录 OpenCode Go，或设置 OPENCODE_API_KEY".to_string())
}

fn parse_meter(value: Option<&Value>) -> Option<Meter> {
    let value = value?;
    let percent = value
        .get("percent")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .clamp(0, 100) as i32;
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("ok")
        .to_string();
    let resets_at = value
        .get("resetsAt")
        .and_then(Value::as_str)
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc));
    Some(Meter {
        percent,
        status,
        resets_at,
    })
}

fn error_message(error: ureq::Error) -> String {
    match error {
        ureq::Error::StatusCode(401) => {
            "认证失败（401）：请确认 OpenCode GUI 中的 Go 订阅仍然有效".to_string()
        }
        ureq::Error::StatusCode(403) => {
            "订阅或权限错误（403）：该密钥未开通 OpenCode Go".to_string()
        }
        ureq::Error::StatusCode(code) => format!("请求失败（HTTP {}）", code),
        other => format!("网络错误：{}", other),
    }
}

fn fetch_usage(token: &str) -> Result<UsageState, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(StdDuration::from_secs(15)))
        .build()
        .into();
    let auth = format!("Bearer {}", token);
    let mut response = agent
        .get(USAGE_URL)
        .header("Authorization", &auth)
        .header("Accept", "application/json")
        .header("User-Agent", "opencode-go-tray/1.0")
        .call()
        .map_err(error_message)?;

    let body: Value = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("解析响应失败：{}", e))?;
    let usage = body
        .get("usage")
        .ok_or_else(|| "响应缺少 usage 字段".to_string())?;

    Ok(UsageState {
        rolling: parse_meter(usage.get("rolling")),
        weekly: parse_meter(usage.get("weekly")),
        monthly: parse_meter(usage.get("monthly")),
        error: None,
        updated: Some(Local::now()),
    })
}

fn status_text(lang: Lang, meter: &Meter) -> &'static str {
    let s = lang.strings();
    if meter.status == "rate-limited" {
        s.status_limited
    } else {
        s.status_ok
    }
}

fn remain_text(lang: Lang, meter: &Meter) -> String {
    let s = lang.strings();
    match meter.resets_at {
        Some(reset) => {
            let minutes = (reset - Utc::now()).num_minutes();
            if minutes <= 0 {
                "0m".to_string()
            } else if minutes >= 60 {
                format!("{}h {}m {}", minutes / 60, minutes % 60, s.reset_after)
            } else {
                format!("{}m {}", minutes, s.reset_after)
            }
        }
        None => "--".to_string(),
    }
}

fn reset_local_text(meter: &Meter) -> String {
    meter
        .resets_at
        .map(|t| t.with_timezone(&Local).format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "未知".to_string())
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn tooltip(lang: Lang, state: &UsageState) -> String {
    let s = lang.strings();
    if let Some(err) = &state.error {
        return format!("OpenCode Go - {}", truncate(err, 58));
    }
    let rolling = state.rolling.as_ref().map(|m| m.percent).unwrap_or(0);
    let monthly = state.monthly.as_ref().map(|m| m.percent).unwrap_or(0);
    format!(
        "OpenCode Go  {} {}%  {} {}%",
        s.tooltip_5h, rolling, s.tooltip_month, monthly
    )
}

fn make_icon(percent: Option<i32>, error: bool) -> Icon {
    let size = 32usize;
    let mut rgba = vec![0u8; size * size * 4];
    let (red, green, blue) = if error {
        (110, 112, 120)
    } else {
        match percent.unwrap_or(0) {
            p if p >= 80 => (220, 70, 70),
            p if p >= 50 => (240, 165, 50),
            _ => (55, 170, 105),
        }
    };

    for y in 0..size {
        for x in 0..size {
            let dx = x as f64 - 15.5;
            let dy = y as f64 - 15.5;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance <= 14.0 {
                let (r, g, b, a) = if distance >= 11.5 {
                    (245, 245, 245, 255)
                } else {
                    (red, green, blue, 255)
                };
                let i = (y * size + x) * 4;
                rgba[i] = r;
                rgba[i + 1] = g;
                rgba[i + 2] = b;
                rgba[i + 3] = a;
            }
        }
    }
    Icon::from_rgba(rgba, size as u32, size as u32).expect("创建托盘图标失败")
}

fn update_menu(
    lang: Lang,
    info5: &MenuItem,
    infow: &MenuItem,
    infom: &MenuItem,
    state: &UsageState,
) {
    let s = lang.strings();
    let set = |item: &MenuItem, label: &str, meter: Option<&Meter>| match meter {
        Some(meter) => item.set_text(format!(
            "{}  {}%  {} {}%  {}  {}",
            label,
            meter.percent,
            s.remaining,
            100 - meter.percent,
            status_text(lang, meter),
            remain_text(lang, meter)
        )),
        None => item.set_text(format!("{}  {}", label, s.no_data)),
    };
    set(info5, s.menu_5h, state.rolling.as_ref());
    set(infow, s.menu_weekly, state.weekly.as_ref());
    set(infom, s.menu_month, state.monthly.as_ref());
    if let Some(err) = &state.error {
        info5.set_text(format!("{}：{}", s.error, truncate(err, 40)));
        infow.set_text(s.no_data);
        infom.set_text(s.reopen_hint);
    }
}

fn open_console() {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer")
            .arg("https://opencode.ai/console")
            .spawn();
    }
}

fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe fn add_control(
    parent: HWND,
    class_name: &str,
    text: &str,
    style: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: usize,
) -> HWND {
    let class = to_wide(class_name);
    let text = to_wide(text);
    CreateWindowExW(
        0,
        class.as_ptr(),
        text.as_ptr(),
        style,
        x,
        y,
        width,
        height,
        parent,
        id as *mut c_void,
        std::ptr::null_mut(),
        std::ptr::null(),
    )
}

unsafe fn add_label(
    parent: HWND,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    right_aligned: bool,
) -> HWND {
    let style = WS_CHILD | WS_VISIBLE | if right_aligned { 2 } else { 0 };
    add_control(parent, "STATIC", text, style, x, y, width, height, 0)
}

unsafe fn add_group(parent: HWND, title: &str, y: i32) -> HWND {
    add_control(
        parent,
        "BUTTON",
        title,
        WS_CHILD | WS_VISIBLE | BS_GROUPBOX as u32,
        20,
        y,
        520,
        106,
        0,
    )
}

unsafe fn add_progress(parent: HWND, percent: i32, x: i32, y: i32) -> HWND {
    let bar = add_control(
        parent,
        "msctls_progress32",
        "",
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        488,
        16,
        0,
    );
    if !bar.is_null() {
        SendMessageW(bar, PBM_SETPOS, percent as usize, 0);
    }
    bar
}

unsafe fn create_usage_window(state: UsageState, lang: Lang) -> HWND {
    let s = lang.strings();
    let instance = GetModuleHandleW(std::ptr::null());
    let class_name = to_wide(WINDOW_CLASS);
    let cursor = LoadCursorW(std::ptr::null_mut(), IDC_ARROW);
    let wc = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: std::ptr::null_mut(),
        hCursor: cursor,
        hbrBackground: (COLOR_WINDOW + 1) as usize as *mut c_void,
        lpszMenuName: std::ptr::null(),
        lpszClassName: class_name.as_ptr(),
    };
    RegisterClassW(&wc);

    let title = to_wide(s.window_title);
    let hwnd = CreateWindowExW(
        0,
        class_name.as_ptr(),
        title.as_ptr(),
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        580,
        560,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        instance,
        std::ptr::null(),
    );
    if hwnd.is_null() {
        return hwnd;
    }

    let font = GetStockObject(DEFAULT_GUI_FONT);
    let mut controls = Vec::new();
    controls.push(add_label(hwnd, s.window_title, 24, 18, 500, 30, false));
    controls.push(add_label(hwnd, s.subtitle, 24, 48, 500, 20, false));

    let mut add_meter = |label: &str, meter: Option<&Meter>, y: i32| {
        let (group_title, value, detail, percent) = match meter {
            Some(meter) => (
                format!("{}  ·  {}%", label, meter.percent),
                format!(
                    "{} {}%      {} {}%      {}",
                    s.used,
                    meter.percent,
                    s.remaining,
                    100 - meter.percent,
                    status_text(lang, meter)
                ),
                format!(
                    "{} · {}",
                    remain_text(lang, meter),
                    reset_local_text(meter)
                ),
                meter.percent,
            ),
            None => (label.to_string(), s.no_data.to_string(), String::new(), 0),
        };
        controls.push(add_group(hwnd, &group_title, y));
        controls.push(add_label(hwnd, &value, 38, y + 24, 300, 22, false));
        controls.push(add_label(
            hwnd,
            &format!("{}%", percent),
            380,
            y + 24,
            140,
            22,
            true,
        ));
        controls.push(add_progress(hwnd, percent, 38, y + 54));
        controls.push(add_label(hwnd, &detail, 38, y + 76, 488, 20, false));
    };

    add_meter(s.group_5h, state.rolling.as_ref(), 78);
    add_meter(s.group_weekly, state.weekly.as_ref(), 192);
    add_meter(s.group_month, state.monthly.as_ref(), 306);

    let updated = state
        .updated
        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "--".to_string());
    let footer = match &state.error {
        Some(error) => format!("{}：{}", s.error, truncate(error, 72)),
        None => format!("{} {}   ·   {}", s.last_updated, updated, s.reopen_hint),
    };
    controls.push(add_label(hwnd, &footer, 24, 424, 520, 20, false));

    controls.push(add_control(
        hwnd,
        "BUTTON",
        s.open_console,
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
        330,
        452,
        110,
        30,
        1,
    ));
    controls.push(add_control(
        hwnd,
        "BUTTON",
        s.close,
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
        450,
        452,
        80,
        30,
        2,
    ));

    for control in controls {
        if !control.is_null() {
            SendMessageW(control, WM_SETFONT, font as usize, 1);
        }
    }

    ShowWindow(hwnd, SW_SHOW);
    UpdateWindow(hwnd);
    hwnd
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_COMMAND => {
            let id = (wparam & 0xffff) as i32;
            if id == 1 {
                open_console();
            } else if id == 2 {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

fn show_usage_window(state: UsageState, lang: Lang) {
    unsafe {
        let icc = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_PROGRESS_CLASS,
        };
        InitCommonControlsEx(&icc);

        let hwnd = create_usage_window(state, lang);
        if hwnd.is_null() {
            return;
        }
        let mut message: MSG = std::mem::zeroed();
        while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

fn open_usage_window(state: &UsageState, lang: Lang) {
    if WINDOW_OPEN.swap(true, Ordering::SeqCst) {
        return;
    }
    let state = state.clone();
    thread::spawn(move || {
        show_usage_window(state, lang);
        WINDOW_OPEN.store(false, Ordering::SeqCst);
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lang = Lang::detect();
    let s = lang.strings();

    if env::args().any(|arg| arg == "--preview") {
        let state = resolve_token()
            .and_then(|token| fetch_usage(&token))
            .unwrap_or_else(|error| UsageState {
                error: Some(error),
                updated: Some(Local::now()),
                ..UsageState::default()
            });
        show_usage_window(state, lang);
        return Ok(());
    }

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let usage_proxy: EventLoopProxy<UserEvent> = event_loop.create_proxy();
    let tray_proxy = event_loop.create_proxy();
    let menu_proxy = event_loop.create_proxy();
    let (refresh_tx, refresh_rx): (Sender<()>, Receiver<()>) = mpsc::channel();

    thread::spawn(move || {
        let token = match resolve_token() {
            Ok(token) => token,
            Err(error) => {
                let mut state = UsageState::default();
                state.error = Some(error);
                let _ = usage_proxy.send_event(UserEvent::Usage(state));
                return;
            }
        };
        loop {
            let state = match fetch_usage(&token) {
                Ok(state) => state,
                Err(error) => UsageState {
                    error: Some(error),
                    updated: Some(Local::now()),
                    ..UsageState::default()
                },
            };
            if usage_proxy.send_event(UserEvent::Usage(state)).is_err() {
                break;
            }
            let _ = refresh_rx.recv_timeout(StdDuration::from_secs(REFRESH_SECS));
        }
    });

    let tray_proxy_handler = tray_proxy;
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = tray_proxy_handler.send_event(UserEvent::Tray(event));
    }));
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_proxy.send_event(UserEvent::Menu(event));
    }));

    let info5 = MenuItem::with_id("info5", s.menu_5h, false, None);
    let infow = MenuItem::with_id("infow", s.menu_weekly, false, None);
    let infom = MenuItem::with_id("infom", s.menu_month, false, None);
    let refresh_item = MenuItem::with_id("refresh", s.menu_refresh, true, None);
    let console_item = MenuItem::with_id("console", s.menu_console, true, None);
    let quit_item = MenuItem::with_id("quit", s.menu_quit, true, None);
    let refresh_id = refresh_item.id().clone();
    let console_id = console_item.id().clone();
    let quit_id = quit_item.id().clone();

    let i5 = info5.clone();
    let iw = infow.clone();
    let im = infom.clone();
    let ri = refresh_item.clone();
    let ci = console_item.clone();
    let qi = quit_item.clone();

    let mut tray: Option<TrayIcon> = None;
    let mut current = UsageState::default();

    event_loop.run(move |event, event_loop| match event {
        Event::NewEvents(StartCause::Init) => {
            let menu = Menu::new();
            let _ = menu.append(&i5);
            let _ = menu.append(&iw);
            let _ = menu.append(&im);
            let _ = menu.append(&PredefinedMenuItem::separator());
            let _ = menu.append(&ri);
            let _ = menu.append(&ci);
            let _ = menu.append(&qi);

            let icon = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip(tooltip(lang, &current))
                .with_icon(make_icon(None, false))
                .build()
                .expect("创建托盘图标失败");
            icon.set_show_menu_on_left_click(false);
            icon.set_show_menu_on_right_click(true);
            tray = Some(icon);
        }
        Event::UserEvent(UserEvent::Usage(state)) => {
            current = state;
            if let Some(icon) = &tray {
                let _ = icon.set_tooltip(Some(tooltip(lang, &current)));
                let percent = current.rolling.as_ref().map(|m| m.percent);
                let _ = icon.set_icon(Some(make_icon(percent, current.error.is_some())));
            }
            update_menu(lang, &i5, &iw, &im, &current);
        }
        Event::UserEvent(UserEvent::Tray(TrayIconEvent::DoubleClick { .. })) => {
            update_menu(lang, &i5, &iw, &im, &current);
            open_usage_window(&current, lang);
        }
        Event::UserEvent(UserEvent::Tray(_)) => {}
        Event::UserEvent(UserEvent::Menu(event)) => {
            if event.id == refresh_id {
                let _ = refresh_tx.send(());
                let mut state = UsageState {
                    error: Some(format!("{}…", s.menu_refresh)),
                    ..UsageState::default()
                };
                state.updated = Some(Local::now());
                current = state;
                if let Some(icon) = &tray {
                    let _ = icon.set_tooltip(Some(tooltip(lang, &current)));
                }
                update_menu(lang, &i5, &iw, &im, &current);
            } else if event.id == console_id {
                open_console();
            } else if event.id == quit_id {
                event_loop.exit();
            }
        }
        _ => {}
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_meter() {
        let meter = parse_meter(Some(&json!({
            "status": "ok",
            "percent": 37,
            "resetsAt": "2026-11-23T12:00:00.000Z"
        })))
        .unwrap();
        assert_eq!(meter.percent, 37);
        assert_eq!(status_text(Lang::Zh, &meter), "正常");
        assert!(meter.resets_at.is_some());
    }

    #[test]
    fn tooltip_shows_both_meters() {
        let state = UsageState {
            rolling: Some(parse_meter(Some(&json!({"percent": 30, "status": "ok"}))).unwrap()),
            weekly: None,
            monthly: Some(
                parse_meter(Some(&json!({"percent": 50, "status": "rate-limited"}))).unwrap(),
            ),
            error: None,
            updated: None,
        };
        assert_eq!(
            tooltip(Lang::Zh, &state),
            "OpenCode Go  5h 30%  月 50%"
        );
    }

    #[test]
    fn error_tooltip_is_truncated() {
        let state = UsageState {
            error: Some("x".repeat(120)),
            ..UsageState::default()
        };
        assert!(tooltip(Lang::En, &state).chars().count() <= 80);
    }

    #[test]
    fn icon_generation_works() {
        let _ = make_icon(Some(10), false);
        let _ = make_icon(Some(90), false);
        let _ = make_icon(None, true);
    }
}
