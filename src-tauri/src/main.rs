#[cfg(target_os = "linux")]
use fork::{daemon, Fork};
use pcap::Capture;
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::Packet;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::from_utf8;
use std::{env, fs};
#[cfg(target_os = "linux")]
use std::{fs::metadata, path::PathBuf};
use tokio::io::{duplex, AsyncWriteExt};
use tokio::runtime::Runtime;
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct TcpFlow {
    src: (std::net::IpAddr, u16),
    dst: (std::net::IpAddr, u16),
}

#[tauri::command]
fn read_pcap_file(pcap_path: String) -> Result<Vec<String>, String> {
    let rt = Runtime::new().unwrap();
    rt.block_on(analyze_pcap(&pcap_path))
}

async fn analyze_pcap(file_path: &str) -> Result<Vec<String>, String> {
    let mut capture = Capture::from_file(file_path).map_err(|e| e.to_string())?;
    let mut file_data = HashMap::new();
    let mut file_names = HashSet::new();
    let mut tcp_streams: HashMap<(Ipv4Addr, Ipv4Addr, u16, u16), BTreeMap<u32, Vec<u8>>> =
        HashMap::new();

    while let Ok(packet) = capture.next_packet() {
        if let Some(ethernet_packet) = EthernetPacket::new(packet.data) {
            if let Some(ip_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                if ip_packet.get_next_level_protocol() == IpNextHeaderProtocols::Tcp {
                    if let Some(tcp_packet) = TcpPacket::new(ip_packet.payload()) {
                        assemble_tcp_streams(
                            &mut tcp_streams,
                            ip_packet.get_source(),
                            ip_packet.get_destination(),
                            tcp_packet.get_source(),
                            tcp_packet.get_destination(),
                            tcp_packet.get_sequence(),
                            tcp_packet.payload().to_vec(),
                        );
                    }
                }
            }
        }
    }

    for ((_src_ip, _dst_ip, _src_port, _dst_port), stream) in tcp_streams {
        let mut full_payload = Vec::new();
        for (_, fragment) in stream {
            full_payload.extend_from_slice(&fragment);
        }

        if let Ok(payload_str) = from_utf8(&full_payload) {
            if payload_str.contains("HTTP/1.1") || payload_str.contains("HTTP/1.0") {
                for line in payload_str.lines() {
                    if line.starts_with("GET ")
                        || line.starts_with("POST ")
                        || line.starts_with("PUT ")
                        || line.starts_with("DELETE ")
                        || line.starts_with("HEAD ")
                    {
                        if let Some(file_path) = line.split_whitespace().nth(1) {
                            if is_valid_file(file_path) {
                                file_names.insert(file_path.to_string());
                                file_data.insert(file_path.to_string(), full_payload.clone());
                            }
                        }
                    }
                }
            } else if full_payload.starts_with(b"PRI * HTTP/2.0") {
                // Handle HTTP/2
                let (_client, mut server) = duplex(1024);
                server.write_all(&full_payload).await.unwrap();
                if let Ok(mut connection) = h2::server::handshake(server).await {
                    while let Some(result) = connection.accept().await {
                        match result {
                            Ok((request, mut respond)) => {
                                if let Some(path) = request.uri().path_and_query() {
                                    if is_valid_file(path.as_str()) {
                                        file_names.insert(path.to_string());
                                        file_data.insert(path.to_string(), full_payload.clone());
                                    }
                                }
                                let response = http::Response::new(());
                                respond.send_response(response, true).unwrap();
                            }
                            Err(e) => eprintln!("Error processing HTTP/2: {}", e),
                        }
                    }
                }
            }
        }
    }

    let export_dir = export_files(&file_data, Path::new(file_path))?;
    println!("Files exported to: {:?}", export_dir);

    Ok(file_names
        .into_iter()
        .map(|name| {
            let mut path = export_dir.clone();
            path.push(Path::new(&name).file_name().unwrap());
            path.into_os_string().into_string().unwrap()
        })
        .collect::<Vec<String>>())
}

fn assemble_tcp_streams(
    tcp_streams: &mut HashMap<(Ipv4Addr, Ipv4Addr, u16, u16), BTreeMap<u32, Vec<u8>>>,
    src_ip: Ipv4Addr,
    dst_ip: Ipv4Addr,
    src_port: u16,
    dst_port: u16,
    sequence: u32,
    payload: Vec<u8>,
) {
    let key = (src_ip, dst_ip, src_port, dst_port);
    let stream = tcp_streams.entry(key).or_insert_with(BTreeMap::new);
    stream.insert(sequence, payload);
}

fn is_valid_file(file_path: &str) -> bool {
    let valid_extensions = vec![
        ".html", ".htm", ".php", ".asp", ".aspx", ".jsp", ".js", ".css", ".json", ".xml", ".png",
        ".jpg", ".jpeg", ".gif", ".bmp", ".svg", ".zip", ".rar", ".7z", ".tar", ".gz", ".bz2",
        ".exe", ".dll", ".bin", ".sh", ".bat", ".cmd", ".pdf", ".doc", ".docx", ".xls", ".xlsx",
        ".ppt", ".pptx", ".txt", ".log", ".csv", ".mp3", ".wav", ".mp4", ".avi", ".mkv", ".mov",
    ];

    valid_extensions.iter().any(|ext| file_path.ends_with(ext))
}

fn export_files(files: &HashMap<String, Vec<u8>>, pcap_path: &Path) -> Result<PathBuf, String> {
    let export_dir = pcap_path.with_file_name("extracted_files");
    fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;

    for (file_path, data) in files {
        let file_name = Path::new(file_path)
            .file_name()
            .ok_or("Invalid file path")?;
        let full_path = export_dir.join(file_name);

        let mut file = File::create(&full_path).map_err(|e| e.to_string())?;
        file.write_all(data).map_err(|e| e.to_string())?;
    }

    Ok(export_dir)
}

#[tauri::command]
fn find_urls(pcap_paths: Vec<String>) -> String {
    let urls = extract_urls_from_pcap(&pcap_paths);
    return urls;
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
    let mut seen_urls: HashSet<String> = HashSet::new(); // To store unique simplified URLs
    urls.push_str("\nWebsites Found in PCAP Files\n");

    for file_path in file_paths {
        let file_content = match fs::read(&file_path) {
            Ok(content) => content,
            Err(_) => continue, // Skip file if unable to read
        };

        // Assuming the file content is packet data in this example
        let text = String::from_utf8_lossy(&file_content);

        // Extract URLs using the regex pattern
        for capture in url_regex.captures_iter(&text) {
            let full_url = &capture[0];
            if let Ok(parsed_url) = url::Url::parse(full_url) {
                let simplified_url = format!(
                    "{}://{}/",
                    parsed_url.scheme(),
                    parsed_url.host_str().unwrap_or("")
                );
                if seen_urls.insert(simplified_url.clone()) {
                    // Add only unique URLs
                    urls.push_str(&simplified_url);
                    urls.push('\n');
                }
            }
        }
    }

    urls
}

#[tauri::command]
fn delete_folder(pcap_path: &Path) -> Result<String, String> {
    if let Some(parent_dir) = pcap_path.parent() {
        let folder_path = parent_dir.join("extracted_files");

        if folder_path.exists() {
            if folder_path.is_dir() {
                match fs::remove_dir_all(&folder_path) {
                    Ok(_) => Ok(format!(
                        "Folder '{}' został usunięty.",
                        folder_path.display()
                    )),
                    Err(e) => Err(format!("Błąd podczas usuwania folderu: {}", e)),
                }
            } else {
                Err(format!(
                    "Ścieżka '{}' nie jest folderem.",
                    folder_path.display()
                ))
            }
        } else {
            Err(format!("Folder '{}' nie istnieje.", folder_path.display()))
        }
    } else {
        Err(format!(
            "Nie udało się znaleźć katalogu nadrzędnego dla '{}'.",
            pcap_path.display()
        ))
    }
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
    let ip_info = extract_ip_addresses_and_ports(&pcap_paths);

    // Creating a TXT file with hash info and website list
    let mut info_file = File::create("info.txt").unwrap();
    info_file.write_all(user_info.as_bytes()).unwrap();
    info_file.write_all(hash_info.as_bytes()).unwrap();
    info_file.write_all(uri_info.as_bytes()).unwrap();
    info_file.write_all(ip_info.as_bytes()).unwrap();

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

fn extract_ip_addresses_and_ports(file_paths: &[String]) -> String {
    let mut ip_port_set: HashSet<String> = HashSet::new();
    let mut ip_port = String::new();

    for file_path in file_paths {
        let mut pcap = match ::pcap::Capture::from_file(file_path) {
            Ok(p) => p,
            Err(_) => continue, // Skip file if unable to read
        };

        while let Ok(packet) = pcap.next_packet() {
            let data = packet.data;
            let eth_packet = pnet::packet::ethernet::EthernetPacket::new(data).unwrap();
            if eth_packet.get_ethertype() == pnet::packet::ethernet::EtherTypes::Ipv4 {
                if let Some(ipv4) =
                    pnet::packet::ipv4::Ipv4Packet::new(pnet::packet::Packet::payload(&eth_packet))
                {
                    let src_ip = format!("{}", ipv4.get_source());
                    let dst_ip = format!("{}", ipv4.get_destination());

                    match ipv4.get_next_level_protocol() {
                        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
                            if let Some(tcp) = pnet::packet::tcp::TcpPacket::new(
                                pnet::packet::Packet::payload(&ipv4),
                            ) {
                                let src_port = tcp.get_source();
                                let dst_port = tcp.get_destination();

                                ip_port_set.insert(format!("src: {}:{}", src_ip, src_port));
                                ip_port_set.insert(format!("dst: {}:{}", dst_ip, dst_port));
                            }
                        }
                        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
                            if let Some(udp) = pnet::packet::udp::UdpPacket::new(
                                pnet::packet::Packet::payload(&ipv4),
                            ) {
                                let src_port = udp.get_source();
                                let dst_port = udp.get_destination();

                                ip_port_set.insert(format!("src: {}:{}", src_ip, src_port));
                                ip_port_set.insert(format!("dst: {}:{}", dst_ip, dst_port));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    ip_port.push_str("\nIP adresses with ports:\n");
    for ip in ip_port_set {
        ip_port.push_str(&ip);
        ip_port.push('\n');
    }

    ip_port
}
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            read_pcap_file,
            zip_and_save_to_directory,
            show_in_folder,
            find_urls,
            delete_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
