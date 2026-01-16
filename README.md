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

### Desktop (Linux, macOS, Windows)

#### Option 1: cargo-binstall

```bash
cargo binstall flying --git https://github.com/wpcRaskolnikov/flying
```

#### Option 2: Download from Releases

Download the latest binary for your platform from [releases page](https://github.com/wpcRaskolnikov/flying/releases).

#### Option 3: Build from Source

```bash
cargo build --release
```

Binary: `target/release/flying`

### Android

Download the Flying Android app from the [releases page](https://github.com/wpcRaskolnikov/flying/releases).

## Quick Start

**Either side must use -l to listen first, the other side will connect**

```bash
# Sender listens (generates password)
# Computer A:
flying send -l myfile.pdf
# Computer B:
flying receive the-generated-password
```

## Command Line Options

### Send Command
```bash
flying send [OPTIONS] <file> [password]
```

Options:
- `-l, --listen` - Listen for connections (generates password)
- `-c, --connect <IP>` - Connect to specific IP
- `-r, --recursive` - Send folders
- `-P, --persistent` - Keep listening after transfer completes (requires -l)

Examples:
```bash
# Listen mode
flying send -l document.pdf

# Send folder
flying send -lr my-project

# Persistent mode (multiple transfers)
flying send -lP video.mp4
```

### Receive Command
```bash
flying receive [OPTIONS] [password]
```

Options:
- `-l, --listen` - Listen for connections
- `-c, --connect <IP>` - Connect to specific IP
- `-o, --output <DIR>` - Output directory (default: current directory)

Examples:
```bash
# Auto-discover sender
flying receive the-password

# Listen mode
flying receive -l

# Custom output directory
flying receive -o ~/Downloads the-password
```

## Contributing

Contributions welcome! Submit issues or pull requests.
