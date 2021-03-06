//! Our Patch enum is intentionally kept in it's own file for easy inclusion into
//! The Percy Book.

use crate::{Attribute, Node, Text};

/// A Patch encodes an operation that modifies a real DOM element.
///
/// To update the real DOM that a user sees you'll want to first diff your
/// old virtual dom and new virtual dom.
///
/// This diff operation will generate `Vec<Patch>` with zero or more patches that, when
/// applied to your real DOM, will make your real DOM look like your new virtual dom.
///
/// Each Patch has a usize node index that helps us identify the real DOM node that it applies to.
///
/// Our old virtual dom's nodes are indexed depth first, as shown in this illustration
/// (0 being the root node, 1 being it's first child, 2 being it's first child's first child).
///
/// ```ignore
///             .─.
///            ( 0 )
///             `┬'
///         ┌────┴──────┐
///         │           │
///         ▼           ▼
///        .─.         .─.
///       ( 1 )       ( 4 )
///        `┬'         `─'
///    ┌────┴───┐       │
///    │        │       ├─────┬─────┐
///    ▼        ▼       │     │     │
///   .─.      .─.      ▼     ▼     ▼
///  ( 2 )    ( 3 )    .─.   .─.   .─.
///   `─'      `─'    ( 5 ) ( 6 ) ( 7 )
///                    `─'   `─'   `─'
/// ```
///
/// The patching process is tested in a real browser in tests/diff_patch.rs
#[derive(Debug, PartialEq)]
pub enum Patch<'a, T, ATT, EVENT, MSG>
where
    MSG: 'static,
    EVENT: 'static,
    ATT: Clone,
{
    /// Append a vector of child nodes to a parent node id.
    AppendChildren(&'a T, NodeIdx, Vec<&'a Node<T, ATT, EVENT, MSG>>),
    /// For a `node_i32`, remove all children besides the first `len`
    TruncateChildren(&'a T, NodeIdx, usize),
    /// Replace a node with another node. This typically happens when a node's tag changes.
    /// ex: <div> becomes <span>
    Replace(&'a T, NodeIdx, &'a Node<T, ATT, EVENT, MSG>),
    /// Add attributes that the new node has that the old node does not
    /// Note: the attributes is not a reference since attributes of same
    /// name are merged to produce a new unify attribute
    AddAttributes(&'a T, NodeIdx, Vec<Attribute<ATT, EVENT, MSG>>),
    /// Remove attributes that the old node had that the new node doesn't
    RemoveAttributes(&'a T, NodeIdx, Vec<ATT>),
    /// Add attributes that the new node has that the old node does not
    AddEventListener(&'a T, NodeIdx, Vec<&'a Attribute<ATT, EVENT, MSG>>),
    /// Remove attributes that the old node had that the new node doesn't
    RemoveEventListener(&'a T, NodeIdx, Vec<ATT>),
    /// Change the text of a Text node.
    ChangeText(NodeIdx, &'a Text),
}

type NodeIdx = usize;

impl<'a, T, ATT, EVENT, MSG> Patch<'a, T, ATT, EVENT, MSG>
where
    ATT: Clone,
{
    /// Every Patch is meant to be applied to a specific node within the DOM. Get the
    /// index of the DOM node that this patch should apply to. DOM nodes are indexed
    /// depth first with the root node in the tree having index 0.
    pub fn node_idx(&self) -> usize {
        match self {
            Patch::AppendChildren(_tag, node_idx, _) => *node_idx,
            Patch::TruncateChildren(_tag, node_idx, _) => *node_idx,
            Patch::Replace(_tag, node_idx, _) => *node_idx,
            Patch::AddAttributes(_tag, node_idx, _) => *node_idx,
            Patch::RemoveAttributes(_tag, node_idx, _) => *node_idx,
            Patch::AddEventListener(_tag, node_idx, _) => *node_idx,
            Patch::RemoveEventListener(_tag, node_idx, _) => *node_idx,
            Patch::ChangeText(node_idx, _) => *node_idx,
        }
    }
}
