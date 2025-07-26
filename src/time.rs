use std::cmp::Ordering;

use chrono::{Duration, TimeDelta, Utc};

const TIME_FORMAT: &str = "%Y-%m-%d %H:%M";

pub fn humanize(time: &str) -> (String, String) {
    let tz = &chrono::Utc::now().timezone();
    let parsed = chrono::DateTime::parse_from_rfc3339(time).unwrap();

    let delta = parsed.signed_duration_since(Utc::now());
    let periods = get_periods(delta);

    let text = periods
        .iter()
        .map(|period| -> String {
            match period {
                TimePeriod::Minutes(n) => format!("{n}m"),
                TimePeriod::Hours(n) => format!("{n}h"),
                TimePeriod::Days(n) => format!("{n}d"),
                TimePeriod::Weeks(n) => format!("{n}w"),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    if text.is_empty() {
        return (
            parsed.with_timezone(tz).format(TIME_FORMAT).to_string(), //
            "now".into(),
        );
    }

    let humanized = match delta.cmp(&TimeDelta::zero()) {
        Ordering::Equal => "now".into(),
        Ordering::Greater => format!("in {text}"),
        Ordering::Less => format!("{text} ago"),
    };

    (
        parsed.with_timezone(tz).format(TIME_FORMAT).to_string(), //
        humanized,
    )
}

#[derive(Clone, Copy, Debug)]
enum TimePeriod {
    Minutes(i64),
    Hours(i64),
    Days(i64),
    Weeks(i64),
}

fn get_periods(td: TimeDelta) -> Vec<TimePeriod> {
    let mut periods = vec![];

    let (weeks, remainder) = get_weeks(td);
    if let Some(weeks) = weeks {
        periods.push(TimePeriod::Weeks(weeks));
    }

    let (days, remainder) = get_days(remainder);
    if let Some(days) = days {
        periods.push(TimePeriod::Days(days));
    }

    let (hours, remainder) = get_hours(remainder);
    if let Some(hours) = hours {
        periods.push(TimePeriod::Hours(hours));
    }

    let (minutes, _) = get_minutes(remainder);
    if let Some(minutes) = minutes {
        periods.push(TimePeriod::Minutes(minutes));
    }

    periods
}

fn get_weeks(td: TimeDelta) -> (Option<i64>, TimeDelta) {
    let weeks = td.num_weeks();
    let remainder = td - Duration::weeks(weeks);
    normalize(weeks, remainder)
}

fn get_days(td: TimeDelta) -> (Option<i64>, TimeDelta) {
    let days = td.num_days();
    let remainder = td - Duration::days(days);
    normalize(days, remainder)
}

fn get_hours(td: TimeDelta) -> (Option<i64>, TimeDelta) {
    let hours = td.num_hours();
    let remainder = td - Duration::hours(hours);
    normalize(hours, remainder)
}

fn get_minutes(td: TimeDelta) -> (Option<i64>, TimeDelta) {
    let minutes = td.num_minutes();
    let remainder = td - Duration::minutes(minutes);
    normalize(minutes, remainder)
}

fn normalize(wholes: impl Into<Option<i64>>, remainder: Duration) -> (Option<i64>, TimeDelta) {
    (wholes.into().map(i64::abs).filter(|x| *x > 0), remainder)
}
