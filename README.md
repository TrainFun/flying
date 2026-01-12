# Flying

Fast, secure, encrypted file transfer tool with automatic peer discovery.

## Features

- **🔒 AES-256-GCM encryption** - All transfers are encrypted
- **📁 Folder support** - Send entire directories with -r flag
- **🚀 Streaming transfer** - Optimized for speed, especially with multiple small files
- **🔍 Auto-discovery** - Finds peers automatically via mDNS
- **♻️ Smart duplicate detection** - Skips identical files (single file transfers only)
- **📊 Real-time progress** - Shows transfer speed and progress

## Installation

### Option 1: cargo-binstall

```bash
cargo binstall flying --git https://github.com/wpcRaskolnikov/flying
```

### Option 2: Download from Releases

Download the latest binary for your platform from [releases page](https://github.com/wpcRaskolnikov/flying/releases).

### Option 3: Build from Source

```bash
cargo build --release
```

Binary: `target/release/flying`

## Quick Start

**Either side must use -l to listen first, the other side will connect**

```bash
# Option 1: Sender listens (generates password)
# Computer A:
flying send -l myfile.pdf
# Computer B:
flying receive the-generated-password

# Option 2: Receiver listens
# Computer A:
flying receive -l
# Computer B:
flying send myfile.pdf the-generated-password
```


## Usage

### Send Files
```bash
# Listen mode (generates password)
flying send -l <file>

# Connect to IP
flying send -c <IP> <file> <password>

# Send folder
flying send -lr <folder>
```

### Receive Files
```bash
# Auto-discover sender
flying receive <password>

# Listen mode
flying receive -l <password>

# Connect to IP
flying receive -c <IP> <password>

# Custom output directory
flying receive -o ~/Downloads <password>
```

## Options

### Send
- `-r, --recursive` - Send folders
- `-l, --listen` - Listen for connections (generates password)
- `-c, --connect <IP>` - Connect to specific IP

### Receive
- `-l, --listen` - Listen for connections
- `-c, --connect <IP>` - Connect to specific IP
- `-o, --output <DIR>` - Output directory (default: current directory)

## Examples
```bash
# Transfer a file (A listens, B connects)
A: flying send -l document.pdf
B: flying receive the-password-from-A

# Transfer a folder
A: flying send -lr my-project
B: flying receive -o ~/Projects the-password

# Direct IP connection
A: flying send -l video.mp4
B: flying receive -c 192.168.1.100 the-password
```

## Contributing

Contributions welcome! Submit issues or pull requests.
