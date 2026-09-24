//! Turns a raw input line into a `Command`.
//!
//! The parser knows nothing about the world. Anything it can't classify
//! comes back as `Command::Other` for the game to resolve against the
//! current room (exit nicknames, room actions, easter eggs, catch-all).

/// Words dropped anywhere in the input before matching.
const FILLER: &[&str] = &["the", "a", "an", "please", "go", "to", "at", "on"];

/// Accepted direction words and the canonical direction each maps to.
const DIRECTIONS: &[(&str, &str)] = &[
    ("n", "north"),
    ("north", "north"),
    ("s", "south"),
    ("south", "south"),
    ("e", "east"),
    ("east", "east"),
    ("w", "west"),
    ("west", "west"),
    ("u", "up"),
    ("up", "up"),
    ("d", "down"),
    ("down", "down"),
    ("in", "in"),
    ("out", "out"),
    ("back", "back"),
];

/// A player action.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Move in a canonical direction ("north", "out", ...).
    Go(&'static str),
    Look,
    Inventory,
    Time,
    Take(String),
    Drop(String),
    /// Blank input.
    Nothing,
    /// Normalized words the parser couldn't classify.
    Other(String),
}

/// Lowercase, split on whitespace, and drop filler words.
pub fn normalize(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(str::to_lowercase)
        .filter(|w| !FILLER.contains(&w.as_str()))
        .collect()
}

/// The canonical direction for `word`, if it is one.
pub fn direction(word: &str) -> Option<&'static str> {
    DIRECTIONS
        .iter()
        .find(|(alias, _)| *alias == word)
        .map(|(_, canonical)| *canonical)
}

/// Parse a raw input line.
///
/// Directions are checked first, then global verbs.
pub fn parse(input: &str) -> Command {
    let words = normalize(input);
    let words: Vec<&str> = words.iter().map(String::as_str).collect();
    if let [word] = words.as_slice()
        && let Some(dir) = direction(word)
    {
        return Command::Go(dir);
    }
    match words.as_slice() {
        [] => Command::Nothing,
        ["look" | "l"] => Command::Look,
        ["inventory" | "i"] => Command::Inventory,
        ["time"] => Command::Time,
        ["take" | "get", noun @ ..] if !noun.is_empty() => Command::Take(noun.join(" ")),
        ["drop", noun @ ..] if !noun.is_empty() => Command::Drop(noun.join(" ")),
        _ => Command::Other(words.join(" ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directions_with_filler_and_case() {
        assert_eq!(parse("N"), Command::Go("north"));
        assert_eq!(parse("  go   north "), Command::Go("north"));
        assert_eq!(parse("please go out"), Command::Go("out"));
        assert_eq!(
            parse("go to the market"),
            Command::Other("market".to_string())
        );
    }

    #[test]
    fn multi_word_nouns() {
        assert_eq!(
            parse("drop five minutes"),
            Command::Drop("five minutes".to_string())
        );
        assert_eq!(
            parse("take the coffee"),
            Command::Take("coffee".to_string())
        );
    }

    #[test]
    fn blank_and_unknown() {
        assert_eq!(parse(""), Command::Nothing);
        assert_eq!(parse("   "), Command::Nothing);
        assert_eq!(parse("take"), Command::Other("take".to_string()));
        assert_eq!(parse("xyzzy"), Command::Other("xyzzy".to_string()));
    }
}
