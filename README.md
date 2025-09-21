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

### Prerequisites

- Rust 1.70+ installed on your system
- [Install Rust](https://rustup.rs/) if you haven't already

### Build from Source

```bash
git clone <your-repo-url>
cd srs
cargo build --release
```

The binary will be available at `target/release/srs`.

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

### Dependencies

- `aes-gcm`: AES-256-GCM encryption
- `clap`: Command-line argument parsing
- `rpassword`: Secure password input
- `serde`: JSON serialization
- `anyhow`: Error handling
- `sha2`: SHA-256 hashing

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

## 🔧 Configuration

### Environment Variables

The `populate` command sets environment variables for the current session:

```bash
srs populate
# Sets: GITHUB_TOKEN=your_actual_token
#       API_KEY=your_actual_key
# etc.
```

## ⚠️ Important Security Notes

1. **Backup Your Master Key**: If you forget it, your data is permanently lost
2. **File Permissions**: Restrict access to `srs_token_store.json`:
   ```bash
   chmod 600 srs_token_store.json
   ```
3. **Master Key Strength**: Use a strong, unique master key
4. **Environment Variables**: Be careful when using `populate` in shared environments

**⚠️ Security Warning**: This tool handles sensitive data. Always use strong master keys and keep your token store file secure.

<!--- P.S. This README is AI Generated. -->
