#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::PathBuf;

/// Get the directory for persist mode socket and lock files
pub fn persist_dir() -> io::Result<PathBuf> {
    // 1. $XDG_RUNTIME_DIR/term39/ (Linux with systemd)
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let dir = PathBuf::from(xdg).join("term39");
        return ensure_dir(dir);
    }

    // 2. $TMPDIR/term39-$UID/ (macOS)
    let uid = unsafe { libc::getuid() };
    if let Ok(tmpdir) = std::env::var("TMPDIR") {
        let dir = PathBuf::from(tmpdir).join(format!("term39-{}", uid));
        return ensure_dir(dir);
    }

    // 3. /tmp/term39-$UID/ (fallback)
    let dir = PathBuf::from(format!("/tmp/term39-{}", uid));
    ensure_dir(dir)
}

/// Get the socket file path
pub fn socket_path() -> io::Result<PathBuf> {
    Ok(persist_dir()?.join("term39.sock"))
}

/// Get the lock file path
pub fn lock_path() -> io::Result<PathBuf> {
    Ok(persist_dir()?.join("term39.lock"))
}

/// Ensure the directory exists with restricted permissions
fn ensure_dir(dir: PathBuf) -> io::Result<PathBuf> {
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
        // Set directory permissions to owner-only (0700)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
        }
    }
    Ok(dir)
}

/// Check if a daemon is alive by checking the lock file and sending signal 0
pub fn is_daemon_alive() -> bool {
    let lock = match lock_path() {
        Ok(p) => p,
        Err(_) => return false,
    };

    if !lock.exists() {
        return false;
    }

    match fs::read_to_string(&lock) {
        Ok(pid_str) => {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                // signal 0 checks if process exists without sending a signal
                unsafe { libc::kill(pid, 0) == 0 }
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Read daemon PID from lock file
pub fn read_daemon_pid() -> Option<i32> {
    let lock = lock_path().ok()?;
    let pid_str = fs::read_to_string(lock).ok()?;
    pid_str.trim().parse::<i32>().ok()
}

/// Write daemon PID to lock file
pub fn write_lock_file(pid: i32) -> io::Result<()> {
    let lock = lock_path()?;
    fs::write(lock, pid.to_string())
}

/// Remove socket and lock files
pub fn cleanup_files() {
    if let Ok(sock) = socket_path() {
        let _ = fs::remove_file(sock);
    }
    if let Ok(lock) = lock_path() {
        let _ = fs::remove_file(lock);
    }
}

/// Check if socket exists (daemon might be running)
pub fn socket_exists() -> bool {
    socket_path().map(|p| p.exists()).unwrap_or(false)
}
