use std::ffi::c_void;
use std::io;
use std::os::fd::RawFd;

/// Abstraction over system calls relative to the terminal.
pub trait System {
    fn open_tty(&self) -> io::Result<RawFd>;
    fn enable_raw(&self, fd: RawFd) -> io::Result<libc::termios>;
    fn disable_raw(&self, fd: RawFd, original: &libc::termios) -> io::Result<()>;
    fn get_window_size(&self, fd: RawFd) -> io::Result<(u16, u16)>;
    fn read(&self, fd: RawFd, buf: &mut [u8]) -> io::Result<usize>;
    fn write(&self, fd: RawFd, buf: &[u8]) -> io::Result<usize>;
}

/// Production implementation using libc.
pub struct LibcSystem;

impl System for LibcSystem {
    fn open_tty(&self) -> io::Result<RawFd> {
        unsafe {
            let path = std::ffi::CString::new("/dev/tty")?;
            let fd = libc::open(path.as_ptr(), libc::O_RDWR);
            if fd < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(fd)
        }
    }

    fn enable_raw(&self, fd: RawFd) -> io::Result<libc::termios> {
        unsafe {
            let mut termios = std::mem::zeroed();

            if libc::tcgetattr(fd, &mut termios) < 0 {
                return Err(io::Error::last_os_error());
            }

            let original = termios.clone();

            termios.c_lflag &=
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
}

use std::fmt;

/// High-level wrapper around the terminal.
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
    pub fn new() -> io::Result<Self> {
        Self::new_with_system(Box::new(LibcSystem))
    }

    /// Creates a new Terminal. immediately enabling raw mode.
    pub fn new_with_system(system: Box<dyn System>) -> io::Result<Self> {
        let fd = system.open_tty()?;
        let termios = system.enable_raw(fd)?;
        Ok(Self {
            system,
            fd,
            original_termios: Some(termios),
        })
    }

    pub fn size(&self) -> io::Result<(u16, u16)> {
        self.system.get_window_size(self.fd)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.system.read(self.fd, buf)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.system.write(self.fd, buf)
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        if let Some(termios) = self.original_termios {
            if let Err(e) = self.system.disable_raw(self.fd, &termios) {
                eprintln!("{}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Arc, Mutex};

    // --- Integration Tests (Ignored by default) ---

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

        // 1. Enable Raw
        let original = sys.enable_raw(fd).expect("Failed to enable raw");

        // 2. Verify ECHO is off
        let mut current: libc::termios = unsafe { std::mem::zeroed() };
        unsafe { libc::tcgetattr(fd, &mut current) };
        assert_eq!(current.c_lflag & libc::ECHO, 0, "ECHO should be disabled");

        // 3. Disable Raw
        sys.disable_raw(fd, &original)
            .expect("Failed to disable raw");

        // 4. Verify ECHO is back on
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

        // Test Write
        let msg = b"Integration Test: Hello World\r\n";
        let written = sys.write(fd, msg).expect("Failed to write");
        assert_eq!(written, msg.len());

        // We can't easily test read without user interaction,
        // but we can verify the call doesn't panic.

        unsafe { libc::close(fd) };
    }

    // --- Unit Tests ---

    #[test]
    fn test_terminal_initialization() {
        let mock = MockSystem::new();
        let log_ref = mock.log.clone();
        let _term = Terminal::new_with_system(Box::new(mock)).unwrap();

        let log = log_ref.lock().unwrap();
        assert!(log.contains(&"open_tty".to_string()));
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

            // Drop happens here
        }

        let log = log_ref.lock().unwrap();
        assert_eq!(log[0], "open_tty");
        assert_eq!(log[1], "enable_raw(100)");
        assert_eq!(log[2], "get_window_size(100)");
        assert_eq!(log[3], "write(100, 3 bytes)");
        assert_eq!(log[4], "read(100)");
        assert_eq!(log[5], "disable_raw(100)");
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

    // --- Mocks ---

    #[derive(Default)]
    struct MockSystem {
        log: Arc<Mutex<Vec<String>>>,
        fail_open: bool,
        fail_enable_raw: bool,
    }

    impl MockSystem {
        fn new() -> Self {
            Self::default()
        }

        fn push_log(&self, msg: &str) {
            self.log.lock().unwrap().push(msg.to_string());
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

        fn enable_raw(&self, fd: RawFd) -> io::Result<libc::termios> {
            self.push_log(&format!("enable_raw({})", fd));
            if self.fail_enable_raw {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Mock Enable Raw Failed",
                ));
            }
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

        fn read(&self, fd: RawFd, _buf: &mut [u8]) -> io::Result<usize> {
            self.push_log(&format!("read({})", fd));
            Ok(0)
        }

        fn write(&self, fd: RawFd, buf: &[u8]) -> io::Result<usize> {
            self.push_log(&format!("write({}, {} bytes)", fd, buf.len()));
            Ok(buf.len())
        }
    }
}
