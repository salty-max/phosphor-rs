//! The `terminal` module provides a low-level abstraction over the terminal device.
//!
//! It handles the interaction with the operating system to:
//! * Enter and exit **Raw Mode** (disabling canonical input and local echo).
//! * Read and write to the underlying TTY file descriptor.
//! * Query terminal capabilities like window size.
//! * Poll for input availability (crucial for handling escape sequences).
//!
//! # Architecture
//! This module uses the **Dependency Injection** pattern to facilitate testing.
//! * [`System`]: A trait defining the low-level OS operations.
//! * [`LibcSystem`]: The production implementation using `libc` FFI.
//! * [`Terminal`]: The high-level wrapper used by the application.

use std::ffi::c_void;
use std::io;
use std::os::fd::RawFd;
use std::time::Duration;

/// Abstraction over system calls relative to the terminal.
///
/// This trait acts as a "seam" for testing, allowing the [`Terminal`] struct to
/// interact with a mock OS during unit tests instead of making real syscalls.
pub trait System {
    /// Opens a file descriptor to the current TTY (usually `/dev/tty`).
    ///
    /// # Errors
    /// Returns an error if the device cannot be opened.
    fn open_tty(&self) -> io::Result<RawFd>;

    /// Closes the given file descriptor.
    ///
    /// # Errors
    /// Returns an error if the system call fails.
    fn close_tty(&self, fd: RawFd) -> io::Result<()>;

    /// Enables "Raw Mode" on the specified file descriptor.
    ///
    /// This disables line buffering, local echo, and signal processing.
    /// Returns the original `termios` configuration so it can be restored later.
    ///
    /// # Errors
    /// Returns an error if the terminal attributes cannot be retrieved or set.
    fn enable_raw(&self, fd: RawFd) -> io::Result<libc::termios>;

    /// Restores the terminal to its original configuration (Canonical Mode).
    ///
    /// # Errors
    /// Returns an error if the terminal attributes cannot be restored.
    fn disable_raw(&self, fd: RawFd, original: &libc::termios) -> io::Result<()>;

    /// Queries the kernel for the current terminal window size (cols, rows).
    ///
    /// # Errors
    /// Returns an error if the `ioctl` call fails.
    fn get_window_size(&self, fd: RawFd) -> io::Result<(u16, u16)>;

    /// Reads raw bytes from the file descriptor into the buffer.
    ///
    /// # Errors
    /// Returns an error if the read operation fails.
    fn read(&self, fd: RawFd, buf: &mut [u8]) -> io::Result<usize>;

    /// Writes raw bytes from the buffer to the file descriptor.
    ///
    /// # Errors
    /// Returns an error if the write operation fails.
    fn write(&self, fd: RawFd, buf: &[u8]) -> io::Result<usize>;

    /// Checks if data is available to read within the given timeout.
    ///
    /// Returns `Ok(true)` if data is ready, `Ok(false)` if the timeout expired,
    /// or `Err` if the system call failed.
    fn poll(&self, fd: RawFd, timeout: Duration) -> io::Result<bool>;
}

/// The production implementation of [`System`] using `libc` calls.
///
/// This struct performs `unsafe` FFI calls to the underlying OS. It is the
/// default backend for [`Terminal`].
pub struct LibcSystem;

impl System for LibcSystem {
    /// Opens `/dev/tty` for read/write access.
    fn open_tty(&self) -> io::Result<RawFd> {
        unsafe {
            let path = std::ffi::CString::new("/dev/tty")
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"))?;

            let fd = libc::open(path.as_ptr(), libc::O_RDWR);
            if fd < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(fd)
        }
    }

    fn close_tty(&self, fd: RawFd) -> io::Result<()> {
        unsafe {
            if libc::close(fd) < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
    }

    /// Configures the terminal for raw I/O.
    ///
    /// Flags modified:
    /// * `c_iflag`: Disables `BRKINT`, `ICRNL`, `INPCK`, `ISTRIP`, `IXON`.
    /// * `c_oflag`: Disables `OPOST`.
    /// * `c_cflag`: Sets `CS8`.
    /// * `c_lflag`: Disables `ECHO`, `ICANON`, `IEXTEN`, `ISIG`.
    fn enable_raw(&self, fd: RawFd) -> io::Result<libc::termios> {
        unsafe {
            let mut termios = std::mem::zeroed();

            if libc::tcgetattr(fd, &mut termios) < 0 {
                return Err(io::Error::last_os_error());
            }

            let original = termios;

            termios.c_iflag &=
                !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
            termios.c_oflag &= !(libc::OPOST);
            termios.c_cflag |= libc::CS8;
            termios.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);

            if libc::tcsetattr(fd, libc::TCSAFLUSH, &termios) < 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(original)
        }
    }

    fn disable_raw(&self, fd: RawFd, original: &libc::termios) -> io::Result<()> {
        unsafe {
            // Flush the screen before exiting
            if libc::tcflush(fd, libc::TCIFLUSH) < 0 {
                return Err(io::Error::last_os_error());
            }

            if libc::tcsetattr(fd, libc::TCSAFLUSH, original) < 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(())
        }
    }

    fn get_window_size(&self, fd: RawFd) -> io::Result<(u16, u16)> {
        unsafe {
            let mut winsize = libc::winsize {
                ws_col: 0,
                ws_row: 0,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };

            if libc::ioctl(fd, libc::TIOCGWINSZ, &mut winsize) < 0 {
                return Err(io::Error::last_os_error());
            }

            Ok((winsize.ws_col, winsize.ws_row))
        }
    }

    fn read(&self, fd: RawFd, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let bytes = libc::read(fd, buf.as_mut_ptr() as *mut c_void, buf.len());
            if bytes < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(bytes as usize)
        }
    }

    fn write(&self, fd: RawFd, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            let bytes = libc::write(fd, buf.as_ptr() as *const c_void, buf.len());
            if bytes < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(bytes as usize)
        }
    }

    fn poll(&self, fd: RawFd, timeout: Duration) -> io::Result<bool> {
        unsafe {
            let mut pfd = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };

            let timeout_ms = timeout.as_millis() as libc::c_int;

            let ret = libc::poll(&mut pfd, 1, timeout_ms);
            if ret < 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(ret > 0)
        }
    }
}

use std::fmt;

/// A high-level wrapper around the terminal state and I/O.
///
/// This struct manages the lifecycle of **Raw Mode** using the RAII pattern.
/// When a `Terminal` is created, it takes control of the TTY. When it is dropped,
/// it automatically restores the original terminal configuration.
pub struct Terminal {
    system: Box<dyn System>,
    fd: RawFd,
    original_termios: Option<libc::termios>,
}

impl fmt::Debug for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Terminal")
            .field("fd", &self.fd)
            .finish_non_exhaustive()
    }
}

impl Terminal {
    /// Creates a new `Terminal` instance using the default [`LibcSystem`].
    ///
    /// This will attempt to open `/dev/tty` and enter Raw Mode immediately.
    ///
    /// # Errors
    /// Returns an error if `/dev/tty` cannot be opened or if Raw Mode cannot be enabled.
    pub fn new() -> io::Result<Self> {
        Self::new_with_system(Box::new(LibcSystem))
    }

    /// Creates a new `Terminal` with a specific system backend.
    ///
    /// This is primarily used for dependency injection in tests.
    pub fn new_with_system(system: Box<dyn System>) -> io::Result<Self> {
        let fd = system.open_tty()?;

        let mut term = Self {
            system,
            fd,
            original_termios: None,
        };

        let termios = term.system.enable_raw(fd)?;
        term.original_termios = Some(termios);

        term.hide_cursor()?;
        term.enable_mouse_capture()?;
        term.enter_alternate_buffer()?;

        Ok(term)
    }

    /// Returns the current size of the terminal as `(cols, rows)`.
    pub fn size(&self) -> io::Result<(u16, u16)> {
        self.system.get_window_size(self.fd)
    }

    /// Reads raw bytes from the terminal into the provided buffer.
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.system.read(self.fd, buf)
    }

    /// Writes raw bytes to the terminal.
    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.system.write(self.fd, buf)
    }

    /// Checks if data is available to read in the given timeout.
    ///
    /// This is used to differentiate between ambiguous keys (like `Esc` vs `Alt+...`).
    /// * `Ok(true)`: Data is waiting in the kernel buffer.
    /// * `Ok(false)`: Timeout expired with no data.
    pub fn poll(&self, timeout: Duration) -> io::Result<bool> {
        self.system.poll(self.fd, timeout)
    }

    /// Shows the terminal cursor.
    pub fn show_cursor(&self) -> io::Result<()> {
        self.write(b"\x1b[?25h")?;
        Ok(())
    }

    /// Hides the terminal cursor.
    pub fn hide_cursor(&self) -> io::Result<()> {
        self.write(b"\x1b[?25l")?;
        Ok(())
    }

    /// Switches the terminal to the alternate screen buffer.
    pub fn enter_alternate_buffer(&self) -> io::Result<()> {
        self.write(b"\x1b[?1049h")?;
        Ok(())
    }

    /// Switches the terminal back to the main screen buffer.
    pub fn exit_alternate_buffer(&self) -> io::Result<()> {
        self.write(b"\x1b[?1049l")?;
        Ok(())
    }

    pub fn enable_mouse_capture(&self) -> io::Result<()> {
        self.write(b"\x1b[?1000h")?;
        Ok(())
    }

    pub fn disable_mouse_capture(&self) -> io::Result<()> {
        self.write(b"\x1b[?1000l")?;
        Ok(())
    }
}

impl Drop for Terminal {
    /// Automatically restores the terminal configuration.
    ///
    /// If restoration fails, the error is logged to `debug.log`.
    fn drop(&mut self) {
        let _ = self.disable_mouse_capture();
        let _ = self.exit_alternate_buffer();
        let _ = self.show_cursor();

        if let Some(termios) = self.original_termios
            && let Err(e) = self.system.disable_raw(self.fd, &termios)
        {
            log!("Error restoring terminal: {}", e);
        }

        let _ = self.system.close_tty(self.fd);
    }
}

#[cfg(test)]
pub(crate) mod mocks {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    pub struct MockSystem {
        pub log: Arc<Mutex<Vec<String>>>,
        pub input_buffer: Arc<Mutex<Vec<u8>>>,
        pub fail_open: bool,
        pub fail_enable_raw: bool,
        pub max_read_size: Option<usize>,
    }

    impl MockSystem {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn with_max_read(mut self, size: usize) -> Self {
            self.max_read_size = Some(size);
            self
        }

        pub fn push_input(&self, data: &[u8]) {
            self.input_buffer.lock().unwrap().extend_from_slice(data);
        }

        fn push_log(&self, msg: &str) {
            if let Ok(mut log) = self.log.lock() {
                log.push(msg.to_string());
            }
        }
    }

    impl System for MockSystem {
        fn open_tty(&self) -> io::Result<RawFd> {
            self.push_log("open_tty");
            if self.fail_open {
                return Err(io::Error::new(io::ErrorKind::Other, "Mock Open Failed"));
            }
            Ok(100)
        }

        fn close_tty(&self, _fd: RawFd) -> io::Result<()> {
            self.push_log("close_tty");
            Ok(())
        }

        fn enable_raw(&self, fd: RawFd) -> io::Result<libc::termios> {
            self.push_log(&format!("enable_raw({})", fd));
            if self.fail_enable_raw {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Mock Enable Raw Failed",
                ));
            }
            // Return empty termios
            Ok(unsafe { std::mem::zeroed() })
        }

        fn disable_raw(&self, fd: RawFd, _original: &libc::termios) -> io::Result<()> {
            self.push_log(&format!("disable_raw({})", fd));
            Ok(())
        }

        fn get_window_size(&self, fd: RawFd) -> io::Result<(u16, u16)> {
            self.push_log(&format!("get_window_size({})", fd));
            Ok((80, 24))
        }

        fn read(&self, fd: RawFd, buf: &mut [u8]) -> io::Result<usize> {
            self.push_log(&format!("read({})", fd));
            let mut input = self.input_buffer.lock().unwrap();
            if input.is_empty() {
                return Ok(0);
            }

            let mut len = std::cmp::min(buf.len(), input.len());
            if let Some(max) = self.max_read_size {
                len = std::cmp::min(len, max);
            }

            // copy_from_slice handles copying data
            buf[..len].copy_from_slice(&input[..len]);
            // Remove read bytes from the "mock input stream"
            input.drain(0..len);
            Ok(len)
        }

        fn write(&self, fd: RawFd, buf: &[u8]) -> io::Result<usize> {
            let content = String::from_utf8_lossy(buf);
            self.push_log(&format!("write({}, \"{}\")", fd, content));
            Ok(buf.len())
        }

        fn poll(&self, _fd: RawFd, _timeout: Duration) -> io::Result<bool> {
            let input = self.input_buffer.lock().unwrap();
            Ok(!input.is_empty())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mocks::MockSystem;

    #[test]
    fn test_terminal_initialization() {
        let mock = MockSystem::new();
        let log_ref = mock.log.clone();
        let _term = Terminal::new_with_system(Box::new(mock)).unwrap();

        let log = log_ref.lock().unwrap();
        // Check that log contains specific strings
        assert!(log.contains(&"open_tty".to_string()));
        // Check that at least one entry starts with "enable_raw"
        assert!(log.iter().any(|entry| entry.starts_with("enable_raw")));
    }

    #[test]
    fn test_lifecycle_and_delegation() {
        let mock = MockSystem::new();
        let log_ref = mock.log.clone();

        {
            let term = Terminal::new_with_system(Box::new(mock)).expect("Failed to init terminal");

            term.size().unwrap();
            term.write(b"foo").unwrap();

            let mut buf = [0u8; 10];
            term.read(&mut buf).unwrap();
        } // Drop happens here

        let log = log_ref.lock().unwrap();
        // Note: Indices depend on exact call order.
        assert_eq!(log[0], "open_tty");
        assert_eq!(log[1], "enable_raw(100)");
        assert_eq!(log[2], "write(100, \"\x1b[?25l\")");
        assert_eq!(log[3], "write(100, \"\x1b[?1000h\")");
        assert_eq!(log[4], "write(100, \"\x1b[?1049h\")");
        assert_eq!(log[5], "get_window_size(100)");
        assert_eq!(log[6], "write(100, \"foo\")");
        assert_eq!(log[7], "read(100)");
        assert_eq!(log[8], "write(100, \"\x1b[?1000l\")");
        assert_eq!(log[9], "write(100, \"\x1b[?1049l\")");
        assert_eq!(log[10], "write(100, \"\x1b[?25h\")");
        assert_eq!(log[11], "disable_raw(100)");
        assert_eq!(log[12], "close_tty");
        assert_eq!(log.len(), 13);
    }

    #[test]
    fn test_initialization_failure_open() {
        let mut mock = MockSystem::new();
        mock.fail_open = true;

        let res = Terminal::new_with_system(Box::new(mock));
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), io::ErrorKind::Other);
    }

    #[test]
    fn test_initialization_failure_enable_raw() {
        let mut mock = MockSystem::new();
        mock.fail_enable_raw = true;

        let res = Terminal::new_with_system(Box::new(mock));
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), io::ErrorKind::Other);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_libc_system_open_tty() {
        let sys = LibcSystem;
        let fd = sys.open_tty().expect("Failed to open /dev/tty");
        assert!(fd > 0);
        unsafe { libc::close(fd) };
    }

    #[test]
    #[ignore]
    fn test_libc_system_raw_mode() {
        let sys = LibcSystem;
        let fd = sys.open_tty().expect("Failed to open TTY");

        let original = sys.enable_raw(fd).expect("Failed to enable raw");

        let mut current: libc::termios = unsafe { std::mem::zeroed() };
        unsafe { libc::tcgetattr(fd, &mut current) };
        assert_eq!(current.c_lflag & libc::ECHO, 0, "ECHO should be disabled");

        sys.disable_raw(fd, &original)
            .expect("Failed to disable raw");

        unsafe { libc::tcgetattr(fd, &mut current) };
        assert_eq!(
            current.c_lflag & libc::ECHO,
            original.c_lflag & libc::ECHO,
            "ECHO state should be restored"
        );

        unsafe { libc::close(fd) };
    }

    #[test]
    #[ignore]
    fn test_libc_system_io() {
        let sys = LibcSystem;
        let fd = sys.open_tty().expect("Failed to open TTY");

        let msg = b"Integration Test: Hello World\r\n";
        let written = sys.write(fd, msg).expect("Failed to write");
        assert_eq!(written, msg.len());

        let size = sys.get_window_size(fd).expect("Failed to get window size");
        assert!(size.0 > 0);
        assert!(size.1 > 0);

        unsafe { libc::close(fd) };
    }

    #[test]
    #[ignore]
    fn test_libc_system_poll() {
        let sys = LibcSystem;
        let fd = sys.open_tty().expect("Failed to open TTY");

        // Should timeout with false
        let res = sys
            .poll(fd, Duration::from_millis(10))
            .expect("Poll failed");
        assert!(!res);

        unsafe { libc::close(fd) };
    }

    #[test]
    #[ignore]
    fn test_libc_system_close_tty() {
        let sys = LibcSystem;
        let fd = sys.open_tty().expect("Failed to open TTY");
        sys.close_tty(fd).expect("Failed to close TTY");
    }
}
