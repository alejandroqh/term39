#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub mod gpm_control;
pub mod keyboard_handlers;
pub mod keyboard_mode;
pub mod mouse;
pub mod mouse_handlers;
