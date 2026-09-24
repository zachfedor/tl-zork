//! Output formatting for a projector: plain text, wrapped narrow.

/// Maximum output width in columns. Readable from the back of the room.
pub const WIDTH: usize = 64;

/// Word-wrap `text` to `width` columns, counted in chars rather than bytes.
///
/// Each `\n`-separated line wraps on its own, so paragraph breaks survive.
/// A line's leading spaces are kept and repeated on its continuation lines,
/// which keeps indented lists (like the inventory) aligned. A single word
/// longer than `width` is left whole rather than split.
pub fn wrap(text: &str, width: usize) -> String {
    let mut out = String::new();
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        // Indent is ASCII spaces, so byte length equals column count
        let indent = line.len() - line.trim_start_matches(' ').len();
        out.push_str(&line[..indent]);
        let mut col = indent;
        for word in line.split_whitespace() {
            let len = word.chars().count();
            if col > indent && col + 1 + len > width {
                out.push('\n');
                out.push_str(&line[..indent]);
                col = indent;
            } else if col > indent {
                out.push(' ');
                col += 1;
            }
            out.push_str(word);
            col += len;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_line_exceeds_width() {
        let text = "word ".repeat(100);
        for line in wrap(&text, WIDTH).lines() {
            assert!(line.chars().count() <= WIDTH, "too long: {line:?}");
        }
    }

    #[test]
    fn counts_chars_not_bytes() {
        // Each "—" is 3 bytes; 20 of them plus spaces fit in 64 columns
        let text = vec!["—"; 20].join(" ");
        assert_eq!(wrap(&text, WIDTH).lines().count(), 1);
    }

    #[test]
    fn keeps_paragraphs_and_indent() {
        let out = wrap("Title\n\n  one two three", 9);
        assert_eq!(out, "Title\n\n  one two\n  three");
    }

    #[test]
    fn empty_input_is_empty() {
        assert_eq!(wrap("", WIDTH), "");
    }
}
