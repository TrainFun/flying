use clap::{Parser, Subcommand};
use flying::{
    ConnectionMode, get_or_prompt_password, print_session_info, run_receiver, run_sender,
};
use std::path::PathBuf;

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
        #[arg(short = 'r', long)]
        recursive: bool,
        #[arg(short = 'P', long)]
        persistent: bool,
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Send {
            file,
            listen,
            connect,
            recursive,
            persistent,
            password,
        } => {
            if !file.exists() {
                eprintln!("Error: File/directory does not exist: {:?}", file);
                std::process::exit(1);
            }

            if file.is_dir() && !recursive {
                eprintln!("Error: Cannot send directory without -r/--recursive flag");
                std::process::exit(1);
            }

            if persistent && !listen {
                eprintln!("Error: --persistent flag requires --listen mode");
                std::process::exit(1);
            }

            let connection_mode = ConnectionMode::from_params(listen, connect);
            let password = get_or_prompt_password(&connection_mode, password);
            print_session_info("SEND", &password, &connection_mode, None);

            if let Err(e) =
                run_sender(&file, &password, connection_mode, recursive, persistent).await
            {
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

            let connection_mode = ConnectionMode::from_params(listen, connect);
            let password = get_or_prompt_password(&connection_mode, password);
            print_session_info("RECEIVE", &password, &connection_mode, Some(&output));

            if let Err(e) = run_receiver(&output, &password, connection_mode).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
