use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use serialport::SerialPort;

pub struct SerialTerm {
    tx: Arc<Mutex<Box<dyn SerialPort>>>,
    rx: Arc<Mutex<Box<dyn SerialPort>>>,
    history: String,
    exit_requested: bool,
}

// 特殊键序列状态
#[derive(Debug, PartialEq)]
enum KeySequenceState {
    Normal,
    CtrlAPressed,
}

impl SerialTerm {
    pub fn new(tx: Box<dyn SerialPort>, rx: Box<dyn SerialPort>, history: &str) -> Self {
        SerialTerm {
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            history: history.to_string(),
            exit_requested: false,
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        // 启用raw模式
        enable_raw_mode()?;
        execute!(io::stdout(), Clear(ClearType::All))?;

        // 设置清理函数
        let cleanup_needed = true;

        let result = self.run_terminal().await;

        // 确保清理终端状态
        if cleanup_needed {
            let _ = disable_raw_mode();
            println!(); // 添加换行符
            eprintln!("\n✓ 已退出串口终端模式");
        }

        result
    }

    async fn run_terminal(&mut self) -> anyhow::Result<()> {
        let tx_port = self.tx.clone();
        let rx_port = self.rx.clone();

        // 创建退出标志
        let exit_flag = Arc::new(Mutex::new(false));
        let exit_flag_rx = exit_flag.clone();

        // 启动串口接收线程
        let rx_handle = thread::spawn(move || {
            Self::handle_serial_receive(rx_port, exit_flag_rx)
        });

        // 主线程处理键盘输入
        let mut key_state = KeySequenceState::Normal;

        loop {
            // 检查退出标志
            if *exit_flag.lock().unwrap() {
                break;
            }

            // 非阻塞读取键盘事件
            if event::poll(Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        // 检测 Ctrl+A+x 退出序列
                        match key_state {
                            KeySequenceState::Normal => {
                                if key.code == KeyCode::Char('a') && key.modifiers.contains(KeyModifiers::CONTROL) {
                                    key_state = KeySequenceState::CtrlAPressed;
                                } else {
                                    // 普通按键，发送到串口
                                    Self::send_key_to_serial(&tx_port, key)?;
                                }
                            }
                            KeySequenceState::CtrlAPressed => {
                                if key.code == KeyCode::Char('x') {
                                    // 用户请求退出
                                    eprintln!("\n检测到退出快捷键 Ctrl+A+x");
                                    *exit_flag.lock().unwrap() = true;
                                    break;
                                } else {
                                    // 不是x键，发送上一个按键并重置状态
                                    if let KeyCode::Char('a') = key.code {
                                        // 如果还是 Ctrl+A，保持状态
                                    } else {
                                        // 发送 Ctrl+A 和当前按键
                                        Self::send_ctrl_a_to_serial(&tx_port)?;
                                        Self::send_key_to_serial(&tx_port, key)?;
                                        key_state = KeySequenceState::Normal;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 等待接收线程结束
        let _ = rx_handle.join();

        Ok(())
    }

    fn handle_serial_receive(
        rx_port: Arc<Mutex<Box<dyn SerialPort>>>,
        exit_flag: Arc<Mutex<bool>>,
    ) -> io::Result<()> {
        let mut buffer = [0u8; 1024];

        loop {
            // 检查退出标志
            if *exit_flag.lock().unwrap() {
                break;
            }

            // 从串口读取数据
            match rx_port.lock().unwrap().read(&mut buffer) {
                Ok(bytes_read) if bytes_read > 0 => {
                    // 将数据直接写入stdout
                    let data = &buffer[..bytes_read];
                    io::stdout().write_all(data)?;
                    io::stdout().flush()?;
                }
                Ok(_) => {
                    // 没有数据可读，短暂休眠
                    thread::sleep(Duration::from_millis(1));
                }
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                    // 超时是正常的，继续
                }
                Err(e) => {
                    eprintln!("\n串口读取错误: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    fn send_key_to_serial(
        tx_port: &Arc<Mutex<Box<dyn SerialPort>>>,
        key: crossterm::event::KeyEvent,
    ) -> io::Result<()> {
        let mut bytes = Vec::new();

        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl 组合键
                    let ctrl_char = match c {
                        'a'..='z' => ((c as u8 - b'a') + 1) as char,
                        'A'..='Z' => ((c as u8 - b'A') + 1) as char,
                        _ => c,
                    };
                    bytes.push(ctrl_char as u8);
                } else {
                    bytes.push(c as u8);
                }
            }
            KeyCode::Enter => bytes.push(b'\r'),
            KeyCode::Backspace => bytes.push(0x7f),
            KeyCode::Tab => bytes.push(b'\t'),
            KeyCode::Esc => bytes.push(0x1b),
            KeyCode::Up => bytes.extend_from_slice(&[0x1b, b'[', b'A']),
            KeyCode::Down => bytes.extend_from_slice(&[0x1b, b'[', b'B']),
            KeyCode::Left => bytes.extend_from_slice(&[0x1b, b'[', b'D']),
            KeyCode::Right => bytes.extend_from_slice(&[0x1b, b'[', b'C']),
            KeyCode::Home => bytes.extend_from_slice(&[0x1b, b'[', b'H']),
            KeyCode::End => bytes.extend_from_slice(&[0x1b, b'[', b'F']),
            KeyCode::PageUp => bytes.extend_from_slice(&[0x1b, b'[', b'5', b'~']),
            KeyCode::PageDown => bytes.extend_from_slice(&[0x1b, b'[', b'6', b'~']),
            KeyCode::Delete => bytes.extend_from_slice(&[0x1b, b'[', b'3', b'~']),
            KeyCode::Insert => bytes.extend_from_slice(&[0x1b, b'[', b'2', b'~']),
            KeyCode::F(1) => bytes.extend_from_slice(&[0x1b, b'O', b'P']),
            KeyCode::F(2) => bytes.extend_from_slice(&[0x1b, b'O', b'Q']),
            KeyCode::F(3) => bytes.extend_from_slice(&[0x1b, b'O', b'R']),
            KeyCode::F(4) => bytes.extend_from_slice(&[0x1b, b'O', b'S']),
            KeyCode::F(5) => bytes.extend_from_slice(&[0x1b, b'[', b'1', b'5', b'~']),
            KeyCode::F(6) => bytes.extend_from_slice(&[0x1b, b'[', b'1', b'7', b'~']),
            KeyCode::F(7) => bytes.extend_from_slice(&[0x1b, b'[', b'1', b'8', b'~']),
            KeyCode::F(8) => bytes.extend_from_slice(&[0x1b, b'[', b'1', b'9', b'~']),
            KeyCode::F(9) => bytes.extend_from_slice(&[0x1b, b'[', b'2', b'0', b'~']),
            KeyCode::F(10) => bytes.extend_from_slice(&[0x1b, b'[', b'2', b'1', b'~']),
            KeyCode::F(11) => bytes.extend_from_slice(&[0x1b, b'[', b'2', b'3', b'~']),
            KeyCode::F(12) => bytes.extend_from_slice(&[0x1b, b'[', b'2', b'4', b'~']),
            _ => {
                // 其他按键忽略
            }
        }

        if !bytes.is_empty() {
            tx_port.lock().unwrap().write_all(&bytes)?;
            tx_port.lock().unwrap().flush()?;
        }

        Ok(())
    }

    fn send_ctrl_a_to_serial(tx_port: &Arc<Mutex<Box<dyn SerialPort>>>) -> io::Result<()> {
        tx_port.lock().unwrap().write_all(&[0x01])?; // Ctrl+A
        tx_port.lock().unwrap().flush()?;
        Ok(())
    }
}