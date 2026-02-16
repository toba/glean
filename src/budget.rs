use crate::types::estimate_tokens;

/// Apply token budget to output. Works backwards from the cap:
/// 1. Reserve 50 tokens for header
/// 2. Truncate content at section boundaries to avoid broken output
/// 3. Never exceed the budget
pub fn apply(output: &str, budget: u64) -> String {
    let current = estimate_tokens(output.len() as u64);
    if current <= budget {
        return output.to_string();
    }

    let header_reserve = 50u64;
    let content_budget = budget.saturating_sub(header_reserve);
    let max_bytes = (content_budget * 4) as usize; // inverse of estimate_tokens

    // Find the first newline after the header (first line)
    let header_end = output.find('\n').unwrap_or(0);
    let header = &output[..header_end];
    let body = &output[header_end..];

    if body.len() <= max_bytes {
        return output.to_string();
    }

    let safe_max = body.floor_char_boundary(max_bytes);
    let truncated = &body[..safe_max];

    // Prefer section boundaries (\n\n##) to avoid cutting mid-match in search results
    let cut_point = truncated
        .rfind("\n\n##")
        .or_else(|| truncated.rfind("\n\n"))
        .or_else(|| truncated.rfind('\n'))
        .unwrap_or(max_bytes);

    let clean_body = &body[..cut_point];

    let omitted_bytes = output.len() - header_end - cut_point;
    let remaining_tokens = estimate_tokens(omitted_bytes as u64);
    format!(
        "{header}{clean_body}\n\n... truncated ({remaining_tokens} tokens omitted, budget: {budget})"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_budget_passes_through() {
        let output = "# header\nshort content";
        let result = apply(output, 1000);
        assert_eq!(result, output);
    }

    fn make_long_body() -> String {
        use std::fmt::Write;
        (0..200).fold(String::new(), |mut acc, i| {
            let _ = write!(acc, "\nline {i} with some content padding");
            acc
        })
    }

    #[test]
    fn over_budget_truncates() {
        let header = "# header";
        let body = make_long_body();
        let output = format!("{header}{body}");
        let result = apply(&output, 100);
        assert!(result.len() < output.len(), "should be shorter");
        assert!(result.contains("truncated"), "should mention truncation");
        assert!(result.contains("budget:"), "should mention budget");
    }

    #[test]
    fn truncation_preserves_header() {
        let header = "# my important header";
        let body = make_long_body();
        let output = format!("{header}{body}");
        let result = apply(&output, 100);
        assert!(
            result.starts_with(header),
            "header should be preserved: {result}"
        );
    }
}
