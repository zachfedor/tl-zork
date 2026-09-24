//! Game state and the per-command step function.
//!
//! Nothing here reads the real clock or prints. `step` takes the elapsed
//! time and returns text, so every timing path is testable instantly.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::clock;
use crate::parser::{self, Command};
use crate::text::{self, WIDTH};
use crate::world::{self, Action, Effect, Route};

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
    visited: HashSet<&'static str>,
    /// How the idea was found; `Some` exactly when the idea is held.
    idea_route: Option<Route>,
    /// False during the dream.
    awake: bool,
    /// Elapsed time at waking; the day's clock starts here.
    woke_at: Duration,
    /// Count of dream loops so far, to rotate their lines.
    dream_loops: usize,
    /// Set by `Effect::Win`; the step that sets it ends the game.
    won: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// A fresh game at the start of the dream with nothing carried.
    pub fn new() -> Self {
        let room_items = world::ROOMS
            .iter()
            .map(|r| (r.id, r.items.to_vec()))
            .collect();
        Game {
            here: world::START,
            inventory: Vec::new(),
            room_items,
            visited: HashSet::new(),
            idea_route: None,
            awake: false,
            woke_at: Duration::ZERO,
            dream_loops: 0,
            won: false,
        }
    }

    /// The opening text: the start room as a first visit.
    pub fn start(&mut self, elapsed: Duration) -> String {
        let body = self.enter(world::START, elapsed);
        text::wrap(&body, WIDTH)
    }

    /// Run one command at the given elapsed time.
    ///
    /// Time checks come before parsing: the ending, then the forced
    /// wake. Either one replaces the command.
    pub fn step(&mut self, input: &str, elapsed: Duration) -> Outcome {
        if clock::is_over(elapsed) {
            return Outcome::Ended(text::wrap(&self.ending(true), WIDTH));
        }
        let body = if !self.awake && clock::is_wake_time(elapsed) {
            format!("{}\n\n{}", world::FORCED_WAKE, self.wake(elapsed))
        } else {
            self.run(parser::parse(input), elapsed)
        };
        if self.won {
            let text = format!("{body}\n\n{}", self.ending(false));
            return Outcome::Ended(text::wrap(&text, WIDTH));
        }
        Outcome::Continue(text::wrap(&body, WIDTH))
    }

    /// Carry out a parsed command and return its text.
    fn run(&mut self, command: Command, elapsed: Duration) -> String {
        match command {
            Command::Go(dir) => self.go(dir, elapsed),
            Command::Look => self.look(elapsed),
            Command::Inventory => self.inventory_text(elapsed),
            Command::Time if !self.awake => "The clocks here don't have hands.".to_string(),
            Command::Time => format!("It's {}.", clock::time_string(elapsed, self.woke_at)),
            Command::Take(noun) => self.take(&noun, elapsed),
            Command::Drop(noun) => self.drop(&noun),
            Command::Nothing => "Time passes. It does that.".to_string(),
            Command::Other(words) => self.other(&words, elapsed),
        }
    }

    /// Leave the dream: start the day's clock and arrive in the bedroom.
    ///
    /// Nothing carried in the dream comes with you.
    fn wake(&mut self, elapsed: Duration) -> String {
        self.awake = true;
        self.woke_at = elapsed;
        self.inventory.clear();
        self.enter(world::WAKE, elapsed)
    }

    /// Move to `id` and describe it on the first visit.
    ///
    /// Revisits get a single line to save screen space, carrying the
    /// same countdown as the inventory once awake; `look` still gives
    /// the full description.
    fn enter(&mut self, id: &'static str, elapsed: Duration) -> String {
        self.here = id;
        if self.visited.insert(id) {
            return self.describe(elapsed, true);
        }
        let title = world::room(id).map_or("somewhere", |r| r.title);
        let back = format!("You're back at {}", mid_sentence(title));
        if !self.awake {
            return format!("{back}.");
        }
        format!("{back}, {}.", clock::countdown(elapsed, self.woke_at))
    }

    /// The current room's title, description, items, and exits.
    ///
    /// The description paragraph is the base text, then the current
    /// phase's text, then the first-visit text when `first` is set, then
    /// the dream's alarm once it has started.
    fn describe(&self, elapsed: Duration, first: bool) -> String {
        let Some(room) = world::room(self.here) else {
            return "You are somewhere. It's fine.".to_string();
        };
        let phase = clock::phase(elapsed, self.woke_at);
        let mut paragraph = vec![room.description];
        if self.awake
            && let Some((_, extra)) = room.phase_text.iter().find(|(p, _)| *p == phase)
        {
            paragraph.push(extra);
        }
        if first && let Some(extra) = room.first_visit {
            paragraph.push(extra);
        }
        if !self.awake && clock::is_beeping(elapsed) {
            paragraph.push(world::DREAM_BEEP);
        }

        let mut out = format!("{}\n\n{}", room.title, paragraph.join(" "));
        for item in self.items_here().iter().filter_map(|id| world::item(id)) {
            out.push_str(&format!("\nYou see {} here.", item.name));
        }
        // Nicknames are exits too, but only real directions get listed
        let exits: Vec<&str> = room
            .exits
            .iter()
            .map(|(dir, _)| *dir)
            .filter(|dir| parser::direction(dir).is_some())
            .collect();
        if !exits.is_empty() {
            out.push_str(&format!("\nExits: {}", exits.join(", ")));
        }
        out
    }

    /// Describe the room, then run its `look` action if it has one.
    fn look(&mut self, elapsed: Duration) -> String {
        let mut out = self.describe(elapsed, false);
        if let Some(action) = self.find_action("look") {
            out.push_str("\n\n");
            out.push_str(&self.perform(action, elapsed));
        }
        out
    }

    /// Move through `dir`, or to the venue once the funnel has started.
    ///
    /// In the dream, any move without an exit loops back to the field.
    fn go(&mut self, dir: &str, elapsed: Duration) -> String {
        if clock::is_funnel(elapsed) {
            if self.here == world::VENUE {
                return "You're already where you need to be.".to_string();
            }
            let arrival = self.enter(world::VENUE, elapsed);
            return format!(
                "You set off. Somehow you're on Buchanan Avenue. The lights are on \
                at West Art and you can hear the room from here.\n\n{arrival}"
            );
        }
        let dest = world::room(self.here)
            .and_then(|r| r.exits.iter().find(|(d, _)| *d == dir))
            .map(|(_, to)| *to);
        match dest {
            Some(to) => self.enter(to, elapsed),
            None if !self.awake => {
                let lines = world::DREAM_LOOPS;
                let line = lines.get(self.dream_loops % lines.len()).unwrap_or(&"");
                self.dream_loops += 1;
                format!("{line}\n\n{}", self.enter(world::START, elapsed))
            }
            None => "You can't go that way.".to_string(),
        }
    }

    /// Resolve words the parser couldn't classify.
    ///
    /// Exit nicknames first, then the current room's actions. Easter eggs
    /// and the rotating catch-all land here in a later slice.
    fn other(&mut self, words: &str, elapsed: Duration) -> String {
        let is_exit =
            world::room(self.here).is_some_and(|r| r.exits.iter().any(|(d, _)| *d == words));
        if is_exit {
            return self.go(words, elapsed);
        }
        if let Some(action) = self.find_action(words) {
            return self.perform(action, elapsed);
        }
        "I don't understand that.".to_string()
    }

    /// The first action in this room that matches `words` and is allowed.
    ///
    /// A disallowed action is skipped rather than refused, so the input
    /// falls through to later layers like any other unmatched command.
    fn find_action(&self, words: &str) -> Option<&'static Action> {
        world::room(self.here)?
            .actions
            .iter()
            .find(|a| a.matches(words) && self.allows(a))
    }

    /// True if every required item is held and no forbidden item is.
    fn allows(&self, action: &Action) -> bool {
        action.requires.iter().all(|id| self.inventory.contains(id))
            && !action.forbids.iter().any(|id| self.inventory.contains(id))
    }

    /// Apply an action's effects in order and return its text.
    ///
    /// The response comes first; effects that move the player append the
    /// new room's description after it.
    fn perform(&mut self, action: &Action, elapsed: Duration) -> String {
        let mut out = action.response.to_string();
        for effect in action.effects {
            match effect {
                Effect::Gain(id) => self.gain(id),
                Effect::GainIdea(route) => {
                    // Only announce the idea when this is the route that grants it
                    if self.idea_route.is_none() {
                        self.idea_route = Some(*route);
                        self.gain(world::IDEA);
                        out = format!("{out} {}", world::IDEA_FOUND);
                    }
                }
                Effect::Teleport(to) => {
                    let arrival = self.enter(to, elapsed);
                    out = format!("{out}\n\n{arrival}");
                }
                Effect::Wake => {
                    let arrival = self.wake(elapsed);
                    out = format!("{out}\n\n{arrival}");
                }
                Effect::Win => self.won = true,
            }
        }
        out
    }

    /// Add an item to the inventory unless it's already held.
    fn gain(&mut self, id: &'static str) {
        if !self.inventory.contains(&id) {
            self.inventory.push(id);
        }
    }

    /// Item ids in the current room.
    fn items_here(&self) -> &[&'static str] {
        self.room_items.get(self.here).map_or(&[], Vec::as_slice)
    }

    /// Move a matching item from the room into the inventory.
    ///
    /// With no such item, a room action for "take <noun>" gets a chance,
    /// since "take letter" reads as picking something up.
    fn take(&mut self, noun: &str, elapsed: Duration) -> String {
        if noun == "five minutes" {
            return "You already have those. For now.".to_string();
        }
        let items = self.room_items.entry(self.here).or_default();
        if let Some(idx) = items.iter().position(|id| matches_noun(id, noun)) {
            self.inventory.push(items.remove(idx));
            return "Taken.".to_string();
        }
        if let Some(action) = self.find_action(&format!("take {noun}")) {
            return self.perform(action, elapsed);
        }
        format!("You don't see any {noun} here.")
    }

    /// Move a matching item from the inventory into the room.
    fn drop(&mut self, noun: &str) -> String {
        if noun == "five minutes" {
            return "Time is not yours to discard.".to_string();
        }
        if matches_noun(world::IDEA, noun) && self.idea_route.is_some() {
            return "You put it down. It follows you anyway.".to_string();
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

    /// Held items, then the time left as the last thing you "have".
    ///
    /// In the dream that's all the time in the world; awake, it's the
    /// countdown. With nothing held, it collapses into one sentence.
    fn inventory_text(&self, elapsed: Duration) -> String {
        if !self.awake && self.inventory.is_empty() {
            return "You have nothing but all the time in the world.".to_string();
        }
        let countdown = if self.awake {
            clock::countdown(elapsed, self.woke_at)
        } else {
            "and all the time in the world".to_string()
        };
        let names: Vec<&str> = self
            .inventory
            .iter()
            .filter_map(|id| world::item(id))
            .map(|item| item.name)
            .collect();
        if names.is_empty() {
            return format!("You have nothing, {countdown}.");
        }
        let mut lines = vec!["You have:".to_string()];
        lines.extend(names.iter().map(|name| format!("  {name}")));
        lines.push(format!("  {countdown}"));
        lines.join("\n")
    }

    /// The closing text: fixed opening, keyed lines, fixed close, event details.
    ///
    /// `called` is true when time ran out, so the room calls the player up;
    /// false when the player gave the talk themselves.
    fn ending(&self, called: bool) -> String {
        let opening = if called {
            "Someone calls your name.\n\n"
        } else {
            ""
        };
        let route = match self.idea_route {
            None => {
                "You never found the idea. You talk anyway, about what you already \
                know, the way you'd explain it to a friend. Halfway through, you \
                notice the room is leaning in."
            }
            Some(Route::Overheard) => {
                "You start with where you heard it: two strangers arguing at Mean \
                Cup. A few people nod like they've sat at that table too."
            }
            Some(Route::WarStory) => {
                "You start with the part that went wrong. The room relaxes. \
                Everyone has one of those."
            }
            Some(Route::ExplainedIt) => {
                "You explain it the way you explained it to the duck. Unlike the \
                duck, the room laughs in the right places."
            }
            Some(Route::StoppedLooking) => {
                "You admit you found it on a bench by the creek, once you stopped \
                looking. Someone in the back laughs, then nods."
            }
            Some(Route::SlowedDown) => {
                "You start with a farmer and a fence post. It sounds like it's going \
                nowhere, right up until it isn't."
            }
        };
        format!(
            "{opening}\
            You plug in. The room quiets. You have five minutes.\n\n\
            {route} Someone asks a question in the hallway afterward. Someone \
            says \"I've been meaning to do one of these.\"\n\n\
            You had something. You always did.\n\n\
            {}\n\n\
            {}",
            world::PUNCHLINE,
            world::EVENT_DETAILS
        )
    }
}

/// A room title that reads naturally after "at": "The Kitchen" becomes
/// "the kitchen", "A Field" becomes "the field", and names like "Mean Cup"
/// stay as they are.
fn mid_sentence(title: &str) -> String {
    let has_article = ["A ", "The ", "Your "]
        .iter()
        .any(|article| title.starts_with(article));
    if !has_article {
        return title.to_string();
    }
    let lowered = title.to_lowercase();
    match lowered.strip_prefix("a ") {
        Some(rest) => format!("the {rest}"),
        None => lowered,
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

    /// Run a sequence of commands at time zero, failing on any ending.
    fn walk(g: &mut Game, inputs: &[&str]) {
        for input in inputs {
            cont(g.step(input, secs(0)));
        }
    }

    /// A game just woken in the bedroom at time zero.
    fn awake() -> Game {
        let mut g = Game::new();
        g.start(secs(0));
        walk(&mut g, &["open mailbox", "read letter"]);
        assert_eq!(g.here, world::WAKE);
        g
    }

    /// A game standing on the square, reached by walking at time zero.
    fn on_square() -> Game {
        let mut g = awake();
        walk(&mut g, &["out", "out"]);
        assert_eq!(g.here, "square");
        g
    }

    // ── The dream ────────────────────────────────────────────────────

    #[test]
    fn game_opens_in_the_dream() {
        let mut g = Game::new();
        let out = g.start(secs(0));
        assert!(out.starts_with("A Field"));
        assert!(said(&out, "small mailbox"));
    }

    #[test]
    fn dream_ring_moves_normally() {
        let mut g = Game::new();
        g.start(secs(0));
        for (dir, room) in [
            ("n", "dream_north"),
            ("e", "dream_east"),
            ("s", "dream_south"),
            ("w", "dream_field"),
        ] {
            cont(g.step(dir, secs(1)));
            assert_eq!(g.here, room, "after {dir}");
        }
    }

    #[test]
    fn walking_away_loops_back_with_rotating_lines() {
        let mut g = Game::new();
        g.start(secs(0));
        cont(g.step("n", secs(1)));
        let first = cont(g.step("n", secs(2)));
        assert_eq!(g.here, "dream_field");
        let second = cont(g.step("in", secs(3)));
        assert_eq!(g.here, "dream_field");
        assert!(said(&first, world::DREAM_LOOPS[0]));
        assert!(said(&second, world::DREAM_LOOPS[1]));
    }

    #[test]
    fn window_teleports_to_the_field() {
        let mut g = Game::new();
        g.start(secs(0));
        walk(&mut g, &["n", "e"]);
        let out = cont(g.step("open window", secs(1)));
        assert_eq!(g.here, "dream_field");
        assert!(said(&out, "climb through"));
        assert!(said(&out, "You're back at the field."));
    }

    #[test]
    fn beeping_starts_at_twenty_seconds() {
        let mut g = Game::new();
        g.start(secs(0));
        assert!(!said(&cont(g.step("look", ms(19_999))), world::DREAM_BEEP));
        assert!(said(&cont(g.step("look", secs(20))), world::DREAM_BEEP));
    }

    #[test]
    fn dream_has_no_time_and_nothing_carried() {
        let mut g = Game::new();
        g.start(secs(0));
        assert!(said(&cont(g.step("time", secs(1))), "don't have hands"));
        assert!(said(
            &cont(g.step("i", secs(1))),
            "all the time in the world"
        ));
    }

    #[test]
    fn every_way_of_getting_the_letter_then_reading_it_wakes() {
        for get in [
            "open mailbox",
            "check the mailbox",
            "take letter",
            "get the letter",
        ] {
            for read in ["read letter", "open the letter"] {
                let mut g = Game::new();
                g.start(secs(0));
                walk(&mut g, &[get]);
                assert!(g.inventory.contains(&world::LETTER), "{get:?}");
                let out = cont(g.step(read, secs(5)));
                assert!(g.awake, "{get:?} then {read:?}");
                assert_eq!(g.here, world::WAKE);
                assert!(said(&out, "Beep!"));
                assert!(said(&out, "all the time in the world"));
            }
        }
    }

    #[test]
    fn opening_mailbox_takes_the_letter_without_waking() {
        let mut g = Game::new();
        g.start(secs(0));
        let out = cont(g.step("open mailbox", secs(1)));
        assert!(said(&out, "reveals a letter"));
        assert!(g.inventory.contains(&world::LETTER));
        assert!(!g.awake);
    }

    #[test]
    fn letter_can_be_read_in_any_dream_room_once_held() {
        for path in [&["n"][..], &["s"], &["n", "e"]] {
            let mut g = Game::new();
            g.start(secs(0));
            walk(&mut g, &["take letter"]);
            walk(&mut g, path);
            assert_ne!(g.here, "dream_field", "{path:?}");
            walk(&mut g, &["read letter"]);
            assert!(g.awake, "{path:?}");
        }
    }

    #[test]
    fn letter_must_be_taken_before_reading() {
        let mut g = Game::new();
        g.start(secs(0));
        let out = cont(g.step("read letter", secs(1)));
        assert!(said(&out, "still in the mailbox"));
        assert!(!g.awake);
    }

    #[test]
    fn dream_inventory_lists_the_letter() {
        let mut g = Game::new();
        g.start(secs(0));
        walk(&mut g, &["take letter"]);
        let out = cont(g.step("i", secs(1)));
        assert_eq!(
            out,
            "You have:\n  a letter\n  and all the time in the world"
        );
    }

    #[test]
    fn the_letter_stays_in_the_dream() {
        let g = awake();
        assert!(g.inventory.is_empty());
    }

    #[test]
    fn revisits_are_one_line_but_look_is_full() {
        let mut g = on_square();
        walk(&mut g, &["w"]);
        let back = cont(g.step("e", secs(0)));
        assert!(said(
            &back,
            "You're back at the square, and only 13 hours until TechLancaster."
        ));
        let look = cont(g.step("look", secs(0)));
        assert!(said(&look, "Red brick buildings"));

        walk(&mut g, &["in"]);
        let back = cont(g.step("back", secs(0)));
        assert!(said(
            &back,
            "You're back at your bedroom, and only 13 hours until TechLancaster."
        ));
    }

    #[test]
    fn revisit_line_counts_down_like_the_inventory() {
        let mut g = on_square();
        walk(&mut g, &["w"]);
        // Revisits happen before the funnel, so the countdown is in hours
        let later = cont(g.step("e", secs(230)));
        assert!(said(&later, "and only 3 hours until TechLancaster."));
        let inventory = cont(g.step("i", secs(230)));
        assert!(said(&inventory, "and only 3 hours until TechLancaster"));
    }

    #[test]
    fn dream_revisits_have_no_countdown() {
        let mut g = Game::new();
        g.start(secs(0));
        walk(&mut g, &["n"]);
        assert_eq!(cont(g.step("w", secs(1))), "You're back at the field.");
    }

    #[test]
    fn titles_read_naturally_mid_sentence() {
        assert_eq!(mid_sentence("A Field"), "the field");
        assert_eq!(mid_sentence("The Square"), "the square");
        assert_eq!(mid_sentence("Your Bedroom"), "your bedroom");
        assert_eq!(mid_sentence("Mean Cup"), "Mean Cup");
        assert_eq!(mid_sentence("Amish Country"), "Amish Country");
    }

    #[test]
    fn dream_forces_wake_at_35_seconds_instead_of_the_command() {
        let mut g = Game::new();
        g.start(secs(0));
        cont(g.step("look", ms(34_999)));
        assert!(!g.awake);

        let out = cont(g.step("n", secs(35)));
        assert!(g.awake);
        // The command was replaced, so the player didn't move past the bedroom
        assert_eq!(g.here, world::WAKE);
        assert!(said(&out, "already been read"));
        assert!(said(&cont(g.step("time", secs(35))), "6:00am"));
    }

    #[test]
    fn day_starts_at_wake() {
        let mut g = Game::new();
        g.start(secs(0));
        cont(g.step("look", secs(40)));
        assert_eq!(g.woke_at, secs(40));
        let out = cont(g.step("time", secs(40)));
        assert!(said(&out, "6:00am"));
    }

    // ── The city ─────────────────────────────────────────────────────

    #[test]
    fn first_visit_text_shows_once() {
        let mut g = awake();
        let out = cont(g.step("look", secs(1)));
        assert!(!said(&out, "as promised"));
    }

    #[test]
    fn phase_text_follows_the_clock() {
        let mut g = on_square();
        let morning = cont(g.step("look", secs(0)));
        let midday = cont(g.step("look", secs(120)));
        let afternoon = cont(g.step("look", secs(200)));
        let evening = cont(g.step("look", secs(250)));

        assert!(said(&morning, "sweeping"));
        assert!(said(&midday, "lunch crowd") && !said(&midday, "sweeping"));
        assert!(said(&afternoon, "long and gold") && !said(&afternoon, "lunch crowd"));
        assert!(said(&evening, "Streetlights"));
        // The base description is always there
        for out in [morning, midday, afternoon, evening] {
            assert!(said(&out, "Red brick buildings"));
        }
    }

    #[test]
    fn room_without_text_for_a_phase_shows_only_base() {
        let mut g = awake();
        let out = cont(g.step("out", secs(120)));
        assert!(said(&out, "The coffee maker has done its best"));
        assert!(!said(&out, "still warm") && !said(&out, "pretzels"));
    }

    #[test]
    fn square_reaches_every_spoke_and_back() {
        for (dir, room, back) in [
            ("w", "mean_cup", "e"),
            ("n", "office", "s"),
            ("s", "county_park", "n"),
            ("e", "amish_country", "w"),
        ] {
            let mut g = on_square();
            walk(&mut g, &[dir]);
            assert_eq!(g.here, room);
            walk(&mut g, &[back]);
            assert_eq!(g.here, "square", "back from {room}");
        }
    }

    #[test]
    fn square_description_points_at_every_spoke() {
        let mut g = on_square();
        let out = cont(g.step("look", secs(0)));
        for place in ["Mean Cup", "work", "County Park", "Amish Country"] {
            assert!(said(&out, place), "{place}");
        }
    }

    #[test]
    fn nicknames_work_but_are_not_listed() {
        let mut g = on_square();
        let out = cont(g.step("look", secs(0)));
        assert!(out.ends_with("Exits: west, north, south, east, in"));
        walk(&mut g, &["go to work"]);
        assert_eq!(g.here, "office");
    }

    #[test]
    fn getting_coffee_at_the_square_heads_to_mean_cup() {
        let mut g = on_square();
        walk(&mut g, &["get coffee"]);
        assert_eq!(g.here, "mean_cup");
    }

    #[test]
    fn invalid_exit_in_the_city_stays_put() {
        let mut g = on_square();
        let out = cont(g.step("up", ms(239_999)));
        assert!(said(&out, "can't go that way"));
        assert_eq!(g.here, "square");
    }

    // ── Idea routes and effects ──────────────────────────────────────

    #[test]
    fn every_spoke_has_a_route_to_the_idea() {
        for (path, route) in [
            (&["w", "eavesdrop"][..], Route::Overheard),
            (&["w", "listen to the strangers"], Route::Overheard),
            (&["n", "describe blocker"], Route::WarStory),
            (&["n", "talk to the duck"], Route::ExplainedIt),
            (&["s", "sit on the bench"], Route::StoppedLooking),
            (&["s", "walk the trail"], Route::StoppedLooking),
            (&["e", "help the farmer"], Route::SlowedDown),
            (&["e", "help"], Route::SlowedDown),
        ] {
            let mut g = on_square();
            walk(&mut g, &path[..path.len() - 1]);
            let out = cont(g.step(path[path.len() - 1], secs(0)));
            assert_eq!(g.idea_route, Some(route), "{path:?}");
            assert!(g.inventory.contains(&world::IDEA), "{path:?}");
            assert!(said(&out, world::IDEA_FOUND), "{path:?}");
        }
    }

    #[test]
    fn idea_is_only_announced_when_granted() {
        let mut g = on_square();
        walk(&mut g, &["w", "eavesdrop", "e", "n"]);
        let out = cont(g.step("describe blocker", secs(0)));
        assert!(said(&out, "war story"));
        assert!(!said(&out, world::IDEA_FOUND));
    }

    #[test]
    fn first_idea_route_wins() {
        let mut g = on_square();
        walk(&mut g, &["w", "eavesdrop", "e", "n", "describe blocker"]);
        assert_eq!(g.idea_route, Some(Route::Overheard));
        let ideas = g.inventory.iter().filter(|id| **id == world::IDEA).count();
        assert_eq!(ideas, 1);
    }

    #[test]
    fn saying_no_blockers_gains_nothing() {
        let mut g = on_square();
        walk(&mut g, &["n", "say no blockers"]);
        assert!(g.inventory.is_empty());
        assert_eq!(g.idea_route, None);
    }

    #[test]
    fn amish_stand_sells_pie_without_the_idea() {
        for input in ["buy pie", "take pie", "buy whoopie pie"] {
            let mut g = on_square();
            walk(&mut g, &["e", input]);
            assert_eq!(g.inventory, vec!["whoopie_pie"], "{input:?}");
            assert_eq!(g.idea_route, None, "{input:?}");
        }
    }

    #[test]
    fn the_idea_cannot_be_dropped() {
        let mut g = on_square();
        walk(&mut g, &["w", "listen"]);
        let out = cont(g.step("drop idea", secs(0)));
        assert!(said(&out, "It follows you anyway."));
        assert!(g.inventory.contains(&world::IDEA));
    }

    // ── Funnel and ending ────────────────────────────────────────────

    #[test]
    fn funnel_sends_any_movement_to_venue() {
        let mut g = on_square();
        let out = cont(g.step("up", secs(240)));
        assert_eq!(g.here, world::VENUE);
        assert!(said(&out, "Buchanan Avenue"));
        assert!(said(&out, "West Art"));

        let again = cont(g.step("s", secs(241)));
        assert!(said(&again, "already where you need to be"));
    }

    #[test]
    fn funnel_overrides_real_exits() {
        let mut g = awake();
        walk(&mut g, &["out"]);
        // "back" is a real kitchen exit, but the funnel wins
        cont(g.step("back", secs(250)));
        assert_eq!(g.here, world::VENUE);
    }

    #[test]
    fn just_before_deadline_commands_still_run() {
        let mut g = awake();
        assert!(matches!(g.step("look", ms(284_999)), Outcome::Continue(_)));
    }

    #[test]
    fn ending_fires_first_for_any_input() {
        for input in ["look", "n", "", "xyzzy", "take coffee", "i", "give talk"] {
            let mut g = on_square();
            let outcome = g.step(input, secs(285));
            assert!(matches!(outcome, Outcome::Ended(_)), "input {input:?}");
            // The command itself was not processed
            assert_eq!(g.here, "square", "input {input:?}");
        }
    }

    #[test]
    fn ending_beats_the_forced_wake() {
        let mut g = Game::new();
        g.start(secs(0));
        assert!(matches!(g.step("look", secs(285)), Outcome::Ended(_)));
    }

    #[test]
    fn ending_has_fixed_close_then_punchline_then_event_details() {
        let mut g = Game::new();
        let Outcome::Ended(out) = g.step("look", secs(300)) else {
            panic!("expected ending");
        };
        // Search the unwrapped text so line breaks can't hide a phrase
        let flat = out.split_whitespace().collect::<Vec<_>>().join(" ");
        let close = flat.find("You had something. You always did.");
        let punch = flat.find("five more minutes");
        let details = flat.find("TechLancaster");
        assert!(close.is_some() && close < punch && punch < details, "{out}");
        assert!(
            out.trim_end()
                .ends_with(world::EVENT_DETAILS.lines().last().unwrap_or(""))
        );
    }

    #[test]
    fn timeout_ending_calls_the_player_up() {
        let mut g = Game::new();
        let Outcome::Ended(out) = g.step("look", secs(285)) else {
            panic!("expected ending");
        };
        assert!(said(&out, "Someone calls your name."));
        assert!(said(&out, "October 22, 2026"));
        assert!(said(&out, "816 Buchanan Ave"));
    }

    #[test]
    fn ending_is_keyed_by_route() {
        let mut g = on_square();
        let Outcome::Ended(none) = g.step("look", secs(285)) else {
            panic!("expected ending");
        };
        assert!(said(&none, "You never found the idea."));

        let mut g = on_square();
        walk(&mut g, &["n", "describe blocker"]);
        let Outcome::Ended(scarred) = g.step("look", secs(285)) else {
            panic!("expected ending");
        };
        assert!(said(&scarred, "the part that went wrong"));
        assert!(!said(&scarred, "never found the idea"));
    }

    #[test]
    fn give_talk_at_venue_wins_without_being_called() {
        for input in ["give talk", "give a talk", "do the talk", "start talk"] {
            let mut g = on_square();
            cont(g.step("n", secs(250)));
            let Outcome::Ended(out) = g.step(input, secs(260)) else {
                panic!("expected ending for {input:?}");
            };
            assert!(said(&out, "You don't wait to be called."), "{input:?}");
            assert!(!said(&out, "Someone calls your name."), "{input:?}");
            assert!(
                said(&out, "You had something. You always did."),
                "{input:?}"
            );
        }
    }

    #[test]
    fn give_talk_elsewhere_does_not_end_the_game() {
        let mut g = on_square();
        assert!(matches!(
            g.step("give talk", secs(10)),
            Outcome::Continue(_)
        ));
    }

    // ── Items and inventory ─────────────────────────────────────────

    #[test]
    fn allows_checks_requires_and_forbids() {
        const NEEDS_COFFEE: Action = Action {
            verbs: &["sip"],
            noun: "",
            requires: &["coffee"],
            forbids: &[],
            response: "",
            effects: &[],
        };
        const NO_COFFEE: Action = Action {
            requires: &[],
            forbids: &["coffee"],
            ..NEEDS_COFFEE
        };
        let mut g = Game::new();
        assert!(!g.allows(&NEEDS_COFFEE));
        assert!(g.allows(&NO_COFFEE));
        g.inventory.push("coffee");
        assert!(g.allows(&NEEDS_COFFEE));
        assert!(!g.allows(&NO_COFFEE));
    }

    #[test]
    fn inventory_lists_held_items_then_countdown() {
        let mut g = awake();
        walk(&mut g, &["out", "take coffee"]);
        let out = cont(g.step("i", secs(0)));
        assert_eq!(
            out,
            "You have:\n  a lukewarm coffee\n  and only 13 hours until TechLancaster"
        );
    }

    #[test]
    fn empty_inventory_is_one_sentence() {
        let mut g = awake();
        let out = cont(g.step("i", secs(0)));
        assert_eq!(
            out,
            "You have nothing, and only 13 hours until TechLancaster."
        );
    }

    #[test]
    fn take_and_drop_move_items_between_room_and_inventory() {
        let mut g = awake();
        walk(&mut g, &["out", "get the mug"]);
        assert_eq!(g.inventory, vec!["coffee"]);
        assert!(g.items_here().is_empty());

        walk(&mut g, &["back", "drop coffee"]);
        assert!(g.inventory.is_empty());
        assert_eq!(g.items_here(), &["coffee"]);
    }

    #[test]
    fn five_minutes_cannot_be_dropped() {
        let mut g = awake();
        let out = cont(g.step("drop five minutes", secs(0)));
        assert!(said(&out, "Time is not yours to discard."));
    }

    #[test]
    fn output_is_only_the_response() {
        let mut g = awake();
        let out = cont(g.step("take the moon", secs(0)));
        assert_eq!(out, "You don't see any moon here.");
    }

    #[test]
    fn no_output_line_exceeds_width() {
        let mut g = Game::new();
        let mut outputs = vec![g.start(secs(0))];
        let script = [
            ("n", 5),
            ("n", 10),
            ("look", 25),
            ("take letter", 28),
            ("read letter", 30),
            ("out", 40),
            ("take coffee", 60),
            ("out", 70),
            ("n", 80),
            ("describe blocker", 90),
            ("out", 100),
            ("e", 110),
            ("help farmer", 120),
            ("buy pie", 125),
            ("w", 130),
            ("s", 140),
            ("sit", 150),
            ("i", 200),
            ("n", 245),
            ("look", 260),
        ];
        for (input, t) in script {
            outputs.push(cont(g.step(input, secs(t))));
        }
        if let Outcome::Ended(t) = g.step("give talk", secs(270)) {
            outputs.push(t);
        }
        for out in outputs {
            for line in out.lines() {
                assert!(line.chars().count() <= WIDTH, "too long: {line:?}");
            }
        }
    }
}
