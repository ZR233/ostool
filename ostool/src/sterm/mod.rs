use serialport::SerialPort;

pub struct SerialTerm {
    tx: Box<dyn SerialPort>,
    rx: Box<dyn SerialPort>,
}

impl SerialTerm {
    pub fn new(tx: Box<dyn SerialPort>, rx: Box<dyn SerialPort>) -> Self {
        SerialTerm { tx, rx }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        // Implement serial shell interaction logic here

        Ok(())
    }
}
