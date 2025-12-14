use std::path::PathBuf;
use pdf_rs::document::PDFDocument;
use pdf_rs::error::Result;
#[test]
fn document() -> Result<()> {
    let mut document = PDFDocument::open(PathBuf::from("document/pdfreference1.0.pdf"))?;
    let xrefs = document.get_xref_slice();
    assert!(!xrefs.is_empty());
    match document.read_object(0)?{
        Some(obj) => assert!(obj.is_indirect_object()),
        _ => assert!(false),
    }
    Ok(())
}
