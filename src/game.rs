//! Game state and the per-command step function.
//!
//! Nothing here reads the real clock or prints. `step` takes the elapsed
//! time and returns text, so every timing path is testable instantly.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::clock;
use crate::parser::{self, Command};
use crate::text::{self, WIDTH};
use crate::world::{self, CONFIDENCE_STATES};

/// What the main loop should do after a command.
#[derive(Debug)]
pub enum Outcome {
    /// Print the text and read the next command.
    Continue(String),
    /// Print the text and exit; the talk is over.
    Ended(String),
}

/// All mutable game state. The world itself is static in `world.rs`.
pub struct Game {
    here: &'static str,
    /// Held item ids, in pickup order.
    inventory: Vec<&'static str>,
    /// Room id -> item ids currently in that room.
    room_items: HashMap<&'static str, Vec<&'static str>>,
    /// Index into `CONFIDENCE_STATES`; saturates at the last entry.
    confidence: usize,
    visited: HashSet<&'static str>,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// A fresh game in the start room with nothing carried.
    pub fn new() -> Self {
        let room_items = world::ROOMS
            .iter()
            .map(|r| (r.id, r.items.to_vec()))
            .collect();
        Game {
            here: world::START,
            inventory: Vec::new(),
            room_items,
            confidence: 0,
            visited: HashSet::new(),
        }
    }

    /// The opening text: the start room as a first visit, plus the status line.
    pub fn start(&mut self, elapsed: Duration) -> String {
        let body = self.enter(world::START, elapsed);
        format!("{}\n\n{}", text::wrap(&body, WIDTH), self.status(elapsed))
    }

    /// Run one command at the given elapsed time.
    ///
    /// The ending check comes before parsing, so once time is up no
    /// command of any kind gets processed.
    pub fn step(&mut self, input: &str, elapsed: Duration) -> Outcome {
        if clock::is_over(elapsed) {
            return Outcome::Ended(text::wrap(&self.ending(), WIDTH));
        }
        let body = match parser::parse(input) {
            Command::Go(dir) => self.go(dir, elapsed),
            Command::Look => self.describe(elapsed, false),
            Command::Inventory => self.inventory_text(elapsed),
            Command::Time => format!("It's {}.", clock::time_string(elapsed)),
            Command::Take(noun) => self.take(&noun),
            Command::Drop(noun) => self.drop(&noun),
            Command::Nothing => "Time passes. It does that.".to_string(),
            Command::Other(words) => self.other(&words, elapsed),
        };
        Outcome::Continue(format!(
            "{}\n\n{}",
            text::wrap(&body, WIDTH),
            self.status(elapsed)
        ))
    }

    /// Move to `id`, mark it visited, and describe it.
    fn enter(&mut self, id: &'static str, elapsed: Duration) -> String {
        self.here = id;
        let first = self.visited.insert(id);
        self.describe(elapsed, first)
    }

    /// The current room's title, description, items, and exits.
    ///
    /// The description paragraph is the base text, then the current
    /// phase's text, then the first-visit text when `first` is set.
    fn describe(&self, elapsed: Duration, first: bool) -> String {
        let Some(room) = world::room(self.here) else {
            return "You are somewhere. It's fine.".to_string();
        };
        let phase = clock::phase(elapsed);
        let mut paragraph = vec![room.description];
        if let Some((_, extra)) = room.phase_text.iter().find(|(p, _)| *p == phase) {
            paragraph.push(extra);
        }
        if first && let Some(extra) = room.first_visit {
            paragraph.push(extra);
        }

        let mut out = format!("{}\n\n{}", room.title, paragraph.join(" "));
        for item in self.items_here().iter().filter_map(|id| world::item(id)) {
            out.push_str(&format!("\nYou see {} here.", item.name));
        }
        if !room.exits.is_empty() {
            let exits: Vec<&str> = room.exits.iter().map(|(dir, _)| *dir).collect();
            out.push_str(&format!("\nExits: {}", exits.join(", ")));
        }
        out
    }

    /// Move through `dir`, or to the venue once the funnel has started.
    fn go(&mut self, dir: &str, elapsed: Duration) -> String {
        if clock::is_funnel(elapsed) {
            if self.here == world::VENUE {
                return "You're already where you need to be.".to_string();
            }
            let arrival = self.enter(world::VENUE, elapsed);
            return format!(
                "You set off. Somehow you're on Water Street. The lights are on at \
                Tellus360 and you can hear the room from here.\n\n{arrival}"
            );
        }
        let dest = world::room(self.here)
            .and_then(|r| r.exits.iter().find(|(d, _)| *d == dir))
            .map(|(_, to)| *to);
        match dest {
            Some(to) => self.enter(to, elapsed),
            None => "You can't go that way.".to_string(),
        }
    }

    /// Resolve words the parser couldn't classify.
    ///
    /// Only exit nicknames for now; room actions, easter eggs, and the
    /// rotating catch-all land here in later slices.
    fn other(&mut self, words: &str, elapsed: Duration) -> String {
        let is_exit =
            world::room(self.here).is_some_and(|r| r.exits.iter().any(|(d, _)| *d == words));
        if is_exit {
            return self.go(words, elapsed);
        }
        "I don't understand that.".to_string()
    }

    /// Item ids in the current room.
    fn items_here(&self) -> &[&'static str] {
        self.room_items.get(self.here).map_or(&[], Vec::as_slice)
    }

    /// Move a matching item from the room into the inventory.
    fn take(&mut self, noun: &str) -> String {
        if noun == "five minutes" {
            return "You already have those. For now.".to_string();
        }
        let items = self.room_items.entry(self.here).or_default();
        match items.iter().position(|id| matches_noun(id, noun)) {
            Some(idx) => {
                self.inventory.push(items.remove(idx));
                "Taken.".to_string()
            }
            None => format!("You don't see any {noun} here."),
        }
    }

    /// Move a matching item from the inventory into the room.
    fn drop(&mut self, noun: &str) -> String {
        if noun == "five minutes" {
            return "Time is not yours to discard.".to_string();
        }
        match self.inventory.iter().position(|id| matches_noun(id, noun)) {
            Some(idx) => {
                let id = self.inventory.remove(idx);
                self.room_items.entry(self.here).or_default().push(id);
                "Dropped.".to_string()
            }
            None => format!("You don't have any {noun}."),
        }
    }

    /// Current confidence as (inventory, status line) text.
    fn confidence_state(&self) -> (&'static str, &'static str) {
        CONFIDENCE_STATES
            .get(self.confidence)
            .or(CONFIDENCE_STATES.last())
            .copied()
            .unwrap_or(("confidence", "confidence"))
    }

    /// Held items, then confidence, five minutes, and the countdown.
    fn inventory_text(&self, elapsed: Duration) -> String {
        let mut lines = vec!["You are carrying:".to_string()];
        for item in self.inventory.iter().filter_map(|id| world::item(id)) {
            lines.push(format!("  {}", item.name));
        }
        lines.push(format!("  {}", self.confidence_state().0));
        lines.push("  five minutes".to_string());
        lines.push(format!("  {}", clock::countdown(elapsed)));
        lines.join("\n")
    }

    /// One-line status: game time, room, and carried things.
    ///
    /// Falls back to three items plus "…", then to no items, so it
    /// always fits in `WIDTH` columns.
    fn status(&self, elapsed: Duration) -> String {
        let time = clock::time_string(elapsed);
        let title = world::room(self.here).map_or("", |r| r.title);
        let mut things: Vec<&str> = self
            .inventory
            .iter()
            .filter_map(|id| world::item(id))
            .map(|item| item.short)
            .collect();
        things.push(self.confidence_state().1);
        things.push("five minutes");

        let full = format!("[ {time} — {title} — {} ]", things.join(", "));
        if full.chars().count() <= WIDTH {
            return full;
        }
        let first_three = things
            .iter()
            .take(3)
            .copied()
            .collect::<Vec<_>>()
            .join(", ");
        let short = format!("[ {time} — {title} — {first_three}, … ]");
        if short.chars().count() <= WIDTH {
            return short;
        }
        format!("[ {time} — {title} ]")
    }

    /// The closing text: fixed opening, keyed lines, fixed close, event details.
    fn ending(&self) -> String {
        // Keyed lines (idea route, confidence, visits, slides) come in a later slice
        let keyed = "Someone laughs at the part you weren't sure about. Someone \
            asks a question in the hallway afterward.";
        format!(
            "Someone calls your name.\n\n\
            You plug in. The room quiets. You have five minutes.\n\n\
            {keyed}\n\n\
            You had something. You always did.\n\n\
            {}",
            world::EVENT_DETAILS
        )
    }
}

/// True if `noun` is one of the item's aliases.
fn matches_noun(id: &str, noun: &str) -> bool {
    world::item(id).is_some_and(|item| item.aliases.contains(&noun))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(s: u64) -> Duration {
        Duration::from_secs(s)
    }

    fn ms(m: u64) -> Duration {
        Duration::from_millis(m)
    }

    /// True if `phrase` appears in `out`, ignoring where wrapping broke lines.
    fn said(out: &str, phrase: &str) -> bool {
        let flat: Vec<&str> = out.split_whitespace().collect();
        flat.join(" ").contains(phrase)
    }

    /// The text of a `Continue`; fails the test on `Ended`.
    fn cont(outcome: Outcome) -> String {
        match outcome {
            Outcome::Continue(t) => t,
            Outcome::Ended(t) => panic!("unexpected ending:\n{t}"),
        }
    }

    /// A game standing on the stoop, reached by walking at time zero.
    fn on_stoop() -> Game {
        let mut g = Game::new();
        g.start(secs(0));
        cont(g.step("out", secs(0)));
        cont(g.step("out", secs(0)));
        assert_eq!(g.here, "stoop");
        g
    }

    #[test]
    fn start_shows_description_phase_and_first_visit() {
        let mut g = Game::new();
        let out = g.start(secs(0));
        assert!(out.starts_with("The Apartment"));
        assert!(said(&out, "Fourteen browser tabs"));
        assert!(said(&out, "Grey light"));
        assert!(said(&out, "Five minutes, as promised"));
    }

    #[test]
    fn first_visit_text_shows_once() {
        let mut g = Game::new();
        g.start(secs(0));
        let out = cont(g.step("look", secs(1)));
        assert!(!said(&out, "as promised"));
    }

    #[test]
    fn phase_text_follows_the_clock() {
        let mut g = on_stoop();
        let morning = cont(g.step("look", secs(0)));
        let midday = cont(g.step("look", secs(120)));
        let afternoon = cont(g.step("look", secs(200)));
        let evening = cont(g.step("look", secs(250)));

        assert!(said(&morning, "Delivery trucks"));
        assert!(said(&midday, "hunting lunch") && !said(&midday, "Delivery trucks"));
        assert!(said(&afternoon, "long and gold") && !said(&afternoon, "hunting lunch"));
        assert!(said(&evening, "Streetlights"));
        // The base description is always there
        for out in [morning, midday, afternoon, evening] {
            assert!(said(&out, "King, Queen, Prince, Duke"));
        }
    }

    #[test]
    fn room_without_text_for_a_phase_shows_only_base() {
        let mut g = Game::new();
        g.start(secs(0));
        let out = cont(g.step("out", secs(120)));
        assert!(said(&out, "The coffee maker has done its best"));
        assert!(!said(&out, "still warm") && !said(&out, "pretzels"));
    }

    #[test]
    fn invalid_exit_before_funnel_stays_put() {
        let mut g = on_stoop();
        let out = cont(g.step("n", ms(239_999)));
        assert!(said(&out, "can't go that way"));
        assert_eq!(g.here, "stoop");
    }

    #[test]
    fn funnel_sends_any_movement_to_venue() {
        let mut g = on_stoop();
        let out = cont(g.step("n", secs(240)));
        assert_eq!(g.here, world::VENUE);
        assert!(said(&out, "Water Street"));
        assert!(said(&out, "Tellus360"));

        let again = cont(g.step("s", secs(241)));
        assert!(said(&again, "already where you need to be"));
    }

    #[test]
    fn funnel_overrides_real_exits() {
        let mut g = Game::new();
        g.start(secs(0));
        cont(g.step("out", secs(0)));
        // "back" is a real kitchen exit, but the funnel wins
        cont(g.step("back", secs(250)));
        assert_eq!(g.here, world::VENUE);
    }

    #[test]
    fn just_before_deadline_commands_still_run() {
        let mut g = Game::new();
        g.start(secs(0));
        assert!(matches!(g.step("look", ms(284_999)), Outcome::Continue(_)));
    }

    #[test]
    fn ending_fires_first_for_any_input() {
        for input in ["look", "n", "", "xyzzy", "take coffee", "i"] {
            let mut g = on_stoop();
            let outcome = g.step(input, secs(285));
            assert!(matches!(outcome, Outcome::Ended(_)), "input {input:?}");
            // The command itself was not processed
            assert_eq!(g.here, "stoop", "input {input:?}");
        }
    }

    #[test]
    fn ending_has_fixed_close_then_event_details() {
        let mut g = Game::new();
        let Outcome::Ended(out) = g.step("look", secs(300)) else {
            panic!("expected ending");
        };
        assert!(said(&out, "You had something. You always did."));
        assert!(
            out.trim_end()
                .ends_with(world::EVENT_DETAILS.lines().last().unwrap_or(""))
        );
    }

    #[test]
    fn inventory_lists_held_items_then_fixed_lines() {
        let mut g = Game::new();
        g.start(secs(0));
        cont(g.step("out", secs(0)));
        cont(g.step("take coffee", secs(0)));
        let out = cont(g.step("i", secs(0)));
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[1], "  a lukewarm coffee");
        assert_eq!(lines[2], "  confidence (factory sealed)");
        assert_eq!(lines[3], "  five minutes");
        assert!(lines[4].contains("Tech Lancaster starts in 13 hours"));
    }

    #[test]
    fn take_and_drop_move_items_between_room_and_inventory() {
        let mut g = Game::new();
        g.start(secs(0));
        cont(g.step("out", secs(0)));
        cont(g.step("get the mug", secs(0)));
        assert_eq!(g.inventory, vec!["coffee"]);
        assert!(g.items_here().is_empty());

        cont(g.step("back", secs(0)));
        cont(g.step("drop coffee", secs(0)));
        assert!(g.inventory.is_empty());
        assert_eq!(g.items_here(), &["coffee"]);
    }

    #[test]
    fn five_minutes_cannot_be_dropped() {
        let mut g = Game::new();
        let out = cont(g.step("drop five minutes", secs(0)));
        assert!(said(&out, "Time is not yours to discard."));
    }

    #[test]
    fn status_line_always_fits() {
        let mut g = Game::new();
        g.inventory = vec!["coffee"; 12];
        let status = g.status(secs(0));
        assert!(status.chars().count() <= WIDTH, "{status}");
        assert!(status.contains('…'));
    }

    #[test]
    fn no_output_line_exceeds_width() {
        let mut g = Game::new();
        let mut outputs = vec![g.start(secs(0))];
        let script = [
            ("look", 0),
            ("out", 30),
            ("take coffee", 60),
            ("out", 120),
            ("i", 200),
            ("n", 245),
            ("look", 260),
        ];
        for (input, t) in script {
            outputs.push(cont(g.step(input, secs(t))));
        }
        if let Outcome::Ended(t) = g.step("give talk", secs(285)) {
            outputs.push(t);
        }
        for out in outputs {
            for line in out.lines() {
                assert!(line.chars().count() <= WIDTH, "too long: {line:?}");
            }
        }
    }
}
