use clap::{Parser, Subcommand};
use crate::application::service::{AddressService, Format, Either};

#[derive(Parser)]
#[command(name = "address_converter", about = "Convert and manage postal addresses (french/iso20022)")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Save a new address
    Save {
        #[arg(long, help = "JSON-formatted address string")]
        address: String,
        #[arg(long, help = "Input format: 'french' or 'iso20022'")]
        from_format: String,
    },
    /// Update an existing address
    Update {
        #[arg(help = "UUID of the address to update")]
        id: String,
        #[arg(long, help = "JSON-formatted address string")]
        address: String,
        #[arg(long, help = "Input format: 'french' or 'iso20022'")]
        from_format: String,
    },
    /// Delete an address
    Delete {
        #[arg(help = "UUID of the address to delete")]
        id: String,
    },
    /// Fetch an address in the specified format
    Fetch {
        #[arg(help = "UUID of the address to fetch")]
        id: String,
        #[arg(long, help = "Output format: 'french' or 'iso20022'")]
        format: String,
    },
}

fn format_to_enum(format: &str) -> Result<Format, String> {
    match format.to_lowercase().as_str() {
        "french" => Ok(Format::French),
        "iso20022" => Ok(Format::Iso20022),
        _ => Err("Invalid format: must be 'french' or 'iso20022'".to_string()),
    }
}

pub fn run_command(cli: Cli, service: AddressService) -> Result<(), String> {
    match cli.command {
        Commands::Save { address, from_format } => {
            let format = format_to_enum(&from_format)?;
            let id = service.save(&address, format).map_err(|e| e.to_string())?;
            println!("Saved address with ID: {}", id);

            Ok(())
        }
        Commands::Update { id, address, from_format } => {
            let format = format_to_enum(&from_format)?;
            service.update(&id, &address, format).map_err(|e| e.to_string())?;
            println!("Updated address with ID: {}", id);

            Ok(())
        }
        Commands::Delete { id } => {
            service.delete(&id).map_err(|e| e.to_string())?;
            println!("Deleted address with ID: {}", id);

            Ok(())
        }
        Commands::Fetch { id, format } => {
            let format_enum = format_to_enum(&format)?;
            let result = service.fetch_format(&id, format_enum).map_err(|e| e.to_string())?;
            
            match result {
                Either::French(french) => println!("{}", serde_json::to_string_pretty(&french).unwrap()),
                Either::Iso20022(iso) => println!("{}", serde_json::to_string_pretty(&iso).unwrap()),
            }
            
            Ok(())
        }
    }
}