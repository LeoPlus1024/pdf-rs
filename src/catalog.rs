use std::fmt::Write;
use crate::constants::{COUNT, FIRST, KIDS, LAST, NEXT, OUTLINES, PAGES, PREV, TITLE, TYPE};
use crate::error::PDFError::{ObjectAttrMiss, PDFParseError, XrefEntryNotFound};
use crate::error::Result;
use crate::objects::{Dictionary, PDFNumber, PDFObject, XEntry};
use crate::parser::parse_with_offset;
use crate::tokenizer::Tokenizer;
use crate::utils::xrefs_search;
use std::collections::HashMap;
use std::fmt::{format, Display, Formatter};
use crate::encoding::PreDefinedEncoding;
use crate::pstr::convert_glyph_text;

macro_rules! mixture_node_id {
    ($obj_num:expr,$gen_num:expr) => {{
        let node_id = ($obj_num as u64) << 16 | $gen_num as u64;
        node_id
    }};
}

macro_rules! separate_node_id {
    ($node_id:expr) => {{
        let obj_num = ($node_id >> 16) as u64;
        let gen_num = ($node_id & 0xFFFF) as u64;
        (obj_num, gen_num)
    }};
}

/// Type alias for node identifiers in the page tree.
type NodeId = u64;

/// Represents a tree structure for organizing pages in a PDF document.
///
/// The `PageTreeArean` manages a hierarchical structure of page nodes,
/// where each node can be either a page tree node (intermediate node) or
/// a page leaf node (terminal node containing actual page content).
pub struct PageTreeArean {
    /// The ID of the root node in the page tree.
    root_id: NodeId,
    /// A collection of all nodes in the page tree, indexed by their IDs.
    nodes: HashMap<NodeId, PageNode>,
}

/// Represents a node in the page tree structure.
///
/// Each node can be either:
/// - A page tree node (intermediate node with children)
/// - A page leaf node (terminal node representing an actual page)
pub(crate) struct PageNode {
    /// The attributes of the page node stored as a dictionary.
    attrs: Dictionary,
    /// The count of pages or child nodes under this node.
    /// For leaf nodes, this is 0. For intermediate nodes, this is the total
    /// number of leaf nodes under this node.
    count: usize,
    /// Optional list of child node IDs for intermediate nodes.
    /// This is None for leaf nodes (actual pages).
    kids: Option<Vec<NodeId>>,
    /// Optional ID of the parent node.
    /// This is None for the root node.
    parent_id: Option<NodeId>,
}

/// Represents the outline (bookmarks) structure of a PDF document.
///
/// The outline provides a hierarchical navigation structure for the document,
/// typically displayed in the PDF viewer's sidebar.
pub struct OutlineTreeArean {
    /// The ID of the root node in the outline tree.
    root_id: NodeId,
    /// A collection of all nodes in the outline tree, indexed by their IDs.
    nodes: HashMap<NodeId, OutlineNode>,
}

/// Represents a node in the outline (bookmark) tree.
///
/// Each outline node corresponds to a bookmark entry in the PDF document.
pub(crate) struct OutlineNode {
    count: i64,
    /// The title of the bookmark.
    title: Option<String>,
    /// Optional ID of the previous sibling node.
    prev_id: Option<NodeId>,
    /// Optional ID of the next sibling node.
    next_id: Option<NodeId>,
    /// Optional ID of the first child node.
    first_id: Option<NodeId>,
    /// Optional ID of the last child node.
    last_id: Option<NodeId>,
    /// Optional ID of the parent node.
    parent_id: Option<NodeId>
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
/// A `Result` containing a tuple with the constructed `PageTreeArean` and an optional `Outline`,
/// or an error if the page catalog cannot be found
pub(crate) fn decode_catalog_data(
    tokenizer: &mut Tokenizer,
    catalog: (u32, u16),
    xrefs: &[XEntry],
) -> Result<(PageTreeArean, Option<OutlineTreeArean>)> {
    let entry = xrefs_search(xrefs, catalog)?;
    let obj = parse_with_offset(tokenizer, entry.value)?;
    let catalog_attr = match obj {
        PDFObject::IndirectObject(_, _, value) => value.to_dict(),
        _ => return Err(ObjectAttrMiss("PDF catalog not found.")),
    };
    match catalog_attr {
        Some(dict) => {
            let page_tree_arean;
            if let Some(PDFObject::ObjectRef(obj_num, gen_num)) = dict.get(PAGES) {
                let mut nodes = HashMap::new();
                let obj_num = *obj_num;
                let gen_num = *gen_num;
                build_page_tree(tokenizer, xrefs, (obj_num, gen_num), None, &mut nodes)?;
                page_tree_arean = PageTreeArean::new(mixture_node_id!(obj_num, gen_num), nodes);
            } else {
                return Err(ObjectAttrMiss("Catalog attribute not contain pages attr."));
            }
            let mut outline = None;
            if let Some(PDFObject::ObjectRef(obj_num, gen_num)) = dict.get(OUTLINES) {
                let mut map = HashMap::<NodeId, OutlineNode>::new();
                let obj_num = *obj_num;
                let gen_num = *gen_num;
                build_outline_tree(tokenizer, xrefs, obj_num, gen_num, None, &mut map)?;
                outline = Some(OutlineTreeArean::new(mixture_node_id!(obj_num, gen_num), map));
            }
            Ok((page_tree_arean, outline))
        }
        _ => Err(ObjectAttrMiss("Catalog attribute not found or not a dict.")),
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
fn build_page_tree(
    tokenizer: &mut Tokenizer,
    xrefs: &[XEntry],
    obj_ref: (u32, u16),
    parent_id: Option<NodeId>,
    nodes: &mut HashMap<NodeId, PageNode>,
) -> Result<()> {
    let entry = xrefs_search(xrefs, obj_ref)?;
    let obj = match parse_with_offset(tokenizer, entry.value)? {
        PDFObject::IndirectObject(_, _, value) => *value,
        _ => return Err(XrefEntryNotFound(obj_ref.0, obj_ref.1)),
    };
    let dict = match obj {
        PDFObject::Dict(dict) => dict,
        _ => return Err(PDFParseError("Page attributes is not a dict")),
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
        let node_id = mixture_node_id!(obj_ref.0, obj_ref.1);
        nodes.insert(node_id, leaf_node);
        return Ok(());
    }
    let count = match dict.get_u64_num(COUNT) {
        Some(count) => count as usize,
        _ => return Err(PDFParseError("Page count not exist or not a number")),
    };
    let mut kids = None;
    if count > 0 {
        let arr = match dict.get_array_value(KIDS) {
            Some(kids) => kids,
            _ => return Err(PDFParseError("Page kids not exist or not an array")),
        };
        let mut children: Vec<NodeId> = Vec::with_capacity(arr.len());
        let tmp = mixture_node_id!(obj_ref.0, obj_ref.1);
        for kid in arr {
            if let PDFObject::ObjectRef(obj_num, gen_num) = kid {
                children.push(mixture_node_id!(*obj_num, *gen_num));
                build_page_tree(tokenizer, xrefs, (*obj_num, *gen_num), Some(tmp), nodes)?;
            } else {
                return Err(PDFParseError(
                    "Page kids not exist or not an object reference",
                ));
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
    nodes.insert(mixture_node_id!(obj_ref.0, obj_ref.1), page_node);
    Ok(())
}

fn build_outline_tree(
    tokenizer: &mut Tokenizer,
    xrefs: &[XEntry],
    obj_num: u32,
    gen_num: u16,
    parent_id: Option<NodeId>,
    map: &mut HashMap<NodeId, OutlineNode>,
) -> Result<()> {
    let entry = xrefs_search(xrefs, (obj_num, gen_num))?;
    let object = parse_with_offset(tokenizer, entry.value)?;
    let (_, _, attrs) = match object.as_indirect_object() {
        Some((obj_num, gen_num, obj)) => match obj.as_dict() {
            Some(dict) => (obj_num, gen_num, dict),
            _ => return Err(PDFParseError("Outline attribute except a dict.")),
        },
        _ => return Err(PDFParseError("Outline object is not an indirect object")),
    };
    let mut title = None;
    let mut prev_id = None;
    let mut next_id = None;
    let mut first_id = None;
    let mut last_id = None;
    let node_id = mixture_node_id!(obj_num, gen_num);
    if let Some(PDFObject::ObjectRef(obj_num, gen_num)) = attrs.get(PREV) {
        prev_id = Some(mixture_node_id!(*obj_num, *gen_num));
    }
    if let Some(PDFObject::ObjectRef(obj_num, gen_num)) = attrs.get(FIRST) {
        first_id = Some(mixture_node_id!(*obj_num, *gen_num));
        build_outline_tree(tokenizer, xrefs, *obj_num, *gen_num, Some(node_id), map)?;
    }
    if let Some(PDFObject::ObjectRef(obj_num, gen_num)) = attrs.get(LAST) {
        last_id = Some(mixture_node_id!(*obj_num, *gen_num));
    }

    if let Some(PDFObject::ObjectRef(obj_num, gen_num)) = attrs.get(NEXT) {
        next_id = Some(mixture_node_id!(*obj_num, *gen_num));
        build_outline_tree(tokenizer, xrefs, *obj_num, *gen_num, Some(node_id), map)?;
    }

    if let Some(PDFObject::String(pstr)) = attrs.get(TITLE){
        title = Some(convert_glyph_text(pstr, &PreDefinedEncoding::PDFDoc));
    }

    let count = match attrs.get(COUNT) {
        Some(PDFObject::Number(PDFNumber::Signed(value))) => *value,
        Some(PDFObject::Number(PDFNumber::Unsigned(value))) => *value as i64,
        _ => 0i64
    };
    let outline_node = OutlineNode {
        count,
        title,
        prev_id,
        next_id,
        first_id,
        last_id,
        parent_id,
    };
    map.insert(node_id, outline_node);
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
        Self { nodes, root_id }
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

/// Formats a page node for display with tree-like indentation.
///
/// This function recursively formats the page tree structure for display purposes,
/// showing the hierarchical relationship between nodes using ASCII characters.
/// Each node is displayed with its object number, generation number, count of children,
/// and any child nodes are indented to show the tree structure.
///
/// # Arguments
///
/// * `page_tree_arean` - A reference to the page tree arena containing all nodes
/// * `node_id` - The ID of the current node being formatted
/// * `f` - A mutable reference to the formatter for writing output
/// * `indent` - The current indentation level for tree display
/// * `is_last` - Boolean indicating if this is the last child at the current level
///
/// # Returns
///
/// A `std::fmt::Result` indicating whether the formatting was successful
fn fmt_page_node(
    page_tree_arean: &PageTreeArean,
    node_id: &NodeId,
    f: &mut Formatter<'_>,
    indent: usize,
    is_last: bool,
) -> std::fmt::Result {
    if let Some(page_node) = page_tree_arean.nodes.get(node_id) {
        let (obj_num, gen_num) = separate_node_id!(node_id);

        let prefix = if indent == 0 {
            String::new()
        } else {
            format!("{}{}", "│   ".repeat(indent - 1), if is_last { "└── " } else { "├── " })
        };
        writeln!(f, "{}Page：[{},{}]", prefix, obj_num, gen_num)?;
        writeln!(f, "{}├── Count：{}",
                 "│   ".repeat(indent),
                 page_node.kids.as_ref().map_or(0, |k| k.len())
        )?;

        writeln!(f, "{}└── Kids", "│   ".repeat(indent))?;

        if let Some(kids) = page_node.kids.as_ref() {
            let total = kids.len();
            for (i, kid_id) in kids.iter().enumerate() {
                let is_kid_last = (i == total - 1);
                fmt_page_node(page_tree_arean, kid_id, f, indent + 1, is_kid_last)?;
            }
        }
    }
    Ok(())
}

impl Display for PageTreeArean {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let root_id = &self.root_id;
        if let Some(page_node) = self.nodes.get(&root_id) {
            fmt_page_node(self, root_id, f, 0,false)?;
        }
        write!(f, "" )
    }
}

impl OutlineTreeArean {
    pub(crate) fn new(root_id: NodeId, nodes: HashMap<NodeId, OutlineNode>) -> Self {
        Self { root_id, nodes }
    }
}
