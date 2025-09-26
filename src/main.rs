use anyhow::Result;
use clap::{Parser, Subcommand};
use rpassword::read_password;
use std::io::{self, Write};

mod crypto;
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
    #[command(about = "Adds a new value corresponding to the name.")]
    Add { name: String, token: Option<String> },
    #[command(about = "Fetches the value of the key corresponding to the name.")]
    Get { name: String },
    #[command(about = "Lists the names of all the available keys.")]
    List,
    #[command(about = "Deletes the value corresponding to the key.")]
    Delete { name: String },
    #[command(about = "Spawns a new shell with all tokens loaded via memory pipe.")]
    Shell,
}

fn main() -> Result<()> {
    let cli = CommandLineInterface::parse();

    let mut storage = TokenStorage::new()?;

    match cli.command {
        Commands::Add { name, token } => {
            let token_value = match token {
                Some(t) => t,
                None => {
                    print!("Enter token for '{}': ", name);
                    io::stdout().flush()?;
                    read_password().expect("Failed to read password")
                }
            };

            storage.store_token(&name, &token_value)?;
            println!("::> Token '{}' stored successfully!", name);
        }
        Commands::Get { name } => match storage.get_token(&name)? {
            Some(token) => println!("{}", token),
            None => println!("::> Token '{}' not found", name),
        },
        Commands::List => {
            let tokens = storage.list_tokens()?;
            println!("Stored tokens:");
            for name in tokens {
                println!("  - {}", name);
            }
        }
        Commands::Delete { name } => {
            storage.delete_token(&name)?;
        }
        Commands::Shell => {
            println!("::> Spawning new shell with SRS tokens loaded...");
            storage.populate_tokens_to_child()?;
        }
    }
    Ok(())
}
