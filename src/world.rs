//! World content: every room and item, as static data.

use crate::clock::Phase;

/// Static room definition. All mutable state lives in `Game`.
pub struct Room {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    /// Appended to the description on the first arrival only.
    pub first_visit: Option<&'static str>,
    /// Appended to the description when the current phase matches.
    pub phase_text: &'static [(Phase, &'static str)],
    /// Direction or nickname -> destination room id.
    pub exits: &'static [(&'static str, &'static str)],
    /// Item ids present when the game starts.
    pub items: &'static [&'static str],
}

/// Something the player can carry.
pub struct Item {
    pub id: &'static str,
    /// Inventory and room listings: "a lukewarm coffee".
    pub name: &'static str,
    /// Status line: "coffee".
    pub short: &'static str,
    /// Nouns the parser accepts for this item.
    pub aliases: &'static [&'static str],
}

/// Room the game starts in.
pub const START: &str = "apartment";

/// Room the funnel delivers the player to.
pub const VENUE: &str = "venue";

/// Printed last, after the ending. The talk doubles as an advertisement.
pub const EVENT_DETAILS: &str = "TECH LANCASTER\n\
    [DATE] at Tellus360, 24 E King St\n\
    Give a talk: [SIGNUP LINK]";

/// Confidence display states as (inventory, status line) text.
///
/// `Game::confidence` indexes this and saturates at the last entry.
pub const CONFIDENCE_STATES: &[(&str, &str)] = &[
    ("confidence (factory sealed)", "confidence"),
    ("confidence (slightly dented)", "confidence (dented)"),
    (
        "confidence (hairline crack, structurally fine)",
        "confidence (cracked)",
    ),
    ("confidence (missing)", "confidence (missing)"),
];

pub static ITEMS: &[Item] = &[Item {
    id: "coffee",
    name: "a lukewarm coffee",
    short: "coffee",
    aliases: &["coffee", "lukewarm coffee", "cup", "mug"],
}];

pub static ROOMS: &[Room] = &[
    Room {
        id: "apartment",
        title: "The Apartment",
        description: "It's dark, and your alarm is going. Fourteen browser tabs glow on \
            the laptop, one of them the Tech Lancaster RSVP with your name on it. \
            The dog has opinions about the door.",
        first_visit: Some("You check your pockets. Five minutes, as promised."),
        phase_text: &[
            (Phase::Morning, "Grey light leaks in around the blinds."),
            (
                Phase::Evening,
                "The apartment has gone quiet. Everyone who matters to tonight is \
                already downtown.",
            ),
        ],
        exits: &[("out", "kitchen")],
        items: &[],
    },
    Room {
        id: "kitchen",
        title: "The Kitchen",
        description: "The coffee maker has done its best. The fridge hums over a \
            whoopie pie of uncertain provenance.",
        first_visit: None,
        phase_text: &[
            (Phase::Morning, "The coffee is still warm, technically."),
            (
                Phase::Afternoon,
                "Lunch was a handful of pretzels eaten over this sink.",
            ),
        ],
        exits: &[("out", "stoop"), ("back", "apartment")],
        items: &["coffee"],
    },
    Room {
        id: "stoop",
        title: "The Stoop",
        description: "The cold gets in immediately. The grid lays itself out ahead: \
            King, Queen, Prince, Duke. Brick sidewalks lose their slow fight with \
            the tree roots.",
        first_visit: None,
        phase_text: &[
            (Phase::Morning, "Delivery trucks idle along Duke."),
            (Phase::Midday, "Office people are out hunting lunch."),
            (
                Phase::Afternoon,
                "The light goes long and gold across the brick.",
            ),
            (
                Phase::Evening,
                "Streetlights come on one block at a time, like they're deciding.",
            ),
        ],
        exits: &[("in", "kitchen")],
        items: &[],
    },
    Room {
        id: "venue",
        title: "Tellus360",
        description: "Warm, loud, a dozen people who are also nervous. There's a \
            projector, a laptop cable, and a spot near the front with your name on it.",
        first_visit: None,
        phase_text: &[(
            Phase::Evening,
            "Someone is testing the mic by saying \"test\" with increasing doubt.",
        )],
        exits: &[],
        items: &[],
    },
];

/// Look up a room by id.
pub fn room(id: &str) -> Option<&'static Room> {
    ROOMS.iter().find(|r| r.id == id)
}

/// Look up an item by id.
pub fn item(id: &str) -> Option<&'static Item> {
    ITEMS.iter().find(|i| i.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn room_ids_are_unique() {
        let mut seen = HashSet::new();
        for r in ROOMS {
            assert!(seen.insert(r.id), "duplicate room id {:?}", r.id);
        }
    }

    #[test]
    fn start_and_venue_exist() {
        assert!(room(START).is_some());
        assert!(room(VENUE).is_some());
    }

    #[test]
    fn every_exit_leads_somewhere() {
        for r in ROOMS {
            for (dir, to) in r.exits {
                assert!(
                    room(to).is_some(),
                    "{}: exit {dir:?} -> missing {to:?}",
                    r.id
                );
            }
        }
    }

    #[test]
    fn every_room_item_exists() {
        for r in ROOMS {
            for id in r.items {
                assert!(item(id).is_some(), "{}: missing item {id:?}", r.id);
            }
        }
    }

    #[test]
    fn phase_text_has_no_duplicate_phases() {
        for r in ROOMS {
            let mut seen = Vec::new();
            for (phase, _) in r.phase_text {
                assert!(!seen.contains(phase), "{}: duplicate {phase:?}", r.id);
                seen.push(*phase);
            }
        }
    }
}
