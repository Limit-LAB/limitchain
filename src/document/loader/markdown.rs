use super::{xml::XMLLoader, Document, DocumentLoader};

/// well as we all know that AI really knows how to read markdown
struct MarkdownLoader {}

impl DocumentLoader for MarkdownLoader {
    fn load_mem(&self, mem: &str) -> anyhow::Result<Vec<Document>> {
        Ok(vec![Document {
            text: mem.to_string(),
            meta: serde_json::json!({}),
        }])
    }
    fn load_file(&self, path: &str) -> anyhow::Result<Vec<Document>> {
        let str = std::fs::read_to_string(path)?;
        Ok(vec![Document {
            text: str,
            meta: serde_json::json!({}),
        }])
    }
}

#[test]
fn test_markdown_loader() {
    let loader = MarkdownLoader {};
    let docs = loader.load_mem(r#"# hello"#).unwrap();
    println!("{:#?}", docs);
}

#[test]
fn test_markdown_loader_file() {
    let loader = MarkdownLoader {};
    let doc = loader.load_file("test_utils/md_example.md").unwrap();
    println!("{:#?}", doc);
    println!(
        "{:#?}",
        doc.iter().map(|s| s.text.clone()).collect::<Vec<String>>()
    );
}
