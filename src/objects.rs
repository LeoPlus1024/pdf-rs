use std::collections::HashMap;


pub enum PDFObject{
    /// The keywords true and false represent boolean objects with values true and false.
    Bool(bool),
    /// ## Numbers
    /// PDF provides two types of numbers, integer and real. Integers may be specified by
    /// signed or unsigned constants. Reals may only be in decimal format. Throughout
    /// this book, number means an object whose type is either integer or real.</br>
    /// `Note Exponential format for numbers (such as 1.0E3) is not supported.`
    Number(PDFNumber),
    /// ## Names
    /// A name, like a string, is a sequence of characters. It must begin with a slash fol-
    /// lowed by a letter, followed by a sequence of characters. Names may contain any
    /// characters except linefeed, carriage return, %, (, ), <, >, [, ], {, and }. Examples of
    /// names are:
    /// ```plaintext
    ///  /Name1
    ///  /ASomewhatLongerName2
    ///  /A;Name_With-various***characters?.
    /// ```
    Named(String),
    String(Vec<u8>),
    /// ## Arrays
    /// An array is a sequence of PDF objects. An array may contain a mixture of object
    /// types. An array is represented as a left square bracket ( [ ), followed by a sequence
    /// of objects, followed by a right square bracket ( ] ). An example of an array is:</br>
    /// ```plaintext
    /// [ 0 (Higgs) false 3.14 3 549 /SomeName ]
    /// ```
    Array(Vec<PDFObject>),
    /// A dictionary is an associative table containing pairs of objects. The first element of
    /// each pair is called the key and the second element is called the value. Unlike dictio-
    /// naries in the PostScript language, a key must
    /// be a name. A value can be any kind of object, including a dictionary.
    /// A dictionary is generally used to collect and tie together the attributes of a complex
    /// object, with each key–value pair specifying the name and value of an attribute.
    ///
    /// A dictionary is represented by two left angle brackets (<<), followed by a sequence
    /// of key–value pairs, followed by two right angle brackets (>>). For example:
    /// Example 4.1 Dictionary
    /// << /Type /Example /Key2 12 /Key3 (a string) >>
    /// Or, in an example of a dictionary within a dictionary:
    /// ```plaintext
    /// << /Type /AlsoAnExample
    /// /Subtype /Bad
    /// /Reason (unsure)
    /// /Version 0.01
    /// /MyInfo <<
    /// /Item1 0.4
    /// /Item2 true
    /// /LastItem (not!)
    /// /VeryLastItem (OK)
    /// >>
    /// >>
    /// ```
    /// Dictionary objects are the main building blocks of a PDF document. Many parts of
    /// a PDF document, such as pages and fonts, are represented using dictionaries. By
    /// convention, the **Type** key of such a dictionary specifies the type of object being
    /// described by the dictionary. Its value is always a name. In some cases, the **Subtype**
    /// key is used to describe a specialization of a particular type. Its value is always a
    /// name. For a font, Type is **Font** and four Subtypes exist: Type1, MMType1,
    /// Type3, and TrueType.
    Dict(HashMap<String, Option<PDFObject>>),
    Null,
    /// Any object used as an element of an array or as a value in a dictionary may be
    /// specified by either a direct object or an indirect reference. An indirect reference is a
    /// reference to an indirect object, and consists of the indirect object’s object number,
    /// generation number, and the **R** keyword:
    /// ```plaintext
    /// <indirect reference> ::=
    /// <object number>
    /// <generation number>
    /// R
    /// ```
    /// Using an indirect reference to the stream’s length, a stream could be written as:
    /// ```plaintext
    /// 7 0 obj
    /// <<
    /// /Length 8 0 R
    /// >>
    /// stream
    /// BT
    /// /F1 12 Tf
    /// 72 712 Td (A stream with an indirect Length) Tj
    /// ET
    /// endstream
    /// endobj
    /// 8 0 obj
    /// 64
    /// endobj
    /// ```
    ObjectRef(u64, u64, Box<PDFObject>),
    /// A direct object is a boolean, number, string, name, array, dictionary, stream, or null,
    /// as described in the previous sections. An indirect object is an object that has been
    /// labeled so that it can be referenced by other objects. Any type of object may be an
    /// indirect object. Indirect objects are very useful; for example, if the length of a
    /// stream is not known before it is written, the value of the stream’s **Length** key may
    /// be specified as an indirect object that is stored in the file after the stream.</br>
    /// An indirect object consists of an object identifier, a direct object, and the **endobj**
    /// keyword. The object identifier consists of an integer object number, an integer gen-
    /// eration number, and the **obj** keyword:
    /// ```plaintext
    /// <indirect object> ::=
    /// <object ID> ::=
    /// <object ID>
    /// <direct object>
    /// endobj
    /// <object number>
    /// <generation number>
    /// obj
    /// ```
    /// The combination of object number and generation number serves as a unique iden-
    /// tifier for an indirect object. Throughout its existence, an indirect object retains the
    /// object number and generation number it was initially assigned, even if the object is
    /// modified.</br>
    /// Each indirect object has a unique object number, and indirect objects are often but
    /// not necessarily numbered sequentially in the file, beginning with o
    IndirectObject(u64, u64),
    Stream,
    /// **Non-standard** pdf object only support current project parse temp storage xref  table
    Xref(PDFXref),
}

#[derive(PartialEq,Clone)]
pub enum PDFNumber {
    Signed(i64),
    Unsigned(u64),
    Real(f64),
}

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