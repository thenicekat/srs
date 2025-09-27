# Secure Rust Storage (SRS)

A secure command-line tool for storing and managing personal access tokens using AES-256-GCM encryption. Built with Rust for maximum security and performance.

## 🔐 Features

- **AES-256-GCM Encryption**: Military-grade encryption for your sensitive data.
- **Master Key Protection**: Your master key is never stored, only used to derive encryption keys.
- **Environment Integration**: Populate environment variables with stored tokens
- **Cross-Platform**: Works on macOS and Linux. (NOTE: Windows to be tested.)
- **Zero Dependencies**: No external services or cloud dependencies. All your data stays on your computer.

## 🚀 Installation

### Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/thenicekat/srs/main/install.sh | bash
```

The install script will:

- Automatically detect your system architecture.
- Download the latest release from GitHub.
- Install the binary and add it to your PATH.

### Manual Installation

#### Prerequisites

- Rust 1.70+ installed on your system
- [Install Rust](https://rustup.rs/) if you haven't already

#### Installing the Binary from cargo

```bash
cargo install --git https://github.com/thenicekat/srs
```

## 📖 Usage

### Command Reference

| Command                | Description                                | Example                                                    |
| ---------------------- | ------------------------------------------ | ---------------------------------------------------------- |
| `add <name> [<value>]` | Store a new token                          | `srs add github_token token_value`, `srs add github_token` |
| `get <name>`           | Retrieve a token                           | `srs get github_token`                                     |
| `list`                 | List all token names                       | `srs list`                                                 |
| `delete <name>`        | Delete a token                             | `srs delete github_token`                                  |
| `shell`                | Creates a new shell with the env populated | `srs shell`                                                |

## 🔒 Security Features

### Encryption Details

- **Algorithm**: AES-256-GCM (Galois/Counter Mode)
- **Key Derivation**: SHA-256 hash of your master key
- **Nonce**: Random 12-byte nonce for each encryption
- **Encoding**: Base64 for safe storage

### Master Key Security

- Your master key is **never stored** on disk
- Entered securely with masked input (asterisks)
- Used only to derive the encryption key
- Forgotten master key = lost data (by design)
- All data is encrypted before storage

## 🛠️ Development

### Project Structure

```
src/
├── main.rs      # CLI interface and command handling
├── crypto.rs    # Encryption/decryption logic
└── storage.rs   # Token storage and management
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code
cargo check
```

## ⚠️ Important Security Notes

1. **Backup Your Master Key**: If you forget it, your data is permanently lost
2. **Master Key Strength**: Use a strong, unique master key
3. **Environment Variables**: Be careful when using `shell` in shared environments

**⚠️ Security Warning**: This tool handles sensitive data. Always use strong master keys and keep your token store file secure.

<!--- P.S. This README is AI Generated. -->
