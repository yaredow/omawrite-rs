#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadingItem {
    pub level: u8,
    pub title: String,
    pub line: u32,
}

/// Extracts Markdown headings from plain text.
///
/// Respects CommonMark rules:
/// - Supports ATX headings (`# Title` up to `###### Title`).
/// - Skips code fences (` ``` ` and `~~~`).
/// - Strips optional trailing closing hashes (e.g. `## Title ##`).
/// - Correctly computes 0-indexed line numbers.
pub fn extract_headings(text: &str) -> Vec<HeadingItem> {
    let mut headings = Vec::new();
    let mut in_code_block = false;
    let mut fence_char = ' ';

    for (line_idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();

        // Check for code fence start/end
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let current_char = trimmed.chars().next().unwrap();
            if !in_code_block {
                in_code_block = true;
                fence_char = current_char;
            } else if fence_char == current_char {
                in_code_block = false;
            }
            continue;
        }

        if in_code_block {
            continue;
        }

        // Check for ATX heading (# ...)
        if trimmed.starts_with('#') {
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
            if (1..=6).contains(&hash_count) {
                let rest = &trimmed[hash_count..];
                // Must be followed by whitespace or be empty
                if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
                    let mut title = rest.trim();
                    // Strip optional trailing hashes: "## Title ##" -> "Title"
                    if title.ends_with('#') {
                        let trailing_count = title.chars().rev().take_while(|&c| c == '#').count();
                        let candidate = title[..title.len() - trailing_count].trim();
                        if !candidate.is_empty() {
                            title = candidate;
                        }
                    }

                    headings.push(HeadingItem {
                        level: hash_count as u8,
                        title: if title.is_empty() {
                            "Untitled section".to_string()
                        } else {
                            title.to_string()
                        },
                        line: line_idx as u32,
                    });
                }
            }
        }
    }

    headings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_headings_hierarchy() {
        let text = r#"
# Document Title
Some introduction paragraph.

## Section 1
Details about section 1.

### Subsection 1.1
Deep details.

## Section 2 ##
Trailing hashes should be stripped.

###### Deep Header
End of document.
"#;

        let headings = extract_headings(text);
        assert_eq!(headings.len(), 5);

        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[0].title, "Document Title");
        assert_eq!(headings[0].line, 1);

        assert_eq!(headings[1].level, 2);
        assert_eq!(headings[1].title, "Section 1");
        assert_eq!(headings[1].line, 4);

        assert_eq!(headings[2].level, 3);
        assert_eq!(headings[2].title, "Subsection 1.1");
        assert_eq!(headings[2].line, 7);

        assert_eq!(headings[3].level, 2);
        assert_eq!(headings[3].title, "Section 2");
        assert_eq!(headings[3].line, 10);

        assert_eq!(headings[4].level, 6);
        assert_eq!(headings[4].title, "Deep Header");
        assert_eq!(headings[4].line, 13);
    }

    #[test]
    fn test_ignores_code_blocks() {
        let text = r#"
# Real Header

```bash
# This is a bash comment, not a markdown header!
echo "hello"
```

## Another Header

~~~python
# Python comment
~~~

### Final Header
"#;

        let headings = extract_headings(text);
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0].title, "Real Header");
        assert_eq!(headings[1].title, "Another Header");
        assert_eq!(headings[2].title, "Final Header");
    }

    #[test]
    fn test_handles_empty_and_non_headings() {
        let text = "C# programming is great\n#tag is not a heading\n# \n# Valid";
        let headings = extract_headings(text);
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0].title, "Untitled section");
        assert_eq!(headings[1].title, "Valid");
    }
}
