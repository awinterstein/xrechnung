use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::writer::Writer;
use std::fs::File;
use std::io::BufWriter;

enum XmlElementContent {
    Content(String),
    Children(Vec<XmlElement>),
}

pub struct XmlElement {
    name: String,
    attributes: Vec<(String, String)>,
    content: XmlElementContent,
}

impl XmlElement {
    fn to_owned_strings_vector(input: Option<Vec<(&str, &str)>>) -> Vec<(String, String)> {
        let input = input.unwrap_or(Vec::new());

        // convert string references to owned strings
        input
            .into_iter()
            .map(|(s1, s2)| (s1.to_string(), s2.to_string()))
            .collect()
    }

    pub fn new(
        name: &str,
        attributes: Option<Vec<(&str, &str)>>,
        children: Option<Vec<XmlElement>>,
    ) -> Self {
        XmlElement {
            name: name.to_string(),
            attributes: XmlElement::to_owned_strings_vector(attributes),
            content: XmlElementContent::Children(children.unwrap_or(Vec::new())),
        }
    }

    pub fn new_leaf(name: &str, attributes: Option<Vec<(&str, &str)>>, content: &str) -> Self {
        XmlElement {
            name: name.to_string(),
            attributes: XmlElement::to_owned_strings_vector(attributes),
            content: XmlElementContent::Content(content.to_string()),
        }
    }

    pub fn push_child(&mut self, child: XmlElement) {
        match &mut self.content {
            XmlElementContent::Children(children) => children.push(child),
            XmlElementContent::Content(_) => {
                panic!("Cannot push child to an element with content.")
            }
        }
    }

    pub fn write<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut elem = BytesStart::new(self.name.clone());

        for (key, value) in &self.attributes {
            elem.push_attribute((key.as_bytes(), value.as_bytes()));
        }

        writer.write_event(Event::Start(elem))?;

        match &self.content {
            XmlElementContent::Children(children) => {
                for child in children {
                    child.write(writer)?;
                }
            }
            XmlElementContent::Content(content) => {
                writer.write_event(Event::Text(BytesText::from_escaped(content)))?;
            }
        }

        writer.write_event(Event::End(BytesEnd::new(self.name.clone())))?;

        Ok(())
    }
}

/// Writes an  XRechnung XML structure to the file with the given name.
///
/// * `file_name` - The name of the file to write the XRechnung XML structure to.
/// * `root_element` - The root element of the XML structure as created by the [`create`][crate::create] function.
pub fn write(file_name: &str, root_element: &XmlElement) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(file_name)?;
    let mut writer: Writer<BufWriter<File>> =
        Writer::new_with_indent(BufWriter::new(file), b' ', 4);

    // xml declaration
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    root_element.write(&mut writer)
}
