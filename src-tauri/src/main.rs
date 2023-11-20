use std::fs::File;
use std::io::Write;
use pcap::Capture;
use pnet::packet::Packet;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use std::collections::HashMap;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;

const JPEG_MAGIC_NUMBER: [u8; 4] = [0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic number
const PNG_MAGIC_NUMBER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG magic number
const GIF_MAGIC_NUMBER: [u8; 6] = [0x47, 0x49, 0x46, 0x38, 0x37, 0x61]; // GIF magic number
const TXT_MAGIC_NUMBER: [u8; 4] = [0x54, 0x45, 0x58, 0x54]; // "TEXT" in ASCII
const PDF_MAGIC_NUMBER: [u8; 4] = [0x25, 0x50, 0x44, 0x46]; // PDF magic number
const ZIP_MAGIC_NUMBER: [u8; 4] = [0x50, 0x4B, 0x03, 0x04]; // ZIP magic number

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

                                if let Some(tcp) = tcp {
                                    let src_port = tcp.get_source();
                                    let seq_number = tcp.get_sequence();
                                    let payload = tcp.payload();
    
                                    let key = format!("{}-{}-{}", ipv4.get_source(), ipv4.get_destination(), src_port);
    
                                    if let Some(data) = tcp_streams.get_mut(&key.clone()) {
                                        if seq_number == 0 {
                                            data.clear();
                                        }
    
                                        data.extend_from_slice(payload);
    
                                        if is_jpeg(data) {
                                            save_file(&data, "jpg");
                                            *data = Vec::new(); // Clear data after saving JPEG
                                        } else if is_png(data) {
                                            save_file(&data, "png");
                                            *data = Vec::new(); // Clear data after saving PNG
                                        } else if is_gif(data) {
                                            save_file(&data, "gif");
                                            *data = Vec::new(); // Clear data after saving GIF
                                        } else if is_txt(data) {
                                            save_file(&data, "txt");
                                            *data = Vec::new(); // Clear data after saving TXT
                                        } else if is_pdf(data) {
                                            save_file(&data, "pdf");
                                            *data = Vec::new(); // Clear data after saving PDF
                                        } else if is_zip(data) {
                                            save_file(&data, "zip");
                                            *data = Vec::new(); // Clear data after saving ZIP
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

    Ok(())
}

fn is_jpeg(data: &[u8]) -> bool {
    data.starts_with(&JPEG_MAGIC_NUMBER) && data.ends_with(&[0xFF, 0xD9])
}

fn is_png(data: &[u8]) -> bool {
    data.starts_with(&PNG_MAGIC_NUMBER)
}

fn is_gif(data: &[u8]) -> bool {
    data.starts_with(&GIF_MAGIC_NUMBER)
}

fn is_txt(data: &[u8]) -> bool {
    data.starts_with(&TXT_MAGIC_NUMBER)
}

fn is_pdf(data: &[u8]) -> bool {
    data.starts_with(&PDF_MAGIC_NUMBER)
}

fn is_zip(data: &[u8]) -> bool {
    data.starts_with(&ZIP_MAGIC_NUMBER)
}

fn save_file(data: &[u8],file_extension: &str) {
    // Save the extracted JPEG file to disk with a unique name
    let file_name = format!("extracted_image_{}.{}", chrono::Utc::now().timestamp_millis(),file_extension);

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
