use crate::constants::{COUNT, KIDS, PAGES, TYPE};
use crate::error::PDFError::{ObjectAttrMiss, PDFParseError, XrefEntryNotFound};
use crate::error::Result;
use crate::objects::{Dictionary, PDFObject, XEntry};
use crate::parser::parse_with_offset;
use crate::tokenizer::Tokenizer;
use crate::utils::xrefs_search;
use std::collections::HashMap;

type NodeId = u64;

/// Represents a tree structure for organizing pages in a PDF document.
///
/// The `PageTreeArean` manages a hierarchical structure of page nodes,
/// where each node can be either a page tree node (intermediate node) or
/// a page leaf node (terminal node containing actual page content).
pub(crate) struct PageTreeArean {
    root_id: NodeId,
    nodes: HashMap<NodeId, PageNode>,
}

/// Represents a node in the page tree structure.
///
/// Each node can be either:
/// - A page tree node (intermediate node with children)
/// - A page leaf node (terminal node representing an actual page)
pub(crate) struct PageNode {
    attrs: Dictionary,
    count: usize,
    kids: Option<Vec<NodeId>>,
    parent_id: Option<NodeId>,
}

/// Creates a page tree arena from the PDF catalog.
///
/// This function builds a hierarchical page tree structure from the PDF's catalog object.
/// It traverses the page tree nodes recursively to construct the complete page hierarchy.
///
/// # Arguments
///
/// * `tokenizer` - A mutable reference to the tokenizer for parsing PDF objects
/// * `catalog` - A tuple containing the object number and generation number of the catalog
/// * `xrefs` - A slice of cross-reference table entries
///
/// # Returns
///
/// A `Result` containing the constructed `PageTreeArean` or an error if the page catalog cannot be found
pub(crate) fn create_page_tree_arena(tokenizer: &mut Tokenizer, catalog: (u64, u64), xrefs: &[XEntry]) -> Result<PageTreeArean> {
    let entry = xrefs_search(xrefs, catalog)?;
    let obj = parse_with_offset(tokenizer, entry.value)?;
    let catalog_attr = match obj {
        PDFObject::IndirectObject(_, _, value) => value.to_dict(),
        _ => return Err(ObjectAttrMiss("PDF catalog not found."))
    };
    match catalog_attr {
        Some(dict) => {
            let mut page_tree_arean = None;
            if let Some(PDFObject::ObjectRef(obj_num, gen_num)) = dict.get(PAGES) {
                let mut nodes = HashMap::new();
                build_page_tree(tokenizer, xrefs, (*obj_num, *gen_num), None, &mut nodes)?;
                page_tree_arean = Some(PageTreeArean::new(*obj_num, nodes));
            }
            match page_tree_arean {
                Some(value) => Ok(value),
                None => Err(ObjectAttrMiss("Catalog attribute not contain pages attr."))
            }
        }
        _ => Err(ObjectAttrMiss("Catalog attribute not found or not a dict."))
    }

}

/// Recursively builds the page tree structure from PDF objects.
///
/// This function traverses the PDF page tree hierarchy, creating nodes for both
/// intermediate page tree nodes and leaf page nodes. It establishes parent-child
/// relationships between nodes and populates node attributes.
///
/// # Arguments
///
/// * `tokenizer` - A mutable reference to the tokenizer for parsing PDF objects
/// * `xrefs` - A slice of cross-reference table entries
/// * `obj_ref` - A tuple containing the object number and generation number of the current node
/// * `parent` - An optional parent node ID
/// * `nodes` - A mutable reference to the HashMap storing all page nodes
///
/// # Returns
///
/// A `Result` indicating success or an error if parsing fails
fn build_page_tree(tokenizer: &mut Tokenizer, xrefs: &[XEntry], obj_ref: (u64, u64), parent_id: Option<NodeId>, nodes: &mut HashMap<NodeId, PageNode>) -> Result<()> {
    let entry = xrefs_search(xrefs, obj_ref)?;
    let obj = match parse_with_offset(tokenizer, entry.value)? {
        PDFObject::IndirectObject(_, _, value) => *value,
        _ => return Err(XrefEntryNotFound(obj_ref.0, obj_ref.1))
    };
    let dict = match obj {
        PDFObject::Dict(dict) => dict,
        _ => return Err(PDFParseError("Page attributes is not a dict"))
    };
    let is_page_tree = dict.named_value_was(TYPE, PAGES);
    // If it is not a page tree, then it is a page
    if !is_page_tree {
        let leaf_node = PageNode {
            attrs: dict,
            kids: None,
            count: 0,
            parent_id,
        };
        nodes.insert(obj_ref.0, leaf_node);
        return Ok(())
    }
    let count = match dict.get_u64_num(COUNT) {
        Some(count) => count as usize,
        _ => return Err(PDFParseError("Page count not exist or not a number"))
    };
    let mut kids = None;
    if count > 0 {
        let arr = match dict.get_array_value(KIDS) {
            Some(kids) => kids,
            _ => return Err(PDFParseError("Page kids not exist or not an array"))
        };
        let mut children: Vec<NodeId> = Vec::with_capacity(arr.len());
        let tmp = Some(obj_ref.0);
        for kid in arr {
            if let PDFObject::ObjectRef(obj_num, gen_num) = kid {
                children.push(*obj_num);
                build_page_tree(tokenizer, xrefs, (*obj_num, *gen_num), tmp, nodes)?;
            } else {
                return Err(PDFParseError("Page kids not exist or not an object reference"));
            }
        }
        kids = Some(children)
    };
    let page_node = PageNode {
        attrs: dict,
        kids,
        count,
        parent_id,
    };
    nodes.insert(obj_ref.0, page_node);
    Ok(())
}

impl PageTreeArean {
    /// Creates a new `PageTreeArean` with the specified root node ID and nodes.
    ///
    /// # Arguments
    ///
    /// * `root_id` - The ID of the root node for this page tree
    /// * `nodes` - A HashMap containing all nodes in the page tree, keyed by their IDs
    ///
    /// # Returns
    ///
    /// A new `PageTreeArean` instance
    pub(crate) fn new(root_id: NodeId, nodes: HashMap<NodeId, PageNode>) -> Self {
        Self {
            nodes,
            root_id
        }
    }

    /// Returns a reference to the root node of the page tree.
    ///
    /// # Returns
    ///
    /// A reference to the root `PageNode`
    pub fn get_root_node(&self) -> Option<&PageNode> {
        self.nodes.get(&self.root_id)
    }

    /// Gets the total number of pages in the document.
    ///
    /// This method counts all leaf nodes in the tree (nodes with count == 0),
    /// which represent actual pages rather than intermediate page tree nodes.
    ///
    /// # Returns
    ///
    /// The total number of pages in the document
    pub(crate) fn get_page_num(&self) -> usize {
        self.nodes.values().filter(|node| node.count == 0).count()
    }
}