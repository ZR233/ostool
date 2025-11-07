#[macro_use]
extern crate log;

use std::{
    fs::File,
    io::*,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

mod crc;
mod ymodem;

macro_rules! dbg {
    ($($arg:tt)*) => {{
        debug!("$ {}", &std::fmt::format(format_args!($($arg)*)));
    }};
}

const CTRL_C: u8 = 0x03;
const INT_STR: &str = "<INTERRUPT>";
const INT: &[u8] = INT_STR.as_bytes();

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
        debug!("shell ready, perfix: `{}`", s.perfix);
        Ok(s)
    }

    fn rx(&mut self) -> &mut Box<dyn Read + Send> {
        self.rx.as_mut().unwrap()
    }

    fn tx(&mut self) -> &mut Box<dyn Write + Send> {
        self.tx.as_mut().unwrap()
    }

    fn wait_for_interrupt(&mut self) -> Result<Vec<u8>> {
        let mut tx = self.tx.take().unwrap();

        let ok = Arc::new(AtomicBool::new(false));

        let tx_handle = thread::spawn({
            let ok = ok.clone();
            move || {
                while !ok.load(Ordering::Acquire) {
                    let _ = tx.write_all(&[CTRL_C]);
                    thread::sleep(Duration::from_millis(20));
                }
                tx
            }
        });
        let mut history: Vec<u8> = Vec::new();
        let mut interrupt_line: Vec<u8> = Vec::new();
        debug!("wait for interrupt");
        loop {
            match self.read_byte() {
                Ok(ch) => {
                    history.push(ch);

                    if history.last() == Some(&b'\n') {
                        let line = history.trim_ascii_end();
                        dbg!("{}", String::from_utf8_lossy(line));
                        let it = line.ends_with(INT);
                        if it {
                            interrupt_line.extend_from_slice(line);
                        }
                        history.clear();
                        if it {
                            ok.store(true, Ordering::Release);
                            break;
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

        self.tx = Some(tx_handle.join().unwrap());

        Ok(interrupt_line)
    }

    fn clear_shell(&mut self) -> Result<()> {
        let _ = self.read_to_end(&mut vec![]);
        Ok(())
    }

    fn wait_for_shell(&mut self) -> Result<()> {
        let mut line = self.wait_for_interrupt()?;
        debug!("got {}", String::from_utf8_lossy(&line));
        line.resize(line.len() - INT.len(), 0);
        self.perfix = String::from_utf8_lossy(&line).to_string();
        self.clear_shell()?;
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8> {
        let mut buff = [0u8; 1];
        let time_out = Duration::from_secs(5);
        let start = Instant::now();

        loop {
            match self.rx().read_exact(&mut buff) {
                Ok(_) => return Ok(buff[0]),
                Err(e) => {
                    if e.kind() == ErrorKind::TimedOut {
                        if start.elapsed() > time_out {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::TimedOut,
                                "Timeout",
                            ));
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    pub fn wait_for_reply(&mut self, val: &str) -> Result<String> {
        let mut reply = Vec::new();
        let mut display = Vec::new();
        debug!("wait for `{}`", val);
        loop {
            let byte = self.read_byte()?;
            reply.push(byte);
            display.push(byte);
            if byte == b'\n' {
                dbg!("{}", String::from_utf8_lossy(&display).trim_end());
                display.clear();
            }

            if reply.ends_with(val.as_bytes()) {
                dbg!("{}", String::from_utf8_lossy(&display).trim_end());
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
        self.tx().write_all("\n".as_bytes())?;
        self.tx().flush()?;
        // self.wait_for_reply(cmd)?;
        // debug!("cmd ok");
        Ok(())
    }

    fn _cmd(&mut self, cmd: &str) -> Result<String> {
        let ok_str = "cmd-ok";
        let cmd_with_id = format!("{cmd}&& echo {ok_str}");
        self.cmd_without_reply(&cmd_with_id)?;
        let perfix = self.perfix.clone();
        let res = self
            .wait_for_reply(&perfix)?
            .trim_end()
            .trim_end_matches(self.perfix.as_str().trim())
            .trim_end()
            .to_string();
        if res.ends_with(ok_str) {
            let res = res
                .trim()
                .trim_end_matches(ok_str)
                .trim_end()
                .trim_start_matches(&cmd_with_id)
                .trim()
                .to_string();
            Ok(res)
        } else {
            Err(Error::other(format!(
                "command `{cmd}` failed, response: {res}",
            )))
        }
    }

    pub fn cmd(&mut self, cmd: &str) -> Result<String> {
        info!("cmd: {cmd}");
        let mut retry = 3;
        while retry > 0 {
            match self._cmd(cmd) {
                Ok(res) => return Ok(res),
                Err(e) => {
                    warn!("cmd `{}` failed: {}, retrying...", cmd, e);
                    retry -= 1;
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
        Err(Error::other(format!(
            "command `{cmd}` failed after retries",
        )))
    }

    pub fn set_env(&mut self, name: impl Into<String>, value: impl Into<String>) -> Result<()> {
        self.cmd(&format!("setenv {} {}", name.into(), value.into()))?;
        Ok(())
    }

    pub fn env(&mut self, name: impl Into<String>) -> Result<String> {
        let name = name.into();
        let s = self.cmd(&format!("echo ${}", name))?;
        let sp = s
            .split("\n")
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>();
        let s = sp
            .last()
            .ok_or(Error::new(
                ErrorKind::NotFound,
                format!("env {} not found", name),
            ))?
            .to_string();
        Ok(s)
    }

    pub fn env_int(&mut self, name: impl Into<String>) -> Result<usize> {
        let name = name.into();
        let line = self.env(&name)?;
        debug!("env {name} = {line}");

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
        loop {
            let byte = self.read_byte()?;
            reply.push(byte);
            print_raw(&[byte]);

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
    #[cfg(target_os = "windows")]
    print_raw_win(buff);
    #[cfg(not(target_os = "windows"))]
    stdout().write_all(buff).unwrap();
}

#[cfg(target_os = "windows")]
fn print_raw_win(buff: &[u8]) {
    use std::sync::Mutex;
    static PRINT_BUFF: Mutex<Vec<u8>> = Mutex::new(Vec::new());

    let mut g = PRINT_BUFF.lock().unwrap();

    g.extend_from_slice(buff);

    if g.ends_with(b"\n") {
        let s = String::from_utf8_lossy(&g[..]);
        println!("{}", s.trim());
        g.clear();
    }
}
