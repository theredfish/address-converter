use address_converter::application::service::AddressService;
use address_converter::infrastructure::JsonAddressRepository;
use address_converter::presentation::cli::commands::{run_command, Cli};
use clap::Parser;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn service(temp_dir: &TempDir) -> AddressService {
    let repo = JsonAddressRepository::new(temp_dir.path());
    AddressService::new(Box::new(repo))
}

/// Helper function that will retrieve the ID from the file stored in the
/// temporary folder. It will to verify that the file exists, contains the
/// correct name and naming consistency with the overall process.
/// Will panic if the file information can't be extracted.
fn get_file_id(path: &Path) -> String {
    let mut files = fs::read_dir(path).unwrap();
    let first_file = files.next().unwrap().unwrap().path();
    let filename_id = first_file
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    filename_id
}

#[test]
fn cli_save_french() {
    let temp_dir = TempDir::new().unwrap();
    let service = service(&temp_dir);

    let cli = Cli::parse_from([
        "address_converter",
        "save",
        "--address",
        r#"{"name": "Monsieur Jean DELHOURME", "street": "25 RUE DE L'EGLISE", "postal": "33380 MIOS", "country": "FRANCE"}"#,
        "--from-format",
        "french",
    ]);
    run_command(cli, &service).unwrap();

    let files = fs::read_dir(temp_dir.path()).unwrap().count();
    assert_eq!(files, 1);
}

#[test]
fn cli_save_duplicate_french() {
    let temp_dir = TempDir::new().unwrap();
    let service = service(&temp_dir);

    let input = r#"{"name": "Monsieur Jean DELHOURME", "street": "25 RUE DE L'EGLISE", "postal": "33380 MIOS", "country": "FRANCE"}"#;

    // Save
    let cli1 = Cli::parse_from([
        "address_converter",
        "save",
        "--address",
        input,
        "--from-format",
        "french",
    ]);
    run_command(cli1, &service).unwrap();

    // Try saving duplicate
    let cli2 = Cli::parse_from([
        "address_converter",
        "save",
        "--address",
        input,
        "--from-format",
        "french",
    ]);
    let result = run_command(cli2, &service);
    assert!(matches!(result, Err(e) if e.contains("Resource already exists:")));
}

#[test]
fn cli_update() {
    let temp_dir = TempDir::new().unwrap();
    let service = service(&temp_dir);

    // Save
    let save_cli = Cli::parse_from([
        "address_converter",
        "save",
        "--address",
        r#"{"name": "Monsieur Jean DELHOURME", "street": "25 RUE DE L'EGLISE", "postal": "33380 MIOS", "country": "FRANCE"}"#,
        "--from-format",
        "french",
    ]);
    run_command(save_cli, &service).unwrap();

    // Retrieve the first file id
    let file_id = get_file_id(temp_dir.path());

    // Update address
    let update_cli = Cli::parse_from([
        "address_converter",
        "update",
        &file_id,
        "--address",
        r#"{"name": "Monsieur Jean DELHOURME", "street": "10 AVENUE DES CHAMPS", "postal": "33380 MIOS", "country": "FRANCE"}"#,
        "--from-format",
        "french",
    ]);
    run_command(update_cli, &service).unwrap();

    // Verify update
    let fetch_result = service.fetch(&file_id).unwrap();
    let street = fetch_result.street.unwrap();
    assert_eq!(street.name, "AVENUE DES CHAMPS");
    assert_eq!(street.number.unwrap(), "10");
}

#[test]
fn cli_fetch() {
    let temp_dir = TempDir::new().unwrap();
    let service = service(&temp_dir);

    // Save
    let save_cli = Cli::parse_from([
        "address_converter",
        "save",
        "--address",
        r#"{"name": "Monsieur Jean DELHOURME", "street": "25 RUE DE L'EGLISE", "postal": "33380 MIOS", "country": "FRANCE"}"#,
        "--from-format",
        "french",
    ]);
    run_command(save_cli, &service).unwrap();

    // Retrieve the first file id
    let file_id = get_file_id(temp_dir.path());

    // Fetch in ISO format
    let fetch_cli = Cli::parse_from([
        "address_converter",
        "fetch",
        &file_id,
        "--format",
        "iso20022",
    ]);
    let result = run_command(fetch_cli, &service);
    assert!(result.is_ok());
}

#[test]
fn cli_delete() {
    let temp_dir = TempDir::new().unwrap();
    let service = service(&temp_dir);

    // Save
    let save_cli = Cli::parse_from([
        "address_converter",
        "save",
        "--address",
        r#"{"name": "Monsieur Jean DELHOURME", "street": "25 RUE DE L’EGLISE", "postal": "33380 MIOS", "country": "FRANCE"}"#,
        "--from-format",
        "french",
    ]);
    run_command(save_cli, &service).unwrap();

    // Retrieve the first file id
    let file_id = get_file_id(temp_dir.path());

    // Delete address
    let delete_cli = Cli::parse_from(["address_converter", "delete", &file_id]);
    let result = run_command(delete_cli, &service);
    assert!(result.is_ok());

    // Verify deleted
    let fetch_result = service.repository.fetch(&file_id);
    assert!(fetch_result.is_err());
}
