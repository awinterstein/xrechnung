//! Functionality to load and parse the configuration for invoice generation.
//!
//! # Example
//!
//! A configuration file can be loaded the following way:
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = xrechnung::config::load("examples/config.toml", "Client Company")?;
//! # Ok(())
//! # }
//! ```
//!
//! The given `buyer_name` "Client Company" needs to exist in the `config.toml` file. See an example of a matching
//! file here:
//!
//! ```toml
#![doc=include_str!("../examples/config.toml")]
//! ```
//!
//! The file must contain one supplier and at least one buyer.

use serde::Deserialize;
use std::fs;

/// Address data for the supplier and buyer.
#[derive(Deserialize)]
pub struct Address {
    /// The first line of the address. It usually contains the street name and number or a P.O. box.
    pub address_line: String,

    /// The city, town or village where the address is located.
    pub city: String,

    /// The country-specific post code of the address.
    pub post_code: String,

    /// The country code of the address as specified in ISO 3166-1 "Codes for the representation of names of countries
    /// and their subdivisions". It is only allowed to use the Alpha-2 code.
    pub country_code: String,
}

/// Supplier data (name, tax data, contact, bank account) for the invoice.
#[derive(Deserialize)]
pub struct Supplier {
    /// The company name of the supplier.
    pub name: String,

    /// The tax identification (e.g., vat number) of the supplier. This is a unique identifier assigned to the
    /// supplier by the tax office.
    pub tax_identification: String,

    /// The address of the supplier.
    pub address: Address,

    /// The phone number of the contact person at the supplier.
    pub phone: String,

    /// The email address of the contact person at the supplier. This email is used as the main contact point for the
    /// supplier in the invoice.
    pub email: String,

    /// The IBAN (International Bank Account Number) of the supplier. This should identify the bank account onto which
    /// the invoice amount should be transferred.
    pub iban: String,

    /// The BIC (Bank Identifier Code) of the supplier. Matching the bank account that is determined by the IBAN field.
    pub bic: String,
}

/// Buyer data (name, tax data, contact, reference number) for the invoice.
#[derive(Deserialize)]
pub struct Buyer {
    /// The company name of the buyer.
    pub name: String,

    /// The tax identification (e.g., vat number) of the buyer. This is a unique identifier assigned to the buyer by
    /// the tax office.
    pub tax_identification: String,

    /// The address of the buyer.
    pub address: Address,

    /// The email address of the contact person at the buyer. This email is used as the main contact point for the
    /// buyer in the invoice.
    pub email: String,

    // Can be order number, internal project number or contact of buyer or even N/A.
    pub reference: String,

    /// After how many days invoices for this buyer are due. This is used to calculated the due date of the invoice
    /// based on the issue date.
    pub due_after_days: i16,
}

/// The complete configuration as deserialized from the configuration file.
/// This includes all available buyers and is reduced to the Config struct before returned to the caller.
#[derive(Deserialize)]
struct CompleteConfig {
    /// The currency used for the invoice, e.g., "EUR", "USD", etc.
    pub currency: String,

    /// The VAT percentage applied to the invoice total.
    pub vat_percent: f32,

    /// The supplier data for the invoice.
    pub supplier: Supplier,

    /// A list of buyers for the invoice creation. Only one will be used for any invoice.
    pub buyer: Vec<Buyer>,
}

/// The reduced configuration for the invoice creation that in contrast to the CompleteConfig struct only contains the
/// about the one buyer for whom the invoice should be created.
pub struct Config {
    /// The currency used for the invoice, e.g., "EUR", "USD", etc.
    pub currency: String,

    /// The VAT percentage applied to the invoice total.
    pub vat_percent: f32,

    /// The supplier data for the invoice.
    pub supplier: Supplier,

    /// The buyer data for the invoice. Selected from all buyers in the CompleteConfig struct.
    pub buyer: Buyer,
}

/// Loads the configuration from the given file and returns a Config struct that can be used to create an invoice.
///
/// * `filename`   - The path to the configuration file in TOML format.
/// * `buyer_name` - The name of the buyer for whom the invoice should be created. The value must match the name
///                  attribute of exactly one Buyer in the configuration file.
///
/// The function returns an error in case that the config file could not be loaded or deserialized or if no buyer with
/// the name `buyer_name` could be found in the configuration file.
pub fn load(filename: &str, buyer_name: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let config_toml = fs::read_to_string(filename)?;
    let complete_config: CompleteConfig = toml::from_str(&config_toml)?;

    // filter buyer from the config file by the given supplier name
    let matching_supplier: Option<Buyer> = complete_config
        .buyer
        .into_iter()
        .filter(|x| x.name == buyer_name)
        .collect::<Vec<_>>()
        .pop(); // get the first element (assuming that there is no more than one match)

    // we can only continue, if a supplier with the given name was found in the config file
    let matching_supplier = matching_supplier.ok_or(format!(
        "Could not find buyer '{buyer_name}' in the configuration file."
    ))?;

    // create the particular config for the given buyer (not returning all buyers)
    let config = Config {
        currency: complete_config.currency,
        vat_percent: complete_config.vat_percent,
        supplier: complete_config.supplier,
        buyer: matching_supplier,
    };

    Ok(config)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_config_file() {
        let config = crate::config::load("examples/config.toml", "Client Company").unwrap();
        assert_eq!(config.supplier.name, "Hans Muster");
        assert_eq!(config.buyer.name, "Client Company");
        assert_eq!(config.buyer.email, "mail@client1.example.com");

        let config = crate::config::load("examples/config.toml", "Another Client").unwrap();
        assert_eq!(config.supplier.name, "Hans Muster");
        assert_eq!(config.buyer.name, "Another Client");
        assert_eq!(config.buyer.email, "mail@client2.example.com");
    }

    #[test]
    fn test_error_on_missing_file() {
        assert!(crate::config::load("examples/config_nonexistent.toml", "Client Company").is_err());
    }

    #[test]
    fn test_error_on_missing_buyer() {
        assert!(crate::config::load("examples/config.toml", "Wrong Company").is_err());
    }
}
