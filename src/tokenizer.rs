use crate::constants::is_key;
use crate::error::PDFError;
use crate::error::PDFError::{PDFParseError0};
use crate::error::Result;
use crate::objects::PDFNumber;
use crate::sequence::Sequence;
use crate::tokenizer::Token::{Bool, Delimiter, Eof, Id, Key, Number};
use crate::utils::{hexdump, line_ending};
use std::ops::Range;

/// Common end characters
const COMMON_END_CHARS: [char; 11] = [
    '<',
    '>',
    '(',
    ')',
    '\r',
    '\n',
    '\\',
    ' ',
    '/',
    '[',
    ']',
];

pub(crate) struct Tokenizer {
    buf: Vec<u8>,
    token_buf: Vec<Token>,
    sequence: Box<dyn Sequence>,
}

#[derive(PartialEq, Clone)]
pub(crate) enum Token {
    Id(String),
    Bool(bool),
    Key(String),
    Number(PDFNumber),
    Delimiter(String),
    Eof,
}

impl Token {
    pub(crate) fn is_numer(&self) -> bool {
        if let Number(_) = self {
            return true;
        }
        false
    }

    pub(crate) fn is_u64(&self) -> bool {
        if let Number(PDFNumber::Unsigned(_)) = self {
            true
        } else {
            false
        }
    }

    pub(crate) fn is_id(&self) -> bool {
        if let Id(_) = self { true } else { false }
    }

    pub(crate) fn to_string(&self) -> String {
        match self {
            Id(id) => id.clone(),
            Key(key) => key.clone(),
            Delimiter(delimiter) => delimiter.clone(),
            Number(PDFNumber::Unsigned(num)) => num.to_string(),
            Number(PDFNumber::Signed(num)) => num.to_string(),
            Number(PDFNumber::Real(num)) => num.to_string(),
            Bool(bool) => bool.to_string(),
            Eof => "_eof".to_string(),
        }
    }

    pub(crate) fn as_u64(&self) -> Result<u64> {
        if let Number(PDFNumber::Unsigned(num)) = self {
            return Ok(*num);
        }
        Err(PDFParseError0(format!("Token can't convert to u64:'{}'",self.to_string())))
    }

    pub(crate) fn as_u32(&self) -> Result<u32> {
        if let Number(PDFNumber::Unsigned(num)) = self {
            return Ok(*num as u32);
        }
        Err(PDFParseError0(format!("Token can't convert to u32:'{}'", self.to_string())))
    }
    
    pub(crate) fn as_u16(&self) -> Result<u16> {
        if let Number(PDFNumber::Unsigned(num)) = self {
            return Ok(*num as u16);
        }
        Err(PDFParseError0(format!("Token can't convert to u16:'{}'", self.to_string())))
    }

    pub(crate) fn except<F>(self, func: F) -> Result<Self>
    where
        F: Fn(&Token) -> bool,
    {
        let m = func(&self);
        if !m {
            return Err(PDFError::PDFParseError("Token kind mistake."));
        }
        Ok(self)
    }

    pub(crate) fn key_was(&self, str: &str) -> bool {
        if let Key(key) = self {
            return key == str;
        }
        false
    }
    
    /// Returns true if the token is a delimiter.
    pub(crate) fn is_delimiter(&self) -> bool {
        if let Delimiter(_) = self { true } else { false }
    }

    /// Returns true if the token is a delimiter and the delimiter is the specified string.
    pub(crate) fn delimiter_was(&self, str: &str) -> bool {
        if let Delimiter(delimiter) = self {
            return delimiter == str;
        }
        false
    }
}

impl Tokenizer {
    pub(crate) fn new(sequence: impl Sequence + 'static) -> Self {
        Self {
            sequence: Box::new(sequence),
            buf: Vec::new(),
            token_buf: Vec::new(),
        }
    }

    pub(crate) fn check_next_token<F>(&mut self, func: F) -> Result<bool>
    where
        F: FnMut(&Token) -> bool,
    {
        self.check_next_token0(true, func)
    }

    pub(crate) fn check_next_token0<F>(&mut self, cache: bool,mut func: F) -> Result<bool>
    where
        F: FnMut(&Token) -> bool,
    {
        let token = if let Some(chr) = self.next_chr()? {
            self.chr2token(chr)?
        } else {
            Eof
        };
        let m = func(&token);
        if !m || cache {
            self.token_buf.push(token);
        }
        Ok(m)
    }

    pub(crate) fn next_token(&mut self) -> Result<Token> {
        let token_buf = &mut self.token_buf;
        if !token_buf.is_empty() {
            return Ok(token_buf.remove(0));
        }
        let option = self.next_chr()?;
        if option.is_none() {
            return Ok(Eof);
        }
        self.chr2token(option.unwrap())
    }

    fn chr2token(&mut self, chr: char) -> Result<Token> {
        let token = match chr {
            '<' => match self.next_chr_was('<') {
                true => Delimiter(String::from("<<")),
                false => Delimiter(String::from('<')),
            },
            '>' => match self.next_chr_was('>') {
                true => Delimiter(String::from(">>")),
                false => Delimiter(String::from(">")),
            },
            '/' | '(' | ')' | '[' | ']' => Delimiter(chr.into()),
            '+' | '-' | '.' => self.num_deco(chr)?,
            chr => {
                // If the character is a digit, then we need to read the number
                if chr.is_digit(10) {
                    self.num_deco(chr)?
                }
                // Identifier
                else {
                    let range = self.loop_util(&COMMON_END_CHARS, |_c| Ok(false))?;
                    let mut buf = self.buf.drain(range).collect::<Vec<u8>>();
                    buf.insert(0, chr as u8);
                    let text = String::from_utf8(buf)?;
                    if is_key(text.as_str()) {
                        return Ok(Key(text));
                    }
                    Id(text)
                }
            }
        };
        Ok(token)
    }

    fn num_deco(&mut self, chr: char) -> Result<Token> {
        let mut is_real = chr == '.';
        let range = self.loop_util(&COMMON_END_CHARS, |c| {
            let is_dot = c == '.';
            // If the character is a dot, then we need to check if it is a valid real number
            if is_dot {
                if is_real {
                    return Err(PDFError::PDFParseError("Multiple dot was found in real number."));
                }
                is_real = true;
            } else {
                let is_digit = c.is_digit(10);
                if !is_digit {
                    return Err(PDFError::PDFParseError0(format!("Invalid number character: {:0x}", c as u8)));
                }
            }
            return Ok(false);
        })?;
        let mut bytes = self.buf.drain(range).collect::<Vec<u8>>();
        bytes.insert(0, chr as u8);
        let text = String::from_utf8(bytes)?;
        let value = if is_real {
            PDFNumber::Real(text.parse::<f64>()?)
        } else {
            let signed = chr == '-';
            if signed {
                PDFNumber::Signed(text.parse::<i64>()?)
            } else {
                PDFNumber::Unsigned(text.parse::<u64>()?)
            }
        };
        Ok(Number(value))
    }

    pub(crate) fn loop_util<F>(&mut self, end_chars: &[char], mut func: F) -> Result<Range<usize>>
    where
        F: FnMut(char) -> Result<bool>,
    {
        let mut index = 0usize;
        let buf = &mut self.buf;
        'ext: loop {
            // If index is equal to buffer length, then we need to read more data
            if index == buf.len() {
                let mut bytes = [0u8; 1024];
                let n = self.sequence.read(&mut bytes)?;
                if n == 0 {
                    return Err(PDFError::EOFError);
                }
                buf.extend_from_slice(&bytes[0..n]);
            }
            let len = buf.len();
            for i in index..len {
                let chr = char::from(buf[i]);
                if end_chars.contains(&chr) || func(chr)? {
                    index = i;
                    break 'ext;
                }
            }
            index = len;
        }
        Ok(0..index)
    }

    fn next_chr(&mut self) -> Result<Option<char>> {
        let option = match self.next_chr0(|_| true)? {
            None => None,
            Some((_, chr)) => Some(chr),
        };
        Ok(option)
    }

    fn next_chr_was(&mut self, chr: char) -> bool {
        match self.next_chr0(|c| c == chr) {
            Ok(Some((equal, _))) => equal,
            _ => false,
        }
    }

    /// Read next byte
    fn next_chr0<F>(&mut self, func: F) -> Result<Option<(bool, char)>>
    where
        F: Fn(char) -> bool,
    {
        let buf = &mut self.buf;
        let mut bytes = [0u8; 1024];
        if buf.is_empty() {
            let n = self.sequence.read(&mut bytes)?;
            if n == 0 {
                return Ok(None);
            }
            buf.extend_from_slice(&bytes[0..n]);
        }
        let len = buf.len();
        let mut skip_cunt = 0;
        for i in 0..len {
            let b = buf[i];
            if line_ending(b) || b == b' ' {
                skip_cunt += 1;
            } else {
                break;
            }
        }
        if skip_cunt > 0 {
            buf.drain(0..skip_cunt);
        }
        // If buffer is empty, then we need to read more data
        if buf.is_empty() {
            return self.next_chr0(func);
        }
        let b = buf[0];
        let chr = char::from(b);
        let equal = func(chr);
        if equal {
            buf.remove(0);
        }
        Ok(Some((equal, chr)))
    }

    pub(crate) fn seek(&mut self, offset: u64) -> Result<u64> {
        let n = self.sequence.seek(offset)?;
        self.token_buf.clear();
        self.buf.clear();
        Ok(n)
    }

    pub(crate) fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>> {
        let buf_len = self.buf.len();
        let buf = if buf_len >= len {
            self.buf.drain(0..len).collect::<Vec<u8>>()
        } else {
            let diff = len - buf_len;
            let mut bytes = vec![0u8; diff];
            let n = self.sequence.read(&mut bytes)?;
            let mut buf = Vec::<u8>::new();
            buf.extend_from_slice(&self.buf);
            buf.extend_from_slice(&bytes[0..n]);
            // Should clear buffer
            self.buf.clear();
            buf
        };
        // #[cfg(debug_assertions)]
        // {
        //     hexdump(&buf);
        // }
        // Clear token buffer
        self.token_buf.clear();
        Ok(buf)
    }

    pub(crate) fn drain_from_buf(&mut self, range: Range<usize>) -> Vec<u8> {
        self.buf.drain(range).collect()
    }

    pub(crate) fn remove_buf_len(&mut self, len: usize) {
        self.buf.drain(0..len);
    }

    /// Skip CRLF
    ///
    /// Return the number of bytes skipped
    pub(crate) fn skip_crlf(&mut self) -> Result<usize> {
        let range = self.loop_util(&[], |chr| Ok(chr != '\r' && chr != '\n'))?;
        let mut count = 0usize;
        if range.start < range.end {
            count = range.end - range.start;
            self.remove_buf_len(range.end);
        }
        Ok(count)
    }
}
