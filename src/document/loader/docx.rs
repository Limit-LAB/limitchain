use std::{fs::File, io::Read};

use super::{xml::XMLLoader, Document, DocumentLoader};

struct DocxLoader {}

impl DocumentLoader for DocxLoader {
    fn load_mem(&self, mem: &str) -> anyhow::Result<Vec<Document>> {
        let reader = std::io::BufReader::new(std::io::Cursor::new(mem.as_bytes()));
        let mut zip = zip::ZipArchive::new(reader)?;
        let mut xml = zip.by_name("word/document.xml")?;
        let mut xml_str = String::new();
        xml.read_to_string(&mut xml_str)?;
        let loader = XMLLoader {};
        loader.load_mem(&xml_str)
    }
    fn load_file(&self, path: &str) -> anyhow::Result<Vec<Document>> {
        let reader = File::open(path)?;
        let mut zip = zip::ZipArchive::new(reader)?;
        let mut xml = zip.by_name("word/document.xml")?;
        let mut xml_str = String::new();
        xml.read_to_string(&mut xml_str)?;
        let loader = XMLLoader {};
        loader.load_mem(&xml_str)
    }
}

#[test]
fn test_docx_loader() {
    let loader = DocxLoader {};
    let doc = loader.load_file("test_utils/md_example.docx").unwrap();
    // println!("{:#?}", doc);
    println!(
        "{:#?}",
        doc.iter().map(|s| s.text.clone()).collect::<Vec<String>>()
    );
}
