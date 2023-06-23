use crate::{movement::Direction, Range, RopeSlice, Selection, Syntax};
use tree_sitter::{Node, Tree};

pub fn expand_selection(syntax: &Syntax, text: RopeSlice, selection: Selection) -> Selection {
    select_node_impl(syntax, text, selection, |mut node, from, to| {
        while node.start_byte() == from && node.end_byte() == to {
            node = node.parent()?;
        }
        Some(node)
    })
}

pub fn shrink_selection(syntax: &Syntax, text: RopeSlice, selection: Selection) -> Selection {
    select_node_impl(syntax, text, selection, |descendant, _from, _to| {
        descendant.child(0).or(Some(descendant))
    })
}

pub fn select_sibling<F>(
    syntax: &Syntax,
    text: RopeSlice,
    selection: Selection,
    sibling_fn: &F,
) -> Selection
where
    F: Fn(Node) -> Option<Node>,
{
    select_node_impl(syntax, text, selection, |descendant, _from, _to| {
        find_sibling_recursive(descendant, sibling_fn)
    })
}

fn find_parent_with_more_children(mut node: Node) -> Option<Node> {
    while let Some(parent) = node.parent() {
        if parent.child_count() > 1 {
            return Some(parent);
        }

        node = parent;
    }

    None
}

pub fn select_all_siblings(tree: &Tree, text: RopeSlice, selection: Selection) -> Selection {
    let root_node = &tree.root_node();

    selection.transform_iter(|range| {
        let from = text.char_to_byte(range.from());
        let to = text.char_to_byte(range.to());

        root_node
            .descendant_for_byte_range(from, to)
            .and_then(find_parent_with_more_children)
            .and_then(|parent| select_children(parent, text, range.direction()))
            .unwrap_or_else(|| vec![range].into_iter())
    })
}

pub fn select_all_children(tree: &Tree, text: RopeSlice, selection: Selection) -> Selection {
    let root_node = &tree.root_node();

    selection.transform_iter(|range| {
        let from = text.char_to_byte(range.from());
        let to = text.char_to_byte(range.to());

        root_node
            .descendant_for_byte_range(from, to)
            .and_then(|parent| select_children(parent, text, range.direction()))
            .unwrap_or_else(|| vec![range].into_iter())
    })
}

fn select_children(
    node: Node,
    text: RopeSlice,
    direction: Direction,
) -> Option<<Vec<Range> as std::iter::IntoIterator>::IntoIter> {
    let mut cursor = node.walk();

    let children = node
        .named_children(&mut cursor)
        .map(|child| {
            let from = text.byte_to_char(child.start_byte());
            let to = text.byte_to_char(child.end_byte());

            if direction == Direction::Backward {
                Range::new(to, from)
            } else {
                Range::new(from, to)
            }
        })
        .collect::<Vec<_>>();

    if !children.is_empty() {
        Some(children.into_iter())
    } else {
        None
    }
}

fn find_sibling_recursive<F>(node: Node, sibling_fn: F) -> Option<Node>
where
    F: Fn(Node) -> Option<Node>,
{
    sibling_fn(node).or_else(|| {
        node.parent()
            .and_then(|node| find_sibling_recursive(node, sibling_fn))
    })
}

fn select_node_impl<F>(
    syntax: &Syntax,
    text: RopeSlice,
    selection: Selection,
    select_fn: F,
) -> Selection
where
    F: Fn(Node, usize, usize) -> Option<Node>,
{
    let tree = syntax.tree();

    selection.transform(|range| {
        let from = text.char_to_byte(range.from());
        let to = text.char_to_byte(range.to());

        let node = match tree
            .root_node()
            .descendant_for_byte_range(from, to)
            .and_then(|node| select_fn(node, from, to))
        {
            Some(node) => node,
            None => return range,
        };

        let from = text.byte_to_char(node.start_byte());
        let to = text.byte_to_char(node.end_byte());

        if range.head < range.anchor {
            Range::new(to, from)
        } else {
            Range::new(from, to)
        }
    })
}
