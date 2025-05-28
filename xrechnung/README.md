# XRechnung

This crate provides functionality to create invoices in the XRechnung format.

[XRechnung](https://xeinkauf.de/xrechnung) is a German standard for electronic invoicing, which is based on the
EN 16931 standard. It is used by public authorities in Germany to receive and process electronic invoices and is
one of the allowed standards for electronic invoicing in Germany.

XRechnung documents can be created in the UBL (Universal Business Language) or (Cross Industry Invoice) CII format,
which are both XML-based formats. This crate provides functionality to create XRechnung documents in the UBL
format.

## Limitations

- the focus is on creating invoices for freelancers and only the needed subset of the XRechnung standard is implemented
- right now, only hourly rate invoices are supported
    - all invoice items are billed as an amount in hours at an hourly rate
    - it is planned to extend the crate to support other items like travel or hardware expenses as well
- invoices are only created in the UBL format

## Example
```rust
// load the configuration for a specific client from the configuration file
let config = xrechnung::config::load("examples/config.toml", "Client Company")?;

// create invoice metadata from the configuration, invoice number and dates
let bill = xrechnung::data::Bill::new(
    "2025-0001".to_string(), // invoice number
    chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(), // issue date of the invoice
    Some(xrechnung::data::Period {
        start: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        // billing period usually ends on the issue date
        end: chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
    }),
    &config,
);

// create some invoice hours elements
// those elements could also be read from a CSV file or other data source
let invoice_hours = vec![
    xrechnung::data::InvoiceHoursElement {
        name: "Example Service".to_string(),
        quantity: 7.0,
        hourly_rate: 110.0,
        date: Some("2025-01-02".to_string()),
    },
    xrechnung::data::InvoiceHoursElement {
        name: "Another Service".to_string(),
        quantity: 6.5,
        hourly_rate: 110.0,
        date: Some("2025-01-03".to_string()),
    },
];

// create XML structure for the invoice from the supplier, buyer, invoice metadata and invoice hours
let xml_root = xrechnung::create(config.supplier, config.buyer, bill, invoice_hours)?;

// finally write the XML structure to a file
xrechnung::write("invoice.xml", &xml_root)?;

```

