use std::fmt::Display;
use crate::error::{Error, error_kind};

macro_rules! pdf_version {
    ($(($name:ident,$version:literal)),+$(,)?) => {
        #[derive(PartialEq,Debug)]
        pub enum PDFVersion{
        $(
            $name,
        )+
        }

        impl TryFrom<&str> for PDFVersion{
            type Error = Error;
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value.as_ref() {
                    $(
                        $version => Ok(PDFVersion::$name),
                    )+
                    _ => Err(error_kind::INVALID_PDF_VERSION.into()),
                }
            }
        }

        impl TryFrom<String> for PDFVersion{
            type Error = Error;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                PDFVersion::try_from(value.as_str())
            }
        }

        impl Display for PDFVersion{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self{
                    $(
                        PDFVersion::$name=>write!(f,"{}",$version),
                    )+
                }
            }
        }
    }
}

pdf_version!(
    (V1_0, "1.0"),
    (V1_1, "1.1"),
    (V1_2, "1.2"),
    (V1_3, "1.3"),
    (V1_4, "1.4"),
    (V1_5, "1.5"),
    (V1_6, "1.6"),
    (V1_7, "1.7"),
    (V2_0, "2.0")
);