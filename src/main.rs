mod mdns;
mod receive;
mod send;
mod utils;

use clap::{Parser, Subcommand};
use std::{net::SocketAddr, path::PathBuf};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

const VERSION: u64 = 3;

#[derive(Parser, Debug)]
#[command(name = "flying")]
#[command(about = "Simple encrypted file transfer tool with automatic peer discovery", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Send {
        file: PathBuf,
        #[arg(short, long, conflicts_with = "connect")]
        listen: bool,
        #[arg(short, long, value_name = "IP")]
        connect: Option<String>,
        password: Option<String>,
    },

    Receive {
        #[arg(short, long, conflicts_with = "connect")]
        listen: bool,
        #[arg(short, long, value_name = "IP")]
        connect: Option<String>,
        password: Option<String>,
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
}

#[derive(Debug, Clone)]
enum ConnectionMode {
    AutoDiscover,
    Listen,
    Connect(String),
}

fn determine_connection_mode(listen: bool, connect: Option<String>) -> ConnectionMode {
    if let Some(ip) = connect {
        ConnectionMode::Connect(ip)
    } else if listen {
        ConnectionMode::Listen
    } else {
        ConnectionMode::AutoDiscover
    }
}

fn get_or_prompt_password(connection_mode: &ConnectionMode, password: Option<String>) -> String {
    match connection_mode {
        ConnectionMode::Listen => utils::generate_password(),
        ConnectionMode::AutoDiscover | ConnectionMode::Connect(_) => {
            password.unwrap_or_else(|| {
                println!("Please enter password:");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                input.trim().to_string()
            })
        }
    }
}

fn print_session_info(
    mode: &str,
    password: &str,
    connection_mode: &ConnectionMode,
    output_dir: Option<&PathBuf>,
) {
    println!("===========================================");
    println!("Flying - File Transfer Tool");
    println!("===========================================");
    println!("Mode: {}", mode);
    println!("Password: {}", password);
    if let Some(dir) = output_dir {
        println!("Output directory: {:?}", dir);
    }
    match connection_mode {
        ConnectionMode::AutoDiscover => {
            println!("Connection: Auto-discovering peers on local network")
        }
        ConnectionMode::Listen => {
            println!("Connection: Listening for incoming connections")
        }
        ConnectionMode::Connect(ip) => println!("Connection: Will connect to {}", ip),
    }
    println!("===========================================\n");
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Send {
            file,
            listen,
            connect,
            password,
        } => {
            if !file.exists() {
                eprintln!("Error: File does not exist: {:?}", file);
                std::process::exit(1);
            }

            let connection_mode = determine_connection_mode(listen, connect);
            let password = get_or_prompt_password(&connection_mode, password);
            print_session_info("SEND", &password, &connection_mode, None);

            if let Err(e) = run_sender(&file, &password, connection_mode).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }

        Commands::Receive {
            listen,
            connect,
            password,
            output,
        } => {
            if !output.exists() {
                eprintln!("Error: Output directory does not exist: {:?}", output);
                std::process::exit(1);
            }

            let connection_mode = determine_connection_mode(listen, connect);
            let password = get_or_prompt_password(&connection_mode, password);
            print_session_info("RECEIVE", &password, &connection_mode, Some(&output));

            if let Err(e) = run_receiver(&output, &password, connection_mode).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

async fn version_handshake(
    stream: &mut TcpStream,
    send_first: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (local_version, peer_version) = if send_first {
        stream.write_u64(VERSION).await?;
        let peer = stream.read_u64().await?;
        (VERSION, peer)
    } else {
        let peer = stream.read_u64().await?;
        stream.write_u64(VERSION).await?;
        (VERSION, peer)
    };

    if peer_version != local_version {
        println!(
            "Warning: Version mismatch (local: {}, peer: {})",
            local_version, peer_version
        );
    }

    Ok(())
}

async fn establish_connection(
    mode: &ConnectionMode,
    port: u16,
) -> Result<TcpStream, Box<dyn std::error::Error>> {
    match mode {
        ConnectionMode::AutoDiscover => {
            println!("Searching for peers on the local network...\n");

            let services = mdns::discover_services(5)?;

            if let Some(service) = mdns::select_service(&services) {
                let addr = SocketAddr::new(service.ip, service.port);
                println!("\nConnecting to {}...", addr);
                let stream = TcpStream::connect(addr).await?;
                println!("Connected!\n");
                Ok(stream)
            } else {
                Err("No peers found on the local network".into())
            }
        }
        ConnectionMode::Listen => {
            // Use [::] for IPv6 dual-stack (accepts both IPv4 and IPv6)
            let addr = format!("[::]:{}", port).parse::<SocketAddr>()?;
            let _mdns = mdns::advertise_service(port)?;
            let listener = TcpListener::bind(&addr).await?;
            println!("Listening on {} (IPv4/IPv6 dual-stack)...", addr);
            println!("Waiting for peer to connect...\n");
            let (stream, socket_addr) = listener.accept().await?;
            println!("Connection accepted from {}\n", socket_addr);
            Ok(stream)
        }
        ConnectionMode::Connect(ip) => {
            let ip: std::net::IpAddr = ip.parse()?;
            let addr = std::net::SocketAddr::new(ip, port);
            println!("Connecting to {}...", addr);
            let stream = TcpStream::connect(addr).await?;
            println!("Connected!\n");
            Ok(stream)
        }
    }
}

async fn run_sender(
    file_path: &PathBuf,
    password: &str,
    connection_mode: ConnectionMode,
) -> Result<(), Box<dyn std::error::Error>> {
    let key = utils::get_key_from_password(password);

    let mut stream = establish_connection(&connection_mode, 3290).await?;

    version_handshake(&mut stream, false).await?;

    // Mode confirmation (1 = send, 0 = receive)
    let peer_mode = stream.read_u64().await?;
    if peer_mode == 1 {
        stream.write_u64(0).await?;
        return Err("Both ends selected send mode".into());
    } else {
        stream.write_u64(1).await?;
    }

    // Send number of files (always 1 in this simple version)
    stream.write_u64(1).await?;

    // Send the file
    send::send_file(file_path, &key, &mut stream).await?;

    println!("\n===========================================");
    println!("Transfer complete!");
    println!("===========================================");

    stream.shutdown().await?;
    Ok(())
}

async fn run_receiver(
    output_dir: &PathBuf,
    password: &str,
    connection_mode: ConnectionMode,
) -> Result<(), Box<dyn std::error::Error>> {
    let key = utils::get_key_from_password(password);

    let mut stream = establish_connection(&connection_mode, 3290).await?;

    version_handshake(&mut stream, true).await?;

    // Mode confirmation (1 = send, 0 = receive)
    stream.write_u64(0).await?;
    let mode_ok = stream.read_u64().await?;
    if mode_ok != 1 {
        return Err("Both ends selected the same mode".into());
    }

    // Receive number of files
    let num_files = stream.read_u64().await?;
    println!("Receiving {} file(s)...\n", num_files);

    // Receive files
    for i in 0..num_files {
        println!("===========================================");
        println!("File {} of {}", i + 1, num_files);
        println!("===========================================");
        let last_file = i == num_files - 1;
        receive::receive_file(output_dir, &key, &mut stream, last_file).await?;
        println!();
    }

    println!("===========================================");
    println!("Transfer complete!");
    println!("===========================================");

    stream.shutdown().await?;
    Ok(())
}
