mod clock;
mod game;
mod parser;
mod text;
mod world;

use std::io::{self, Write};
use std::panic::{self, AssertUnwindSafe};
use std::time::{Duration, Instant};

use game::{Game, Outcome};

/// First line printed, Zork-style, before the opening room.
const BANNER: &str = "Welcome to TechLancaster.      This version created Sep 24, 2026";

/// Printed if a command panics, instead of crashing on stage.
const RECOVERED: &str = "You lose your train of thought for a second. It comes back.";

fn main() {
    // Never show a Rust panic message on the projector
    panic::set_hook(Box::new(|_| {}));

    let started = Instant::now();
    let mut game = Game::new();
    println!("{BANNER}\n\n{}", game.start(Duration::ZERO));

    loop {
        print!("\n> ");
        let _ = io::stdout().flush();
        let Some(line) = read_line() else { break };

        let elapsed = started.elapsed();
        match panic::catch_unwind(AssertUnwindSafe(|| game.step(&line, elapsed))) {
            Ok(Outcome::Continue(text)) => println!("\n{text}"),
            Ok(Outcome::Ended(text)) => {
                println!("\n{text}");
                break;
            }
            Err(_) => println!("\n{RECOVERED}"),
        }
    }
}

/// Read one line from stdin, or `None` on EOF or a read error.
fn read_line() -> Option<String> {
    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_fits_the_projector() {
        assert!(BANNER.chars().count() <= text::WIDTH);
    }
}
