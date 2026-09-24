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
    /// Tried on every arrival, after the description; `verbs` is ignored.
    pub on_enter: Option<Action>,
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
    Perspective,
    StoppedLooking,
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

/// Item id of the idea, gained through `Effect::GainIdea`.
pub const IDEA: &str = "idea";

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
        id: "indignation",
        name: "righteous indignation",
        aliases: &["indignation", "righteous indignation"],
    },
    Item {
        id: "perspective",
        name: "a moment of perspective",
        aliases: &["perspective", "moment", "moment of perspective"],
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
    on_enter: None,
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

/// Reading the letter, however it's phrased, ends the dream.
const READ_LETTER: Action = Action {
    verbs: &["read", "open", "take", "get"],
    noun: "letter",
    response: "The letter is short. It says:\n\n    Beep!\n    Beep!\n    Beep!",
    effects: &[Effect::Wake],
    ..ACTION
};

/// The market sells you a whoopie pie, whatever you came for.
const MARKET_BUY: Action = Action {
    verbs: &["buy"],
    response: "You go in for cider donuts and come out with a whoopie pie. That's \
        all the market is selling you today.",
    effects: &[Effect::Gain("whoopie_pie")],
    ..ACTION
};

/// Listening to the busker, with or without naming them.
const BUSKER: Action = Action {
    verbs: &["listen"],
    response: "You stand through a whole verse and put a dollar in the case. It's a \
        good song. It doesn't need to be anything else.",
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
                response: "Opening the small mailbox reveals a letter.",
                ..ACTION
            },
            Action {
                verbs: &["read"],
                noun: "mailbox",
                ..READ_LETTER
            },
            READ_LETTER,
        ],
        ..ROOM
    },
    Room {
        id: "dream_north",
        title: "North of the House",
        description: "A narrow path runs along the north side of the house, toward \
            a wall of trees. You have the feeling you've been here before.",
        exits: &[("west", "dream_field"), ("east", "dream_east")],
        ..ROOM
    },
    Room {
        id: "dream_east",
        title: "Behind the House",
        description: "Behind the house, a window stands slightly ajar. The house \
            looks smaller from back here, or you are larger.",
        exits: &[("north", "dream_north"), ("south", "dream_south")],
        actions: &[Action {
            verbs: &["open", "climb", "enter"],
            noun: "window",
            response: "You ease the window open and climb through, landing solidly in grass.",
            effects: &[Effect::Teleport("dream_field")],
            ..ACTION
        }],
        ..ROOM
    },
    Room {
        id: "dream_south",
        title: "A Porch",
        description: "A porch that was not there a moment ago. The swing is still \
            moving. Nobody is on it.",
        exits: &[("west", "dream_field"), ("east", "dream_east")],
        ..ROOM
    },
    // ── Act II: the city ─────────────────────────────────────────────
    Room {
        id: "bedroom",
        title: "Your Bedroom",
        description: "It's dark, and your alarm is going off. Twenty browser tabs glare \
            at you on the glowing screen of your laptop. One of them, is a signup sheet \
            for the TechLancaster Lightning Talk Event.\n\n And it has your name on it!",
        first_visit: Some(
            "You had all the time in the world to come up with an idea.\n\nNow you have a day.",
        ),
        phase_text: &[
            (Phase::Morning, "Grey light leaks in around the blinds."),
            (
                Phase::Evening,
                "The room has gone quiet. Everyone who matters to tonight is \
                already out there.",
            ),
        ],
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
            ("out", "stoop"),
            ("back", "bedroom"),
            ("bedroom", "bedroom"),
            ("stoop", "stoop"),
            ("outside", "stoop"),
        ],
        items: &["coffee"],
        ..ROOM
    },
    Room {
        id: "stoop",
        title: "The Stoop",
        description: "The cold gets in immediately. The grid lays itself out ahead: \
            King, Queen, Prince, Duke. Brick sidewalks lose their slow fight with \
            the tree roots.",
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
        exits: &[
            ("north", "market"),
            ("east", "mean_cup"),
            ("south", "office"),
            ("west", "square"),
            ("in", "kitchen"),
            ("market", "market"),
            ("central market", "market"),
            ("mean cup", "mean_cup"),
            ("cafe", "mean_cup"),
            ("office", "office"),
            ("work", "office"),
            ("square", "square"),
            ("penn square", "square"),
            ("kitchen", "kitchen"),
            ("home", "kitchen"),
        ],
        ..ROOM
    },
    Room {
        id: "market",
        title: "Central Market",
        description: "Central Market is loud and crowded, every aisle its own \
            conversation. Somewhere close, cider donuts.",
        phase_text: &[
            (
                Phase::Morning,
                "The stands are still setting up and the good stuff is going fast.",
            ),
            (Phase::Midday, "Lunch lines braid through the aisles."),
            (
                Phase::Afternoon,
                "Vendors are packing up. The donuts are gone.",
            ),
            (
                Phase::Evening,
                "The market is closed. You're here anyway, looking through the glass.",
            ),
        ],
        exits: &[("out", "stoop"), ("south", "stoop"), ("stoop", "stoop")],
        actions: &[
            Action {
                verbs: &["listen", "eavesdrop"],
                response: "You stop trying to think and just listen. Two strangers \
                    argue about something small with enormous care. Something in it \
                    catches. You have the idea now.",
                effects: &[Effect::GainIdea(Route::Overheard)],
                ..ACTION
            },
            MARKET_BUY,
            Action {
                noun: "donut",
                ..MARKET_BUY
            },
            Action {
                noun: "donuts",
                ..MARKET_BUY
            },
            Action {
                noun: "something",
                ..MARKET_BUY
            },
        ],
        ..ROOM
    },
    Room {
        id: "mean_cup",
        title: "Mean Cup",
        description: "Someone at the next table is pitching something with a lot of \
            hand gestures. Someone else has a keyboard you could hear from the door.",
        phase_text: &[
            (Phase::Morning, "The line is out to the door."),
            (
                Phase::Afternoon,
                "Laptop campers have claimed every outlet.",
            ),
        ],
        exits: &[("out", "stoop"), ("west", "stoop"), ("stoop", "stoop")],
        actions: &[
            Action {
                verbs: &["eavesdrop", "listen"],
                response: "You don't mean to listen. You listen. You leave with a \
                    strong opinion about semicolons, and it isn't even yours.",
                effects: &[Effect::Gain("opinion")],
                ..ACTION
            },
            Action {
                verbs: &["refill", "buy", "order"],
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
        description: "Standup is in five minutes; it's always in five minutes. Jira \
            is open on someone's monitor. A rubber duck on your desk waits patiently.",
        phase_text: &[
            (
                Phase::Morning,
                "Everyone is pretending to have read the ticket.",
            ),
            (
                Phase::Midday,
                "Half the floor is at lunch. The other half is in a meeting about it.",
            ),
            (
                Phase::Afternoon,
                "The afternoon slump has set in. Only the duck is alert.",
            ),
        ],
        exits: &[
            ("out", "stoop"),
            ("north", "stoop"),
            ("down", "garage"),
            ("stoop", "stoop"),
            ("garage", "garage"),
        ],
        actions: &[
            Action {
                verbs: &["say"],
                noun: "no blockers",
                response: "\"No blockers.\" Everyone nods. You survive. Nothing is gained.",
                ..ACTION
            },
            Action {
                verbs: &["describe", "admit", "mention"],
                noun: "blocker",
                response: "You tell them what actually went wrong. It stings a little, \
                    saying it out loud. Someone says \"oh no, we had that too.\" You now \
                    have a war story, and the idea came with it.",
                effects: &[Effect::Gain("war_story"), Effect::GainIdea(Route::WarStory)],
                ..ACTION
            },
            Action {
                verbs: &["talk", "speak", "explain"],
                noun: "duck",
                response: "You explain everything to the duck. The duck says nothing, \
                    supportively. You feel a little more put together.",
                ..ACTION
            },
        ],
        ..ROOM
    },
    Room {
        id: "garage",
        title: "Parking Garage, Level 4",
        description: "Concrete, echo, one flickering light. Your car has a ticket on it.",
        exits: &[("up", "office"), ("out", "stoop"), ("office", "office")],
        actions: &[Action {
            verbs: &["read", "take", "get"],
            noun: "ticket",
            response: "You read the ticket. You read it again. You are filled with \
                righteous indignation, which, to be clear, is a completely valid \
                talk topic.",
            effects: &[Effect::Gain("indignation")],
            ..ACTION
        }],
        ..ROOM
    },
    Room {
        id: "square",
        title: "Penn Square",
        description: "The Soldiers and Sailors Monument, a busker working through a \
            song everyone knows the chorus to, and pigeons who clearly have a plan.",
        phase_text: &[
            (
                Phase::Midday,
                "The lunch crowd is eating on the monument steps.",
            ),
            (Phase::Evening, "The square is emptying toward dinner."),
        ],
        exits: &[
            ("east", "stoop"),
            ("north", "park"),
            ("up", "monument"),
            ("stoop", "stoop"),
            ("park", "park"),
            ("musser park", "park"),
            ("monument", "monument"),
            ("steps", "monument"),
        ],
        actions: &[
            BUSKER,
            Action {
                noun: "busker",
                ..BUSKER
            },
        ],
        ..ROOM
    },
    Room {
        id: "monument",
        title: "Top of the Monument Steps",
        description: "Not high, exactly, but high enough. The whole grid runs out \
            from here in four directions.",
        exits: &[("down", "square"), ("square", "square")],
        actions: &[Action {
            verbs: &["look"],
            response: "From up here, everything you were worried about is roughly the \
                size of a pigeon. You have a moment of perspective. The idea was in \
                there too.",
            effects: &[
                Effect::Gain("perspective"),
                Effect::GainIdea(Route::Perspective),
            ],
            ..ACTION
        }],
        ..ROOM
    },
    Room {
        id: "park",
        title: "Musser Park",
        description: "Quiet, a little overgrown, with a bench whose plaque remembers \
            someone who liked it here.",
        exits: &[
            ("south", "square"),
            ("out", "stoop"),
            ("square", "square"),
            ("stoop", "stoop"),
        ],
        on_enter: Some(Action {
            forbids: &[IDEA],
            response: "You sit on the bench. You stop looking for it. That's when the \
                idea shows up, the way a cat does.",
            effects: &[Effect::GainIdea(Route::StoppedLooking)],
            ..ACTION
        }),
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

    /// Every action in a room, including its arrival action.
    fn actions(r: &Room) -> impl Iterator<Item = &Action> {
        r.actions.iter().chain(r.on_enter.as_ref())
    }

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
            for a in actions(r) {
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
            let teleports = actions(r).flat_map(|a| a.effects).filter_map(|e| match e {
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
    fn city_rooms_return_to_the_stoop() {
        for id in ["market", "mean_cup", "office", "garage", "park"] {
            let r = room(id).expect("room exists");
            assert!(
                r.exits.iter().any(|(_, to)| *to == "stoop"),
                "{id} has no way back to the stoop"
            );
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
