use std::{
    io::*,
    time::{Duration, Instant},
};

pub struct UbootCli {
    pub tx: Option<Box<dyn Write + Send>>,
    pub rx: Option<Box<dyn Read + Send>>,
}

impl UbootCli {
    pub fn new(tx: impl Write + Send + 'static, rx: impl Read + Send + 'static) -> Self {
        Self {
            tx: Some(Box::new(tx)),
            rx: Some(Box::new(rx)),
        }
    }

    fn rx(&mut self) -> &mut Box<dyn Read + Send> {
        self.rx.as_mut().unwrap()
    }

    fn tx(&mut self) -> &mut Box<dyn Write + Send> {
        self.tx.as_mut().unwrap()
    }

    pub fn wait_for_shell(&mut self) {
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

                        if history.ends_with(c"<INTERRUPT>".to_bytes())
                        // || history.ends_with("=>".as_bytes())
                        {
                            return;
                        }

                        if last.elapsed() > Duration::from_millis(100) {
                            let _ = self.tx().write_all(&[CTRL_C]);
                            last = Instant::now();
                        }
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::TimedOut => {}
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }

    fn read_line(&mut self) -> String {
        let mut line_raw = Vec::new();
        let mut byte = [0; 1];

        while let Ok(n) = self.rx().read(&mut byte) {
            if n == 0 {
                break;
            }

            if byte[0] == b'\r' {
                continue;
            }

            if byte[0] == b'\n' {
                break;
            }

            line_raw.push(byte[0]);
        }

        if line_raw.is_empty() {
            return String::new();
        }

        let line = String::from_utf8_lossy(&line_raw);
        line.trim().to_string()
    }

    pub fn cmd(&mut self, cmd: &str) -> String {
        self.tx().write_all(cmd.as_bytes()).unwrap();
        self.tx().write_all("\r\n".as_bytes()).unwrap();
        self.tx().flush().unwrap();
        let shell_start;
        loop {
            let line = self.read_line();
            println!("{line}");
            if line.contains(cmd) {
                shell_start = line.trim().trim_end_matches(cmd).trim().to_string();
                break;
            }
        }
        let mut reply = Vec::new();
        let mut buff = [0u8; 1];
        loop {
            self.rx().read_exact(&mut buff).unwrap();
            reply.push(buff[0]);

            if reply.ends_with(shell_start.as_bytes()) {
                break;
            }
        }

        String::from_utf8_lossy(&reply)
            .trim()
            .trim_end_matches(&shell_start)
            .to_string()
    }

    pub fn set_env(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.cmd(&format!("setenv {} {}", name.into(), value.into()));
    }

    pub fn env(&mut self, name: impl Into<String>) -> String {
        self.cmd(&format!("echo ${}", name.into()))
    }

    pub fn env_int(&mut self, name: impl Into<String>) -> Option<usize> {
        let line = self.env(name);
        parse_int(&line)
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
