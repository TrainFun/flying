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

            let connection_mode = if let Some(ip) = connect {
                ConnectionMode::Connect(ip)
            } else if listen {
                ConnectionMode::Listen
            } else {
                ConnectionMode::AutoDiscover
            };

            let password = match &connection_mode {
                ConnectionMode::Listen => password.unwrap_or_else(|| utils::generate_password()),
                ConnectionMode::AutoDiscover | ConnectionMode::Connect(_) => password
                    .unwrap_or_else(|| {
                        println!("Please enter password:");
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();
                        input.trim().to_string()
                    }),
            };

            println!("===========================================");
            println!("Flying - File Transfer Tool");
            println!("===========================================");
            println!("Mode: SEND");
            println!("Password: {}", password);
            match &connection_mode {
                ConnectionMode::AutoDiscover => {
                    println!("Connection: Auto-discovering peers on local network")
                }
                ConnectionMode::Listen => {
                    println!("Connection: Listening for incoming connections")
                }
                ConnectionMode::Connect(ip) => println!("Connection: Will connect to {}", ip),
            }
            println!("===========================================\n");

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

            let connection_mode = if let Some(ip) = connect {
                ConnectionMode::Connect(ip)
            } else if listen {
                ConnectionMode::Listen
            } else {
                ConnectionMode::AutoDiscover
            };

            let password = match &connection_mode {
                ConnectionMode::Listen => password.unwrap_or_else(|| utils::generate_password()),
                ConnectionMode::AutoDiscover | ConnectionMode::Connect(_) => password
                    .unwrap_or_else(|| {
                        println!("Please enter password:");
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();
                        input.trim().to_string()
                    }),
            };

            println!("===========================================");
            println!("Flying - File Transfer Tool");
            println!("===========================================");
            println!("Mode: RECEIVE");
            println!("Password: {}", password);
            println!("Output directory: {:?}", output);
            match &connection_mode {
                ConnectionMode::AutoDiscover => {
                    println!("Connection: Auto-discovering peers on local network")
                }
                ConnectionMode::Listen => {
                    println!("Connection: Listening for incoming connections")
                }
                ConnectionMode::Connect(ip) => println!("Connection: Will connect to {}", ip),
            }
            println!("===========================================\n");

            if let Err(e) = run_receiver(&output, &password, connection_mode).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
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
                let addr = format!("{}:{}", service.ip, service.port).parse::<SocketAddr>()?;
                println!("\nConnecting to {}...", addr);
                let stream = TcpStream::connect(addr).await?;
                println!("Connected!\n");
                Ok(stream)
            } else {
                // No peers found, fall back to listen mode
                println!("\nNo peers found. Falling back to listen mode...");
                let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>()?;
                let _mdns = mdns::advertise_service(port)?;
                let listener = TcpListener::bind(&addr).await?;
                println!("Listening on {}...", addr);
                println!("Waiting for peer to connect...\n");
                let (stream, socket_addr) = listener.accept().await?;
                println!("Connection accepted from {}\n", socket_addr);
                Ok(stream)
            }
        }
        ConnectionMode::Listen => {
            let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>()?;
            let _mdns = mdns::advertise_service(port)?;
            let listener = TcpListener::bind(&addr).await?;
            println!("Listening on {}...", addr);
            println!("Waiting for peer to connect...\n");
            let (stream, socket_addr) = listener.accept().await?;
            println!("Connection accepted from {}\n", socket_addr);
            Ok(stream)
        }
        ConnectionMode::Connect(ip) => {
            let addr = format!("{}:{}", ip, port).parse::<SocketAddr>()?;
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

    // Establish connection based on mode
    let mut stream = establish_connection(&connection_mode, 3290).await?;

    // Version exchange
    let peer_version = stream.read_u64().await?;
    stream.write_u64(VERSION).await?;

    if peer_version != VERSION {
        println!(
            "Warning: Version mismatch (local: {}, peer: {})",
            VERSION, peer_version
        );
    }

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

    // Establish connection based on mode
    let mut stream = establish_connection(&connection_mode, 3290).await?;

    // Version exchange
    stream.write_u64(VERSION).await?;
    let peer_version = stream.read_u64().await?;

    if peer_version != VERSION {
        println!(
            "Warning: Version mismatch (local: {}, peer: {})",
            VERSION, peer_version
        );
    }

    // Mode confirmation (1 = send, 0 = receive)
    stream.write_u64(0).await?; // We are receiver
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
