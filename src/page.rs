use std::collections::HashMap;
use crate::constants::{KIDS, PAGES, TYPE};
use crate::error::error_kind::PAGE_NOT_FOUND;
use crate::error::{Error, Result};
use crate::objects::{Dictionary, PDFObject, XEntry};
use crate::parser::parse_with_offset;
use crate::tokenizer::Tokenizer;

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
    children: Option<Vec<NodeId>>,
    parent: Option<NodeId>,
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
    if let Some(entry) = xrefs.iter().find(|x| x.obj_num == catalog.0 && x.gen_num == catalog.1) {
        let obj = parse_with_offset(tokenizer, entry.value)?;
        if let PDFObject::IndirectObject(_, _, value) = obj {
            if let PDFObject::Dict(dict) = *value {
                match dict.get(PAGES).map(|obj| obj.as_object_ref().unwrap()) {
                    Some((obj_num, gen_num)) => {
                        let mut nodes = HashMap::<NodeId, PageNode>::new();
                        build_page_tree(tokenizer, xrefs, (obj_num, gen_num), None, &mut nodes)?;
                        return Ok(PageTreeArean::new(obj_num, nodes))
                    }
                    _ => {}
                }
            }
        }
    }
    Err(Error::new(PAGE_NOT_FOUND, format!("Can not find page catalog with {} {}", catalog.0, catalog.1)))
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
fn build_page_tree(tokenizer: &mut Tokenizer, xrefs: &[XEntry], obj_ref: (u64, u64), parent: Option<NodeId>, nodes: &mut HashMap<NodeId, PageNode>) -> Result<()> {
    if let Some(entry) = xrefs.iter().find(|x| x.obj_num == obj_ref.0 && x.gen_num == obj_ref.1) {
        if let PDFObject::IndirectObject(_, _, value) = parse_with_offset(tokenizer, entry.value)? {
            if let PDFObject::Dict(dict) = *value {
                let is_page_tree = dict.named_value_was(TYPE, PAGES);
                // If it is not a page tree, then it is a page
                if !is_page_tree {
                    let leaf_node = PageNode {
                        attrs: dict,
                        children: None,
                        count: 0,
                        parent: None,
                    };
                    nodes.insert(obj_ref.0, leaf_node);
                    return Ok(())
                }
                if let Some(kids) = dict.get_array_value(KIDS) {
                    let len = kids.len();
                    let children = if len > 0 {
                        let parent = Some(obj_ref.0);
                        let mut children: Vec<NodeId> = Vec::new();
                        for kid in kids {
                            if let PDFObject::ObjectRef(obj_num, gen_num) = kid {
                                children.push(*obj_num);
                                build_page_tree(tokenizer, xrefs, (*obj_num, *gen_num), parent, nodes)?;
                            }
                        }
                        Some(children)
                    } else {
                        None
                    };
                    let count = children.as_ref().map(|children| children.len()).unwrap_or(0);
                    let page_node = PageNode {
                        attrs: dict,
                        children,
                        count,
                        parent,
                    };
                    nodes.insert(obj_ref.0, page_node);
                }
            }
        }
    }
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
    pub fn get_root_node(&self) -> &PageNode {
        self.nodes.get(&self.root_id).unwrap()
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