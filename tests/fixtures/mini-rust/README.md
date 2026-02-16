# Mini Rust

## Usage

```rust
use mini_rust::Matcher;
use mini_rust::RegexMatcher;

fn main() {
    let matcher = RegexMatcher::new("pattern");
    let searcher = Searcher::new(matcher);
    searcher.search("input");
}
```

## API

The `Matcher` trait defines the core matching interface.
The `RegexMatcher` struct provides regex-based matching.
