use crate::constants::pdf_key::{END_OBJ, OBJ, R};
use crate::constants::*;
use crate::error::error_kind::{EOF, EXCEPT_TOKEN, ILLEGAL_TOKEN, STR_NOT_ENCODED};
use crate::error::{Error, Result};
use crate::objects::{Dictionary, PDFNumber, PDFObject, XEntry};
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
            "<<" => parse_dict(tokenizer),
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
                let mut objects = Vec::<PDFObject>::new();
                // Parse indirect object contain all object
                loop {
                    let token = tokenizer.next_token()?;
                    if token.key_was(END_OBJ) {
                        return Ok(PDFObject::IndirectObject(obj_num, gen_num, objects));
                    }
                    let object = parser0(tokenizer, token)?;
                    objects.push(object);
                }
            },
            _ => {
                PDFObject::ObjectRef(obj_num,gen_num)
            }
        };
        return Ok(object)
    }
    Err(Error::new(EXCEPT_TOKEN, "Except a token with R or obj".to_string()))

}
fn parse_dict(mut tokenizer: &mut Tokenizer) -> Result<PDFObject> {
    let mut entries = HashMap::<String, Option<PDFObject>>::new();
    'ext:loop {
        let token = tokenizer.next_token()?;
        if let Delimiter(ref delimiter) = token {
            if delimiter == DOUBLE_RIGHT_BRACKET {
                break;
            }
        }
        let object = parser0(&mut tokenizer, token)?;
        if let PDFObject::Named(named) = object {
            let token = tokenizer.next_token()?;
            if let Delimiter(ref delimiter) = token {
                let dict_close = *delimiter == DOUBLE_RIGHT_BRACKET;
                let is_named = *delimiter == String::from(SPLASH);
                if is_named || dict_close {
                    entries.insert(named, None);
                    if dict_close {
                        continue 'ext;
                    }
                    continue;
                }
            }
            let value = parser0(&mut tokenizer, token)?;
            entries.insert(named, Some(value));
        } else {
            return Err(Error::new(EXCEPT_TOKEN, "Except a named token.".into()));
        }
    }
    Ok(PDFObject::Dict(Dictionary::new(entries)))
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
            let buf = tokenizer.buf.drain(range).collect::<Vec<u8>>();
            let buf = if post_script {
                hex2bytes(&buf)
            } else {
                buf
            };
            // Remove '>' or ')'
            tokenizer.buf.remove(0);
            Ok(PDFObject::String(buf))
        }
        Err(_e) => Err(STR_NOT_ENCODED.into()),
    }
}
