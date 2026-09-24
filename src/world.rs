//! World content: every room and item, as static data.
//!
//! Act I is a short dream that opens like Zork; Act II is the city.

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
    /// Room-specific commands, tried in order.
    pub actions: &'static [Action],
}

/// A room-specific command and what it does.
pub struct Action {
    /// Any of these verbs triggers the action: ["drink", "sip", "chug"].
    pub verbs: &'static [&'static str],
    /// Must follow the verb; "" means the verb alone.
    pub noun: &'static str,
    /// Item ids that must all be held.
    pub requires: &'static [&'static str],
    /// Item ids none of which may be held.
    pub forbids: &'static [&'static str],
    pub response: &'static str,
    /// Applied in order; may be empty.
    pub effects: &'static [Effect],
}

/// A change an action makes to the game.
pub enum Effect {
    /// Add an item to the inventory unless already held.
    Gain(&'static str),
    /// Gain the idea; only the first route counts.
    GainIdea(Route),
    /// Move to a room and describe it.
    Teleport(&'static str),
    /// Leave the dream and start the day.
    Wake,
    /// Run the ending.
    Win,
}

/// How the player got the idea. Keys the ending without naming the idea.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Overheard,
    WarStory,
    ExplainedIt,
    StoppedLooking,
    SlowedDown,
}

impl Action {
    /// True if `words` is one of the verbs, followed by the noun if there is one.
    ///
    /// `words` is already normalized, so filler like "the" is gone.
    pub fn matches(&self, words: &str) -> bool {
        self.verbs.iter().any(|verb| {
            if self.noun.is_empty() {
                words == *verb
            } else {
                words
                    .strip_prefix(verb)
                    .and_then(|rest| rest.strip_prefix(' '))
                    == Some(self.noun)
            }
        })
    }
}

/// Something the player can carry.
pub struct Item {
    pub id: &'static str,
    /// Inventory and room listings: "a lukewarm coffee".
    pub name: &'static str,
    /// Nouns the parser accepts for this item.
    pub aliases: &'static [&'static str],
}

/// Room the game starts in: the dream.
pub const START: &str = "dream_field";

/// Room the player wakes up in.
pub const WAKE: &str = "bedroom";

/// Room the funnel delivers the player to.
pub const VENUE: &str = "venue";

/// Item id of the dream's letter; reading it wakes the player.
pub const LETTER: &str = "letter";

/// Item id of the idea, gained through `Effect::GainIdea`.
pub const IDEA: &str = "idea";

/// Appended to an action's response when it actually grants the idea.
pub const IDEA_FOUND: &str = "There it is: the idea for your talk.";

/// Appended to dream descriptions once the alarm starts bleeding through.
pub const DREAM_BEEP: &str = "You hear a faint beeping sound.";

/// Shown when the dream runs out of time and wakes the player itself.
pub const FORCED_WAKE: &str = "The field tilts. A letter you never opened has somehow \
    already been read.\n\n    Beep.\n    Beep.\n    Beep.";

/// Rotating lines for walking away from the house in the dream.
pub const DREAM_LOOPS: &[&str] = &[
    "You walk away from the house for a long time. But now the house is somehow back in front of you.",
    "The field stretches on into infinity, until it wraps back around on itself.",
    "You set off with real purpose, but then it fell out of your inventory. You're back in the field.",
    "The trees part politely and deliver you back to the field.",
    "You're fairly sure you walked in a straight line.",
    "A path appears, then thinks better of it. You're lost.",
];

/// The ending's last line before the event details, on every path.
///
/// The joke isn't that time ran out; it's that there's always more.
pub const PUNCHLINE: &str = "Afterward, the host checks the schedule. \"We've got five \
    more minutes,\" they say, looking out at the room. \"Who's next?\"";

/// Printed last, after the ending. The talk doubles as an advertisement.
pub const EVENT_DETAILS: &str = "TechLancaster\n\
    Thursday, October 22, 2026, at West Art\n\
    816 Buchanan Ave, Lancaster, PA 17603\n\
    Give a talk: https://bit.ly/tl-lightning-signup-2026";

pub static ITEMS: &[Item] = &[
    Item {
        id: LETTER,
        name: "a letter",
        aliases: &["letter"],
    },
    Item {
        id: "coffee",
        name: "a lukewarm coffee",
        aliases: &["coffee", "lukewarm coffee", "cup", "mug"],
    },
    Item {
        id: "whoopie_pie",
        name: "a whoopie pie",
        aliases: &["whoopie pie", "whoopie", "pie"],
    },
    Item {
        id: "opinion",
        name: "a strong opinion about semicolons",
        aliases: &["opinion", "strong opinion"],
    },
    Item {
        id: "war_story",
        name: "a war story",
        aliases: &["war story", "story"],
    },
    Item {
        id: IDEA,
        name: "the idea [REDACTED]",
        aliases: &["idea"],
    },
];

/// Blank room for struct-update syntax, so each room lists only what it uses.
const ROOM: Room = Room {
    id: "",
    title: "",
    description: "",
    first_visit: None,
    phase_text: &[],
    exits: &[],
    items: &[],
    actions: &[],
};

/// Blank action for struct-update syntax.
const ACTION: Action = Action {
    verbs: &[],
    noun: "",
    requires: &[],
    forbids: &[],
    response: "",
    effects: &[],
};

/// Reading the letter ends the dream, in any dream room, once it's held.
const READ_LETTER: Action = Action {
    verbs: &["read", "open"],
    noun: "letter",
    requires: &[LETTER],
    response: "The letter is short. It says:\n\n    Beep!\n    Beep!\n    Beep!",
    effects: &[Effect::Wake],
    ..ACTION
};

/// Listening in at Mean Cup, with or without naming who.
const EAVESDROP: Action = Action {
    verbs: &["eavesdrop", "listen"],
    response: "You don't mean to listen. You listen. You leave with a strong opinion \
        about semicolons, and it isn't even yours.",
    effects: &[Effect::Gain("opinion"), Effect::GainIdea(Route::Overheard)],
    ..ACTION
};

/// Sitting down in County Park.
const SIT: Action = Action {
    verbs: &["sit", "rest", "relax"],
    response: "You sit by the water. You stop looking for it. The creek keeps going.",
    effects: &[Effect::GainIdea(Route::StoppedLooking)],
    ..ACTION
};

/// Walking the County Park trail.
const WALK: Action = Action {
    verbs: &["walk", "hike", "follow"],
    response: "You walk until you stop thinking about it. The trail loops back to \
        where it started, and so do you, sort of.",
    effects: &[Effect::GainIdea(Route::StoppedLooking)],
    ..ACTION
};

/// Helping the farmer in Amish Country.
const HELP_FARMER: Action = Action {
    verbs: &["help"],
    noun: "farmer",
    response: "You hold the fence post while the farmer works. He talks about doing \
        one thing at a time, all the way through, and doesn't check his phone once. \
        He doesn't have one.",
    effects: &[Effect::GainIdea(Route::SlowedDown)],
    ..ACTION
};

/// The roadside stand's honor-system pie.
const BUY_PIE: Action = Action {
    verbs: &["buy", "take", "get"],
    noun: "pie",
    response: "You leave cash in the tin and take a whoopie pie. Nobody checks.",
    effects: &[Effect::Gain("whoopie_pie")],
    ..ACTION
};

pub static ROOMS: &[Room] = &[
    // ── Act I: the dream ─────────────────────────────────────────────
    Room {
        id: "dream_field",
        title: "A Field",
        description: "You are in an open field west of a big white house with a \
            boarded front door.\nThere is a small mailbox here.",
        exits: &[("north", "dream_north"), ("south", "dream_south")],
        actions: &[
            Action {
                verbs: &["open", "check"],
                noun: "mailbox",
                forbids: &[LETTER],
                response: "Opening the small mailbox reveals a letter. You take it.",
                effects: &[Effect::Gain(LETTER)],
                ..ACTION
            },
            Action {
                verbs: &["take", "get"],
                noun: "letter",
                forbids: &[LETTER],
                response: "You take the letter out of the mailbox.",
                effects: &[Effect::Gain(LETTER)],
                ..ACTION
            },
            Action {
                verbs: &["read", "open"],
                noun: "letter",
                forbids: &[LETTER],
                response: "The letter is still in the mailbox.",
                ..ACTION
            },
            READ_LETTER,
        ],
        ..ROOM
    },
    Room {
        id: "dream_north",
        title: "The North Side of the House",
        description: "A narrow path runs along the north side of the house, toward \
            a wall of trees. You have the feeling you've been here before.",
        exits: &[("west", "dream_field"), ("east", "dream_east")],
        actions: &[READ_LETTER],
        ..ROOM
    },
    Room {
        id: "dream_east",
        title: "The Back of the House",
        description: "Behind the house, a window stands slightly ajar. The house \
            looks smaller from back here, or you are larger.",
        exits: &[("north", "dream_north"), ("south", "dream_south")],
        actions: &[
            Action {
                verbs: &["open", "climb", "enter"],
                noun: "window",
                response: "You ease the window open and climb through, landing solidly in grass.",
                effects: &[Effect::Teleport("dream_field")],
                ..ACTION
            },
            READ_LETTER,
        ],
        ..ROOM
    },
    Room {
        id: "dream_south",
        title: "A Porch",
        description: "A porch that was not there a moment ago. The swing is still \
            moving. Nobody is on it.",
        exits: &[("west", "dream_field"), ("east", "dream_east")],
        actions: &[READ_LETTER],
        ..ROOM
    },
    // ── Act II: the city ─────────────────────────────────────────────
    Room {
        id: "bedroom",
        title: "Your Bedroom",
        description: "It's dark, and your alarm is going off. Twenty browser tabs glare \
            at you on the glowing screen of your laptop. One of them is a signup sheet \
            for the TechLancaster Lightning Talk Event.\n\nAnd it has your name on it!",
        first_visit: Some(
            "You had all the time in the world to come up with an idea. Now you only have a day.",
        ),
        phase_text: &[],
        exits: &[("out", "kitchen"), ("kitchen", "kitchen")],
        ..ROOM
    },
    Room {
        id: "kitchen",
        title: "The Kitchen",
        description: "The coffee maker has done its best. The fridge hums over a \
            whoopie pie of uncertain provenance.",
        phase_text: &[
            (Phase::Morning, "The coffee is still warm, technically."),
            (
                Phase::Afternoon,
                "Lunch was a handful of pretzels eaten over this sink.",
            ),
        ],
        exits: &[
            ("out", "square"),
            ("back", "bedroom"),
            ("bedroom", "bedroom"),
            ("square", "square"),
            ("outside", "square"),
        ],
        items: &["coffee"],
        ..ROOM
    },
    Room {
        id: "square",
        title: "The Square",
        description: "Center city Lancaster. Red brick buildings stand shoulder to \
            shoulder, and the air smells like fall. An old Amish man hums past on an \
            electric scooter. Coffee is west at Mean Cup, work is north, County Park \
            is south, and Amish Country is out east.",
        phase_text: &[
            (
                Phase::Morning,
                "Shop owners are sweeping their front steps.",
            ),
            (Phase::Midday, "The lunch crowd moves through in waves."),
            (
                Phase::Afternoon,
                "The light goes long and gold across the brick.",
            ),
            (
                Phase::Evening,
                "Streetlights come on one block at a time, like they're deciding.",
            ),
        ],
        exits: &[
            ("west", "mean_cup"),
            ("north", "office"),
            ("south", "county_park"),
            ("east", "amish_country"),
            ("in", "kitchen"),
            ("mean cup", "mean_cup"),
            ("coffee", "mean_cup"),
            ("cafe", "mean_cup"),
            ("office", "office"),
            ("work", "office"),
            ("park", "county_park"),
            ("county park", "county_park"),
            ("amish country", "amish_country"),
            ("country", "amish_country"),
            ("farm", "amish_country"),
            ("kitchen", "kitchen"),
            ("home", "kitchen"),
        ],
        actions: &[Action {
            verbs: &["take", "get", "buy"],
            noun: "coffee",
            response: "You head west for coffee.",
            effects: &[Effect::Teleport("mean_cup")],
            ..ACTION
        }],
        ..ROOM
    },
    Room {
        id: "mean_cup",
        title: "Mean Cup",
        description: "Warm and loud. At the next table, two strangers are arguing \
            about something small with enormous care, and it's hard not to \
            eavesdrop. The barista will happily refill your coffee.",
        phase_text: &[
            (Phase::Morning, "The line is out to the door."),
            (
                Phase::Afternoon,
                "Laptop campers have claimed every outlet.",
            ),
        ],
        exits: &[("east", "square"), ("out", "square"), ("square", "square")],
        actions: &[
            EAVESDROP,
            Action {
                noun: "strangers",
                ..EAVESDROP
            },
            Action {
                verbs: &["refill", "buy", "order", "get"],
                noun: "coffee",
                response: "The barista tops you off without asking. Nothing changes. \
                    You feel better anyway.",
                ..ACTION
            },
        ],
        ..ROOM
    },
    Room {
        id: "office",
        title: "The Office",
        description: "Standup is about to start. You could say \"no blockers\" like \
            always, or finally describe the blocker that's been eating your week. \
            A rubber duck on your desk looks ready to talk it through.",
        phase_text: &[
            (
                Phase::Midday,
                "Half the floor is at lunch. The other half is in a meeting about it.",
            ),
            (
                Phase::Afternoon,
                "The afternoon slump has set in. Only the duck is alert.",
            ),
        ],
        exits: &[("south", "square"), ("out", "square"), ("square", "square")],
        actions: &[
            Action {
                verbs: &["say", "standup"],
                noun: "no blockers",
                response: "\"No blockers.\" Everyone nods. You survive. Nothing is gained.",
                ..ACTION
            },
            Action {
                verbs: &["describe", "admit", "mention", "explain"],
                noun: "blocker",
                response: "You tell them what actually went wrong. It stings a little, \
                    saying it out loud. Someone says \"oh no, we had that too.\" Now \
                    you have a war story.",
                effects: &[Effect::Gain("war_story"), Effect::GainIdea(Route::WarStory)],
                ..ACTION
            },
            Action {
                verbs: &["talk", "speak", "explain"],
                noun: "duck",
                response: "You explain the whole problem to the duck. The duck says \
                    nothing, supportively. Halfway through, you hear yourself.",
                effects: &[Effect::GainIdea(Route::ExplainedIt)],
                ..ACTION
            },
        ],
        ..ROOM
    },
    Room {
        id: "county_park",
        title: "County Park",
        description: "Leaves are turning along the Conestoga, and a bench by the water \
            looks like it has been waiting for you. A trail wanders off into the \
            trees, going nowhere in particular.",
        phase_text: &[
            (Phase::Morning, "Mist is still lifting off the creek."),
            (
                Phase::Evening,
                "The light is going, and so are the dog walkers.",
            ),
        ],
        exits: &[("north", "square"), ("out", "square"), ("square", "square")],
        actions: &[
            SIT,
            Action {
                noun: "bench",
                ..SIT
            },
            Action {
                noun: "down",
                ..SIT
            },
            WALK,
            Action {
                noun: "trail",
                ..WALK
            },
        ],
        ..ROOM
    },
    Room {
        id: "amish_country",
        title: "Amish Country",
        description: "The city gives way to farmland. A buggy clops past. A roadside \
            stand sells pies on the honor system, and a farmer mending a fence nods \
            like you could stop and help.",
        phase_text: &[
            (Phase::Midday, "Somewhere, a dinner bell rings."),
            (Phase::Evening, "The fields go gold, then grey."),
        ],
        exits: &[("west", "square"), ("out", "square"), ("square", "square")],
        actions: &[
            HELP_FARMER,
            Action {
                noun: "",
                ..HELP_FARMER
            },
            Action {
                verbs: &["talk"],
                ..HELP_FARMER
            },
            BUY_PIE,
            Action {
                noun: "whoopie pie",
                ..BUY_PIE
            },
        ],
        ..ROOM
    },
    Room {
        id: "venue",
        title: "West Art",
        description: "Warm, loud, a dozen people who are also nervous. There's a \
            projector, a laptop cable, and a spot near the front with your name on it.",
        phase_text: &[(
            Phase::Evening,
            "Someone is testing the mic by saying \"test\" with increasing doubt.",
        )],
        actions: &[Action {
            verbs: &["give", "do", "start"],
            noun: "talk",
            response: "You don't wait to be called. You walk to the front of the room.",
            effects: &[Effect::Win],
            ..ACTION
        }],
        ..ROOM
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
    use crate::clock::Phase;
    use crate::text::{wrap, WIDTH};
    use std::collections::HashSet;

    #[test]
    fn room_and_item_ids_are_unique() {
        let mut seen = HashSet::new();
        for r in ROOMS {
            assert!(seen.insert(r.id), "duplicate room id {:?}", r.id);
        }
        let mut seen = HashSet::new();
        for i in ITEMS {
            assert!(seen.insert(i.id), "duplicate item id {:?}", i.id);
        }
    }

    #[test]
    fn item_aliases_are_unambiguous() {
        let mut seen = HashSet::new();
        for i in ITEMS {
            for alias in i.aliases {
                assert!(seen.insert(alias), "alias {alias:?} used twice");
            }
        }
    }

    #[test]
    fn special_rooms_and_items_exist() {
        assert!(room(START).is_some());
        assert!(room(WAKE).is_some());
        assert!(room(VENUE).is_some());
        assert!(item(IDEA).is_some());
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
    fn every_referenced_item_and_room_exists() {
        for r in ROOMS {
            for id in r.items {
                assert!(item(id).is_some(), "{}: missing item {id:?}", r.id);
            }
            for a in r.actions {
                for id in a.requires.iter().chain(a.forbids) {
                    assert!(
                        item(id).is_some(),
                        "{}: action needs missing item {id:?}",
                        r.id
                    );
                }
                for effect in a.effects {
                    match effect {
                        Effect::Gain(id) => {
                            assert!(item(id).is_some(), "{}: gains missing {id:?}", r.id)
                        }
                        Effect::Teleport(to) => {
                            assert!(room(to).is_some(), "{}: teleports to missing {to:?}", r.id)
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    #[test]
    fn every_action_has_verbs() {
        for r in ROOMS {
            for a in r.actions {
                assert!(!a.verbs.is_empty(), "{}: action with no verbs", r.id);
            }
        }
    }

    #[test]
    fn every_room_is_reachable() {
        // The wake and the funnel reach their rooms without an exit
        let mut seen: HashSet<&str> = [START, WAKE, VENUE].into();
        let mut queue: Vec<&str> = seen.iter().copied().collect();
        while let Some(id) = queue.pop() {
            let Some(r) = room(id) else { continue };
            let teleports = r
                .actions
                .iter()
                .flat_map(|a| a.effects)
                .filter_map(|e| match e {
                    Effect::Teleport(to) => Some(*to),
                    _ => None,
                });
            for to in r.exits.iter().map(|(_, to)| *to).chain(teleports) {
                if seen.insert(to) {
                    queue.push(to);
                }
            }
        }
        for r in ROOMS {
            assert!(seen.contains(r.id), "{} is unreachable", r.id);
        }
    }

    #[test]
    fn spokes_return_to_the_square_and_offer_the_idea() {
        for id in [
            "kitchen",
            "mean_cup",
            "office",
            "county_park",
            "amish_country",
        ] {
            let r = room(id).expect("room exists");
            assert!(
                r.exits.iter().any(|(_, to)| *to == "square"),
                "{id} has no way back to the square"
            );
        }
        for id in ["mean_cup", "office", "county_park", "amish_country"] {
            let r = room(id).expect("room exists");
            let offers_idea = r
                .actions
                .iter()
                .flat_map(|a| a.effects)
                .any(|e| matches!(e, Effect::GainIdea(_)));
            assert!(offers_idea, "{id} has no way to get the idea");
        }
    }

    #[test]
    fn descriptions_fit_on_a_projector() {
        // Base text, the longest phase line, and first-visit text together
        for r in ROOMS {
            let longest_phase = r
                .phase_text
                .iter()
                .map(|(_, t)| *t)
                .max_by_key(|t| t.len())
                .unwrap_or("");
            let full = [r.description, longest_phase, r.first_visit.unwrap_or("")].join(" ");
            let lines = wrap(&full, WIDTH).lines().count();
            assert!(lines <= 6, "{}: {lines} lines", r.id);
        }
    }

    #[test]
    fn phase_text_has_no_duplicate_phases() {
        for r in ROOMS {
            let mut seen: Vec<Phase> = Vec::new();
            for (phase, _) in r.phase_text {
                assert!(!seen.contains(phase), "{}: duplicate {phase:?}", r.id);
                seen.push(*phase);
            }
        }
    }

    #[test]
    fn action_matches_verb_and_noun() {
        let a = Action {
            verbs: &["give", "do"],
            noun: "talk",
            ..ACTION
        };
        assert!(a.matches("give talk"));
        assert!(a.matches("do talk"));
        assert!(!a.matches("give"));
        assert!(!a.matches("givetalk"));
        assert!(!a.matches("give talk now"));
    }
}
