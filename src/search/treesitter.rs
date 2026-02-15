//! Shared tree-sitter utilities used by symbol search and caller search.

/// Parse content into a tree-sitter Tree. Returns `None` if the language
/// can't be set or parsing fails.
pub(crate) fn parse_tree(
    content: &str,
    ts_lang: &tree_sitter::Language,
) -> Option<tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(ts_lang).ok()?;
    parser.parse(content, None)
}

/// Definition node kinds across tree-sitter grammars.
pub(crate) const DEFINITION_KINDS: &[&str] = &[
    // Functions
    "function_declaration",
    "function_definition",
    "function_item",
    "method_definition",
    "method_declaration",
    // Classes & structs
    "class_declaration",
    "class_definition",
    "struct_item",
    // Interfaces & types (TS)
    "interface_declaration",
    "type_alias_declaration",
    "type_item",
    // Enums
    "enum_item",
    "enum_declaration",
    // Variables & constants
    "lexical_declaration",
    "variable_declaration",
    "const_item",
    "static_item",
    // Rust-specific
    "trait_item",
    "impl_item",
    "mod_item",
    // Python
    "decorated_definition",
    // Go
    "type_declaration",
    // Swift
    "protocol_declaration",
    "init_declaration",
    "typealias_declaration",
    "property_declaration",
    // Exports
    "export_statement",
];

/// Extract the name defined by a tree-sitter definition node.
///
/// Walks standard field names (`name`, `identifier`, `declarator`) and handles
/// nested declarators and export statements.
pub(crate) fn extract_definition_name(node: tree_sitter::Node, lines: &[&str]) -> Option<String> {
    // Try standard field names
    for field in &["name", "identifier", "declarator"] {
        if let Some(child) = node.child_by_field_name(field) {
            let text = node_text_simple(child, lines);
            if !text.is_empty() {
                // For variable_declarator, get the identifier inside
                if child.kind().contains("declarator")
                    && let Some(id) = child.child_by_field_name("name")
                {
                    return Some(node_text_simple(id, lines));
                }
                return Some(text);
            }
        }
    }

    // For export_statement, check the declaration child
    if node.kind() == "export_statement" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if DEFINITION_KINDS.contains(&child.kind()) {
                return extract_definition_name(child, lines);
            }
        }
    }

    None
}

/// Get the text of a single-line node from pre-split source lines.
///
/// Returns the text slice for single-line nodes, or the text from the start
/// column to end-of-line for multi-line nodes.
pub(crate) fn node_text_simple(node: tree_sitter::Node, lines: &[&str]) -> String {
    let row = node.start_position().row;
    let col_start = node.start_position().column;
    let end_row = node.end_position().row;
    if row < lines.len() && row == end_row {
        let col_end = node.end_position().column.min(lines[row].len());
        lines[row][col_start..col_end].to_string()
    } else if row < lines.len() {
        lines[row][col_start..].to_string()
    } else {
        String::new()
    }
}
