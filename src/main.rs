mod clock;
mod game;
mod parser;
mod text;
mod world;

use std::io::{self, Write};
use std::panic::{self, AssertUnwindSafe};
use std::time::{Duration, Instant};

use game::{Game, Outcome};

/// Shown before the clock starts, so setup time isn't game time.
const TITLE: &str =
    "FIVE MINUTES\n\nA text adventure in one lightning talk.\n\nPress Enter to begin.";

/// Printed if a command panics, instead of crashing on stage.
const RECOVERED: &str = "You lose your train of thought for a second. It comes back.";

fn main() {
    // Never show a Rust panic message on the projector
    panic::set_hook(Box::new(|_| {}));

    print!("{}", text::wrap(TITLE, text::WIDTH));
    let _ = io::stdout().flush();
    if read_line().is_none() {
        return;
    }

    let started = Instant::now();
    let mut game = Game::new();
    println!("\n{}", game.start(Duration::ZERO));

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
