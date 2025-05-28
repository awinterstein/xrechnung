/// A command line tool to create an XRechnung invoice from a CSV file with invoice hours.
///
/// ```
///
/// ```
use chrono::{Datelike, NaiveDate};
use clap::Parser;
use csv;
use std::fs::File;

use xrechnung::data::{Bill, InvoiceHoursElement, Period};

/// Command line tool to create an XRechnung invoice from a CSV file with invoice hours.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The unique number of the invoice
    #[arg(short, long)]
    invoice_id: String,

    /// Config file that provides supplier and buyer information
    #[arg(short, long)]
    config: String,

    /// Buyer of the invoice
    #[arg(short, long)]
    buyer: String,

    /// Issue date of the invoice
    #[arg(short = 'd', long)]
    issue_date: NaiveDate,

    /// CSV file that contains the invoice lines
    #[arg(short = 'l', long)]
    invoice_hours: String,

    /// Output XML file for the invoice to be written
    #[arg(short, long)]
    output: String,
}

fn read_invoice_hours(
    file_name: &str,
) -> Result<Vec<InvoiceHoursElement>, Box<dyn std::error::Error>> {
    let mut invoice_hours: Vec<InvoiceHoursElement> = Vec::new();
    let file = File::open(file_name)?;

    for result in csv::Reader::from_reader(file).deserialize() {
        invoice_hours.push(result?);
    }

    Ok(invoice_hours)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse command line arguments and load configuration based on them
    let args = Args::parse();
    let config = xrechnung::config::load(&args.config, &args.buyer)?;

    // read the invoice hours from the given CSV file
    let invoice_hours = read_invoice_hours(&args.invoice_hours)?;

    // the start of the billing period is either the first date of the invoice hours, or if that does not exist
    // then the billing period is the first day of the month of the issue date of the bill
    let start = if invoice_hours.iter().count() > 0 && invoice_hours[0].date.is_some() {
        NaiveDate::parse_from_str(&invoice_hours[0].date.as_ref().unwrap(), "%Y-%m-%d").unwrap()
    } else {
        args.issue_date.with_day(1).unwrap() // billing period starts on first day of the month of the issue date
    };

    let bill = Bill::new(
        args.invoice_id,
        args.issue_date,
        Some(Period {
            start: start,
            end: args.issue_date, // billing period ends on the issue date
        }),
        &config,
    );

    // create XML structure for the invoice from the supplier, buyer, invoice metadata and invoice hours
    let xml_root = xrechnung::create(config.supplier, config.buyer, bill, invoice_hours)?;

    // finally write the XML structure to a file
    xrechnung::write(&args.output, &xml_root)
}
