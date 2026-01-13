use std::path::PathBuf;
use pdf_rs::document::PDFDocument;
use pdf_rs::error::Result;
use pdf_rs::helper::extract_page_text;

#[test]
fn document() -> Result<()> {
    let mut document = PDFDocument::open(PathBuf::from("document/pdfreference1.0.pdf"))?;
    let xrefs = document.get_xref_slice();
    assert!(!xrefs.is_empty());
    assert_eq!(document.get_page_num(), 230);
    match document.read_object(0)?{
        Some(obj) => assert!(obj.is_indirect_object()),
        _ => assert!(false),
    }
    Ok(())
}


#[test]
fn test_stream_read() -> Result<()> {
    let mut document = PDFDocument::open(PathBuf::from("document/pdfreference1.0.pdf"))?;
    match document.find_xref_index(|entry| entry.get_obj_num() == 1354) {
        Some(index) => {
            let object = document.read_object(index)?.unwrap();
            match object.as_indirect_object() {
                Some((obj_num, gen_num, obj)) => {
                    assert!(obj.is_stream());
                    assert!(obj_num == 1354 && gen_num == 0)
                }
                _ => assert!(false),
            }
        }
        _ => assert!(false),
    }
    Ok(())
}

#[test]
fn test_page_tree() -> Result<()> {
    let mut document = PDFDocument::open(PathBuf::from("document/pdfreference1.0.pdf"))?;
    let page_ids = document.get_page_ids();
    assert_eq!(page_ids.len(), document.get_page_num());
    for page_id in page_ids {
        extract_page_text(&mut document, page_id)?;
    }
    Ok(())
}