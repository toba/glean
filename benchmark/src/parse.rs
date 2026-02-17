use serde_json::Value;
use std::collections::HashMap;

/// A single tool invocation.
#[expect(dead_code)]
pub struct ToolCall {
    pub name: String,
    pub input: HashMap<String, Value>,
    pub tool_use_id: String,
    pub turn_index: usize,
}

/// One assistant turn with usage and tool calls.
#[expect(dead_code)]
pub struct Turn {
    pub index: usize,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub tool_calls: Vec<ToolCall>,
}

impl Turn {
    /// Total context processed this turn (input + cached).
    pub fn context_tokens(&self) -> u64 {
        self.input_tokens + self.cache_creation_tokens + self.cache_read_tokens
    }
}

/// Complete parsed result from a `claude -p` run.
#[expect(dead_code)]
pub struct RunResult {
    pub session_id: String,
    pub turns: Vec<Turn>,
    pub num_turns: u64,
    pub duration_ms: u64,
    pub duration_api_ms: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub result_text: String,
    // Filled in by runner after parsing
    pub task_name: String,
    pub mode_name: String,
    pub model_name: String,
    pub repetition: u32,
    pub correct: bool,
    pub correctness_reason: String,
}

/// Parse newline-delimited JSON output from `claude -p --output-format stream-json --verbose`.
pub fn parse_stream_json(raw_output: &str) -> RunResult {
    let mut session_id = String::new();
    let mut turns: Vec<Turn> = Vec::new();
    let mut all_text_parts: Vec<String> = Vec::new();
    let mut final_summary = Value::Null;
    let mut turn_index: usize = 0;

    for line in raw_output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let event: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        match event.get("type").and_then(Value::as_str) {
            Some("system") => {
                if let Some(sid) = event.get("session_id").and_then(Value::as_str) {
                    session_id = sid.to_string();
                }
            }
            Some("assistant") => {
                let message = event.get("message").cloned().unwrap_or(Value::Null);
                let usage = message.get("usage").cloned().unwrap_or(Value::Null);
                let content_blocks = message
                    .get("content")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();

                let mut tool_calls = Vec::new();
                let mut text_blocks: Vec<String> = Vec::new();

                for block in &content_blocks {
                    match block.get("type").and_then(Value::as_str) {
                        Some("tool_use") => {
                            let input_obj = block
                                .get("input")
                                .and_then(Value::as_object)
                                .cloned()
                                .unwrap_or_default();
                            tool_calls.push(ToolCall {
                                name: block
                                    .get("name")
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .to_string(),
                                input: input_obj.into_iter().collect(),
                                tool_use_id: block
                                    .get("id")
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .to_string(),
                                turn_index,
                            });
                        }
                        Some("text") => {
                            if let Some(t) = block.get("text").and_then(Value::as_str) {
                                text_blocks.push(t.to_string());
                            }
                        }
                        _ => {}
                    }
                }

                let turn = Turn {
                    index: turn_index,
                    input_tokens: usage
                        .get("input_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                    output_tokens: usage
                        .get("output_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                    cache_creation_tokens: usage
                        .get("cache_creation_input_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                    cache_read_tokens: usage
                        .get("cache_read_input_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                    tool_calls,
                };
                turns.push(turn);
                turn_index += 1;

                if !text_blocks.is_empty() {
                    all_text_parts.push(text_blocks.join("\n"));
                }
            }
            Some("result") => {
                final_summary = event;
            }
            _ => {}
        }
    }

    let usage = final_summary.get("usage").cloned().unwrap_or(Value::Null);

    RunResult {
        session_id,
        num_turns: final_summary
            .get("num_turns")
            .and_then(Value::as_u64)
            .unwrap_or(turns.len() as u64),
        duration_ms: final_summary
            .get("duration_ms")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        duration_api_ms: final_summary
            .get("duration_api_ms")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        total_input_tokens: usage
            .get("input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        total_output_tokens: usage
            .get("output_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        total_cache_creation_tokens: usage
            .get("cache_creation_input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        total_cache_read_tokens: usage
            .get("cache_read_input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        result_text: all_text_parts.join("\n"),
        turns,
        task_name: String::new(),
        mode_name: String::new(),
        model_name: String::new(),
        repetition: 0,
        correct: false,
        correctness_reason: String::new(),
    }
}

/// Count tool calls by name across all turns.
pub fn tool_call_counts(result: &RunResult) -> HashMap<String, u64> {
    let mut counts: HashMap<String, u64> = HashMap::new();
    for turn in &result.turns {
        for tc in &turn.tool_calls {
            *counts.entry(tc.name.clone()).or_insert(0) += 1;
        }
    }
    counts
}
