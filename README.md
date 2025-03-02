# Address Converter

## Prerequisites

- Rust
- Cargo

## Design

### Domain

This project follows a Domain-Driven Design (DDD) approach, where the domain
handles converting addresses to and from the following formats:

- French addresses in the NF Z10-011 standard
- ISO 20022 addresses

The `Address` entity is format-agnostic. It provides a unified way to store and
manage addresses, which is useful for storage purposes and enforces domain
invariants. The `AddressConvertible` trait offers the interface to convert to
and from the desired formats using value objects for each: `FrenchAddress` and
`IsoAddress`.

During conversion, a `ConvertedAddress` value object is produced, representing
the conversion lifecycle within the domain. An `Address` gains its identity only
when created from a `ConvertedAddress`. Similar to a `ConvertedAddress`, an
`Address` includes unique identifier (UUID) and an `updated_at` field for
tracking purposes.

### Library and binaries

The library exposes the following layers for consumers:

- Application
- Domain
- Infrastructure
- Presentation

This structure allows reuse and extension if the library's functionalities.
Currently, only one binary, `bin/cli`, is available. The project is configured
to easily extend entry points and enable or disable them via cargo features:
`cli` and `api`. The latter is a placeholder example and would need
implementation.

Binaries can be tweaked to change the persistence solution. We currently provide
JSON persistence, which could be swapped for a real database later.

## Getting started

```bash
cargo run --bin cli -- --help
```

```
Convert and manage postal addresses (french/iso20022)

Usage: cli <COMMAND>

Commands:
  save    Save a new address
  update  Update an existing address
  delete  Delete an address
  fetch   Fetch an address in the specified format
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Different storage folder

The CLI binary uses the json repository to store addresses on the filesystem.
By default, addresses are storedin a folder named `json_storage` in your current
directory. TO change this folder, prefix your commands with the `STORAGE_DIR`
environment variable . For example:

```bash
STORAGE_DIR="${HOME}/json_storage" cargo run --bin cli -- --help
```

### Save and Fetch

This section provides examples of how to save addresses from a specific
format (french or ISO 20022) and fetch them in the format of your choice. The
`save` command gives you an ID, which you can use to fetch the saved address and
format it. Note that you may need to delete the created address if you try to
save the same data from a different format.

#### Individual

##### French -> ISO20022

```bash
cargo run --bin cli save --from-format=french --address='
{
    "name": "Monsieur Jean DELHOURME",
    "internal_delivery": "Chez Mireille COPEAU Appartement 2",
    "external_delivery": "Entrée A Bâtiment Jonquille",
    "street": "25 RUE DE L’EGLISE",
    "distribution_info": "CAUDOS",
    "postal": "33380 MIOS",
    "country": "FRANCE"
}'

Saved address with ID: ea8bf423-198c-4ec3-a890-5832af32bdc7
```

```bash
cargo run --bin cli fetch ea8bf423-198c-4ec3-a890-5832af32bdc7 --format=iso20022
```

##### ISO20022 -> French

```bash
cargo run --bin cli save --from-format=iso20022 --address='
{
    "name": "Monsieur Jean DELHOURME",
    "postal_address": {
        "street_name": "RUE DE L’EGLISE",
        "building_number": "25",
        "room": "Chez Mireille COPEAU Appartement 2",
        "postbox": "CAUDOS",
        "postcode": "33380",
        "town_name": "MIOS",
        "country": "FR"
    }
}'

Saved address with ID: ea8bf423-198c-4ec3-a890-5832af32bdc7
```

```bash
cargo run --bin cli fetch ea8bf423-198c-4ec3-a890-5832af32bdc7 --format=french
```

#### Business

##### French -> ISO20022

```bash
cargo run --bin cli save --from-format=french --address='
{
    "business_name": "Société DUPONT",
    "recipient": "Mademoiselle Lucie MARTIN",
    "external_delivery": "Résidence des Capucins Bâtiment Quater",
    "street": "56 RUE EMILE ZOLA",
    "distribution_info": "BP 90432 MONTFERRIER SUR LEZ",
    "postal": "34092 MONTPELLIER CEDEX 5",
    "country": "FRANCE"
}'

Saved address with ID: ea8bf423-198c-4ec3-a890-5832af32bdc7
```

```bash
cargo run --bin cli fetch ea8bf423-198c-4ec3-a890-5832af32bdc7 --format=iso20022
```

##### ISO20022 -> French

```bash
cargo run --bin cli save --from-format=iso20022 --address='
{
    "business_name": "Société DUPONT",
    "postal_address": {
        "street_name": "RUE EMILE ZOLA",
        "building_number": "56",
        "department": "Mademoiselle Lucie MARTIN",
        "postbox": "BP 90432",
        "town_location_name": "MONTFERRIER SUR LEZ",
        "postcode": "34092",
        "town_name": "MONTPELLIER CEDEX 5",
        "country": "FR"
    }
}'

Saved address with ID: ea8bf423-198c-4ec3-a890-5832af32bdc7
```

```bash
cargo run --bin cli fetch ea8bf423-198c-4ec3-a890-5832af32bdc7 --format=french
```

### Update and Fetch

```bash
cargo run --bin cli update 7d793037-a8d9-4e2e-9e4f-3c6d8e4f5b6c --from-format=iso20022 --address='
{
    "business_name": "Société DUPONT",
    "postal_address": {
        "street_name": "RUE DE LA RÉPUBLIQUE",
        "building_number": "15",
        "department": "Mademoiselle Lucie MARTIN",
        "postbox": "BP 69001",
        "town_location_name": "PART-DIEU",
        "postcode": "69001",
        "town_name": "LYON",
        "country": "FR"
    }
}'

Updated address with ID: ea8bf423-198c-4ec3-a890-5832af32bdc7
```

```bash
cargo run --bin cli fetch ea8bf423-198c-4ec3-a890-5832af32bdc7 --format=french
```

### Delete

```bash
cargo run --bin cli delete ea8bf423-198c-4ec3-a890-5832af32bdc7

Deleted address with ID: ea8bf423-198c-4ec3-a890-5832af32bdc7
```