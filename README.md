# Secure Rust Storage (SRS)

A secure command-line tool for storing and managing personal access tokens using AES-256-GCM encryption. Built with Rust for maximum security and performance.

## 🔐 Features

- **AES-256-GCM Encryption**: Military-grade encryption for your sensitive data.
- **Master Key Protection**: Your master key is never stored, only used to derive encryption keys.
- **Masked Input**: Secure password input with asterisk masking.
- **Environment Integration**: Populate environment variables with stored tokens
- **Cross-Platform**: Works on Windows, macOS, and Linux.
- **Zero Dependencies**: No external services or cloud dependencies. All your data stays on your computer.

## 🚀 Installation

### Quick Install (Recommended)

#### Universal Install (Auto-detects platform)

```bash
curl -fsSL https://raw.githubusercontent.com/thenicekat/srs/main/install.sh | bash
```

The install script will:

- Automatically detect your system architecture
- Download the latest release from GitHub
- Install the binary to `~/.local/bin`
- Set up the shell alias for automatic environment variable sourcing
- Add the binary to your PATH (if needed)

### Manual Installation

#### Prerequisites

- Rust 1.70+ installed on your system
- [Install Rust](https://rustup.rs/) if you haven't already

#### Installing the Binary from cargo

```bash
cargo install --git https://github.com/thenicekat/srs
```

#### Setting the Platform specific alias

Add the appropriate alias to your shell's RC file to ensure automatic environment file sourcing:

Bash(.bashrc):

```bash
echo 'srs() {
    srs "$@"
    source ~/.local/share/__srs__.env 2>/dev/null || true
    rm -f ~/.local/share/__srs__.env
}' >> ~/.bashrc
```

Zsh(.zshrc):

```bash
echo 'srs() {
    srs "$@"
    source ~/.local/share/__srs__.env 2>/dev/null || true
    rm -f ~/.local/share/__srs__.env
}' >> ~/.zshrc
```

## 📖 Usage

### Basic Commands

```bash
# Add a new token.
srs add github_token

# Add a token with value directly.
srs add github_token --token "ghp_xxxxxxxxxxxx"

# Retrieve a token.
srs get github_token

# List all stored tokens.
srs list

# Delete a token.
srs delete github_token

# Populate environment variables.
srs populate
```

### Command Reference

| Command                        | Description               | Example                   |
| ------------------------------ | ------------------------- | ------------------------- |
| `add <name> [--token <value>]` | Store a new token         | `srs add github_token`    |
| `get <name>`                   | Retrieve a token          | `srs get github_token`    |
| `list`                         | List all token names      | `srs list`                |
| `delete <name>`                | Delete a token            | `srs delete github_token` |
| `populate`                     | Set environment variables | `srs populate`            |

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

### Data Storage

- Tokens stored in `srs_token_store.json`
- All data encrypted before storage
- File permissions should be restricted (600 recommended)

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
3. **Environment Variables**: Be careful when using `populate` in shared environments

**⚠️ Security Warning**: This tool handles sensitive data. Always use strong master keys and keep your token store file secure.

<!--- P.S. This README is AI Generated. -->
