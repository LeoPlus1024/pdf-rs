use std::str::FromStr;
use pdf_rs::error::Result;
use pdf_rs::vpdf::PDFVersion;
mod common;

#[test]
fn vpdf_test() -> Result<()> {
    assert_eq!(PDFVersion::V1_0, PDFVersion::from_str("1.0")?);
    assert_eq!(PDFVersion::V1_1, PDFVersion::from_str("1.1")?);
    assert_eq!(PDFVersion::V1_2, PDFVersion::from_str("1.2")?);
    assert_eq!(PDFVersion::V1_3, PDFVersion::from_str("1.3")?);
    assert_eq!(PDFVersion::V1_4, PDFVersion::from_str("1.4")?);
    assert_eq!(PDFVersion::V1_5, PDFVersion::from_str("1.5")?);
    assert_eq!(PDFVersion::V1_6, PDFVersion::from_str("1.6")?);
    assert_eq!(PDFVersion::V1_7, PDFVersion::from_str("1.7")?);
    assert_eq!(PDFVersion::V2_0, PDFVersion::from_str("2.0")?);
    Ok(())
}
