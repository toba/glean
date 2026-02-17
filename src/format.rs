use std::fmt::Write;
use std::path::Path;

use crate::types::{ViewMode, estimate_tokens};

/// Build the standard header line:
/// `# path/to/file.ts (N lines, ~X.Xk tokens) [mode]`
pub fn file_header(path: &Path, byte_len: u64, line_count: u32, mode: ViewMode) -> String {
    let tokens = estimate_tokens(byte_len);
    let token_str = if tokens >= 1000 {
        format!("~{}.{}k tokens", tokens / 1000, (tokens % 1000) / 100)
    } else {
        format!("~{tokens} tokens")
    };
    format!(
        "# {} ({line_count} lines, {token_str}) [{mode}]",
        path.display()
    )
}

/// Build header for binary files: `# path (binary, size, mime) [skipped]`
pub fn binary_header(path: &Path, byte_len: u64, mime: &str) -> String {
    let size_str = format_size(byte_len);
    format!(
        "# {} (binary, {size_str}, {mime}) [skipped]",
        path.display()
    )
}

/// Build header for search results.
pub fn search_header(
    query: &str,
    scope: &Path,
    total: usize,
    defs: usize,
    usages: usize,
) -> String {
    let parts = match (defs, usages) {
        (0, _) => format!("{total} matches"),
        (d, u) => format!("{total} matches ({d} definitions, {u} usages)"),
    };
    format!("# Search: \"{query}\" in {} — {parts}", scope.display())
}

/// Strip the scope prefix from a path to produce a relative display path.
/// Falls back to the full path if stripping fails.
pub fn rel(path: &Path, scope: &Path) -> String {
    path.strip_prefix(scope)
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Human-readable file size. Integer math only — no floats.
fn format_size(bytes: u64) -> String {
    match bytes {
        b if b < 1024 => format!("{b}B"),
        b if b < 1024 * 1024 => format!("{}KB", b / 1024),
        b => format!(
            "{}.{}MB",
            b / (1024 * 1024),
            (b % (1024 * 1024)) * 10 / (1024 * 1024)
        ),
    }
}

/// Prefix each line with its 1-indexed line number, right-aligned.
pub fn number_lines(content: &str, start: u32) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let last = (start as usize + lines.len()).max(1);
    let width = (last.ilog10() + 1) as usize;
    let mut out = String::with_capacity(content.len() + lines.len() * (width + 2));
    for (i, line) in lines.iter().enumerate() {
        let num = start as usize + i;
        let _ = writeln!(out, "{num:>width$}  {line}");
    }
    out
}

// ---------------------------------------------------------------------------
// Hashline support (edit mode)
// ---------------------------------------------------------------------------

/// FNV-1a hash of a line, truncated to 12 bits (3 hex chars).
/// Used as a per-line content checksum for edit-mode anchors.
pub(crate) fn line_hash(bytes: &[u8]) -> u16 {
    let mut h: u32 = 0x811c_9dc5;
    for &b in bytes {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    (h & 0xFFF) as u16
}

/// Format lines with hashline anchors: `{line}:{hash}|{content}`
/// Used in edit mode so the agent can reference lines by content hash.
pub fn hashlines(content: &str, start: u32) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = String::with_capacity(content.len() + lines.len() * 8);
    for (i, line) in lines.iter().enumerate() {
        let num = start as usize + i;
        let hash = line_hash(line.as_bytes());
        let _ = writeln!(out, "{num}:{hash:03x}|{line}");
    }
    out
}

/// Parse a hashline anchor `"42:a3f"` into `(line_number, hash)`.
/// Inverse of the format produced by [`hashlines`].
pub(crate) fn parse_anchor(s: &str) -> Option<(usize, u16)> {
    let (line_str, hash_str) = s.split_once(':')?;
    let line: usize = line_str.trim().parse().ok()?;
    if line == 0 {
        return None; // 1-indexed
    }
    let hash = u16::from_str_radix(hash_str.trim(), 16).ok()?;
    Some((line, hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn line_hash_deterministic() {
        let input = b"pub fn hello() -> String {";
        assert_eq!(line_hash(input), line_hash(input));
        assert_eq!(line_hash(b""), line_hash(b""));
    }

    #[test]
    fn line_hash_12_bit_range() {
        let inputs = [
            b"short".as_slice(),
            b"a much longer line with plenty of content to hash",
            b"",
            b"\t\tindented line",
            b"unicode: \xc3\xa9\xc3\xa0\xc3\xbc",
            b"pub struct Foo { bar: i32 }",
        ];
        for input in inputs {
            let h = line_hash(input);
            assert!(h <= 0xFFF, "hash {h:#x} exceeds 12-bit range");
        }
    }

    #[test]
    fn hashlines_format() {
        let content = "first line\nsecond line\nthird line";
        let output = hashlines(content, 1);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3);
        // Each line: N:HHH|content
        for (i, line) in lines.iter().enumerate() {
            let num = i + 1;
            assert!(
                line.starts_with(&format!("{num}:")),
                "line should start with line number"
            );
            assert!(line.contains('|'), "line should contain pipe separator");
            let after_pipe = line.split_once('|').unwrap().1;
            let expected = content.lines().nth(i).unwrap();
            assert_eq!(after_pipe, expected);
        }
    }

    #[test]
    fn parse_anchor_valid() {
        assert_eq!(parse_anchor("42:a3f"), Some((42, 0xa3f)));
        assert_eq!(parse_anchor("1:000"), Some((1, 0)));
        assert_eq!(parse_anchor("999:fff"), Some((999, 0xfff)));
    }

    #[test]
    fn parse_anchor_invalid() {
        assert_eq!(parse_anchor("0:a3f"), None); // line 0 invalid
        assert_eq!(parse_anchor("nocolon"), None);
        assert_eq!(parse_anchor(""), None);
        assert_eq!(parse_anchor(":abc"), None); // no line number
        assert_eq!(parse_anchor("42:ggg"), None); // invalid hex
    }

    #[test]
    fn number_lines_formatting() {
        let content = "alpha\nbeta\ngamma";
        let output = number_lines(content, 1);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains('1') && lines[0].contains("alpha"));
        assert!(lines[1].contains('2') && lines[1].contains("beta"));
        assert!(lines[2].contains('3') && lines[2].contains("gamma"));
        // Right-alignment: single-digit lines with width 1
        assert!(lines[0].starts_with("1  "));
    }

    #[test]
    fn search_header_format() {
        let header = search_header("foo", Path::new("/tmp/scope"), 10, 3, 7);
        assert!(header.contains("foo"), "should contain query");
        assert!(header.contains("/tmp/scope"), "should contain scope");
        assert!(header.contains("10 matches"), "should contain total");
        assert!(header.contains("3 definitions"), "should contain def count");
        assert!(header.contains("7 usages"), "should contain usage count");
    }
}
