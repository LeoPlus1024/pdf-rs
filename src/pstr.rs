use crate::encoding::{PreDefinedEncoding, mapper_chr_from_u8};
use crate::objects::PDFString;
use crate::utils::{hex2byte, hex2bytes};

#[macro_export] macro_rules! convert_glyph_from_dict {
    ($dict:ident,$key:ident,$encoding:expr) => {
        match $dict.get($key) {
            Some(PDFObject::String(pstr)) => Some(convert_glyph_text(pstr, $encoding)),
            _ => None,
        }
    };
}

pub(crate) fn convert_glyph_text(str: &PDFString, encoding: &PreDefinedEncoding) -> String {
    // If the string is utf16be, convert it to string
    if str.is_utf16be() {
        return to_utf16be_text(str);
    }
    let buf = str.get_buf();
    let mut chr_buf = Vec::<char>::new();
    for b in buf {
        let t = mapper_chr_from_u8(*b, encoding);
        if let Some(chr) = t {
            chr_buf.push(chr);
        }
    }
    chr_buf.iter().collect()
}

// Convert utf16be byte array to string
fn to_utf16be_text(str: &PDFString) -> String {
    let buf = str.get_buf();
    // UTF-16BE strings in PDF start with a BOM (0xFE 0xFF)
    // Skip the BOM and convert the remaining bytes to UTF-16
    if buf.len() >= 2 && buf.len() % 2 == 0 {
        let utf16_data: Vec<u16> = buf[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();
        String::from_utf16(&utf16_data).unwrap_or_default()
    } else {
        // todo Convert byte to hex string?
        String::new()
    }
}