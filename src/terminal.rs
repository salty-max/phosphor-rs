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

    fn enable_raw(&self, _fd: RawFd) -> io::Result<libc::termios> {
        // TODO: Implement using libc::tcgetattr and libc::tcsetattr
        // Remember to disable ECHO, ICANON, ISIG, IEXTEN
        todo!("Implement enable_raw_mode")
    }

    fn disable_raw(&self, _fd: RawFd, _original: &libc::termios) -> io::Result<()> {
        // TODO: Restore original attributes
        todo!("Implement disable_raw_mode")
    }

    fn get_window_size(&self, _fd: RawFd) -> io::Result<(u16, u16)> {
        // TODO: Use libc::ioctl with TIOCGWINSZ
        todo!("Implement get_window_size")
    }

    fn read(&self, _fd: RawFd, _buf: &mut [u8]) -> io::Result<usize> {
        // TODO: Use libc::read
        todo!("Implement read")
    }

    fn write(&self, _fd: RawFd, _buf: &[u8]) -> io::Result<usize> {
        // TODO: Use libc::write
        todo!("Implement write")
    }
}

/// High-level wrapper around the terminal.
pub struct Terminal {
    system: Box<dyn System>,
    fd: RawFd,
    original_termios: Option<libc::termios>,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        Self::new_with_system(Box::new(LibcSystem))
    }

    /// Creates a new Terminal. immediately enabling raw mode.
    pub fn new_with_system(system: Box<dyn System>) -> io::Result<Self> {
        let fd = system.open_tty().unwrap();
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

    #[test]
    #[ignore]
    fn test_libc_system_open_tty() {
        let sys = LibcSystem;
        let fd = sys.open_tty().expect("Failed to open /dev/tty");
        assert!(fd > 0);
        unsafe { libc::close(fd) };
    }

    // MockSystem to record calls
    #[derive(Default)]
    struct MockSystem {
        log: Arc<Mutex<Vec<String>>>,
    }

    impl MockSystem {
        fn new() -> Self {
            Self::default()
        }

        fn push_log(&self, msg: &str) {
            self.log.lock().unwrap().push(msg.to_string());
        }

        fn get_log(&self) -> Vec<String> {
            self.log.lock().unwrap().clone()
        }
    }

    impl System for MockSystem {
        fn open_tty(&self) -> io::Result<RawFd> {
            self.push_log("open_tty");
            Ok(100)
        }

        fn enable_raw(&self, fd: RawFd) -> io::Result<libc::termios> {
            self.push_log(&format!("enable_raw({})", fd));
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

    #[test]
    fn test_terminal_initialization() {
        let mock = MockSystem::new();
        let log_ref = mock.log.clone();
        let _term = Terminal::new_with_system(Box::new(mock)).unwrap();

        let log = log_ref.lock().unwrap();
        assert!(log.contains(&"open_tty".to_string()));
        // We don't know the FD yet, but we know it should have enabled raw mode
        assert!(log.iter().any(|entry| entry.starts_with("enable_raw")));
    }

    #[test]
    fn test_lifecycle_and_delegation() {
        let mock = MockSystem::new();
        let log_ref = mock.log.clone();

        {
            let term = Terminal::new_with_system(Box::new(mock)).expect("Failed to init terminal");

            // Should have opened and enabled
            term.size().unwrap();
            term.write(b"foo").unwrap();

            // Drop happens here
        }

        let log = log_ref.lock().unwrap();
        assert_eq!(log[0], "open_tty");
        assert_eq!(log[1], "enable_raw(100)");
        assert_eq!(log[2], "get_window_size(100)");
        assert_eq!(log[3], "write(100, 3 bytes)");
        assert_eq!(log[4], "disable_raw(100)");
    }
}
