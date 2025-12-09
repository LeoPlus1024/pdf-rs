use ipdf::document::PDFDocument;
use ipdf::error::Result;
use ipdf::sequence::{FileSequence};
use ipdf::vpdf::PDFVersion;

#[test]
fn vpdf_test() -> Result<()> {
    assert_eq!(PDFVersion::V1_0, "1.0".try_into()?);
    assert_eq!(PDFVersion::V1_1, "1.1".try_into()?);
    assert_eq!(PDFVersion::V1_2, "1.2".try_into()?);
    assert_eq!(PDFVersion::V1_3, "1.3".try_into()?);
    assert_eq!(PDFVersion::V1_4, "1.4".try_into()?);
    assert_eq!(PDFVersion::V1_5, "1.5".try_into()?);
    assert_eq!(PDFVersion::V1_6, "1.6".try_into()?);
    assert_eq!(PDFVersion::V1_7, "1.7".try_into()?);
    assert_eq!(PDFVersion::V2_0, "2.0".try_into()?);
    Ok(())
}

#[test]
fn parse_pdf() -> Result<()> {
    let file = std::fs::File::open("document/pdfreference1.0.pdf")?;
    let sequence = FileSequence::new(file);
    let t = PDFDocument::new(sequence)?;
    Ok(())
}
