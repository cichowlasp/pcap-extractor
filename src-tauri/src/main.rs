use std::fs::File;
use std::io::Write;
use std::path::Path;
use pcap::Capture;
use pnet::packet::Packet;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use std::collections::HashMap;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use zip::write::FileOptions;
use zip::ZipWriter;
use std::fs;


const JPEG_MAGIC_NUMBER: [u8; 4] = [0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic number
const PNG_MAGIC_NUMBER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG magic number
const GIF_MAGIC_NUMBER: [u8; 6] = [0x47, 0x49, 0x46, 0x38, 0x37, 0x61]; // GIF magic number
const TXT_MAGIC_NUMBER: [u8; 4] = [0x54, 0x45, 0x58, 0x54]; // "TEXT" in ASCII
const PDF_MAGIC_NUMBER: [u8; 4] = [0x25, 0x50, 0x44, 0x46]; // PDF magic number
const ZIP_MAGIC_NUMBER: [u8; 4] = [0x50, 0x4B, 0x03, 0x04]; // ZIP magic number

#[tauri::command]
fn read_pcap_file(file_path: &str) ->  Vec<String> {
    let mut file_paths: Vec<String> = Vec::new();
    if let Ok(mut cap) = Capture::from_file(file_path) {
        let mut tcp_streams: HashMap<String, Vec<u8>> = HashMap::new();
        let temp_dir = std::env::temp_dir();
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
                                            if let Some(file_path) = save_file(&data, "jpg", &temp_dir) {
                                                file_paths.push(file_path); // Add the file path to the vector
                                            }
                                            *data = Vec::new(); // Clear data after saving JPEG
                                        } else if is_png(data) {
                                            if let Some(file_path) = save_file(&data, "jpg", &temp_dir) {
                                                file_paths.push(file_path); // Add the file path to the vector
                                            }
                                            *data = Vec::new(); // Clear data after saving PNG
                                        } else if is_gif(data) {
                                            if let Some(file_path) = save_file(&data, "jpg", &temp_dir) {
                                                file_paths.push(file_path); // Add the file path to the vector
                                            }
                                            *data = Vec::new(); // Clear data after saving GIF
                                        } else if is_txt(data) {
                                            if let Some(file_path) = save_file(&data, "jpg", &temp_dir) {
                                                file_paths.push(file_path); // Add the file path to the vector
                                            }
                                            *data = Vec::new(); // Clear data after saving TXT
                                        } else if is_pdf(data) {
                                            if let Some(file_path) = save_file(&data, "jpg", &temp_dir) {
                                                file_paths.push(file_path); // Add the file path to the vector
                                            }
                                            *data = Vec::new(); // Clear data after saving PDF
                                        } else if is_zip(data) {
                                            if let Some(file_path) = save_file(&data, "jpg", &temp_dir) {
                                                file_paths.push(file_path); // Add the file path to the vector
                                            }
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
    return file_paths;
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

fn save_file(data: &[u8],file_extension: &str, temp_dir: &Path) -> Option<String> {
    // Save the extracted JPEG file to disk with a unique name
    let file_name = format!("extracted_image_{}.{}", chrono::Utc::now().timestamp_millis(),file_extension);
    let path = temp_dir.join(file_name);
    let display_path = path.clone();
    if let Ok(mut file) = File::create(path) {
        if let Err(e) = file.write_all(data) {
            eprintln!("Error writing JPEG file: {:?}", e);
            return None;
        } else {
            println!("JPEG file saved in: {}", display_path.display());
            return Some(format!("{}",display_path.display()));
        }
    } else {
        eprintln!("Error creating JPEG files");
        return None;
    }
}

// Function to zip files from an array of paths and save the resulting zip file to a directory
#[tauri::command]
fn zip_and_save_to_directory(file_paths: Vec<String>, output_directory: String, zip_file_name: String) -> Result<String, String> {
    let output_zip_path = std::path::Path::new(&output_directory).join(&zip_file_name);
    let output_file = match File::create(&output_zip_path) {
        Ok(file) => file,
        Err(_) => return Err("Failed to create output zip file".to_string()),
    };
    let mut zip = ZipWriter::new(output_file);

    for file_path in file_paths {
        let metadata = match fs::metadata(&file_path) {
            Ok(metadata) => metadata,
            Err(_) => return Err(format!("Failed to get metadata for: {}", &file_path)),
        };

        if metadata.is_file() {
            let file_contents = match std::fs::read(&file_path) {
                Ok(contents) => contents,
                Err(_) => return Err(format!("Failed to read file: {}", &file_path)),
            };

            let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

            if let Err(_) = zip.start_file(&file_path, options) {
                return Err(format!("Failed to add file: {}", &file_path));
            }

            if let Err(_) = zip.write_all(&file_contents) {
                return Err(format!("Failed to write file content: {}", &file_path));
            }
        }
    }

    if let Err(_) = zip.finish() {
        return Err("Failed to finalize the zip file".to_string());
    }

    Ok(format!("Zip file created at: {:?}", &output_zip_path))
}
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![read_pcap_file, zip_and_save_to_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
