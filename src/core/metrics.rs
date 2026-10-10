pub fn count_words(text: &str) -> usize {
    let mut count = 0;
    let mut in_word = false;

    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_alphanumeric() {
            if !in_word {
                in_word = true;
                count += 1;
            }
        } else if (c == '\'' || c == '-') && in_word {
            if let Some(&next) = chars.peek() {
                if !next.is_alphanumeric() {
                    in_word = false;
                }
            } else {
                in_word = false;
            }
        } else {
            in_word = false;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_count() {
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("Hello world"), 2);
        assert_eq!(count_words("It's a state-of-the-art markdown editor."), 5);
        assert_eq!(count_words("Word 123 another"), 3);
    }
}
