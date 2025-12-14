macro_rules! pdf_key {
    ($(($ident:ident,$value:literal)),+$(,)?) => {
        pub(crate) mod pdf_key {
            $(
                pub(crate) const $ident: &str = $value;
            )+
        }
        pub(crate) fn is_key(str:&str)->bool{
            match str {
                $(
                    $value => true,
                )+
                _ => false
            }
        }
    }
}

pdf_key!(
    (TRAILER,"trailer"),
    (XREF,"xref"),
    (R,"R"),
    (OBJ,"obj"),
    (START_XREF,"startxref"),
    (TURE,"true"),
    (FALSE,"false"),
    (NULL,"null"),
    (END_OBJ,"endobj")
);


pub(crate) const LEFT_BRACKET: char = '<';
pub(crate) const RIGHT_BRACKET: char = '>';
pub(crate) const LEFT_PARENTHESIS: char = '(';
pub(crate) const RIGHT_PARENTHESIS: char = ')';
pub(crate) const DOT: char = '.';
pub(crate) const SUB: char = '-';
pub(crate) const ADD: char = '+';
pub(crate) const DOUBLE_LEFT_BRACKET: &str = "<<";
pub(crate) const DOUBLE_RIGHT_BRACKET: &str = ">>";

pub(crate) const PREV: &str = "Prev";
pub(crate) const SIZE: &str = "Size";
pub(crate) const ROOT: &str = "Root";
pub(crate) const CATALOG: &str = "Catalog";

pub(crate) const CR: char = '\r';
pub(crate) const LF: char = '\n';

pub(crate) const ESCAPE: char = '\\';

pub(crate) const SPACE: char = ' ';

pub(crate) const SPLASH: char = '/';

pub(crate) const LEFT_SQUARE_BRACKET: char = '[';
pub(crate) const RIGHT_SQUARE_BRACKET: char = ']';
pub(crate) const END_CHARS: [char; 11] = [
    LEFT_BRACKET,
    RIGHT_BRACKET,
    LEFT_PARENTHESIS,
    RIGHT_PARENTHESIS,
    CR,
    LF,
    ESCAPE,
    SPACE,
    SPLASH,
    LEFT_SQUARE_BRACKET,
    RIGHT_SQUARE_BRACKET,
];