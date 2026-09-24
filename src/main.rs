use std::collections::HashMap;
use std::io::{self, Write};

/// A location in the World
///
/// `exits` maps a full direction word (e.g. "north") to the index
/// of the destination room in `World::rooms`.
struct Room {
    name: &'static str,
    description: &'static str,
    exits: HashMap<&'static str, usize>,
    items: Vec<&'static str>,
}

/// Build a `Room` from slices so the map definition stays compact.
///
/// Converts the exit pairs into a `HashMap` and copies the items
/// into an owned `Vec` so they can be taken and dropped later.
fn room(
    name: &'static str,
    description: &'static str,
    exits: &[(&'static str, usize)],
    items: &[&'static str],
) -> Room {
    Room {
        name,
        description,
        exits: exits.iter().copied().collect(),
        items: items.to_vec(),
    }
}

/// Game state: the map, player's current location, and their inventory.
struct World {
    rooms: Vec<Room>,
    current: usize,
    inventory: Vec<&'static str>,
}

impl World {
    /// Initialize new world with a given map of rooms.
    ///
    /// Room order matters, exits refer to rooms by their index.
    fn new() -> Self {
        let rooms = vec![
            room(
                "West of House",
                "You are standing in an open field west of a white house, with a boarded front door.",
                &[("north", 1), ("south", 2)],
                &["leaflet"],
            ),
            room(
                "North of House",
                "You are facing the north side of a white house. A narrow path winds north through the trees.",
                &[("south", 0), ("north", 3)],
                &[],
            ),
            room(
                "South of House",
                "You are facing the south side of a white house. All the windows are boarded.",
                &[("north", 0)],
                &["lamp"],
            ),
            room(
                "Forest Path",
                "A path winds through a dimly lit forest. A large tree with low branches stands here.",
                &[("south", 1)],
                &["egg"],
            ),
        ];
        World {
            rooms,
            current: 0,
            inventory: Vec::new(),
        }
    }

    /// Print current room's description, items, and exits
    fn describe(&self) {
        let room = &self.rooms[self.current];
        println!("\n{}\n\n{}", room.name, room.description);
        for item in &room.items {
            println!("There is a {item} here.");
        }
        let exits: Vec<&str> = room.exits.keys().copied().collect();
        println!("Exits: {}", exits.join(", "));
    }

    /// Move the player through `dir` if the current room has that exit available
    fn go(&mut self, dir: &str) {
        match self.rooms[self.current].exits.get(dir) {
            Some(&next) => {
                self.current = next;
                self.describe();
            }
            None => println!("You can't go that way."),
        }
    }

    /// Move `item` from the current room into the inventory.
    fn take(&mut self, item: &str) {
        let room = &mut self.rooms[self.current];
        match room.items.iter().position(|i| *i == item) {
            Some(idx) => {
                self.inventory.push(room.items.remove(idx));
                println!("Taken.");
            }
            None => println!("You see no {item} here."),
        }
    }

    /// Move `item` from inventory into the current room.
    fn drop(&mut self, item: &str) {
        let room = &mut self.rooms[self.current];
        match self.inventory.iter().position(|i| *i == item) {
            Some(idx) => room.items.push(self.inventory.remove(idx)),
            None => println!("You don't have a {item}."),
        }
    }

    /// List everything the player is carrying.
    fn show_inventory(&self) {
        if self.inventory.is_empty() {
            println!("You are empty handed.");
        } else {
            println!("You are carrying: {}", self.inventory.join(", "));
        }
    }
}

/// A player action produced by `parse`.
enum Command {
    Go(String),
    Look,
    Take(String),
    Drop(String),
    Inventory,
    Quit,
    Unknown,
}

/// Exit keys use full words, so expand abbreviations
fn expand(dir: &str) -> String {
    match dir {
        "n" => "north",
        "s" => "south",
        "e" => "east",
        "w" => "west",
        other => other,
    }
    .to_string()
}

/// Turn a raw input line into a `Command`.
///
/// Input is lowercased and split on whitespace, then matched as a
/// slice pattern. Bare directions ("n", "north") count as movement.
fn parse(input: &str) -> Command {
    let lower = input.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    match words.as_slice() {
        ["look" | "l"] => Command::Look,
        ["inventory" | "i"] => Command::Inventory,
        ["quit" | "q"] => Command::Quit,
        ["take" | "get", item] => Command::Take(item.to_string()),
        ["drop" | "leave", item] => Command::Drop(item.to_string()),
        ["go", dir] | [dir @ ("n" | "s" | "e" | "w" | "north" | "south" | "east" | "west")] => {
            Command::Go(expand(dir))
        }
        // [""] => Command::,
        _ => Command::Unknown,
    }
}

fn main() {
    println!("Welcome to TechLancaster.            This version created Sep 24, 2026");

    let mut world = World::new();
    world.describe();

    loop {
        print!("\n> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        // 0 bytes read = EOF (Crtl-D)
        if io::stdin().read_line(&mut line).unwrap() == 0 {
            break;
        }

        match parse(&line) {
            Command::Go(dir) => world.go(&dir),
            Command::Look => world.describe(),
            Command::Take(item) => world.take(&item),
            Command::Drop(item) => world.drop(&item),
            Command::Inventory => world.show_inventory(),
            Command::Unknown => println!("I don't understand that."),
            Command::Quit => {
                println!("Goodbye!");
                break;
            }
        }

        if world.current == 0 && world.inventory.contains(&"egg") {
            println!("\nYou return with the egg. You win!");
            break;
        }
    }
}
