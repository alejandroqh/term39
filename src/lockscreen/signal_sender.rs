//! Signal sender for triggering lockscreen on other term39 instances.
//!
//! This module provides functionality to send SIGUSR1 to all running term39
//! processes, triggering their lockscreen feature.

use std::io;
use std::process::Command;

/// Send SIGUSR1 to all other running term39 instances to trigger lockscreen.
///
/// This function is used when term39 is invoked with the `--lock` flag.
/// It finds all running term39 processes and sends them the SIGUSR1 signal,
/// which triggers the lockscreen feature.
///
/// # Returns
/// - `Ok(())` if at least one process was signaled
/// - `Err(_)` if no running term39 instance was found
pub fn send_lock_signal() -> io::Result<()> {
    let current_pid = std::process::id();

    // Use pgrep to find term39 processes
    let output = Command::new("pgrep").arg("-x").arg("term39").output();

    match output {
        Ok(result) => {
            let pids_str = String::from_utf8_lossy(&result.stdout);
            let mut found = false;

            for line in pids_str.lines() {
                if let Ok(pid) = line.trim().parse::<u32>() {
                    // Don't signal ourselves
                    if pid != current_pid {
                        // Send SIGUSR1 to the process
                        unsafe {
                            if libc::kill(pid as i32, libc::SIGUSR1) == 0 {
                                println!("Sent lock signal to term39 (PID: {})", pid);
                                found = true;
                            }
                        }
                    }
                }
            }

            if !found {
                eprintln!("No running term39 instance found to lock.");
                std::process::exit(1);
            }
        }
        Err(_) => {
            // pgrep not available, try reading /proc directly
            if let Ok(entries) = std::fs::read_dir("/proc") {
                let mut found = false;
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name() {
                        if let Ok(pid) = name.to_string_lossy().parse::<u32>() {
                            if pid != current_pid {
                                let comm_path = path.join("comm");
                                if let Ok(comm) = std::fs::read_to_string(&comm_path) {
                                    if comm.trim() == "term39" {
                                        unsafe {
                                            if libc::kill(pid as i32, libc::SIGUSR1) == 0 {
                                                println!(
                                                    "Sent lock signal to term39 (PID: {})",
                                                    pid
                                                );
                                                found = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !found {
                    eprintln!("No running term39 instance found to lock.");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Could not find running term39 instances.");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
