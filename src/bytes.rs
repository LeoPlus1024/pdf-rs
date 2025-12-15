use std::cmp::min;

macro_rules! hex_map {
    ($hex:ident,$(($val:literal, $char:literal)),+) => {
        match $hex {
            $($char => $val,)+
            _=> panic!("Invalid hex byte")
        }
    };
}
macro_rules! hex_convert {
    ($(($val:literal, $char:literal)),+$(,)?) => {
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
pub(crate) fn literal_to_u64(bytes: &[u8]) -> u64 {
    let len = bytes.len();
    let mut value: u64 = 0;
    for i in 0..len {
        let b = bytes[i] - 48;
        value = (value * 10) + b as u64;
    }
    value
}

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

#[inline]
pub(crate) fn line_ending(b: u8) -> bool {
    b == 10 || b == 13
}


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

#[test]
fn test_hex2bytes(){
    let hex = "012F3D4C".as_bytes();
    let buf = hex2bytes(hex);
    assert_eq!(buf, [0x01, 0x2F, 0x3D, 0x4c]);
    let hex = "012F3D4".as_bytes();
    // Test if the last byte is not a hex digit
    assert_eq!(hex2bytes(hex), [0x01, 0x2F, 0x3D, 0x40])
}

