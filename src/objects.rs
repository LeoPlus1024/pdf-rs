use std::collections::HashMap;


pub enum PDFObject{
    Bool(PDFBool),
    Number(PDFNumber),
    Named(PDFNamed),
    String(Vec<u8>),
    Array(PDFArray),
    Dict(PDFDict),
    Null,
    DirectObject(PDFDirectObject),
    IndirectObject(PDFIndirectObject),
    Stream(PDFStream),
    Xref(PDFXref),
}

pub struct PDFBool {
    value: bool,
}

#[derive(Eq, Hash, PartialEq)]
pub struct PDFNamed {
    pub(crate) name: String,
}

#[derive(PartialEq,Clone)]
pub enum PDFNumber {
    Signed(i64),
    Unsigned(u64),
    Real(f64),
}

pub struct PDFArray {
    pub(crate) elements: Vec<PDFObject>,
}

pub struct PDFDict {
    pub(crate) entries: HashMap<PDFNamed, Option<PDFObject>>,
}

pub struct PDFDirectObject {
    pub(crate) obj_num: u64,
    pub(crate) gen_num: u64,
    pub(crate) metadata: Box<PDFObject>,
}

pub struct PDFIndirectObject {
    pub(crate) obj_num: u64,
    pub(crate) gen_num: u64,
}

pub struct PDFStream;


pub(crate) enum EntryState {
    Using(u64),
    Deleted(u64)
}


pub struct Entry {
    pub(crate) state: EntryState,
    /// The maximum generation number is 65535. Once that number is reached, that entry in the crossreference table will not be reused.
    pub(crate) gen_num: u64,
}
pub struct PDFXref {
    pub(crate) obj_num: u64,
    pub(crate) length: u64,
    pub(crate) entries: Vec<Entry>,
}

pub struct Trailer {
    pub(crate) metadata: PDFDict,
    pub(crate) byte_offset: u64,
}