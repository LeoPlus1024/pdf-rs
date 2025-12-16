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
    (END_OBJ,"endobj"),
    (STREAM,"stream"),
    (END_STREAM,"endstream")
);

pub(crate) const KIDS: &str = "Kids";
pub(crate) const TYPE: &str = "Type";
pub(crate) const PREV: &str = "Prev";
pub(crate) const SIZE: &str = "Size";
pub(crate) const ROOT: &str = "Root";
pub(crate) const COUNT: &str = "Count";
pub(crate) const PAGES: &str = "Pages";
pub(crate) const CATALOG: &str = "Catalog";

pub(crate) const LENGTH: &str = "Length";
