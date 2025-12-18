use std::cmp::min;
use crate::error::PDFError::XrefEntryNotFound;
use crate::error::Result;
use crate::objects::XEntry;

/// Maps a hexadecimal character to its corresponding numeric value.
///
/// This macro creates a match expression that converts hexadecimal characters
/// (0-9, a-f, A-F) to their numeric values (0-15).
///
/// # Arguments
///
/// * `$hex` - The hexadecimal character to convert
/// * `$val` - The numeric value
/// * `$char` - The character representation
macro_rules! hex_map {
    ($hex:ident,$(($val:literal, $char:literal)),+) => {
        match $hex {
            $($char => $val,)+
            _=> panic!("Invalid hex byte")
        }
    };
}

/// Generates a function to convert two hexadecimal characters to a byte value.
///
/// This macro creates the `hex2byte` function which combines two hexadecimal
/// characters (least significant and most significant) into a single byte value.
///
/// # Arguments
///
/// * `$val` - The numeric value for each hex character
/// * `$char` - The character representation for each hex digit
macro_rules! hex_convert {
    ($(($val:literal, $char:literal)),+$(,)?) => {
        /// Converts two hexadecimal characters to a single byte value.
        ///
        /// Combines a least significant byte (lsb) and most significant byte (msb)
        /// into a single u8 value.
        ///
        /// # Arguments
        ///
        /// * `lsb` - The least significant hexadecimal character
        /// * `msb` - The most significant hexadecimal character
        ///
        /// # Returns
        ///
        /// The combined byte value
        pub(crate) fn hex2byte(lsb: u8 ,msb: u8)-> u8 {
           let lsb = char::from(lsb);
           let msb = char::from(msb);
           let lv =  hex_map!(lsb, $(($val, $char)),+);
           let mv =  hex_map!(msb, $(($val, $char)),+);
           return lv | (mv<< 4);
        }
    }
}

hex_convert!(
    (0,'0'),
    (1,'1'),
    (2,'2'),
    (3,'3'),
    (4,'4'),
    (5,'5'),
    (6,'6'),
    (7,'7'),
    (8,'8'),
    (9,'9'),
    (10,'a'),
    (11,'b'),
    (12,'c'),
    (13,'d'),
    (14,'e'),
    (15,'f'),
    (10,'A'),
    (11,'B'),
    (12,'C'),
    (13,'D'),
    (14,'E'),
    (15,'F')
);

/// Converts a byte slice representing a decimal number to a u64 value.
///
/// Parses a sequence of ASCII digits and converts them to their numeric value.
///
/// # Arguments
///
/// * `bytes` - A slice of bytes representing ASCII digits ('0'-'9')
///
/// # Returns
///
/// The parsed u64 value
pub(crate) fn literal_to_u64(bytes: &[u8]) -> u64 {
    let len = bytes.len();
    let mut value: u64 = 0;
    for i in 0..len {
        let b = bytes[i] - 48;
        value = (value * 10) + b as u64;
    }
    value
}

/// Counts the number of leading line ending characters in a byte slice.
///
/// Iterates through the beginning of a byte slice and counts consecutive
/// line ending characters ('\r' or '\n').
///
/// # Arguments
///
/// * `bytes` - A slice of bytes to check for leading line endings
///
/// # Returns
///
/// The count of leading line ending characters
pub(crate) fn count_leading_line_endings(bytes: &[u8]) -> u64 {
    let mut count = 0u64;
    for i in 0..bytes.len() {
        if !line_ending(bytes[i]) {
            break;
        }
        count += 1;
    }
    count
}

/// Checks if a byte represents a line ending character.
///
/// Determines if the given byte is either a carriage return ('\r') or
/// line feed ('\n') character.
///
/// # Arguments
///
/// * `b` - The byte to check
///
/// # Returns
///
/// True if the byte is a line ending character, false otherwise
#[inline]
pub(crate) fn line_ending(b: u8) -> bool {
    b == b'\r' || b == b'\n'
}

/// Converts a hexadecimal string representation to a vector of bytes.
///
/// Takes a byte slice containing hexadecimal characters and converts pairs
/// of characters to their corresponding byte values.
///
/// # Arguments
///
/// * `bytes` - A slice of bytes containing hexadecimal characters
///
/// # Returns
///
/// A vector of bytes representing the parsed hexadecimal values
pub(crate) fn hex2bytes(bytes: &[u8])-> Vec<u8>{
    let len = bytes.len();
    let mut buf = Vec::new();
    if len == 0 {
        return buf;
    }
    for i in (0..len).step_by(2) {
        let msb = bytes[i];
        let lsb = if i < len - 1 {
            bytes[i + 1]
        } else {
            b'0'
        };
        let value = hex2byte(lsb, msb);
        buf.push(value);
    }
    buf
}

/// Utility function to dump a byte slice in hexadecimal and output it to stdout.
pub(crate) fn hexdump(bytes: &[u8]) {
    let len = bytes.len();
    let groups = len / 16 + if len % 16 == 0 { 0 } else { 1 };
    let mut hex = Vec::<String>::new();
    let mut ascii = [' '; 16];
    for group in 0..groups {
        let offset = group * 16;
        let bound = min(offset + 16, len);
        for i in offset..bound {
            hex.push(format!("{:02x}",bytes[i]));
            let chr =  bytes[i] as char;
            if chr.is_ascii_graphic() {
                ascii[i - offset] = chr;
            }else {
                ascii[i - offset] = '.';
            }
        }
        let len = hex.len();
        if len < 16 {
            hex.extend(vec!["  ".to_string(); 16 - len]);
        }
        let (left, right) = hex.split_at(8);
        println!("{:08x}  {}  {}  |{}|", offset, left.join(" "), right.join(" "), ascii.iter().collect::<String>());
        hex.clear();
        ascii.fill('.');
    }
}

/// Searches for an XRef entry that matches the given object reference.
///
/// This function iterates through the provided XRef entries to find one that
/// matches both the object number and generation number of the given reference.
///
/// # Arguments
///
/// * `xrefs` - A slice of XRef entries to search through
/// * `obj_ref` - A tuple containing the object number and generation number to search for
///
/// # Returns
///
/// * `Ok(&XEntry)` - A reference to the matching XRef entry if found
/// * `Err(Error)` - An error if no matching entry is found, with a message indicating
///                  the object number and generation number that could not be found
///
/// # Errors
///
/// Returns an error with kind PAGE_PARSE_ERROR if no XRef entry matches the given object reference.
pub(crate) fn xrefs_search(xrefs: &[XEntry], obj_ref: (u64, u64)) -> Result<&XEntry> {
    xrefs.iter()
        .find(|x| x.obj_num == obj_ref.0 && x.gen_num == obj_ref.1)
        .ok_or_else(|| XrefEntryNotFound(obj_ref.0, obj_ref.1))
}

#[test]
fn test_hex2bytes(){
    let hex = "012F3D4C".as_bytes();
    let buf = hex2bytes(hex);
    assert_eq!(buf, [0x01, 0x2F, 0x3D, 0x4c]);
    let hex = "012F3D4".as_bytes();
    // Test if the last byte is not a hex digit
    assert_eq!(hex2bytes(hex), [0x01, 0x2F, 0x3D, 0x40])
}