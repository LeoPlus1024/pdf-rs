use crate::encoding::{PreDefinedEncoding, mapper_chr_from_u8};
use crate::objects::PDFString;

#[macro_export] macro_rules! convert_glyph_from_dict {
    ($dict:ident,$key:ident,$encoding:expr) => {
        match $dict.get($key) {
            Some(PDFObject::String(pstr)) => Some(convert_glyph_text(pstr, $encoding)),
            _ => None,
        }
    };
}

pub(crate) fn convert_glyph_text(str: &PDFString, encoding: &PreDefinedEncoding) -> String {
    let buf = str.get_buf();
    let mut chr_buf = Vec::<char>::new();
    for b in buf {
        let t = mapper_chr_from_u8(*b - 1, encoding);
        if let Some(chr) = t {
            chr_buf.push(chr);
        }
    }
    chr_buf.iter().collect()
}
