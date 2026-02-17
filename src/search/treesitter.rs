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
    // Zig
    "test_declaration",
    "using_namespace_declaration",
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

    // Rust impl_item: `impl Type { ... }` — the type is in the `type` field, not `name`.
    if node.kind() == "impl_item"
        && let Some(type_node) = node.child_by_field_name("type")
    {
        let text = node_text_simple(type_node, lines);
        if !text.is_empty() {
            return Some(text);
        }
    }

    // Fallback: scan positional children for an identifier node.
    // Needed for Zig's variable_declaration where identifier is a child, not a field.
    {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let text = node_text_simple(child, lines);
                if !text.is_empty() {
                    return Some(text);
                }
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

/// Extract the trait name from a Rust `impl_item` node.
/// For `impl Trait for Type`, returns the trait name.
/// For bare `impl Type`, returns `None`.
pub(crate) fn extract_impl_trait(node: tree_sitter::Node, lines: &[&str]) -> Option<String> {
    debug_assert_eq!(node.kind(), "impl_item");
    let trait_node = node.child_by_field_name("trait")?;
    let text = node_text_simple(trait_node, lines);
    if text.is_empty() { None } else { Some(text) }
}

/// Extract the implementing type from a Rust `impl_item` node.
/// For `impl Trait for Type` or `impl Type`, returns the type name.
pub(crate) fn extract_impl_type(node: tree_sitter::Node, lines: &[&str]) -> Option<String> {
    debug_assert_eq!(node.kind(), "impl_item");
    let type_node = node.child_by_field_name("type")?;
    let text = node_text_simple(type_node, lines);
    if text.is_empty() { None } else { Some(text) }
}

/// Extract interface names from a class declaration's `implements` clause.
/// Works for TypeScript (`class Foo implements Bar, Baz`) and Java.
/// Handles nesting: `class_declaration` → `class_heritage` → `implements_clause`.
pub(crate) fn extract_implemented_interfaces(
    node: tree_sitter::Node,
    lines: &[&str],
) -> Vec<String> {
    if let Some(clause) = find_implements_clause(node) {
        collect_interfaces_from_clause(clause, lines)
    } else {
        Vec::new()
    }
}

fn collect_interfaces_from_clause(clause: tree_sitter::Node, lines: &[&str]) -> Vec<String> {
    let mut interfaces = Vec::new();
    let mut cursor = clause.walk();
    for child in clause.children(&mut cursor) {
        let kind = child.kind();
        if kind == "type_identifier" || kind == "identifier" {
            let text = node_text_simple(child, lines);
            if !text.is_empty() {
                interfaces.push(text);
            }
        } else if kind == "generic_type"
            && let Some(name) = child.child_by_field_name("name")
        {
            let text = node_text_simple(name, lines);
            if !text.is_empty() {
                interfaces.push(text);
            }
        }
    }
    interfaces
}

fn find_implements_clause(node: tree_sitter::Node) -> Option<tree_sitter::Node> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        if kind == "implements_clause" || kind == "super_interfaces" {
            return Some(child);
        }
        // TypeScript nests: class_declaration → class_heritage → implements_clause
        if kind == "class_heritage" {
            let mut inner = child.walk();
            for grandchild in child.children(&mut inner) {
                let gk = grandchild.kind();
                if gk == "implements_clause" || gk == "super_interfaces" {
                    return Some(grandchild);
                }
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
