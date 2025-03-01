use address_converter::presentation::cli::commands::{run_command, Cli};
use address_converter::application::service::AddressService;
use address_converter::infrastructure::JsonAddressRepository;
use clap::Parser;
use std::env;

#[cfg(feature = "cli")]
fn main() {

    let storage_dir = env::var("STORAGE_DIR").unwrap_or_else(|_| "./json_storage".to_string());
    let service = AddressService::new(Box::new(JsonAddressRepository::new(storage_dir)));
    
    let cli = Cli::parse();
    if let Err(e) = run_command(cli, &service) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("CLI support is disabled. Enable the 'cli' feature to use this binary.");
    std::process::exit(1);
}