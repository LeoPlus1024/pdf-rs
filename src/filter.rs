use crate::error::{PDFError, Result};
use crate::objects::Stream;
use crate::utils::hex2bytes;
use flate2::read::ZlibDecoder;
use std::io::Read;

/// Decodes ASCII85 encoded data.
///
/// ASCII85 (also known as Base85) is an encoding scheme that converts binary data
/// into ASCII characters. It uses 5 ASCII characters to represent 4 bytes of data.
/// Special handling for the 'z' character which represents 4 zero bytes.
///
/// # Arguments
///
/// * `buf` - A slice of bytes containing ASCII85 encoded data
///
/// # Returns
///
/// A vector of decoded bytes
///
fn ascii_85_decode(buf: &[u8]) -> Vec<u8> {
    static ASCII_85_LOOKUP: [u8; 5] = [
        1, 1, 2, 3, 4
    ];
    let mut bytes = Vec::new();
    let l = buf.len();
    let mut t = [0u8; 5];
    let mut w = 0;
    for i in 0..l {
        let b = buf[i];
        if b == b'z' {
            bytes.extend_from_slice([0u8; 4].as_slice());
            continue;
        }
        if b == b'\n' || b == b'\r' || b == b'\t' || b == b' ' {
            continue;
        }
        t[4 - w] = b - 33;
        w += 1;
        if w == 5 || i == l - 1 {
            let mut value = 0u32;
            for (i, v) in t.iter_mut().enumerate() {
                value = value + (*v as u32) * 85u32.pow((i) as u32);
            }
            let k = value.to_be_bytes();
            bytes.extend_from_slice(&k[0..ASCII_85_LOOKUP[w - 1] as usize]);
            w = 0;
            t.fill(0);
        }
    }
    bytes
}

/// Decodes stream data using the specified filter.
///
/// This function applies the appropriate decoding filter based on the filter name.
/// Supported filters include FlateDecode, ASCIIHexDecode, and ASCII85Decode.
///
/// # Arguments
///
/// * `filter` - The name of the filter to apply
/// * `buf` - A slice of bytes containing the encoded data
///
/// # Returns
///
/// A `Result` containing the decoded byte vector, or an error if the filter is not supported
///
/// # Errors
///
/// Returns an error if the filter is not supported
fn decode_stream_xx_decode(filter: &str, buf: &[u8]) -> Result<Vec<u8>> {
    let bytes = match filter {
        "FlateDecode" => {
            let mut zlib_decoder = ZlibDecoder::new(buf);
            let mut flate_bytes = Vec::new();
            zlib_decoder.read_to_end(&mut flate_bytes)?;
            flate_bytes
        }
        "ASCIIHexDecode" => hex2bytes(buf),
        "ASCII85Decode" => ascii_85_decode(buf),
        _ => return Err(PDFError::NotSupportFilter(filter.to_string()))
    };
    Ok(bytes)
}

/// Decodes a PDF stream by applying all its filters in reverse order.
///
/// PDF streams can have multiple filters applied in sequence. This function
/// applies the filters from last to first (reverse order) to properly decode
/// the stream data.
///
/// # Arguments
///
/// * `stream` - A reference to the Stream to decode
///
/// # Returns
///
/// A `Result` containing the decoded byte vector, or an error if decoding fails
///
/// # Errors
///
/// Returns an error if any filter fails to decode the data
pub(crate) fn decode_stream(stream: &Stream) -> Result<Vec<u8>> {
    let filters = stream.get_filters();
    let len = filters.len();
    let mut bytes = Vec::new();
    for i in (0..len).rev() {
        let filter = &filters[i];
        let slice = if bytes.is_empty() {
            stream.as_slice()
        } else {
            bytes.as_slice()
        };
        bytes = decode_stream_xx_decode(filter, &slice)?;
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the ASCII85 decode function with various inputs.
    ///
    /// Verifies that:
    /// - The 'z' character correctly decodes to 4 zero bytes
    /// - Whitespace characters are properly ignored
    /// - Standard ASCII85 encoded strings decode correctly
    #[test]
    fn test_ascii_85_decode() {
        let bytes = ascii_85_decode(b"z");
        assert_eq!(bytes, [0u8; 4]);
        let bytes = ascii_85_decode(b"z\n");
        assert_eq!(bytes, [0u8; 4]);
        let bytes = ascii_85_decode(b"87cURDn");
        assert_eq!(bytes, b"Hello");
    }
}