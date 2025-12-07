use crate::runtime::value::Value;
use std::collections::HashMap;
use std::sync::Arc;
use sxd_document::parser;
use sxd_xpath::{ Value as XPathValue, evaluate_xpath };

pub fn parse_xml(xml_content: &str) -> Result<Value, String> {
    match parser::parse(xml_content) {
        Ok(_) => Ok(create_document_object(xml_content.to_string())),
        Err(e) => Err(format!("XML Parse Error: {}", e)),
    }
}

pub fn create_xml_module() -> Value {
    let mut methods = HashMap::new();

    methods.insert(
        "Parse".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(|args| {
                    if args.len() != 1 {
                        return Err("XML.Parse requires 1 argument (xml_string)".to_string());
                    }

                    let xml_content = args[0].to_display_string();
                    parse_xml(&xml_content)
                })
            )
        )
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}

fn create_document_object(xml: String) -> Value {
    let doc_string = xml.clone();
    let mut doc_methods = HashMap::new();

    doc_methods.insert(
        "XPath".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(move |args| {
                    if args.len() != 1 {
                        return Err(
                            "Document.XPath requires 1 argument (xpath_expression)".to_string()
                        );
                    }

                    let xpath_expr = args[0].to_display_string();

                    let package = match parser::parse(&doc_string) {
                        Ok(p) => p,
                        Err(e) => {
                            return Err(format!("XML Parse Error: {}", e));
                        }
                    };

                    let document = package.as_document();

                    let xpath_result = match evaluate_xpath(&document, &xpath_expr) {
                        Ok(result) => result,
                        Err(e) => {
                            return Err(format!("XPath Error: {}", e));
                        }
                    };

                    Ok(convert_xpath_to_object(xpath_result))
                })
            )
        )
    );

    let doc_string_2 = xml.clone();
    doc_methods.insert(
        "Text".to_string(),
        Value::NativeFunction(
            Arc::new(
                Box::new(move |args| {
                    if !args.is_empty() {
                        return Err("Document.Text requires no arguments".to_string());
                    }

                    let package = match parser::parse(&doc_string_2) {
                        Ok(p) => p,
                        Err(e) => {
                            return Err(format!("XML Parse Error: {}", e));
                        }
                    };

                    let document = package.as_document();
                    let root = document.root();

                    let text = extract_text_recursive(root.children());
                    Ok(Value::String(text))
                })
            )
        )
    );

    Value::Map(Arc::new(std::sync::RwLock::new(doc_methods)))
}

fn convert_xpath_to_object(xpath_val: XPathValue) -> Value {
    match xpath_val {
        XPathValue::Boolean(b) => Value::Boolean(b),
        XPathValue::Number(n) => {
            Value::from_number_string(&n.to_string()).unwrap_or(Value::default_number())
        }
        XPathValue::String(s) => Value::String(s),
        XPathValue::Nodeset(nodeset) => {
            let mut results = Vec::new();
            for node in nodeset.document_order() {
                let text = match node {
                    sxd_xpath::nodeset::Node::Element(el) => extract_element_text(el),
                    sxd_xpath::nodeset::Node::Text(text) => text.text().to_string(),
                    sxd_xpath::nodeset::Node::Attribute(attr) => attr.value().to_string(),
                    _ => String::new(),
                };

                if !text.is_empty() {
                    results.push(Value::String(text));
                }
            }
            Value::List(Arc::new(std::sync::RwLock::new(results)))
        }
    }
}

fn extract_element_text(element: sxd_document::dom::Element) -> String {
    let mut text = String::new();
    for child in element.children() {
        match child {
            sxd_document::dom::ChildOfElement::Element(el) => {
                text.push_str(&extract_element_text(el));
            }
            sxd_document::dom::ChildOfElement::Text(t) => {
                text.push_str(t.text());
            }
            _ => {}
        }
    }
    text
}

fn extract_text_recursive(children: Vec<sxd_document::dom::ChildOfRoot>) -> String {
    let mut text = String::new();
    for child in children {
        match child {
            sxd_document::dom::ChildOfRoot::Element(el) => {
                text.push_str(&extract_element_text(el));
            }
            _ => {}
        }
    }
    text
}
