/// Macro to define PDF key constants and a utility function for key checking.
///
/// This macro generates a module containing string constants for common PDF keys
/// and a function to check if a string is a valid PDF key.
///
/// # Arguments
///
/// * `$ident` - The identifier for the constant
/// * `$value` - The string value of the constant
macro_rules! pdf_key {
    ($(($ident:ident,$value:literal)),+$(,)?) => {
        /// Module containing PDF key string constants.
        pub(crate) mod pdf_key {
            $(
                /// PDF key constant.
                pub(crate) const $ident: &str = $value;
            )+
        }
        /// Checks if a string is a valid PDF key.
        ///
        /// # Arguments
        ///
        /// * `str` - The string to check
        ///
        /// # Returns
        ///
        /// True if the string is a valid PDF key, false otherwise
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

/// Key for page tree nodes.
pub(crate) const KIDS: &str = "Kids";
/// Key for object type.
pub(crate) const TYPE: &str = "Type";
/// Key for previous cross-reference section.
pub(crate) const PREV: &str = "Prev";
/// Key for cross-reference table size.
pub(crate) const SIZE: &str = "Size";
/// Key for document catalog.
pub(crate) const ROOT: &str = "Root";
/// Key for count of pages or objects.
pub(crate) const COUNT: &str = "Count";
/// Key for pages object type.
pub(crate) const PAGES: &str = "Pages";
/// Key for catalog object type.
pub(crate) const CATALOG: &str = "Catalog";
/// Key for outlines.
pub(crate) const OUTLINES: &str = "Outlines";
/// Key for stream length.
pub(crate) const LENGTH: &str = "Length";

pub(crate) const FIRST: &str = "First";
pub(crate) const LAST: &str = "Last";

pub(crate) const NEXT: &str = "Next";

pub(crate) const INFO: &str = "Info";
pub(crate) const PRODUCER: &str = "Producer";
pub(crate) const CREATOR: &str = "Creator";
pub(crate) const CREATION_DATE: &str = "CreationDate";
pub(crate) const AUTHOR: &str = "Author";
pub(crate) const TITLE: &str = "Title";
pub(crate) const MOD_DATE:&str = "ModDate";