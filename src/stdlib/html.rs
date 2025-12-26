use crate::runtime::value::Value;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::Arc;

pub fn parse_html(html_content: &str) -> Result<Value, String> {
    Ok(create_page_object(html_content.to_string()))
}

pub fn create_html_module() -> Value {
    let mut methods = HashMap::new();

    methods.insert(
        "Parse".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() != 1 {
                return Err("HTML.Parse requires 1 argument (html_string)".to_string());
            }

            let html_content = args[0].to_display_string();
            parse_html(&html_content)
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}

fn create_page_object(html: String) -> Value {
    let _document = Html::parse_document(&html);

    let doc_string = html.clone();
    let mut page_methods = HashMap::new();

    page_methods.insert(
        "SelectText".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 1 {
                return Err("Page.SelectText requires 1 argument (selector)".to_string());
            }

            let selector_str = args[0].to_display_string();
            let selector = Selector::parse(&selector_str)
                .map_err(|_| format!("Invalid CSS selector: {}", selector_str))?;

            let fragment = Html::parse_document(&doc_string);

            let mut results = Vec::new();
            for element in fragment.select(&selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                results.push(Value::String(text));
            }

            Ok(Value::List(Arc::new(std::sync::RwLock::new(results))))
        }))),
    );

    let doc_string_2 = html.clone();

    page_methods.insert(
        "SelectAttr".to_string(),
        Value::NativeFunction(Arc::new(Box::new(move |args| {
            if args.len() != 2 {
                return Err(
                    "Page.SelectAttr requires 2 arguments (selector, attribute)".to_string()
                );
            }

            let selector_str = args[0].to_display_string();
            let attr_name = args[1].to_display_string();

            let selector = Selector::parse(&selector_str)
                .map_err(|_| format!("Invalid CSS selector: {}", selector_str))?;

            let fragment = Html::parse_document(&doc_string_2);

            let mut results = Vec::new();
            for element in fragment.select(&selector) {
                if let Some(val) = element.value().attr(&attr_name) {
                    results.push(Value::String(val.to_string()));
                }
            }

            Ok(Value::List(Arc::new(std::sync::RwLock::new(results))))
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(page_methods)))
}
