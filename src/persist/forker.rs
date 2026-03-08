use std::io;

/// Fork the current process and run the daemon in the child.
///
/// Returns:
/// - `Ok(ForkResult::Parent(child_pid))` in the parent process
/// - `Ok(ForkResult::Child)` in the child (daemon) process
/// - `Err(_)` if fork fails
///
/// The child process:
/// 1. Calls setsid() to become a session leader
/// 2. Redirects stdin/stdout/stderr to /dev/null
/// 3. Does NOT call any thread-spawning code before this point
pub fn fork_daemon() -> io::Result<ForkResult> {
    // Safety: fork() must be called before any threads are spawned.
    // The caller (main.rs) ensures this by forking before setup_terminal(),
    // initialize_mouse_input(), or any TerminalEmulator creation.
    let pid = unsafe { libc::fork() };

    match pid {
        -1 => Err(io::Error::last_os_error()),
        0 => {
            // Child process - become the daemon

            // Create a new session (detach from controlling terminal)
            if unsafe { libc::setsid() } == -1 {
                eprintln!("setsid failed: {}", io::Error::last_os_error());
                std::process::exit(1);
            }

            // Redirect stdin/stdout/stderr to /dev/null
            redirect_stdio_to_devnull();

            Ok(ForkResult::Child)
        }
        child_pid => {
            // Parent process - continue as client
            Ok(ForkResult::Parent(child_pid))
        }
    }
}

/// Result of a fork operation
pub enum ForkResult {
    /// We are in the parent process. Contains the child PID.
    Parent(i32),
    /// We are in the child (daemon) process.
    Child,
}

/// Redirect stdin, stdout, stderr to /dev/null
fn redirect_stdio_to_devnull() {
    unsafe {
        let devnull = libc::open(c"/dev/null".as_ptr(), libc::O_RDWR);
        if devnull >= 0 {
            let _ = libc::dup2(devnull, libc::STDIN_FILENO);
            let _ = libc::dup2(devnull, libc::STDOUT_FILENO);
            let _ = libc::dup2(devnull, libc::STDERR_FILENO);
            if devnull > libc::STDERR_FILENO {
                let _ = libc::close(devnull);
            }
        }
    }
}
