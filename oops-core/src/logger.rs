//! Shell logger: captures terminal output to a log file via a PTY.
//!
//! "Instant mode" support — instead of re-running a failed command to capture
//! its error output, a background PTY process continuously logs terminal
//! output so the error text is already available when needed.
//!
//! **Unix only.** On Windows this feature prints an error and exits.

use std::path::Path;

/// Size of the ring-buffer log file (2 MiB).
#[cfg_attr(not(unix), allow(dead_code))]
const LOG_SIZE: usize = 2 * 1024 * 1024;
/// How much to clear when the ring buffer wraps (512 KiB).
#[cfg_attr(not(unix), allow(dead_code))]
const CLEAN_SIZE: usize = 512 * 1024;
/// I/O buffer size.
#[cfg_attr(not(unix), allow(dead_code))]
const BUF_SIZE: usize = 4096;

// ===========================================================================
// Unix implementation
// ===========================================================================
#[cfg(unix)]
mod imp {
    use super::*;
    use std::ffi::CString;
    use std::fs::File;
    use std::io::{self, Seek, Write};
    use std::os::unix::io::{AsRawFd, RawFd};
    use std::sync::atomic::{AtomicI32, Ordering};

    /// PTY master fd, stored globally so the SIGWINCH handler can access it.
    static MASTER_FD: AtomicI32 = AtomicI32::new(-1);

    // ---------------------------------------------------------------
    // SIGWINCH handler
    // ---------------------------------------------------------------

    extern "C" fn sigwinch_handler(_: libc::c_int) {
        let fd = MASTER_FD.load(Ordering::Relaxed);
        if fd < 0 {
            return;
        }
        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        if unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) } == 0 {
            unsafe {
                libc::ioctl(fd, libc::TIOCSWINSZ, &mut ws);
            }
        }
    }

    fn install_sigwinch(master_fd: RawFd) {
        MASTER_FD.store(master_fd, Ordering::Relaxed);
        unsafe {
            libc::signal(libc::SIGWINCH, sigwinch_handler as libc::sighandler_t);
        }
    }

    // ---------------------------------------------------------------
    // Entry point
    // ---------------------------------------------------------------

    pub fn run(log_path: &Path) -> ! {
        let shell =
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        // -- open ring-buffer log file --
        let mut log_file = File::create(log_path).unwrap_or_else(|e| {
            eprintln!("Oops: cannot create log file '{}': {}", log_path.display(), e);
            std::process::exit(1);
        });
        log_file.set_len(LOG_SIZE as u64).ok();
        log_file.seek(io::SeekFrom::Start(0)).ok();
        let _ = log_file.write_all(&vec![0u8; LOG_SIZE]);
        log_file.seek(io::SeekFrom::Start(0)).ok();

        // -- fork PTY --
        let winsize = get_winsize();
        let pty_master = {
            let mut master: libc::c_int = 0;
            let pid = unsafe {
                libc::forkpty(
                    &mut master,
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    &winsize,
                )
            };
            if pid == -1 {
                eprintln!("Oops: forkpty failed");
                std::process::exit(1);
            }
            if pid == 0 {
                exec_shell(&shell);
            }
            master
        };

        let stdin_fd = io::stdin().as_raw_fd() as libc::c_int;

        // -- raw mode --
        let saved_termios = set_raw_termios(stdin_fd);

        // -- SIGWINCH --
        install_sigwinch(pty_master);

        // ====================================================
        // Main event loop
        // ====================================================
        let mut buf = [0u8; BUF_SIZE];
        let mut write_pos: usize = 0;

        let mut poll_fds = [
            libc::pollfd {
                fd: stdin_fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: pty_master,
                events: libc::POLLIN,
                revents: 0,
            },
        ];

        loop {
            let ret = unsafe { libc::poll(poll_fds.as_mut_ptr(), poll_fds.len() as _, -1) };
            if ret < 0 {
                let err = io::Error::last_os_error().raw_os_error().unwrap_or(0);
                if err == libc::EINTR {
                    continue;
                }
                break;
            }

            // stdin → PTY
            if poll_fds[0].revents & libc::POLLIN != 0 {
                let n = read_fd(stdin_fd, &mut buf);
                if n > 0 {
                    let _ = write_fd(pty_master, &buf[..n]);
                } else {
                    break;
                }
            }

            // PTY → stdout + log
            if poll_fds[1].revents & libc::POLLIN != 0 {
                let n = read_fd(pty_master, &mut buf);
                if n > 0 {
                    let _ = io::stdout().write_all(&buf[..n]);
                    let _ = io::stdout().flush();
                    ring_write(&mut log_file, &mut write_pos, &buf[..n]);
                } else {
                    break;
                }
            }
        }

        // -- cleanup --
        restore_termios(stdin_fd, saved_termios);
        unsafe {
            libc::close(pty_master);
        }
        std::process::exit(0);
    }

    // ---------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------

    fn get_winsize() -> libc::winsize {
        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) };
        if ret == 0 && ws.ws_row > 0 && ws.ws_col > 0 {
            ws
        } else {
            libc::winsize {
                ws_row: 24,
                ws_col: 80,
                ws_xpixel: 0,
                ws_ypixel: 0,
            }
        }
    }

    fn exec_shell(shell: &str) -> ! {
        let c_shell =
            CString::new(shell).unwrap_or_else(|_| CString::new("/bin/sh").unwrap());
        unsafe {
            libc::execlp(
                c_shell.as_ptr(),
                c_shell.as_ptr(),
                std::ptr::null::<libc::c_char>(),
            );
        }
        eprintln!("Oops: execlp '{}' failed", shell);
        std::process::exit(1);
    }

    fn set_raw_termios(fd: libc::c_int) -> Option<libc::termios> {
        let mut saved: libc::termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(fd, &mut saved) } != 0 {
            return None;
        }
        let mut raw = saved;
        unsafe { libc::cfmakeraw(&mut raw) };
        raw.c_cc[libc::VMIN] = 1;
        raw.c_cc[libc::VTIME] = 0;
        unsafe {
            libc::tcsetattr(fd, libc::TCSANOW, &raw);
        }
        Some(saved)
    }

    fn restore_termios(fd: libc::c_int, saved: Option<libc::termios>) {
        if let Some(term) = saved {
            unsafe {
                libc::tcsetattr(fd, libc::TCSANOW, &term);
            }
        }
    }

    fn ring_write(f: &mut File, pos: &mut usize, data: &[u8]) {
        let end = *pos + data.len();
        if end > LOG_SIZE {
            let remaining = LOG_SIZE - *pos;
            if remaining > 0 {
                let _ = f.seek(io::SeekFrom::Start(*pos as u64));
                let _ = f.write_all(&data[..remaining]);
            }
            let _ = f.seek(io::SeekFrom::Start(0));
            let clean = CLEAN_SIZE.min(LOG_SIZE);
            let _ = f.write_all(&vec![0u8; clean]);
            let _ = f.seek(io::SeekFrom::Start(0));
            let _ = f.write_all(&data[remaining..]);
            *pos = data.len() - remaining;
        } else {
            let _ = f.seek(io::SeekFrom::Start(*pos as u64));
            let _ = f.write_all(data);
            *pos = end;
        }
    }

    fn read_fd(fd: libc::c_int, buf: &mut [u8]) -> usize {
        match unsafe { libc::read(fd, buf.as_mut_ptr() as *mut _, buf.len()) } {
            n if n > 0 => n as usize,
            _ => 0,
        }
    }

    fn write_fd(fd: libc::c_int, buf: &[u8]) -> usize {
        match unsafe { libc::write(fd, buf.as_ptr() as *const _, buf.len()) } {
            n if n > 0 => n as usize,
            _ => 0,
        }
    }
}

// ===========================================================================
// Windows / non-Unix
// ===========================================================================
#[cfg(not(unix))]
mod imp {
    use super::*;

    pub fn run(_log_path: &Path) -> ! {
        eprintln!("Oops: --shell-logger is not supported on Windows.");
        eprintln!("      The shell logger requires a Unix PTY subsystem.");
        eprintln!("      On this platform oops re-runs the failed command");
        eprintln!("      to capture its error output — no logger is needed.");
        std::process::exit(1);
    }
}

// ===========================================================================
// Public API
// ===========================================================================

/// Start the shell logger, capturing all terminal output to `log_path`.
///
/// This function **never returns**. It spawns a PTY with the userʼs `$SHELL`
/// and copies all output into a fixed-size ring-buffer file.
///
/// # Platform support
/// - **Unix** (Linux/macOS/BSD): full PTY-based shell logger.
/// - **Windows**: prints an error and exits.
pub fn run_shell_logger(log_path: &Path) -> ! {
    eprintln!(
        "Shell logger started. Logging to {}. Press Ctrl+D or type 'exit' to stop.",
        log_path.display()
    );
    imp::run(log_path)
}
