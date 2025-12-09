use crate::error::Result;
use crate::objects::Xref;
use crate::parser::{parse_version, parse_xref};
use crate::sequence::Sequence;
use crate::vpdf::PDFVersion;
pub struct PDFDocument {
    xref: Xref,
    version: PDFVersion,
    sequence: Box<dyn Sequence>
}

impl PDFDocument {
    pub fn new(mut sequence: impl Sequence + 'static) -> Result<PDFDocument> {
        let version = parse_version(&mut sequence)?;
        let xref = parse_xref(&mut sequence)?;
        let pdf = PDFDocument {
            xref,
            version,
            sequence: Box::new(sequence),
        };
        Ok(pdf)
    }
}
