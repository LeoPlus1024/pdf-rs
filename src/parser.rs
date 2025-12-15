use crate::constants::pdf_key::{END_OBJ, END_STREAM, OBJ, R, STREAM};
use crate::constants::*;
use crate::error::error_kind::{EOF, EXCEPT_TOKEN, ILLEGAL_STREAM, ILLEGAL_TOKEN, STR_NOT_ENCODED};
use crate::error::{Error, Result};
use crate::objects::{Dictionary, PDFNumber, PDFObject, Stream, XEntry};
use crate::tokenizer::Token::{Delimiter, Id, Key, Number};
use crate::tokenizer::{Token, Tokenizer};
use std::collections::HashMap;
use crate::bytes::hex2bytes;

pub(crate) fn parse(mut tokenizer: &mut Tokenizer) -> Result<PDFObject>
{
    let token = tokenizer.next_token()?;
    let object = parser0(&mut tokenizer, token)?;
    Ok(object)
}

fn parser0(tokenizer: &mut Tokenizer, token: Token) -> Result<PDFObject> {
    match token {
        Delimiter(delimiter) => match delimiter.as_str() {
            "<<" =>{
                let dict = parse_dict(tokenizer)?;
                // If the next token is stream, then it is a stream
                if tokenizer.check_next_token0(false,|token| token.key_was(STREAM))? {
                    return parse_stream(tokenizer, dict);
                }
                Ok(PDFObject::Dict(dict))
            }
            "[" => parse_array(tokenizer),
            "/" => parse_named(tokenizer),
            "<" | "(" => parse_string(tokenizer, delimiter == "("),
            &_ => todo!(),
        },
        Key(key) => match key.as_str() {
            pdf_key::NULL => Ok(PDFObject::Null),
            pdf_key::TURE => Ok(PDFObject::Bool(true)),
            pdf_key::FALSE => Ok(PDFObject::Bool(false)),
            &_ => Err(Error::new(ILLEGAL_TOKEN, format!("Key '{}' not implemented", key))),
        }
        Number(number) => match number {
            PDFNumber::Unsigned(value) => {
                let is_num = tokenizer.check_next_token(|token| token.is_u64())?;
                if !is_num {
                    return Ok(PDFObject::Number(number));
                }
                let is_obj = tokenizer.check_next_token(|token| token.key_was(R) || token.key_was(OBJ))?;
                if is_obj {
                    return parse_obj(tokenizer, Some(value))
                }
                Ok(PDFObject::Number(number))
            }
            _ => Ok(PDFObject::Number(number))
        },
        Token::Eof => Err(EOF.into()),
        _ => Err(Error::new(ILLEGAL_TOKEN, format!("Illegal token:{}", token.to_string())))
    }
}

pub(crate) fn parse_text_xref(tokenizer: &mut Tokenizer) -> Result<Vec<XEntry>> {
    let obj_num = tokenizer.next_token()?.as_u64()?;
    let length = tokenizer.next_token()?.as_u64()?;
    let mut entries = Vec::<XEntry>::new();
    for i in 0..length {
        let value = tokenizer.next_token()?.as_u64()?;
        let gen_num = tokenizer.next_token()?.as_u64()?;
        let state = tokenizer.next_token()?.to_string();
        let using = match state.as_str() {
            "n" => true,
            "f" => false,
            _ => return Err(Error::new(EXCEPT_TOKEN, format!("Except a token with 'f' or 'n' but it is '{}'", state)))
        };
        let obj_num = obj_num + i;
        let entry = XEntry::new (
            obj_num,
            gen_num,
            value,
            using
        );
        entries.push(entry);
    }
    Ok(entries)
}

fn parse_obj(tokenizer: &mut Tokenizer, option: Option<u64>) -> Result<PDFObject> {
    let obj_num = match option {
        Some(num) => num,
        None => tokenizer.next_token()?.as_u64()?
    };
    let obj_gen_token = tokenizer.next_token()?.except(|token| token.is_u64())?;
    let type_token = tokenizer.next_token()?.except(|token| token.key_was(R) || token.key_was(OBJ))?;
    let gen_num = obj_gen_token.as_u64()?;
    if let Key(ref key) = type_token {
        let object = match key.as_str() {
            OBJ => {
                let token = tokenizer.next_token()?;
                let value = parser0(tokenizer, token)?;
                // Except a token with 'endobj'
                tokenizer.next_token()?.except(|token| token.key_was(END_OBJ))?;
                return Ok(PDFObject::IndirectObject(obj_num, gen_num, Box::new(value)));
            },
            _ => {
                PDFObject::ObjectRef(obj_num,gen_num)
            }
        };
        return Ok(object)
    }
    Err(Error::new(EXCEPT_TOKEN, "Except a token with R or obj".to_string()))

}
fn parse_dict(mut tokenizer: &mut Tokenizer) -> Result<Dictionary> {
    let mut entries = HashMap::<String, Option<PDFObject>>::new();
    loop {
        let token = tokenizer.next_token()?;
        if let Delimiter(ref delimiter) = token {
            if delimiter == DOUBLE_RIGHT_BRACKET {
                break;
            }
        }
        let object = parser0(&mut tokenizer, token)?;
        if let PDFObject::Named(named) = object {
            let empty_or_end = tokenizer.check_next_token(|token| token.delimiter_was(DOUBLE_RIGHT_BRACKET) || token.delimiter_was("/"))?;
            if empty_or_end {
                entries.insert(named, None);
                continue;
            }
            let token = tokenizer.next_token()?;
            let value = parser0(&mut tokenizer, token)?;
            entries.insert(named, Some(value));
        } else {
            return Err(Error::new(EXCEPT_TOKEN, "Except a named token.".into()));
        }
    }
    Ok(Dictionary::new(entries))
}

fn parse_named(tokenizer: &mut Tokenizer) -> Result<PDFObject> {
    let token = tokenizer.next_token()?;
    if let Id(name) = token {
        return Ok(PDFObject::Named(name));
    }
    Err(Error::new(
        EXCEPT_TOKEN,
        "Except a identifier token.".to_string(),
    ))
}

fn parse_array(tokenizer: &mut Tokenizer) -> Result<PDFObject> {
    let mut elements = Vec::<PDFObject>::new();
    loop {
        let token = tokenizer.next_token()?;
        if let Delimiter(ref delimiter) = token {
            if delimiter == "]" {
                return Ok(PDFObject::Array(elements));
            }
        }
        let object = parser0(tokenizer, token)?;
        elements.push(object);

    }
}

fn parse_string(tokenizer: &mut Tokenizer, post_script: bool) -> Result<PDFObject> {
    let end_chr = if post_script { RIGHT_PARENTHESIS } else { RIGHT_BRACKET };
    let mut is_escape = true;
    let result = tokenizer.loop_util(&[], |chr| {
        is_escape = (chr == ESCAPE) && !is_escape;
        Ok(is_escape || chr == end_chr)
    });
    match result {
        Ok(range) => {
            let buf = tokenizer.drain_from_buf(range);
            let buf = if post_script {
                hex2bytes(&buf)
            } else {
                buf
            };
            // Remove '>' or ')'
            tokenizer.remove_buf_len(1);
            Ok(PDFObject::String(buf))
        }
        Err(_e) => Err(STR_NOT_ENCODED.into()),
    }
}

/// A stream has a `/Length` entry that specifies the number of bytes of data
/// between the `stream` and `endstream` keywords. This length does not include
/// the `stream` or `endstream` keywords themselves, nor the required
/// end-of-line marker (CRLF or LF) immediately following `stream`.
pub(crate) fn parse_stream(tokenizer: &mut Tokenizer, metadata: Dictionary) -> Result<PDFObject> {
    if let Some(PDFObject::Number(PDFNumber::Unsigned(length))) = metadata.get(LENGTH) {
        // Skip CRLF
        tokenizer.skip_crlf()?;
        let length = *length as usize;
        let buf = tokenizer.read_bytes(length)?;
        if buf.len() != length {
            return Err(Error::new(ILLEGAL_STREAM, format!("Require Stream length is {} but it is {}", length, buf.len()).into()));
        }
        let stream = Stream::new(metadata, buf);
        // Except next token is `endstream`
        tokenizer.next_token()?.except(|token| token.key_was(END_STREAM))?;
        return Ok(PDFObject::Stream(stream))
    }
    Err(Error::new(ILLEGAL_STREAM, "Stream length is not found".into()))
}