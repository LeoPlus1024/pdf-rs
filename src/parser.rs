use crate::constants::pdf_key::{OBJ, R};
use crate::constants::*;
use crate::error::error_kind::{EOF, EXCEPT_TOKEN, ILLEGAL_TOKEN, STR_NOT_ENCODED};
use crate::error::{Error, Result};
use crate::objects::{Entry, EntryState, PDFArray, PDFDict, PDFDirectObject, PDFIndirectObject, PDFNamed, PDFNumber, PDFObject, PDFXref};
use crate::tokenizer::Token::{Delimiter, Id, Key, Number};
use crate::tokenizer::{Token, Tokenizer};
use std::collections::HashMap;
use crate::bytes::hex2bytes;

pub(crate) fn parse<F>(mut tokenizer: &mut Tokenizer, visit: F) -> Result<Option<PDFObject>>
where
    F: Fn(&Token) -> bool,
{
    let token = tokenizer.next_token()?;
    if !visit(&token) {
        return Ok(None);
    }
    let object = parser0(&mut tokenizer, token)?;
    Ok(Some(object))
}

fn parser0(tokenizer: &mut Tokenizer, token: Token) -> Result<PDFObject> {
    match token {
        Delimiter(delimiter) => match delimiter.as_str() {
            DOUBLE_LEFT_BRACKET => parse_dict(tokenizer),
            "[" => parse_array(tokenizer),
            "/" => parse_named(tokenizer),
            "<" | "(" => parse_string(tokenizer, delimiter == "("),
            &_ => todo!(),
        },
        Key(key) => match key.as_str() {
            pdf_key::XREF => parse_xref(tokenizer),
            pdf_key::TRAILER => {
                if let Delimiter(ref delimiter) = tokenizer.next_token()?{
                    if *delimiter != DOUBLE_LEFT_BRACKET {
                        return Err(Error::new(EXCEPT_TOKEN, format!("Trailer after must follow a '<<' but it was {}", delimiter)));
                    }
                    return parse_dict(tokenizer)
                }
                Err(Error::new(EXCEPT_TOKEN, "Trailer after must follow a dict".into()))
            },
            &_ => todo!(),
        }
        Number(number) => match number {
            PDFNumber::Unsigned(value) => {
                let is_num = tokenizer.check_next_token(|token| Ok(token.is_u64()))?;
                if !is_num {
                    return Ok(PDFObject::Number(number));
                }
                let is_obj = tokenizer.check_next_token(|token| Ok(token.key_was(R) || token.key_was(OBJ)))?;
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

fn parse_xref(tokenizer: &mut Tokenizer) -> Result<PDFObject> {
    let obj_num = tokenizer.next_token()?.as_u64()?;
    let length = tokenizer.next_token()?.as_u64()?;
    let mut entries = Vec::<Entry>::new();
    for i in 0..length {
        let offset = tokenizer.next_token()?.as_u64()?;
        let gen_num = tokenizer.next_token()?.as_u64()?;
        let state = tokenizer.next_token()?.to_string();
        let state = match state.as_str() {
            "n" => EntryState::Using(offset),
            "f" => EntryState::Deleted(offset),
            _ => return Err(Error::new(EXCEPT_TOKEN, format!("Except a token with 'f' or 'n' but it is '{}'", state)))
        };
        let entry = Entry {
            state,
            gen_num,
        };
        entries.push(entry);
    }
    let xref = PDFXref {
        obj_num,
        length,
        entries,
    };
    Ok(PDFObject::Xref(xref))
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
                let metadata = parse_dict(tokenizer)?;
                let object = PDFDirectObject {
                    obj_num,
                    gen_num,
                    metadata: Box::new(metadata),
                };
                PDFObject::DirectObject(object)
            },
            _ => {
                let object = PDFIndirectObject {
                    obj_num,
                    gen_num,
                };
                PDFObject::IndirectObject(object)
            }
        };
        return Ok(object)
    }
    Err(Error::new(EXCEPT_TOKEN, "Except a token with R or obj".to_string()))

}
fn parse_dict(mut tokenizer: &mut Tokenizer) -> Result<PDFObject> {
    let mut entries = HashMap::<PDFNamed, Option<PDFObject>>::new();
    loop {
        let token = tokenizer.next_token()?;
        if let Delimiter(ref delimiter) = token {
            if delimiter == DOUBLE_RIGHT_BRACKET {
                return Ok(PDFObject::Dict(PDFDict { entries }));
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
                        return Ok(PDFObject::Dict(PDFDict { entries }));
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
}

fn parse_named(tokenizer: &mut Tokenizer) -> Result<PDFObject> {
    let token = tokenizer.next_token()?;
    if let Id(name) = token {
        return Ok(PDFObject::Named(PDFNamed { name }));
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
                return Ok(PDFObject::Array(PDFArray { elements }));
            }
        }
        let object = parser0(tokenizer, token)?;
        elements.push(object);

    }
}

fn parse_string(tokenizer: &mut Tokenizer, post_script: bool) -> Result<PDFObject> {
    let end_chr = if post_script { ')' } else { '>' };
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
