use std::fs::File;
use std::io::Write;
use pcap::{Capture};
use pnet::packet::Packet;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::{IpNextHeaderProtocols};
use std::collections::HashMap;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;

const JPEG_MAGIC_NUMBER: [u8; 4] = [0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic number
const EOI_MARKER: [u8; 2] = [0xFF, 0xD9]; // EOI marker

#[tauri::command]
fn read_pcap_file(file_path: &str) -> Result<(), String> {
    if let Ok(mut cap) = Capture::from_file(file_path) {
        let mut tcp_streams: HashMap<String, Vec<u8>> = HashMap::new();
        while let Ok(packet) = cap.next_packet() {
            // Process each captured packet
            let data= packet.data;
                // Use pnet to decode Ethernet, IP, and TCP layers
                if let Some(eth) = EthernetPacket::new(&data) {
                    if eth.get_ethertype() == EtherTypes::Ipv4 {
                        if let Some(ipv4) = Ipv4Packet::new(eth.payload()) {
                            if ipv4.get_next_level_protocol() == IpNextHeaderProtocols::Tcp {
                                let tcp = TcpPacket::new(ipv4.payload());

                                if ipv4.get_next_level_protocol() == IpNextHeaderProtocols::Tcp {
                                    let tcp = TcpPacket::new(ipv4.payload());
                                    if let Some(tcp) = tcp {
                                        let src_port = tcp.get_source();
                                        let dst_port = tcp.get_destination();
                                        let seq_number = tcp.get_sequence();
                                        let payload = tcp.payload();
        
                                        let key = format!("{}-{}-{}", ipv4.get_source(), ipv4.get_destination(), src_port);
        
                                        if let Some(data) = tcp_streams.get_mut(&key.clone()) {
                                            if seq_number == 0 {
                                                data.clear();
                                            }
        
                                            data.extend_from_slice(payload);
        
                                            if data.ends_with(&[0xFF, 0xD9]) {
                                                save_jpeg_file(&data);
                                                *data = Vec::new(); // Clear data after saving JPEG
                                            }
                                        } else {
                                            let mut data = Vec::new();
                                            data.extend_from_slice(payload);
                                            tcp_streams.insert(key, data);
                                        }
                                    }
                                }
                            }
                        }
                    }
                
            }
        }
    }

    Ok(())
}


fn save_jpeg_file(data: &[u8]) {
    // Save the extracted JPEG file to disk with a unique name
    let file_name = format!("extracted_image_{}.jpg", chrono::Utc::now().timestamp_millis());

    if let Ok(mut file) = File::create(format!("/Users/cichowlasp/Documents/test/{file_name}")) {
        if let Err(e) = file.write_all(data) {
            eprintln!("Error writing JPEG file: {:?}", e);
        } else {
            println!("JPEG file saved as: {}", file_name);
        }
    } else {
        eprintln!("Error creating JPEG file: {}", file_name);
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![read_pcap_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
