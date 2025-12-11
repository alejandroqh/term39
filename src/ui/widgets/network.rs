//! Network widget for the top bar
//!
//! Shows network interface status with signal strength for WiFi
//! or connection indicator for Ethernet.

use super::{Widget, WidgetAlignment, WidgetClickResult, WidgetContext};
use crate::rendering::{Cell, Theme, VideoBuffer};
use crate::window::manager::FocusState;
use crossterm::style::Color;
use std::cell::RefCell;
use std::time::{Duration, Instant};

/// Network information including interface name, connection state, and signal
#[derive(Clone)]
pub struct NetworkInfo {
    pub interface: String,
    pub is_connected: bool,
    pub is_wifi: bool,
    pub signal_strength: Option<u8>, // 0-100 for WiFi, None for Ethernet
}

/// Cached network info with last update time
struct NetworkCache {
    info: Option<NetworkInfo>,
    interface: String,
    last_update: Instant,
}

thread_local! {
    static NETWORK_CACHE: RefCell<NetworkCache> = RefCell::new(NetworkCache {
        info: None,
        interface: String::new(),
        last_update: Instant::now() - Duration::from_secs(2), // Force initial fetch
    });
}

/// Get the current network info (cached for 1 second)
pub fn get_network_info(interface: &str) -> Option<NetworkInfo> {
    NETWORK_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();

        // Check if interface changed (avoid allocation if same)
        let interface_changed = cache.interface != interface;

        // Refresh if interface changed or more than 1 second has passed
        if interface_changed || cache.last_update.elapsed() >= Duration::from_secs(1) {
            cache.info = fetch_network_info(interface);
            // Only allocate new string when interface actually changed
            if interface_changed {
                cache.interface = interface.to_string();
            }
            cache.last_update = Instant::now();
        }

        cache.info.clone()
    })
}

/// Actually fetch network info from the system
fn fetch_network_info(interface: &str) -> Option<NetworkInfo> {
    if interface.is_empty() {
        return None;
    }

    let is_wifi = is_wifi_interface(interface);
    let is_connected = check_interface_connected(interface);
    let signal_strength = if is_wifi && is_connected {
        get_wifi_signal_strength(interface)
    } else {
        None
    };

    Some(NetworkInfo {
        interface: interface.to_string(),
        is_connected,
        is_wifi,
        signal_strength,
    })
}

/// Check if interface is a WiFi interface based on common naming conventions
fn is_wifi_interface(interface: &str) -> bool {
    // Linux WiFi interfaces
    interface.starts_with("wlan")
        || interface.starts_with("wlp")
        || interface.starts_with("wifi")
        || interface.starts_with("ath")
        || interface.starts_with("ra")
        // macOS: en0 is typically WiFi on laptops, but we need to check
        || (cfg!(target_os = "macos") && is_macos_wifi_interface(interface))
}

/// Check if a macOS interface is WiFi by looking at ifconfig output
#[cfg(target_os = "macos")]
fn is_macos_wifi_interface(interface: &str) -> bool {
    // On macOS, check if the interface type indicates WiFi
    if let Ok(output) = std::process::Command::new("networksetup")
        .args(["-listallhardwareports"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse output looking for Wi-Fi followed by our interface
        let mut is_wifi_section = false;
        for line in stdout.lines() {
            if line.contains("Wi-Fi") || line.contains("AirPort") {
                is_wifi_section = true;
            } else if line.starts_with("Hardware Port:") {
                is_wifi_section = false;
            } else if is_wifi_section && line.contains("Device:") && line.contains(interface) {
                return true;
            }
        }
    }
    false
}

#[cfg(not(target_os = "macos"))]
fn is_macos_wifi_interface(_interface: &str) -> bool {
    false
}

/// Check if interface is connected (up and running) - Linux
#[cfg(target_os = "linux")]
fn check_interface_connected(interface: &str) -> bool {
    let operstate_path = format!("/sys/class/net/{}/operstate", interface);
    if let Ok(state) = std::fs::read_to_string(&operstate_path) {
        state.trim() == "up"
    } else {
        false
    }
}

/// Check if interface is connected - macOS
#[cfg(target_os = "macos")]
fn check_interface_connected(interface: &str) -> bool {
    // Use ifconfig to check interface status on macOS
    if let Ok(output) = std::process::Command::new("ifconfig")
        .arg(interface)
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Check if interface is up and has an IP address
        stdout.contains("status: active") || (stdout.contains("UP") && stdout.contains("inet "))
    } else {
        false
    }
}

/// Check if interface is connected - other platforms (basic existence check)
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn check_interface_connected(_interface: &str) -> bool {
    // On other platforms, we can't easily check interface state
    // Return false to indicate unknown/disconnected
    false
}

/// Get WiFi signal strength from /proc/net/wireless (Linux only)
#[cfg(target_os = "linux")]
fn get_wifi_signal_strength(interface: &str) -> Option<u8> {
    let contents = std::fs::read_to_string("/proc/net/wireless").ok()?;

    for line in contents.lines().skip(2) {
        // Skip header lines
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        // Interface name ends with ':'
        let iface = parts[0].trim_end_matches(':');
        if iface == interface {
            // Column 3 is link quality (0-70 typically)
            if let Some(quality_str) = parts.get(2) {
                if let Ok(quality) = quality_str.trim_end_matches('.').parse::<f32>() {
                    // Convert to percentage (70 = 100%)
                    let percentage = ((quality / 70.0) * 100.0).min(100.0) as u8;
                    return Some(percentage);
                }
            }
        }
    }
    None
}

/// Get WiFi signal strength on macOS using airport utility
#[cfg(target_os = "macos")]
fn get_wifi_signal_strength(_interface: &str) -> Option<u8> {
    // Use the airport command-line tool to get signal strength
    let airport_path =
        "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport";
    if let Ok(output) = std::process::Command::new(airport_path).arg("-I").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("agrCtlRSSI:") {
                // RSSI is typically -30 (excellent) to -90 (poor)
                if let Some(rssi_str) = line.split(':').nth(1) {
                    if let Ok(rssi) = rssi_str.trim().parse::<i32>() {
                        // Convert RSSI to percentage: -30 = 100%, -90 = 0%
                        let percentage = ((rssi + 90) * 100 / 60).clamp(0, 100) as u8;
                        return Some(percentage);
                    }
                }
            }
        }
    }
    None
}

/// Get WiFi signal strength - other platforms (not supported)
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_wifi_signal_strength(_interface: &str) -> Option<u8> {
    None
}

/// Get signal bars string based on strength percentage
fn get_signal_bars(strength: u8) -> &'static str {
    if strength >= 80 {
        "\u{2582}\u{2584}\u{2586}\u{2588}" // ▂▄▆█ - full signal
    } else if strength >= 60 {
        "\u{2582}\u{2584}\u{2586} " // ▂▄▆  - good signal
    } else if strength >= 40 {
        "\u{2582}\u{2584}  " // ▂▄   - fair signal
    } else if strength >= 20 {
        "\u{2582}   " // ▂    - weak signal
    } else {
        "    " // no bars - very weak
    }
}

/// Get color based on signal strength
fn get_signal_color(strength: u8) -> Color {
    if strength >= 60 {
        Color::Green
    } else if strength >= 40 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// Widget displaying network status
pub struct NetworkWidget {
    hovered: bool,
    cached_info: Option<NetworkInfo>,
    interface: String,
    enabled: bool,
}

impl NetworkWidget {
    pub fn new() -> Self {
        Self {
            hovered: false,
            cached_info: None,
            interface: String::new(),
            enabled: false,
        }
    }

    /// Configure the widget with interface name and enabled state
    pub fn configure(&mut self, interface: &str, enabled: bool) {
        self.interface = interface.to_string();
        self.enabled = enabled;
    }

    /// Returns whether the network widget is currently hovered
    #[allow(dead_code)]
    pub fn is_hovered(&self) -> bool {
        self.hovered
    }
}

impl Default for NetworkWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for NetworkWidget {
    fn width(&self) -> u16 {
        if !self.enabled || self.interface.is_empty() {
            return 0;
        }

        if let Some(ref info) = self.cached_info {
            // Format: " wlan0 ▂▄▆█  " or " eth0 ▣  " or " wlan0 ✗  "
            let status_len = if info.is_wifi && info.is_connected && info.signal_strength.is_some()
            {
                4 // signal bars
            } else {
                1 // single char (▣ or ✗)
            };
            // " " + interface + " " + status + "  " (two trailing spaces for margin)
            (info.interface.len() + status_len + 4) as u16
        } else {
            0
        }
    }

    fn render(&self, buffer: &mut VideoBuffer, x: u16, theme: &Theme, focus: FocusState) {
        let info = match &self.cached_info {
            Some(info) => info,
            None => return,
        };

        let bg_color = match focus {
            FocusState::Desktop | FocusState::Topbar => theme.topbar_bg_focused,
            FocusState::Window(_) => theme.topbar_bg_unfocused,
        };
        let fg_color = theme.window_border_unfocused_fg;

        let mut current_x = x;

        // Leading space
        buffer.set(current_x, 0, Cell::new_unchecked(' ', fg_color, bg_color));
        current_x += 1;

        // Interface name
        for ch in info.interface.chars() {
            buffer.set(current_x, 0, Cell::new_unchecked(ch, fg_color, bg_color));
            current_x += 1;
        }

        // Space before status
        buffer.set(current_x, 0, Cell::new_unchecked(' ', fg_color, bg_color));
        current_x += 1;

        if info.is_connected {
            if info.is_wifi {
                // WiFi: show signal bars if available
                if let Some(strength) = info.signal_strength {
                    let bars = get_signal_bars(strength);
                    let color = get_signal_color(strength);
                    for ch in bars.chars() {
                        buffer.set(current_x, 0, Cell::new_unchecked(ch, color, bg_color));
                        current_x += 1;
                    }
                } else {
                    // Connected but no signal info - show connected icon
                    buffer.set(
                        current_x,
                        0,
                        Cell::new_unchecked('\u{25A3}', Color::Green, bg_color),
                    ); // ▣
                    current_x += 1;
                }
            } else {
                // Ethernet: show connected icon
                buffer.set(
                    current_x,
                    0,
                    Cell::new_unchecked('\u{25A3}', Color::Green, bg_color),
                ); // ▣
                current_x += 1;
            }
        } else {
            // Disconnected: show X
            buffer.set(
                current_x,
                0,
                Cell::new_unchecked('\u{2717}', Color::Red, bg_color),
            ); // ✗
            current_x += 1;
        }

        // Trailing spaces (margin)
        buffer.set(current_x, 0, Cell::new_unchecked(' ', fg_color, bg_color));
        current_x += 1;
        buffer.set(current_x, 0, Cell::new_unchecked(' ', fg_color, bg_color));
    }

    fn is_visible(&self, _ctx: &WidgetContext) -> bool {
        self.enabled && !self.interface.is_empty() && self.cached_info.is_some()
    }

    fn contains(&self, point_x: u16, point_y: u16, widget_x: u16) -> bool {
        point_y == 0 && point_x >= widget_x && point_x < widget_x + self.width()
    }

    fn update_hover(&mut self, mouse_x: u16, mouse_y: u16, widget_x: u16) {
        self.hovered = self.contains(mouse_x, mouse_y, widget_x);
    }

    fn handle_click(&mut self, _mouse_x: u16, _mouse_y: u16, _widget_x: u16) -> WidgetClickResult {
        // Network widget doesn't respond to clicks
        WidgetClickResult::NotHandled
    }

    fn reset_state(&mut self) {
        self.hovered = false;
    }

    fn update(&mut self, _ctx: &WidgetContext) {
        if self.enabled && !self.interface.is_empty() {
            self.cached_info = get_network_info(&self.interface);
        } else {
            self.cached_info = None;
        }
    }

    fn alignment(&self) -> WidgetAlignment {
        WidgetAlignment::Right
    }
}
