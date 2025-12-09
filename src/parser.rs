use crate::bytes::{line_ending, literal_to_u64};
use crate::error::error_kind::INVALID_PDF_FILE;
use crate::objects::{Entry, Xref};
use crate::sequence::Sequence;
use crate::vpdf::PDFVersion;


pub(crate) fn parse_version(sequence: &mut impl Sequence) -> crate::error::Result<PDFVersion> {
    let mut buf = [0u8; 1024];
    let n = sequence.read(&mut buf)?;
    if n < 8 {
        return Err(INVALID_PDF_FILE.into());
    }
    if buf.len() < 8
        || buf[0] != 37
        || buf[1] != 80
        || buf[2] != 68
        || buf[3] != 70
        || buf[4] != 45
    {
        return Err(INVALID_PDF_FILE.into());
    }
    let version = String::from_utf8(buf[5..8].to_vec())?;
    Ok(version.try_into()?)
}

pub(crate) fn parse_xref(sequence: &mut impl Sequence) -> crate::error::Result<Xref> {
    let offset = parse_trailer(sequence)?;
    sequence.seek(offset)?;
    // Skip xref
    sequence.read_line()?;
    let xref_meta = String::from_utf8(sequence.read_line()?)?;
    let values = xref_meta.split_whitespace().collect::<Vec<&str>>();
    let obj_num = values[0].parse::<u64>()?;
    let length = values[1].parse::<u64>()?;
    let mut entries = Vec::<Entry>::new();
    for _ in 0..length {
        let line = sequence.read_line()?;
        String::from_utf8(line)?;
    }
    Ok(Xref {
        obj_num,
        length,
        entries: vec![],
    })
}

pub(crate) fn parse_trailer(sequence: &mut impl Sequence) -> crate::error::Result<u64> {
    let size = sequence.size()?;
    let pos = if size > 1024 { size - 1024 } else { 0 };
    let mut buf = [0u8; 1024];
    sequence.seek(pos)?;
    let n = sequence.read(&mut buf)?;
    let mut list = Vec::<u8>::new();
    let mut index = 0;
    for i in (0..n).rev() {
        // 't'
        let b = buf[i];
        if b == 102 {
            break;
        }
        // '%'
        if b == 37 {
            index = i;
        } else {
            if index != 0 && !line_ending(b) {
                list.push(b);
            }
        }
    }
    list.reverse();
    Ok(literal_to_u64(&list))
}

