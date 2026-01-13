use crate::catalog::NodeId;
use crate::constants::CONTENTS;
use crate::document::PDFDocument;
use crate::error::PDFError::{ObjectAttrMiss, PageNotFound};
use crate::error::Result;
use crate::objects::PDFObject;

pub fn extract_page_text(document: &mut PDFDocument, page_id: NodeId) -> Result<Option<String>> {
    let page = match document.get_page(page_id) {
        Some(page) => page,
        None => return Err(PageNotFound(format!("Page not found:{}", page_id))),
    };
    let contents = match page.get_attr(CONTENTS) {
        Some(PDFObject::ObjectRef(obj_num,gen_num)) => {
            document.read_object_with_ref((*obj_num, *gen_num))
        },
        None => return Err(ObjectAttrMiss(CONTENTS)),
        _ => todo!(),
    };
    Ok(Some(String::new()))
}