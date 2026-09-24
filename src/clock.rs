//! Wall-clock driven game time.
//!
//! Every function takes real `elapsed` time since the program started,
//! so tests can simulate any moment without waiting. Game time also takes
//! `woke_at`, the elapsed time when the player woke from the dream: the day
//! runs from there to the ending, so a long dream makes the day go faster.

use std::time::Duration;

/// Real seconds until the ending fires. This is 7:00pm game time.
pub const ENDING_SECS: u64 = 285;

/// Real seconds after which every movement command leads to the venue.
pub const FUNNEL_SECS: u64 = 240;

/// Real seconds after which dream descriptions pick up the alarm.
pub const BEEP_SECS: u64 = 20;

/// Real seconds after which the next command wakes the player.
pub const WAKE_SECS: u64 = 35;

/// Game minutes from 6:00am to 7:00pm.
const DAY_MINUTES: u64 = 780;

/// Game minutes past midnight at which the day starts (6:00am).
const DAY_START: u64 = 6 * 60;

/// Real milliseconds until the ending.
const ENDING_MS: u64 = ENDING_SECS * 1000;

/// Time-of-day flavor. Phases only change description text, never gate anything.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Morning,
    Midday,
    Afternoon,
    Evening,
}

/// Milliseconds in `d`, clamped at the ending so the cast can't truncate.
fn ms(d: Duration) -> u64 {
    d.as_millis().min(u128::from(ENDING_MS)) as u64
}

/// Game minutes since 6:00am, clamped at 7:00pm.
///
/// The day spans real time from `woke_at` to the ending. Integer math on
/// milliseconds keeps this exact and free of float rounding.
pub fn game_minutes(elapsed: Duration, woke_at: Duration) -> u64 {
    // max(1) guards a wake at the very last moment against dividing by zero
    let day = (ENDING_MS - ms(woke_at)).max(1);
    let since = ms(elapsed).saturating_sub(ms(woke_at)).min(day);
    since * DAY_MINUTES / day
}

/// The phase of the day, by game time.
///
/// Morning until 11:00am, Midday until 2:00pm, Afternoon until 5:00pm,
/// then Evening.
pub fn phase(elapsed: Duration, woke_at: Duration) -> Phase {
    match game_minutes(elapsed, woke_at) {
        0..300 => Phase::Morning,
        300..480 => Phase::Midday,
        480..660 => Phase::Afternoon,
        _ => Phase::Evening,
    }
}

/// True once dream descriptions should include the alarm.
pub fn is_beeping(elapsed: Duration) -> bool {
    elapsed >= Duration::from_secs(BEEP_SECS)
}

/// True once a dreaming player should be woken instead of running the command.
pub fn is_wake_time(elapsed: Duration) -> bool {
    elapsed >= Duration::from_secs(WAKE_SECS)
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
pub fn time_string(elapsed: Duration, woke_at: Duration) -> String {
    let total = DAY_START + game_minutes(elapsed, woke_at);
    let (hour24, minute) = (total / 60, total % 60);
    let suffix = if hour24 < 12 { "am" } else { "pm" };
    // 12-hour clocks say 12, not 0
    let hour = match hour24 % 12 {
        0 => 12,
        h => h,
    };
    format!("{hour}:{minute:02}{suffix}")
}

/// The inventory's last line: how little time is left until TechLancaster.
///
/// Phrased in rough units on purpose, and always "only", even at 13 hours.
pub fn countdown(elapsed: Duration, woke_at: Duration) -> String {
    let remaining = DAY_MINUTES.saturating_sub(game_minutes(elapsed, woke_at));
    match remaining {
        90.. => {
            // Round to the nearest hour; 90+ minutes never rounds down to 1
            let hours = (remaining + 30) / 60;
            format!("and only {hours} hours until TechLancaster")
        }
        45..90 => "and only about an hour until TechLancaster".to_string(),
        10..45 => format!("and only {remaining} minutes until TechLancaster"),
        _ => "and TechLancaster is starting".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZERO: Duration = Duration::ZERO;

    fn secs(s: u64) -> Duration {
        Duration::from_secs(s)
    }

    fn millis(m: u64) -> Duration {
        Duration::from_millis(m)
    }

    #[test]
    fn game_minutes_spans_the_day_and_clamps() {
        assert_eq!(game_minutes(secs(0), ZERO), 0);
        assert_eq!(game_minutes(secs(ENDING_SECS), ZERO), DAY_MINUTES);
        assert_eq!(game_minutes(secs(10_000), ZERO), DAY_MINUTES);
    }

    #[test]
    fn late_wake_compresses_the_day() {
        let woke = secs(WAKE_SECS);
        assert_eq!(game_minutes(woke, woke), 0);
        // Halfway through the 250s day is 390 minutes: 12:30pm
        assert_eq!(time_string(secs(160), woke), "12:30pm");
        assert_eq!(game_minutes(secs(ENDING_SECS), woke), DAY_MINUTES);
    }

    #[test]
    fn wake_at_or_after_ending_does_not_divide_by_zero() {
        assert_eq!(game_minutes(secs(300), secs(300)), 0);
    }

    #[test]
    fn time_string_formats_12_hour_clock() {
        assert_eq!(time_string(secs(0), ZERO), "6:00am");
        // 360 game minutes after 6am is noon
        assert_eq!(time_string(millis(131_539), ZERO), "12:00pm");
        assert_eq!(time_string(secs(ENDING_SECS), ZERO), "7:00pm");
    }

    #[test]
    fn phase_boundaries_match_spec_table() {
        assert_eq!(phase(secs(0), ZERO), Phase::Morning);
        assert_eq!(phase(millis(109_615), ZERO), Phase::Morning);
        assert_eq!(phase(millis(109_616), ZERO), Phase::Midday);
        // 2:00pm falls at 175.38s
        assert_eq!(phase(secs(175), ZERO), Phase::Midday);
        assert_eq!(phase(secs(176), ZERO), Phase::Afternoon);
        assert_eq!(phase(millis(241_153), ZERO), Phase::Afternoon);
        assert_eq!(phase(millis(241_154), ZERO), Phase::Evening);
        assert_eq!(phase(secs(10_000), ZERO), Phase::Evening);
    }

    #[test]
    fn real_time_thresholds() {
        assert!(!is_beeping(millis(19_999)));
        assert!(is_beeping(secs(BEEP_SECS)));
        assert!(!is_wake_time(millis(34_999)));
        assert!(is_wake_time(secs(WAKE_SECS)));
        assert!(!is_funnel(millis(239_999)));
        assert!(is_funnel(secs(FUNNEL_SECS)));
        assert!(!is_over(millis(284_999)));
        assert!(is_over(secs(ENDING_SECS)));
    }

    #[test]
    fn countdown_tiers() {
        assert_eq!(
            countdown(secs(0), ZERO),
            "and only 13 hours until TechLancaster"
        );
        // Funnel start is ~4:56pm, about two hours out
        assert_eq!(
            countdown(secs(FUNNEL_SECS), ZERO),
            "and only 2 hours until TechLancaster"
        );
        assert_eq!(
            countdown(secs(260), ZERO),
            "and only about an hour until TechLancaster"
        );
        assert_eq!(
            countdown(secs(270), ZERO),
            "and only 42 minutes until TechLancaster"
        );
        assert_eq!(
            countdown(secs(ENDING_SECS), ZERO),
            "and TechLancaster is starting"
        );
    }
}
