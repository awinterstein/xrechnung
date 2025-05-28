use chrono::NaiveDate;

use crate::config::{Address, Buyer, Supplier};
use crate::data::{Bill, InvoiceHoursElement, Period};
use crate::xml_writer::XmlElement;

const XMLNS_UBL: &'static str = "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2";
const XMLNS_CAC: &'static str =
    "urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2";
const XMLNS_CBC: &'static str =
    "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2";
const CUSTOMIZATION_ID: &'static str =
    "urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0";
const PROFILE_ID: &'static str = "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0";
const PAYMENT_MEANS_CODE: &'static str = "42"; // payment to bank account
const ENDPOINT_SCHEME_ID: &'static str = "EM"; // use email addresses as the contact points
const QUANTITY_UNIT_CODE: &'static str = "HUR"; // HUR is code for 'hour' from Codes for Units of Measure used in International Trade

/// Rounds a floating point number to two decimal places and formats it as a string.
fn rounded_string(input: f32) -> String {
    format!("{:.2}", (input * 100.0).round() / 100.0)
}

fn create_root_element() -> XmlElement {
    XmlElement::new(
        "ubl:Invoice",
        Some(vec![
            ("xmlns:ubl", XMLNS_UBL),
            ("xmlns:cac", XMLNS_CAC),
            ("xmlns:cbc", XMLNS_CBC),
        ]),
        Some(vec![
            XmlElement::new_leaf("cbc:CustomizationID", None, CUSTOMIZATION_ID),
            XmlElement::new_leaf("cbc:ProfileID", None, PROFILE_ID),
        ]),
    )
}

fn create_endpoint_id_element(scheme_id: &str, endpoint: &str) -> XmlElement {
    XmlElement::new_leaf(
        "cbc:EndpointID",
        Some(vec![("schemeID", scheme_id)]),
        endpoint,
    )
}

fn create_invoice_period_element(period: &Period) -> XmlElement {
    XmlElement::new(
        "cac:InvoicePeriod",
        None,
        Some(vec![
            XmlElement::new_leaf("cbc:StartDate", None, &period.start.to_string()),
            XmlElement::new_leaf("cbc:EndDate", None, &period.end.to_string()),
        ]),
    )
}

fn create_supplier_element(supplier: &Supplier) -> XmlElement {
    // supplier elements with only one party
    XmlElement::new(
        "cac:AccountingSupplierParty",
        None,
        Some(vec![XmlElement::new(
            "cac:Party",
            None,
            Some(vec![
                create_endpoint_id_element(ENDPOINT_SCHEME_ID, &supplier.email),
                create_address_element(&supplier.address),
                create_party_tax_scheme_element(&supplier.tax_identification),
                create_legal_entity_element(&supplier.name, &supplier.tax_identification),
                create_contact_element(&supplier.name, &supplier.phone, &supplier.email),
            ]),
        )]),
    )
}

fn create_buyer_element(buyer: &Buyer) -> XmlElement {
    // buyer elements with only one party
    XmlElement::new(
        "cac:AccountingCustomerParty",
        None,
        Some(vec![XmlElement::new(
            "cac:Party",
            None,
            Some(vec![
                create_endpoint_id_element(ENDPOINT_SCHEME_ID, &buyer.email),
                create_address_element(&buyer.address),
                create_legal_entity_element(&buyer.name, &buyer.tax_identification),
            ]),
        )]),
    )
}

fn create_delivery_element(issue_date: &NaiveDate) -> XmlElement {
    XmlElement::new(
        "cac:Delivery",
        None,
        Some(vec![XmlElement::new_leaf(
            "cbc:ActualDeliveryDate",
            None,
            &issue_date.to_string(),
        )]),
    )
}

fn create_payment_means_element(name: &str, iban: &str, bic: &str) -> XmlElement {
    XmlElement::new(
        "cac:PaymentMeans",
        None,
        Some(vec![
            XmlElement::new_leaf("cbc:PaymentMeansCode", None, PAYMENT_MEANS_CODE),
            XmlElement::new(
                "cac:PayeeFinancialAccount",
                None,
                Some(vec![
                    XmlElement::new_leaf("cbc:ID", None, iban),
                    XmlElement::new_leaf("cbc:Name", None, name),
                    XmlElement::new(
                        "cac:FinancialInstitutionBranch",
                        None,
                        Some(vec![XmlElement::new_leaf("cbc:ID", None, bic)]),
                    ),
                ]),
            ),
        ]),
    )
}

fn create_tax_total_element(bill: &Bill, value: f32) -> XmlElement {
    // add tax amounts with only VAT
    XmlElement::new(
        "cac:TaxTotal",
        None,
        Some(vec![
            create_element_with_currency(
                &bill.currency,
                "cbc:TaxAmount",
                &rounded_string(value * (bill.vat_percent / 100.0)),
            ),
            XmlElement::new(
                "cac:TaxSubtotal",
                None,
                Some(vec![
                    create_element_with_currency(
                        &bill.currency,
                        "cbc:TaxableAmount",
                        &rounded_string(value),
                    ),
                    create_element_with_currency(
                        &bill.currency,
                        "cbc:TaxAmount",
                        &rounded_string(value * (bill.vat_percent / 100.0)),
                    ),
                    XmlElement::new(
                        "cac:TaxCategory",
                        None,
                        Some(vec![
                            XmlElement::new_leaf("cbc:ID", None, "S"),
                            XmlElement::new_leaf(
                                "cbc:Percent",
                                None,
                                &rounded_string(bill.vat_percent),
                            ),
                            create_tax_scheme_vat_element(),
                        ]),
                    ),
                ]),
            ),
        ]),
    )
}

fn create_legal_monetary_total_element(bill: &Bill, value: f32) -> XmlElement {
    XmlElement::new(
        "cac:LegalMonetaryTotal",
        None,
        Some(vec![
            create_element_with_currency(
                &bill.currency,
                "cbc:LineExtensionAmount",
                &rounded_string(value),
            ),
            create_element_with_currency(
                &bill.currency,
                "cbc:TaxExclusiveAmount",
                &rounded_string(value),
            ),
            create_element_with_currency(
                &bill.currency,
                "cbc:TaxInclusiveAmount",
                &rounded_string(value * ((bill.vat_percent / 100.0) + 1.0)),
            ),
            create_element_with_currency(&bill.currency, "cbc:AllowanceTotalAmount", "0.00"),
            create_element_with_currency(&bill.currency, "cbc:ChargeTotalAmount", "0.00"),
            create_element_with_currency(&bill.currency, "cbc:PrepaidAmount", "0.00"),
            create_element_with_currency(&bill.currency, "cbc:PayableRoundingAmount", "0.00"),
            create_element_with_currency(
                &bill.currency,
                "cbc:PayableAmount",
                &rounded_string(value * ((bill.vat_percent / 100.0) + 1.0)),
            ),
        ]),
    )
}

fn create_element_with_currency(currency: &str, tag: &str, content: &str) -> XmlElement {
    XmlElement::new_leaf(tag, Some(vec![("currencyID", currency)]), content)
}

fn create_address_element(address: &Address) -> XmlElement {
    XmlElement::new(
        "cac:PostalAddress",
        None,
        Some(vec![
            XmlElement::new_leaf("cbc:StreetName", None, &address.address_line),
            XmlElement::new_leaf("cbc:CityName", None, &address.city),
            XmlElement::new_leaf("cbc:PostalZone", None, &address.post_code),
            create_country_element(&address.country_code),
        ]),
    )
}

fn create_country_element(country_code: &str) -> XmlElement {
    XmlElement::new(
        "cac:Country",
        None,
        Some(vec![XmlElement::new_leaf(
            "cbc:IdentificationCode",
            None,
            country_code,
        )]),
    )
}

fn create_tax_scheme_vat_element() -> XmlElement {
    XmlElement::new(
        "cac:TaxScheme",
        None,
        Some(vec![XmlElement::new_leaf("cbc:ID", None, "VAT")]),
    )
}

fn create_party_tax_scheme_element(company_id: &str) -> XmlElement {
    XmlElement::new(
        "cac:PartyTaxScheme",
        None,
        Some(vec![
            XmlElement::new_leaf("cbc:CompanyID", None, company_id),
            create_tax_scheme_vat_element(),
        ]),
    )
}

fn create_classified_tax_category_element(vat_percent: f32) -> XmlElement {
    XmlElement::new(
        "cac:ClassifiedTaxCategory",
        None,
        Some(vec![
            XmlElement::new_leaf("cbc:ID", None, "S"),
            XmlElement::new_leaf("cbc:Percent", None, &rounded_string(vat_percent)),
            create_tax_scheme_vat_element(),
        ]),
    )
}

fn create_legal_entity_element(name: &str, id: &str) -> XmlElement {
    XmlElement::new(
        "cac:PartyLegalEntity",
        None,
        Some(vec![
            XmlElement::new_leaf("cbc:RegistrationName", None, name),
            XmlElement::new_leaf("cbc:CompanyID", None, id),
        ]),
    )
}

fn create_contact_element(name: &str, phone: &str, email: &str) -> XmlElement {
    XmlElement::new(
        "cac:Contact",
        None,
        Some(vec![
            XmlElement::new_leaf("cbc:Name", None, name),
            XmlElement::new_leaf("cbc:Telephone", None, phone),
            XmlElement::new_leaf("cbc:ElectronicMail", None, email),
        ]),
    )
}

fn create_invoice_hours_element(
    id: &str,
    currency: &str,
    vat_percent: f32,
    element: InvoiceHoursElement,
) -> Result<XmlElement, Box<dyn std::error::Error>> {
    let mut line_element = XmlElement::new(
        "cac:InvoiceLine",
        None,
        Some(vec![
            XmlElement::new_leaf("cbc:ID", None, id),
            XmlElement::new_leaf(
                "cbc:InvoicedQuantity",
                Some(vec![("unitCode", QUANTITY_UNIT_CODE)]),
                &rounded_string(element.quantity),
            ),
            create_element_with_currency(
                currency,
                "cbc:LineExtensionAmount",
                &rounded_string(element.quantity * element.hourly_rate),
            ),
        ]),
    );

    if element.date.is_some() {
        let date = NaiveDate::parse_from_str(&element.date.unwrap(), "%Y-%m-%d")?;

        line_element.push_child(create_invoice_period_element(&Period {
            start: date,
            end: date,
        }));
    }

    line_element.push_child(XmlElement::new(
        "cac:Item",
        None,
        Some(vec![
            XmlElement::new_leaf("cbc:Name", None, &element.name),
            create_classified_tax_category_element(vat_percent),
        ]),
    ));

    line_element.push_child(XmlElement::new(
        "cac:Price",
        None,
        Some(vec![create_element_with_currency(
            currency,
            "cbc:PriceAmount",
            &rounded_string(element.hourly_rate),
        )]),
    ));

    Ok(line_element)
}

/// Creates an XML structure for an invoice based on the provided supplier, buyer, bill metadata, and invoice hours.
///
/// The returned XML structure can than be given to the [`write`][crate::write] function to write the invoice to a file.
///
/// * `supplier` - The supplier information (name, address, contact, bank data).
/// * `buyer` - The buyer information (name, address, contact).
/// * `bill` - The bill metadata (invoice number, issue date, due date, currency).
/// * `invoice_hours` - A vector of `InvoiceHoursElement` representing the hours worked and their rates.
pub fn create(
    supplier: Supplier,
    buyer: Buyer,
    bill: Bill,
    invoice_hours: Vec<InvoiceHoursElement>,
) -> Result<XmlElement, Box<dyn std::error::Error>> {
    let mut value = 0.0;
    for line in &invoice_hours {
        value += line.quantity * line.hourly_rate;
    }

    let mut root = create_root_element();

    root.push_child(XmlElement::new_leaf("cbc:ID", None, &bill.number));
    root.push_child(XmlElement::new_leaf(
        "cbc:IssueDate",
        None,
        &bill.issue_date.to_string(),
    ));
    root.push_child(XmlElement::new_leaf(
        "cbc:DueDate",
        None,
        &bill.due_date.to_string(),
    ));
    root.push_child(XmlElement::new_leaf("cbc:InvoiceTypeCode", None, "380"));
    root.push_child(XmlElement::new_leaf(
        "cbc:DocumentCurrencyCode",
        None,
        &bill.currency,
    ));
    root.push_child(XmlElement::new_leaf(
        "cbc:BuyerReference",
        None,
        &buyer.reference,
    ));

    if bill.period.is_some() {
        root.push_child(create_invoice_period_element(
            &bill.period.as_ref().unwrap(),
        ));
    }

    root.push_child(create_supplier_element(&supplier));
    root.push_child(create_buyer_element(&buyer));
    root.push_child(create_delivery_element(&bill.issue_date));
    root.push_child(create_payment_means_element(
        &supplier.name,
        &supplier.iban,
        &supplier.bic,
    ));
    root.push_child(create_tax_total_element(&bill, value));
    root.push_child(create_legal_monetary_total_element(&bill, value));

    let mut count = 0;
    for invoice_hours_element in invoice_hours {
        count += 1;
        root.push_child(create_invoice_hours_element(
            &count.to_string(),
            &bill.currency,
            bill.vat_percent,
            invoice_hours_element,
        )?);
    }

    Ok(root)
}
