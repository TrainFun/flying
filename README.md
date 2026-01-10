# Flying

Fast, secure, encrypted file transfer tool with automatic peer discovery.

## Features

- 🔒 AES-256-GCM encryption - All transfers are encrypted
- 📁 Folder support - Send entire directories with -r flag
- 🚀 Streaming transfer - Optimized for speed, especially with multiple small files
- 🔍 Auto-discovery - Finds peers automatically via mDNS
- ♻️ Smart duplicate detection - Skips identical files (single file transfers only)
- 📊 Real-time progress - Shows transfer speed and progress

## Installation
```bash
cargo build --release
```

Binary: target/release/flying

## Quick Start

*One side must use -l to listen first*

### Computer A: Start listening (generates password)
`flying send -l myfile.pdf`

### Computer B: Connect with password
`flying receive the-generated-password`

# Usage

## Send Files
```
# Listen mode (generates password)
flying send -l <file>

# Connect to IP
flying send -c <IP> <file> <password>

# Send folder
flying send -lr <folder>
```

## Receive Files
```
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
- -r, --recursive - Send folders
- -l, --listen - Listen for connections (generates password)
- -c, --connect <IP> - Connect to specific IP
___
### Receive
- -l, --listen - Listen for connections
- -c, --connect <IP> - Connect to specific IP
- -o, --output <DIR> - Output directory (default: current directory)
___
## Examples
```
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
