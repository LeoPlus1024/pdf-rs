use std::collections::HashMap;

#[derive(PartialEq, Clone)]
pub enum PDFNumber {
    Signed(i64),
    Unsigned(u64),
    Real(f64),
}

#[derive(Clone)]
pub struct XEntry {
    /// The value of the entry.
    pub(crate) value: u64,
    /// The entry is either in use or deleted.
    pub(crate) using: bool,
    /// The object number of the entry.
    pub(crate) obj_num: u64,
    /// The generation number of the entry.
    pub(crate) gen_num: u64,
}

pub struct Dictionary {
    entries: HashMap<String, PDFObject>,
}

pub struct Stream {
    metadata: Dictionary,
}

pub enum PDFObject {
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
    Dict(Dictionary),
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
    ObjectRef(u64, u64),
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
    IndirectObject(u64, u64, Box<PDFObject>),
    /// ## Streams
    /// A stream, like a string, is a sequence of characters. However, an application can
    /// read a small portion of a stream at a time, while a string must be read in its entirety.
    /// For this reason, objects with potentially large amounts of data, such as images and
    /// page descriptions, are represented as streams.
    ///
    /// A stream consists of a dictionary that describes a sequence of characters, followed
    /// by the keyword stream, followed by one or more lines of characters, followed by
    /// the keyword endstream.
    /// ```plaintext
    /// <stream> ::= <dictionary>
    /// stream
    /// {<lines of characters>}*
    /// endstream
    /// ```
    Stream(Stream),
}

impl PDFObject {
    /// Returns true if the object is a boolean.
    pub fn is_bool(&self) -> bool {
        match self {
            PDFObject::Bool(_) => true,
            _ => false,
        }
    }
    /// Returns the boolean value of the object if it is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PDFObject::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns true if the object is a number.
    pub fn is_number(&self) -> bool {
        match self {
            PDFObject::Number(_) => true,
            _ => false,
        }
    }
    /// Returns the number value of the object if it is a number.
    pub fn as_number(&self) -> Option<&PDFNumber> {
        match self {
            PDFObject::Number(n) => Some(n),
            _ => None,
        }
    }
    /// Returns true if the object is a string.
    pub fn is_string(&self) -> bool {
        match self {
            PDFObject::String(_) => true,
            _ => false,
        }
    }
    /// Returns the string byte sequence of the object if it is a string.
    pub fn as_str_bytes(&self) -> Option<&[u8]> {
        match self {
            PDFObject::String(buf) => Some(buf),
            _ => None,
        }
    }

    /// Returns the string value of the object if it is a string.
    pub fn is_array(&self) -> bool {
        match self {
            PDFObject::Array(_) => true,
            _ => false,
        }
    }
    /// Returns the array of objects if it is an array.
    pub fn as_array(&self) -> Option<&[PDFObject]> {
        match self {
            PDFObject::Array(a) => Some(a),
            _ => None,
        }
    }
    /// Returns true if the object is a dictionary.
    pub fn is_dict(&self) -> bool {
        match self {
            PDFObject::Dict(_) => true,
            _ => false,
        }
    }
    /// Returns the dictionary if it is one.
    pub fn as_dict(&self) -> Option<&Dictionary> {
        match self {
            PDFObject::Dict(d) => Some(d),
            _ => None,
        }
    }
    /// Returns true if the object is an indirect object.
    pub fn is_object_ref(&self) -> bool {
        match self {
            PDFObject::ObjectRef(_, ..) => true,
            _ => false,
        }
    }
    /// Returns the object reference if it is one.
    pub fn as_object_ref(&self) -> Option<(u64, u64)> {
        match self {
            PDFObject::ObjectRef(n, g) => Some((*n, *g)),
            _ => None,
        }
    }

    /// Returns true if the object is an indirect object.
    pub fn is_indirect_object(&self) -> bool {
        match self {
            PDFObject::IndirectObject(_, _, _) => true,
            _ => false,
        }
    }
    /// Returns the indirect object if it is one.
    pub fn as_indirect_object(&self) -> Option<(u64, u64, &PDFObject)> {
        match self {
            PDFObject::IndirectObject(n, g, data) => Some((*n, *g, data)),
            _ => None,
        }
    }

    /// Returns true if the object is null.
    pub fn is_null(&self) -> bool {
        match self {
            PDFObject::Null => true,
            _ => false,
        }
    }
    /// Returns true if the object is a stream.
    pub fn is_stream(&self)->bool{
        match self {
            PDFObject::Stream(_) => true,
            _ => false,
        }
    }

    /// Returns the stream if it is one.
    pub fn as_stream(&self)->Option<&Stream>{
        match self {
            PDFObject::Stream(s) => Some(s),
            _ => None,
        }
    }

}

impl Dictionary {
    /// Creates a new dictionary with the given entries.
    pub(crate) fn new(entries: HashMap<String, PDFObject>) -> Self {
        Dictionary { entries }
    }
    /// Returns the value of the entry with the given key.
    pub fn get(&self, key: &str)-> Option<&PDFObject> {
        self.entries.get(key)
    }
}

impl XEntry {
    pub(crate) fn new(obj_num: u64, gen_num: u64, value: u64, using: bool) -> Self {
        XEntry {
            obj_num,
            gen_num,
            using,
            value,
        }
    }
    /// Returns the object number of the entry.
    pub fn get_obj_num(&self)->u64{
        self.obj_num
    }
    /// Returns the generation number of the entry.
    pub fn get_gen_num(&self)->u64{
        self.gen_num
    }
    /// Returns true if the entry is currently being used.
    pub fn is_using(&self) -> bool {
        self.using
    }

    /// Returns true if the entry is freed.
    pub fn is_freed(&self)->bool{
        !self.using
    }
    /// Returns the value of the entry.
    pub fn get_value(&self)->u64{
        self.value
    }
}

impl Stream {
    /// Creates a new stream with the given metadata.
    pub(crate) fn new(metadata: Dictionary,buf:Vec<u8>) -> Self {
        Stream { metadata }
    }
}