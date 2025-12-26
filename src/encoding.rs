/// Enum for pdf predefined encodings
pub(crate) enum PreDefinedEncoding {
    MacRoman,
    Standard,
    WinAnsi,
    PDFDoc,
    MacExpert,
}
type EncodingEntry = (u8, &'static str, Option<char>);

include!("../encoding/MacRoman");
include!("../encoding/Standard");
include!("../encoding/WinAnsi");
include!("../encoding/PDFDoc");
include!("../encoding/MacExpert");


pub(crate) fn mapper_chr_from_u8(bytes: u8, encoding: &PreDefinedEncoding) -> Option<char> {
    match encoding {
        PreDefinedEncoding::PDFDoc => {
            PDF_DOC_ENCODING[bytes as usize]
        }
        _ => {
            let look_table = match encoding {
                PreDefinedEncoding::MacRoman => MAC_ROMAN_ENCODING,
                PreDefinedEncoding::Standard => STANDARD_ENCODING,
                PreDefinedEncoding::WinAnsi => WIN_ANSI_ENCODING,
                PreDefinedEncoding::MacExpert => MAC_EXPERT_ENCODING,
                _ => return None
            };
            look_table.iter()
                .filter(|e| e.0 == bytes)
                .map(|e| e.2)
                .next()?
        }
    }
}