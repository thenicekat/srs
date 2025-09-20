use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{self, Write};

mod crypto;
use crypto::CryptoManager;

mod storage;
use storage::TokenStorage;

#[derive(Parser)]
#[command(name = "srs")]
#[command(about = "Secure Rust Storage - A tool to store personal access tokens securely")]
struct CommandLineInterface {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        name: String,
        token: Option<String>,
    },
    Get {
        name: String,
    },
    List,
    Delete {
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = CommandLineInterface::parse();
    print!("Enter your master key: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let master_key = input.trim().to_string();

    let crypto_manager = CryptoManager::new(&master_key)?;
    let mut storage = TokenStorage::new("tokens.json", crypto_manager)?;
    
    match cli.command {
        Commands::Add { name, token } => {
            let token_value = match token {
                Some(t) => t,
                None => {
                    print!("Enter token for '{}': ", name);
                    io::stdout().flush()?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    input.trim().to_string()
                }
            };
            
            storage.store_token(&name, &token_value)?;
            println!("✅ Token '{}' stored successfully!", name);
        }
        Commands::Get { name } => {
            match storage.get_token(&name)? {
                Some(token) => println!("{}", token),
                None => println!("❌ Token '{}' not found", name),
            }
        }
        Commands::List => {
            let tokens = storage.list_tokens()?;
            if tokens.is_empty() {
                println!("No tokens stored.");
            } else {
                println!("Stored tokens:");
                for name in tokens {
                    println!("  - {}", name);
                }
            }
        }
        Commands::Delete { name } => {
            if storage.delete_token(&name)? {
                println!("✅ Token '{}' deleted successfully!", name);
            } else {
                println!("❌ Token '{}' not found", name);
            }
        }
    }
    
    Ok(())
}
