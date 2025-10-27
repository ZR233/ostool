use fitimage::fit::FdtHeader;

fn main() {
    let header = FdtHeader::new();
    println!("FDT header size: {} bytes", FdtHeader::size());

    let mut buffer = Vec::new();
    header.write_to_buffer(&mut buffer);
    println!("Written header size: {} bytes", buffer.len());

    // Print first 56 bytes
    println!("First 56 bytes:");
    for (i, byte) in buffer.iter().take(56).enumerate() {
        if i % 16 == 0 {
            print!("\n{:04x}: ", i);
        }
        print!("{:02x} ", byte);
    }
    println!();
}
