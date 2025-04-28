use std::{
    fs::File,
    io::*,
    path::PathBuf,
    time::{Duration, Instant},
};

use colored::Colorize;

mod crc;
mod ymodem;

macro_rules! trace {
    ($($arg:tt)*) => {{
        println!("\r\n{}", &std::fmt::format(format_args!($($arg)*)).bright_black());
    }};
}

pub struct UbootShell {
    pub tx: Option<Box<dyn Write + Send>>,
    pub rx: Option<Box<dyn Read + Send>>,
    perfix: String,
}

impl UbootShell {
    /// Create a new UbootShell instance, block wait for uboot shell.
    pub fn new(tx: impl Write + Send + 'static, rx: impl Read + Send + 'static) -> Result<Self> {
        let mut s = Self {
            tx: Some(Box::new(tx)),
            rx: Some(Box::new(rx)),
            perfix: "".to_string(),
        };
        s.wait_for_shell()?;
        trace!("shell ready, perfix: `{}`", s.perfix);
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
        let mut is_interrupt_found = false;
        let mut is_shell_ok = false;

        loop {
            match self.rx().read(&mut buf) {
                Ok(n) => {
                    if n == 1 {
                        let ch = buf[0];
                        if ch == b'\n' && history.last() != Some(&b'\r') {
                            print_raw(b"\r");
                            history.push(b'\r');
                        }
                        history.push(ch);

                        print_raw(&buf);

                        if history.ends_with(c"<INTERRUPT>".to_bytes()) && !is_interrupt_found {
                            let line = history.split(|n| *n == b'\n').next_back().unwrap();
                            let s = String::from_utf8_lossy(line);
                            self.perfix = s.trim().replace("<INTERRUPT>", "").trim().to_string();
                            is_interrupt_found = true;
                            let _ = self.tx().write_all("testshell\r\n".as_bytes());
                        }

                        if is_interrupt_found && history.ends_with("\'help\'".as_bytes()) {
                            is_shell_ok = true;
                        }

                        if is_shell_ok && history.ends_with(self.perfix.as_bytes()) {
                            return Ok(());
                        }

                        if last.elapsed() > Duration::from_millis(20) && !is_interrupt_found {
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

    // fn read_line(&mut self) -> Result<String> {
    //     let mut line_raw = Vec::new();
    //     let mut byte = [0; 1];

    //     loop {
    //         let n = self.rx().read(&mut byte)?;
    //         if n == 0 {
    //             break;
    //         }

    //         print_raw(&byte);

    //         if byte[0] == b'\r' {
    //             continue;
    //         }

    //         if byte[0] == b'\n' {
    //             break;
    //         }

    //         line_raw.push(byte[0]);
    //     }

    //     if line_raw.is_empty() {
    //         return Ok(String::new());
    //     }

    //     let line = String::from_utf8_lossy(&line_raw);
    //     Ok(line.trim().to_string())
    // }

    pub fn wait_for_reply(&mut self, val: &str) -> Result<String> {
        let mut reply = Vec::new();
        let mut buff = [0u8; 1];

        trace!("wait for `{}`", val);
        loop {
            self.rx().read_exact(&mut buff)?;
            reply.push(buff[0]);
            print_raw(&buff);

            if reply.ends_with(val.as_bytes()) {
                break;
            }
        }
        Ok(String::from_utf8_lossy(&reply)
            .trim()
            .trim_end_matches(&self.perfix)
            .to_string())
    }

    pub fn cmd_without_reply(&mut self, cmd: &str) -> Result<()> {
        self.tx().write_all(cmd.as_bytes())?;
        self.tx().write_all("\r\n".as_bytes())?;
        self.tx().flush()?;
        self.wait_for_reply(cmd)?;
        trace!("cmd ok");
        Ok(())
    }

    pub fn cmd(&mut self, cmd: &str) -> Result<String> {
        self.cmd_without_reply(cmd)?;
        let perfix = self.perfix.clone();
        self.wait_for_reply(&perfix)
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
        self.cmd_without_reply(&format!("loady {:#x}", addr,))?;
        let crc = self.wait_for_load_crc()?;
        let mut p = ymodem::Ymodem::new(crc);

        let file = file.into();
        let name = file.file_name().unwrap().to_str().unwrap();

        let mut file = File::open(&file).unwrap();

        let size = file.metadata().unwrap().len() as usize;

        p.send(self, &mut file, name, size, |p| {
            on_progress(p, size);
        })?;
        let perfix = self.perfix.clone();
        self.wait_for_reply(&perfix)
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

fn print_raw(buff: &[u8]) {
    stdout().write_all(buff).unwrap();
}
