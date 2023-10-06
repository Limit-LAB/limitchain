use anyhow::anyhow;
use quick_xml::{events::Event, Reader};
use serde_json::json;

use super::{Document, DocumentLoader};

pub struct XMLLoader {}

use std::io::BufRead;
fn reader_to_docs<R>(mut reader: Reader<R>) -> anyhow::Result<Vec<Document>>
where
    R: BufRead,
{
    let mut result = Vec::new();

    let mut buf = Vec::new();
    let mut tag_stack = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(event) => {
                match event {
                    Event::Start(tag) => {
                        let tag_name = String::from_utf8(tag.name().0.to_vec())?;
                        let mut attrs = json!({});
                        for attr in tag.attributes() {
                            let attr = attr?;
                            let key = String::from_utf8(attr.key.0.to_vec())?;
                            // try parse value as utf8 string
                            if let Ok(value) = String::from_utf8(attr.value.to_vec()) {
                                attrs.as_object_mut().unwrap().insert(key, value.into());
                            } else {
                                let value = attr.value.to_vec();
                                attrs.as_object_mut().unwrap().insert(key, value.into());
                            }
                        }
                        if attrs == json!({}) {
                            tag_stack.push(json!({
                                "name": tag_name,
                            }));
                        } else {
                            tag_stack.push(json!({
                                "name": tag_name,
                                "attrs": attrs,
                            }));
                        }
                    }
                    Event::End(tag) => {
                        if tag_stack.last().is_some() {
                            tag_stack.pop();
                        }
                    }
                    Event::Text(txt) => result.push(Document {
                        text: txt.unescape().map_or("".to_string(), |s| s.to_string()),
                        meta: json!({
                            "tag_stack": tag_stack.clone(),
                        }),
                    }),
                    Event::Eof => {
                        break;
                    }
                    _ => {}
                }
            }
            Err(e) => {
                println!("{:#?}", e);
                break;
            }
        }
    }
    Ok(result
        .into_iter()
        .filter(|s| s.text.trim().len() > 0)
        .collect())
}

impl DocumentLoader for XMLLoader {
    fn load_mem(&self, mem: &str) -> anyhow::Result<Vec<Document>> {
        let reader = Reader::from_str(mem);
        reader_to_docs(reader)
    }

    fn load_file(&self, path: &str) -> anyhow::Result<Vec<Document>> {
        let reader = Reader::from_file(path)?;
        reader_to_docs(reader)
    }
}

#[test]
fn test_xml_loader_mem() {
    let loader = XMLLoader {};
    let doc = loader
        .load_mem(
            r#"
    <html>
        <head>
            <title>hello</title>
        </head>
        <body>
            <div class="content">
                <p>hello world</p>
            </div>
        </body>
    </html>
    "#,
        )
        .unwrap();
    println!("{:#?}", doc);
}
