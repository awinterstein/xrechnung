//! Data structures representing an invoice. The XRechnung format is created from these structures by the `xml_bill`
//! module.

use crate::config::Config;
use chrono::{Days, NaiveDate};
use serde::Deserialize;

/// Definition of a period for the invoice (e.g., billing period).
pub struct Period {
    /// The start date of the period.
    pub start: NaiveDate,

    /// The end date of the period.
    pub end: NaiveDate,
}

/// Data structure containing the metadata of an invoice (bill).
pub struct Bill {
    /// The unique number of the invoice (as required by law).
    pub number: String,

    /// The currency of the invoice, e.g., EUR.
    pub currency: String,

    /// The VAT percentage applied to the invoice total.
    pub vat_percent: f32,

    /// The issue date of the invoice.
    pub issue_date: NaiveDate,

    /// The due date of the invoice.
    pub due_date: NaiveDate,

    /// The billing period for the invoice, if applicable.
    pub period: Option<Period>,
}

/// Data structure representing an invoice line item for hours worked.
/// From a list of those items, the billable amount for the invoice is calculated.
#[derive(Deserialize)]
pub struct InvoiceHoursElement {
    /// The name / description of the line item, e.g., "Development", "Consulting", etc.
    pub name: String,

    /// The quantity of hours worked for this line item.
    pub quantity: f32,

    /// The hourly rate for this line item in the specified currency.
    pub hourly_rate: f32,

    /// The date of the line item in ISO 8601 format (YYYY-MM-DD), if applicable.
    pub date: Option<String>,
}

impl Bill {
    pub fn new(
        number: String,
        issue_date: NaiveDate,
        period: Option<Period>,
        config: &Config,
    ) -> Self {
        Bill {
            number,
            currency: config.currency.clone(),
            vat_percent: config.vat_percent,
            issue_date: issue_date,

            // bill is due configured amount of days after issue date
            due_date: (issue_date + Days::new(config.buyer.due_after_days as u64)),

            period,
        }
    }
}
