#[cfg(target_os = "linux")]
use fork::{daemon, Fork};
use pcap::Capture;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::Packet;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
#[cfg(target_os = "linux")]
use std::{fs::metadata, path::PathBuf};
use zip::write::FileOptions;
use zip::ZipWriter; // dep: fork = "0.1"

const JPEG_MAGIC_NUMBER: [u8; 4] = [0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic number
const PNG_MAGIC_NUMBER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG magic number
const GIF_MAGIC_NUMBER: [u8; 6] = [0x47, 0x49, 0x46, 0x38, 0x37, 0x61]; // GIF magic number
const TXT_MAGIC_NUMBER: [u8; 4] = [0x54, 0x45, 0x58, 0x54]; // "TEXT" in ASCII
const PDF_MAGIC_NUMBER: [u8; 4] = [0x25, 0x50, 0x44, 0x46]; // PDF magic number
const ZIP_MAGIC_NUMBER: [u8; 4] = [0x50, 0x4B, 0x03, 0x04]; // ZIP magic number

#[tauri::command]
fn read_pcap_file(file_path: &str) -> Vec<String> {
    let mut file_paths: Vec<String> = Vec::new();
    if let Ok(mut cap) = Capture::from_file(file_path) {
        let mut tcp_streams: HashMap<String, Vec<u8>> = HashMap::new();
        let temp_dir = std::env::temp_dir();
        while let Ok(packet) = cap.next_packet() {
            // Process each captured packet
            let data = packet.data;
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

                                let key = format!(
                                    "{}-{}-{}",
                                    ipv4.get_source(),
                                    ipv4.get_destination(),
                                    src_port
                                );

                                if let Some(data) = tcp_streams.get_mut(&key.clone()) {
                                    if seq_number == 0 {
                                        data.clear();
                                    }

                                    data.extend_from_slice(payload);

                                    if is_jpeg(data) {
                                        if let Some(file_path) = save_file(&data, "jpg", &temp_dir)
                                        {
                                            file_paths.push(file_path); // Add the file path to the vector
                                        }
                                        *data = Vec::new(); // Clear data after saving JPEG
                                    } else if is_png(data) {
                                        if let Some(file_path) = save_file(&data, "jpg", &temp_dir)
                                        {
                                            file_paths.push(file_path); // Add the file path to the vector
                                        }
                                        *data = Vec::new(); // Clear data after saving PNG
                                    } else if is_gif(data) {
                                        if let Some(file_path) = save_file(&data, "jpg", &temp_dir)
                                        {
                                            file_paths.push(file_path); // Add the file path to the vector
                                        }
                                        *data = Vec::new(); // Clear data after saving GIF
                                    } else if is_txt(data) {
                                        if let Some(file_path) = save_file(&data, "jpg", &temp_dir)
                                        {
                                            file_paths.push(file_path); // Add the file path to the vector
                                        }
                                        *data = Vec::new(); // Clear data after saving TXT
                                    } else if is_pdf(data) {
                                        if let Some(file_path) = save_file(&data, "jpg", &temp_dir)
                                        {
                                            file_paths.push(file_path); // Add the file path to the vector
                                        }
                                        *data = Vec::new(); // Clear data after saving PDF
                                    } else if is_zip(data) {
                                        if let Some(file_path) = save_file(&data, "jpg", &temp_dir)
                                        {
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

fn save_file(data: &[u8], file_extension: &str, temp_dir: &Path) -> Option<String> {
    // Save the extracted JPEG file to disk with a unique name
    let file_name = format!(
        "extracted_image_{}.{}",
        chrono::Utc::now().timestamp_millis(),
        file_extension
    );
    let path = temp_dir.join(file_name);
    let display_path = path.clone();
    if let Ok(mut file) = File::create(path) {
        if let Err(e) = file.write_all(data) {
            eprintln!("Error writing JPEG file: {:?}", e);
            return None;
        } else {
            println!("JPEG file saved in: {}", display_path.display());
            return Some(format!("{}", display_path.display()));
        }
    } else {
        eprintln!("Error creating JPEG files");
        return None;
    }
}

fn hash_files(file_paths: &[String]) -> String {
    let mut hash_info = String::new();
    hash_info.push_str("Extracted Files Hashes\n");

    for file_path in file_paths {
        let metadata = match fs::metadata(&file_path) {
            Ok(metadata) => metadata,
            Err(_) => return format!("Failed to get metadata for: {}", &file_path),
        };

        if metadata.is_file() {
            let file_name = match std::path::Path::new(&file_path).file_name() {
                Some(name) => name.to_string_lossy().into_owned(),
                None => return format!("Failed to extract file name: {}", &file_path),
            };
            let file_content = fs::read(&file_path).unwrap();
            let file_hash = format!("{:x}", Sha256::digest(&file_content)); // Calculate hash using SHA-256
            hash_info.push_str(&format!(
                "File: {} - Hash (SHA256): {}\n",
                file_name, file_hash
            ));
        }
    }
    hash_info
}

fn extract_urls_from_pcap(file_paths: &[String]) -> String {
    let mut urls = String::new();
    let url_regex = Regex::new(r#"(https?|ftp)://[^\s/$.?#].[^\s]*"#).unwrap(); // Regular expression for matching URLs
    urls.push_str("\nWebistes Found in PCAP Files\n");
    for file_path in file_paths {
        let file_content = match fs::read(&file_path) {
            Ok(content) => content,
            Err(_) => continue, // Skip file if unable to read
        };

        // Assuming the file content is packet data in this example
        let text = String::from_utf8_lossy(&file_content);

        // Extract URLs using the regex pattern
        for capture in url_regex.captures_iter(&text) {
            urls.push_str(&capture[0]);
            urls.push('\n');
        }
    }
    urls
}

// Function to zip files from an array of paths and save the resulting zip file to a directory
#[tauri::command]
fn zip_and_save_to_directory(
    file_paths: Vec<String>,
    output_directory: String,
    zip_file_name: String,
    pcap_paths: Vec<String>,
    name: String,
    surname: String,
    time_start: String,
    time_end: String,
) -> Result<String, String> {
    let output_zip_path = std::path::Path::new(&output_directory).join(&zip_file_name);
    let output_file = match File::create(&output_zip_path) {
        Ok(file) => file,
        Err(_) => return Err("Failed to create output zip file".to_string()),
    };
    let mut zip = ZipWriter::new(output_file);
    let user_info = format!(
        "Export informations:\nUser: {} {}\nTime Start:{}\nTime End:{}\n\n",
        name, surname, time_start, time_end
    );
    let hash_info = hash_files(&file_paths);
    let uri_info = extract_urls_from_pcap(&pcap_paths);

    // Creating a TXT file with hash info and website list
    let mut info_file = File::create("info.txt").unwrap();
    info_file.write_all(user_info.as_bytes()).unwrap();
    info_file.write_all(hash_info.as_bytes()).unwrap();
    info_file.write_all(uri_info.as_bytes()).unwrap();

    // Adding the info TXT file to the zip archive
    let info_file_content = fs::read("info.txt").unwrap();
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file("info.txt", options).unwrap();
    zip.write_all(&info_file_content).unwrap();
    fs::remove_file("info.txt").unwrap();

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

            let file_name = match std::path::Path::new(&file_path).file_name() {
                Some(name) => name.to_string_lossy().into_owned(),
                None => return Err(format!("Failed to extract file name: {}", &file_path)),
            };

            let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

            if let Err(_) = zip.start_file(&file_name, options) {
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

#[tauri::command]
fn show_in_folder(path: String) {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .args(["/select,", &path]) // The comma after select is not a typo
            .spawn()
            .unwrap();
    }

    #[cfg(target_os = "linux")]
    {
        if path.contains(",") {
            // see https://gitlab.freedesktop.org/dbus/dbus/-/issues/76
            let new_path = match metadata(&path).unwrap().is_dir() {
                true => path,
                false => {
                    let mut path2 = PathBuf::from(path);
                    path2.pop();
                    path2.into_os_string().into_string().unwrap()
                }
            };
            Command::new("xdg-open").arg(&new_path).spawn().unwrap();
        } else {
            if let Ok(Fork::Child) = daemon(false, false) {
                Command::new("dbus-send")
                    .args([
                        "--session",
                        "--dest=org.freedesktop.FileManager1",
                        "--type=method_call",
                        "/org/freedesktop/FileManager1",
                        "org.freedesktop.FileManager1.ShowItems",
                        format!("array:string:\"file://{path}\"").as_str(),
                        "string:\"\"",
                    ])
                    .spawn()
                    .unwrap();
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").args(["-R", &path]).spawn().unwrap();
    }
}
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            read_pcap_file,
            zip_and_save_to_directory,
            show_in_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
