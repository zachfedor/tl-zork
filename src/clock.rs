//! Wall-clock driven game time.
//!
//! Every function takes real `elapsed` time since the presenter pressed Enter,
//! so tests can simulate any moment without waiting.

use std::time::Duration;

/// Real seconds until the ending fires. This is 7:00pm game time.
pub const ENDING_SECS: u64 = 285;

/// Real seconds after which every movement command leads to the venue.
pub const FUNNEL_SECS: u64 = 240;

/// Game minutes from 6:00am to 7:00pm.
const DAY_MINUTES: u64 = 780;

/// Game minutes past midnight at which the day starts (6:00am).
const DAY_START: u64 = 6 * 60;

/// Time-of-day flavor. Phases only change description text, never gate anything.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Morning,
    Midday,
    Afternoon,
    Evening,
}

/// Game minutes since 6:00am, clamped at 7:00pm.
///
/// Integer math on milliseconds keeps this exact and free of float rounding.
pub fn game_minutes(elapsed: Duration) -> u64 {
    let end_ms = ENDING_SECS * 1000;
    let ms = elapsed.as_millis().min(u128::from(end_ms)) as u64;
    ms * DAY_MINUTES / end_ms
}

/// The phase of the day for a given real elapsed time.
///
/// Boundaries are in real seconds (see SPEC §2) so Evening begins
/// exactly when the funnel does.
pub fn phase(elapsed: Duration) -> Phase {
    match elapsed.as_secs() {
        0..110 => Phase::Morning,
        110..175 => Phase::Midday,
        175..FUNNEL_SECS => Phase::Afternoon,
        _ => Phase::Evening,
    }
}

/// True once movement should deliver the player to the venue.
pub fn is_funnel(elapsed: Duration) -> bool {
    elapsed >= Duration::from_secs(FUNNEL_SECS)
}

/// True once the next command should be replaced by the ending.
pub fn is_over(elapsed: Duration) -> bool {
    elapsed >= Duration::from_secs(ENDING_SECS)
}

/// Game time as a 12-hour clock string, e.g. "2:41pm".
pub fn time_string(elapsed: Duration) -> String {
    let total = DAY_START + game_minutes(elapsed);
    let (hour24, minute) = (total / 60, total % 60);
    let suffix = if hour24 < 12 { "am" } else { "pm" };
    // 12-hour clocks say 12, not 0
    let hour = match hour24 % 12 {
        0 => 12,
        h => h,
    };
    format!("{hour}:{minute:02}{suffix}")
}

/// The inventory's last line: a loose sense of how long until Tech Lancaster.
///
/// Phrased in rough units on purpose; see the table in SPEC §7.
pub fn countdown(elapsed: Duration) -> String {
    let remaining = DAY_MINUTES.saturating_sub(game_minutes(elapsed));
    match remaining {
        90.. => {
            // Round to the nearest hour; 90+ minutes never rounds down to 1
            let hours = (remaining + 30) / 60;
            format!("a nagging awareness that Tech Lancaster starts in {hours} hours")
        }
        45..90 => "a nagging awareness that Tech Lancaster starts in about an hour".to_string(),
        10..45 => {
            format!("a growing certainty that Tech Lancaster starts in {remaining} minutes")
        }
        _ => "the distinct sound of your name being called".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(s: u64) -> Duration {
        Duration::from_secs(s)
    }

    #[test]
    fn game_minutes_spans_the_day_and_clamps() {
        assert_eq!(game_minutes(secs(0)), 0);
        assert_eq!(game_minutes(secs(ENDING_SECS)), DAY_MINUTES);
        assert_eq!(game_minutes(secs(10_000)), DAY_MINUTES);
    }

    #[test]
    fn time_string_formats_12_hour_clock() {
        assert_eq!(time_string(secs(0)), "6:00am");
        // 360 game minutes after 6am is noon
        assert_eq!(time_string(Duration::from_millis(131_539)), "12:00pm");
        assert_eq!(time_string(secs(ENDING_SECS)), "7:00pm");
    }

    #[test]
    fn phase_boundaries_match_spec_table() {
        assert_eq!(phase(secs(0)), Phase::Morning);
        assert_eq!(phase(Duration::from_millis(109_999)), Phase::Morning);
        assert_eq!(phase(secs(110)), Phase::Midday);
        assert_eq!(phase(secs(175)), Phase::Afternoon);
        assert_eq!(phase(Duration::from_millis(239_999)), Phase::Afternoon);
        assert_eq!(phase(secs(240)), Phase::Evening);
        assert_eq!(phase(secs(10_000)), Phase::Evening);
    }

    #[test]
    fn funnel_and_ending_thresholds() {
        assert!(!is_funnel(Duration::from_millis(239_999)));
        assert!(is_funnel(secs(240)));
        assert!(!is_over(Duration::from_millis(284_999)));
        assert!(is_over(secs(285)));
    }

    #[test]
    fn countdown_tiers() {
        assert!(countdown(secs(0)).ends_with("in 13 hours"));
        // Funnel start is ~4:56pm, about two hours out
        assert!(countdown(secs(FUNNEL_SECS)).ends_with("in 2 hours"));
        assert!(countdown(secs(270)).contains("minutes"));
        assert_eq!(
            countdown(secs(ENDING_SECS)),
            "the distinct sound of your name being called"
        );
    }
}
