use pcap::{Capture, Packet};
use std::fs;


#[tauri::command]
fn read_pcap_file(file_path: &str) -> Result<(), String> {
    let mut cap = Capture::from_file(file_path).unwrap();

    while let Ok(packet) = cap.next_packet() {
        println!("Packet captured with length {}", packet.header.len);

        // Print the packet data
        print_packet_data(packet);
    }

    Ok(())
}

fn print_packet_data(packet: Packet) {
    // Assuming Ethernet frame for simplicity
    let eth_bytes = packet.data;

    // Print the hexadecimal representation of packet data
    for (i, byte) in eth_bytes.iter().enumerate() {
        print!("{:02X} ", byte);
        if (i + 1) % 16 == 0 {
            println!();
        }
    }
    println!();
}


fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![read_pcap_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
