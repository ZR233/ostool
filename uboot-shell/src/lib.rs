use std::{
    fs::File,
    io::*,
    path::PathBuf,
    time::{Duration, Instant},
};

mod crc;
mod ymodem;

pub struct UbootShell {
    pub tx: Option<Box<dyn Write + Send>>,
    pub rx: Option<Box<dyn Read + Send>>,
}

impl UbootShell {
    /// Create a new UbootShell instance, block wait for uboot shell.
    pub fn new(tx: impl Write + Send + 'static, rx: impl Read + Send + 'static) -> Result<Self> {
        let mut s = Self {
            tx: Some(Box::new(tx)),
            rx: Some(Box::new(rx)),
        };
        s.wait_for_shell()?;
        Ok(s)
    }

    fn rx(&mut self) -> &mut Box<dyn Read + Send> {
        self.rx.as_mut().unwrap()
    }

    fn tx(&mut self) -> &mut Box<dyn Write + Send> {
        self.tx.as_mut().unwrap()
    }

    fn wait_for_shell(&mut self) -> Result<()> {
        let mut buf = [0u8; 1];
        let mut history: Vec<u8> = Vec::new();
        const CTRL_C: u8 = 0x03;

        let mut last = Instant::now();

        loop {
            match self.rx().read(&mut buf) {
                Ok(n) => {
                    if n == 1 {
                        let ch = buf[0];
                        if ch == b'\n' && history.last() != Some(&b'\r') {
                            stdout().write_all(b"\r").unwrap();
                            history.push(b'\r');
                        }
                        history.push(ch);

                        stdout().write_all(&buf).unwrap();

                        if history.ends_with(c"<INTERRUPT>".to_bytes()) {
                            return Ok(());
                        }

                        if last.elapsed() > Duration::from_millis(20) {
                            let _ = self.tx().write_all(&[CTRL_C]);
                            last = Instant::now();
                        }
                    }
                }

                Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    fn read_line(&mut self) -> Result<String> {
        let mut line_raw = Vec::new();
        let mut byte = [0; 1];

        loop {
            let n = self.rx().read(&mut byte)?;
            if n == 0 {
                break;
            }

            stdout().write_all(&byte).unwrap();

            if byte[0] == b'\r' {
                continue;
            }

            if byte[0] == b'\n' {
                break;
            }

            line_raw.push(byte[0]);
        }

        if line_raw.is_empty() {
            return Ok(String::new());
        }

        let line = String::from_utf8_lossy(&line_raw);
        Ok(line.trim().to_string())
    }

    pub fn wait_for_reply(&mut self, val: &str) -> Result<String> {
        let mut reply = Vec::new();
        let mut buff = [0u8; 1];
        loop {
            self.rx().read_exact(&mut buff)?;
            reply.push(buff[0]);

            if reply.ends_with(val.as_bytes()) {
                break;
            }
        }
        Ok(String::from_utf8_lossy(&reply).trim().to_string())
    }

    pub fn cmd_without_reply(&mut self, cmd: &str, want_echo: bool) -> Result<String> {
        self.tx().write_all(cmd.as_bytes())?;
        self.tx().write_all("\r\n".as_bytes())?;
        self.tx().flush()?;

        if !want_echo {
            return Ok(String::new());
        }

        let shell_start;
        loop {
            let line = self.read_line()?;
            if line.contains(cmd) {
                shell_start = line.trim().trim_end_matches(cmd).trim().to_string();
                break;
            }
        }
        Ok(shell_start)
    }

    pub fn cmd(&mut self, cmd: &str) -> Result<String> {
        let shell_start = self.cmd_without_reply(cmd, true)?;

        Ok(self
            .wait_for_reply(&shell_start)?
            .trim_end_matches(&shell_start)
            .to_string())
    }

    pub fn set_env(&mut self, name: impl Into<String>, value: impl Into<String>) -> Result<()> {
        self.cmd(&format!("setenv {} {}", name.into(), value.into()))?;
        Ok(())
    }

    pub fn env(&mut self, name: impl Into<String>) -> Result<String> {
        let name = name.into();
        let s = self.cmd(&format!("echo ${}", name))?;
        if s.is_empty() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("env {} not found", name),
            ));
        }
        Ok(s)
    }

    pub fn env_int(&mut self, name: impl Into<String>) -> Result<usize> {
        let name = name.into();
        let line = self.env(&name)?;
        parse_int(&line).ok_or(Error::new(
            ErrorKind::InvalidData,
            format!("env {name} is not a number"),
        ))
    }

    pub fn loady(
        &mut self,
        addr: usize,
        file: impl Into<PathBuf>,
        on_progress: impl Fn(usize, usize),
    ) -> Result<String> {
        let shell_start = self.cmd_without_reply(&format!("loady {:#x}", addr,), true)?;
        let crc = self.wait_for_load_crc()?;
        let mut p = ymodem::Ymodem::new(crc);

        let file = file.into();
        let name = file.file_name().unwrap().to_str().unwrap();

        let mut file = File::open(&file).unwrap();

        let size = file.metadata().unwrap().len() as usize;

        p.send(self, &mut file, name, size, |p| {
            on_progress(p, size);
        })?;

        Ok(self
            .wait_for_reply(&shell_start)?
            .trim_end_matches(&shell_start)
            .to_string())
    }

    fn wait_for_load_crc(&mut self) -> Result<bool> {
        let mut reply = Vec::new();
        let mut buff = [0u8; 1];
        loop {
            self.rx().read_exact(&mut buff)?;
            reply.push(buff[0]);
            let _ = stdout().write_all(&buff);

            if reply.ends_with(b"C") {
                return Ok(true);
            }
            let res = String::from_utf8_lossy(&reply);
            if res.contains("try 'help'") {
                panic!("{}", res);
            }
        }
    }
}

impl Read for UbootShell {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.rx().read(buf)
    }
}

impl Write for UbootShell {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.tx().write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.tx().flush()
    }
}

fn parse_int(line: &str) -> Option<usize> {
    let mut line = line.trim();
    let mut radix = 10;
    if line.starts_with("0x") {
        line = &line[2..];
        radix = 16;
    }
    u64::from_str_radix(line, radix).ok().map(|o| o as _)
}
