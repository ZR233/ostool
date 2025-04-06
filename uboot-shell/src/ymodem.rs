use std::io::*;

use crate::crc::crc16_ccitt;

const SOH: u8 = 0x01;
const STX: u8 = 0x02;
const EOT: u8 = 0x04;
const ACK: u8 = 0x06;
const NAK: u8 = 0x15;
// const CAN: u8 = 0x18;
const EOF: u8 = 0x1A;
const CRC: u8 = 0x43;

pub struct Ymodem {
    crc_mode: bool,
    blk: u8,
    retries: usize,
}

impl Ymodem {
    pub fn new(crc_mode: bool) -> Self {
        Self {
            crc_mode,
            blk: 0,
            retries: 10,
        }
    }

    fn nak(&self) -> u8 {
        if self.crc_mode { CRC } else { NAK }
    }

    fn getc<D: Read>(&mut self, dev: &mut D) -> Result<u8> {
        let mut buff = [0u8; 1];
        dev.read_exact(&mut buff)?;
        Ok(buff[0])
    }

    fn wait_for_start<D: Read>(&mut self, dev: &mut D) -> Result<()> {
        loop {
            match self.getc(dev)? {
                NAK => {
                    self.crc_mode = false;
                    return Ok(());
                }
                CRC => {
                    self.crc_mode = true;
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    pub fn send<D: Write + Read, F: Read>(
        &mut self,
        dev: &mut D,
        file: &mut F,
        name: &str,
        size: usize,
        on_progress: impl Fn(usize),
    ) -> Result<()> {
        println!("Sending file: {name}");

        self.send_header(dev, name, size)?;

        let mut buff = [0u8; 1024];

        while let Ok(n) = file.read(&mut buff) {
            if n == 0 {
                break;
            }
            self.send_blk(dev, &buff[..n], EOF, false)?;

            on_progress(self.blk as usize * 1024);
        }

        dev.write_all(&[EOT])?;
        dev.flush()?;
        self.wait_ack(dev)?;

        self.send_blk(dev, &[0], 0, true)?;

        self.wait_for_start(dev)?;
        Ok(())
    }

    fn wait_ack<D: Read>(&mut self, dev: &mut D) -> Result<()> {
        let nak = self.nak();
        loop {
            let c = self.getc(dev)?;
            match c {
                ACK => return Ok(()),
                _ => {
                    if c == nak {
                        return Err(Error::new(ErrorKind::BrokenPipe, "NAK"));
                    }
                    stdout().write_all(&[c])?;
                }
            }
        }
    }

    fn send_header<D: Write + Read>(&mut self, dev: &mut D, name: &str, size: usize) -> Result<()> {
        let mut buff = Vec::new();

        buff.append(&mut name.as_bytes().to_vec());

        buff.push(0);

        buff.append(&mut format!("{}", size).as_bytes().to_vec());

        buff.push(0);

        self.send_blk(dev, &buff, 0, false)
    }

    fn send_blk<D: Write + Read>(
        &mut self,
        dev: &mut D,
        data: &[u8],
        pad: u8,
        last: bool,
    ) -> Result<()> {
        let len;
        let p;

        if data.len() > 128 {
            len = 1024;
            p = STX;
        } else {
            len = 128;
            p = SOH;
        }
        let blk = if last { 0 } else { self.blk };
        let mut err = None;
        loop {
            if self.retries == 0 {
                return Err(err.unwrap_or(Error::new(ErrorKind::BrokenPipe, "retry too much")));
            }

            dev.write_all(&[p, blk, !blk])?;

            let mut buf = vec![pad; len];
            buf[..data.len()].copy_from_slice(data);

            dev.write_all(&buf)?;

            if self.crc_mode {
                let chsum = crc16_ccitt(0, &buf);
                let crc1 = (chsum >> 8) as u8;
                let crc2 = (chsum & 0xff) as u8;

                dev.write_all(&[crc1, crc2])?;
            }
            dev.flush()?;

            match self.wait_ack(dev) {
                Ok(_) => break,
                Err(e) => {
                    err = Some(e);
                    self.retries -= 1;
                }
            }
        }

        if self.blk == u8::MAX {
            self.blk = 0;
        } else {
            self.blk += 1;
        }

        Ok(())
    }
}
