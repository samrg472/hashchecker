use std::{collections::HashMap, io::Read};
use xml::reader::XmlEvent;

#[derive(Debug)]
pub struct XmlDoc {
    name: String,
    value: Option<String>,
    attributes: HashMap<String, String>,
    children: Vec<XmlDoc>,
}

impl XmlDoc {
    pub fn parse(doc: &str) -> Self {
        let mut reader = xml::ParserConfig::new()
            .trim_whitespace(true)
            .create_reader(doc.as_bytes());
        loop {
            let data = reader.next().unwrap();
            match data {
                XmlEvent::StartDocument { .. } => {}
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    let attributes = {
                        let mut map = HashMap::new();
                        for attrib in attributes {
                            map.insert(attrib.name.local_name, attrib.value);
                        }
                        map
                    };

                    let mut parent = XmlDoc::new(name.local_name, attributes);
                    parse_children(&mut reader, &mut parent);
                    break parent;
                }
                _ => panic!("Unhandled event: {:?}", data),
            }
        }
    }

    pub fn new(name: String, attributes: HashMap<String, String>) -> Self {
        Self {
            name,
            value: None,
            attributes,
            children: Vec::new(),
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn children(&self) -> &[XmlDoc] {
        &self.children
    }

    #[inline]
    pub fn value(&self) -> Option<&String> {
        self.value.as_ref()
    }

    #[inline]
    pub fn get_attrib(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    #[inline]
    pub fn get_child(&self, name: &str) -> Option<&XmlDoc> {
        self.children.iter().find(|c| c.name == name)
    }
}

fn parse_children<R>(reader: &mut xml::EventReader<R>, parent: &mut XmlDoc)
where
    R: Read,
{
    loop {
        let data = reader.next();
        match data {
            Ok(data) => match data {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    let attributes = {
                        let mut map = HashMap::new();
                        for attrib in attributes {
                            map.insert(attrib.name.local_name, attrib.value);
                        }
                        map
                    };
                    let mut child = XmlDoc::new(name.local_name, attributes);
                    parse_children(reader, &mut child);
                    parent.children.push(child);
                }
                XmlEvent::Characters(data) => {
                    parent.value = Some(data);
                }
                XmlEvent::EndElement { .. } => {
                    break;
                }
                _ => panic!("Unhandled event: {:?}", data),
            },
            Err(e) => {
                panic!("Unable to parse xml: {}", e);
            }
        }
    }
}
