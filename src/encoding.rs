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
