use crate::utils;
use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
use std::{
    fs,
    io::Write,
    path::Path,
    time::Instant,
};
use tokio::{
    io::AsyncReadExt,
    net::TcpStream,
};

pub async fn receive_file(
    folder: &Path,
    key: &[u8],
    stream: &mut TcpStream,
    check_duplicate: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let start = Instant::now();

    // Check destination folder
    fs::read_dir(folder)?;

    // Receive file details
    let (filename, file_size) = receive_file_details(stream).await?;
    println!("Receiving: {}", filename);
    println!("File size: {}", utils::make_size_readable(file_size));

    let mut full_path = folder.to_path_buf();
    full_path.push(&filename);

    // For single file, check if we already have it
    if check_duplicate {
        let need_transfer = check_for_file(&full_path, file_size, stream).await?;
        if !need_transfer {
            println!("Already have this file, skipping.");
            return Ok(());
        }
    }

    // Create parent directories if necessary
    utils::make_parent_directories(&full_path)?;

    // Find unique filename if file exists
    let mut i = 1;
    while full_path.is_file() {
        let file_name = full_path.file_name().unwrap().to_str().unwrap();
        let new_name = format!("({}) {}", i, file_name);
        full_path.pop();
        full_path.push(new_name);
        i += 1;
    }

    // Open output file
    let mut out_file = fs::File::create(&full_path)?;

    // Stream receive: start receiving immediately without waiting
    receive_file_streaming(&cipher, stream, &mut out_file, file_size).await?;

    let elapsed = (Instant::now() - start).as_secs_f64();
    println!("Receiving took {}", utils::format_time(elapsed));

    let megabits = 8.0 * (file_size as f64 / 1_000_000.0);
    let mbps = megabits / elapsed;
    println!("Speed: {:.2}mbps", mbps);

    Ok(())
}

async fn receive_file_streaming(
    cipher: &Aes256Gcm,
    stream: &mut TcpStream,
    out_file: &mut fs::File,
    file_size: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut progress = utils::ProgressTracker::new();
    let mut bytes_received = 0u64;

    // Stream receive and decrypt chunks continuously
    loop {
        let chunk_size = stream.read_u64().await? as usize;
        if chunk_size == 0 {
            break;
        }

        let mut chunk = vec![0u8; chunk_size];
        stream.read_exact(&mut chunk).await?;

        // Decrypt
        let nonce = &chunk[..12];
        let ciphertext = &chunk[12..];
        let nonce = aes_gcm::Nonce::from_slice(nonce);
        let decrypted_chunk = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption error: {:?}", e))?;

        bytes_received += decrypted_chunk.len() as u64;
        out_file.write_all(&decrypted_chunk)?;
        progress.update(bytes_received, file_size)?;
    }

    progress.finish()?;

    Ok(())
}

async fn receive_file_details(
    stream: &mut TcpStream,
) -> Result<(String, u64), Box<dyn std::error::Error>> {
    let filename_size = stream.read_u64().await? as usize;
    let mut filename_bytes = vec![0; filename_size];
    stream.read_exact(&mut filename_bytes).await?;
    let filename = String::from_utf8_lossy(&filename_bytes).to_string();
    let file_size = stream.read_u64().await?;
    Ok((filename, file_size))
}

async fn check_for_file(
    filename: &Path,
    size: u64,
    stream: &mut TcpStream,
) -> Result<bool, Box<dyn std::error::Error>> {
    use tokio::io::AsyncWriteExt;

    if filename.is_file() {
        let metadata = fs::metadata(filename)?;
        let local_size = metadata.len();
        if size == local_size {
            stream.write_u64(1).await?;
            let local_hash = utils::hash_file(filename)?;
            let mut peer_hash = vec![0; 32];
            stream.read_exact(&mut peer_hash).await?;
            let hashes_match = local_hash == peer_hash;
            stream.write_u64(if hashes_match { 1 } else { 0 }).await?;
            Ok(!hashes_match)
        } else {
            stream.write_u64(0).await?;
            Ok(true)
        }
    } else {
        stream.write_u64(0).await?;
        Ok(true)
    }
}
