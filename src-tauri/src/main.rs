use pcap::{Capture, Packet};
use std::fs;
use std::io::Write;

const JPEG_MAGIC_NUMBER: [u8; 2] = [0xFF, 0xD8];

#[tauri::command]
fn read_pcap_file(file_path: &str) -> Result<(), String> {
    let mut cap = Capture::from_file(file_path).unwrap();

    // HashMap to store TCP streams
    let mut tcp_streams: std::collections::HashMap<(u32, u32, u16, u16), Vec<u8>> = std::collections::HashMap::new();

    let mut file = 0;

    while let Ok(packet) = cap.next_packet() {

        let ethertype = u16::from_be_bytes([packet.data[12], packet.data[13]]);
        if ethertype != 0x0800 {
            continue; // Skip non-IPv4 packets
        }

        let ip_header_length = (packet.data[14] & 0x0F) as usize * 4;
        let protocol = packet.data[23];
        if protocol != 6 {
            continue; // Skip non-TCP packets
        }

        let src_ip = u32::from_be_bytes([
            packet.data[26],
            packet.data[27],
            packet.data[28],
            packet.data[29],
        ]);
        let dst_ip = u32::from_be_bytes([
            packet.data[30],
            packet.data[31],
            packet.data[32],
            packet.data[33],
        ]);

        let src_port = u16::from_be_bytes([packet.data[ip_header_length + 0], packet.data[ip_header_length + 1]]);
        let dst_port = u16::from_be_bytes([packet.data[ip_header_length + 2], packet.data[ip_header_length + 3]]);
        
        // Identifying TCP streams by 4-tuple (source IP, destination IP, source port, destination port)
        let key = (src_ip, dst_ip, src_port, dst_port);

        // Reassemble TCP streams
        let tcp_data = &packet.data[ip_header_length + 20..]; // Assuming no options in TCP header
        if let Some(data) = tcp_streams.get_mut(&key) {
            // Assuming data is raw file content
            data.extend_from_slice(tcp_data);

            // Here, you can implement logic to detect file boundaries or specific file formats
            // For example, searching for file headers, footers, or specific signatures

            // Detect JPEG file boundary
            while let Some(jpeg_start) = find_jpeg_boundary(data) {
                let jpeg_end = find_jpeg_boundary(&data[jpeg_start + 1..]).map(|pos| jpeg_start + pos + 1).unwrap_or(data.len());

                // Save the JPEG file with a unique name
                save_jpeg_file(&data[jpeg_start..jpeg_end], file);
                file += 1;

                // Remove the extracted JPEG data
                data.drain(..jpeg_end);
            }
        } else {
            // Create a new entry for this TCP stream
            tcp_streams.insert(key, tcp_data.to_vec());
        }
    

        // Print the packet data
        //print_packet_data(packet);

    }

    Ok(())
}

fn find_jpeg_boundary(data: &[u8]) -> Option<usize> {
    // Find the JPEG magic number in the data stream
    data.windows(2).position(|window| window == JPEG_MAGIC_NUMBER)
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

fn save_jpeg_file(data: &[u8], counter: usize) {
    // Save the extracted JPEG file to disk
    if let Ok(mut file) = fs::File::create(format!("/Users/cichowlasp/Documents/test2/extracted_image{counter}.jpg")) {
        if let Err(e) = file.write_all(data) {
            eprintln!("Error writing JPEG file: {:?}", e);
        } else {
            println!("JPEG file extracted and saved!");
        }
    } else {
        eprintln!("Error creating JPEG file");
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![read_pcap_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
