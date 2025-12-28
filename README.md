# Flying

A simple, secure, and fast file transfer tool with automatic peer discovery via mDNS.

## Features

- **Encrypted transfers**: All files are encrypted using AES-256-GCM encryption
- **Automatic peer discovery**: Uses mDNS to automatically discover peers on the local network
- **Manual connection options**: Connect directly via IP address or listen for incoming connections
- **Resume capability**: Detects duplicate files via SHA-256 hashing to avoid redundant transfers
- **Progress tracking**: Real-time progress indicators with transfer speed statistics
- **Cross-platform**: Works on Linux, macOS, and Windows
  
## Installation

### Prerequisites

- Rust toolchain (1.70 or newer recommended)
  
### Build from source
```
cargo build --release
```

The binary will be available at `target/release/flying`

## Usage

Flying has two modes: **send** and **receive**.

### Sending a file
```
flying send <FILE> [OPTIONS] [PASSWORD]
```

#### Options:
- `-l, --listen`: Listen for incoming connections (generates a random password)
- `-c, --connect <IP>`: Connect directly to a specific IP address
- No flags: Auto-discover peers on the local network
  
#### Examples:

Auto-discover receiver:
```
flying send document.pdf my-secret-password
```

Listen mode (generates password):
```
flying send document.pdf --listen
```

Connect to specific IP:
```
flying send document.pdf --connect 192.168.1.100 my-secret-password
```

### Receiving a file
```
flying receive [OPTIONS] [PASSWORD]
```

#### Options:
- `-l, --listen`: Listen for incoming connections
- `-c, --connect <IP>`: Connect directly to a specific IP address
- `-o, --output <DIR>`: Output directory (default: current directory)
- No flags: Auto-discover peers on the local network
  
#### Examples:

Auto-discover sender:
```
flying receive my-secret-password
```

Listen mode:
```
flying receive --listen my-secret-password
```

Connect to specific IP:
```
flying receive --connect 192.168.1.100 my-secret-password
```

Specify output directory:
```
flying receive --output ~/Downloads my-secret-password
```

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
