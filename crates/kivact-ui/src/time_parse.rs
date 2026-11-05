//! Role: Parse wall-clock text independently of mounted field controllers.
use kivact_core::i18n::TimeOfDay;
pub(crate) fn parse_time(input: &str, seconds: bool) -> Option<TimeOfDay> {
    let lower = input.trim().to_ascii_lowercase();
    let (input, meridiem) = if let Some(time) = lower
        .strip_suffix("am")
        .or_else(|| lower.strip_suffix("a.m."))
    {
        (time.trim_end(), Some(false))
    } else if let Some(time) = lower
        .strip_suffix("pm")
        .or_else(|| lower.strip_suffix("p.m."))
    {
        (time.trim_end(), Some(true))
    } else {
        (lower.as_str(), None)
    };
    if input.is_empty() || !input.bytes().all(|b| b.is_ascii_digit() || b == b':') {
        return None;
    }
    let fields = input.split(':').collect::<Vec<_>>();
    let (hour, minute, second) = match fields.as_slice() {
        [hour] if hour.len() <= 2 => (*hour, "0", "0"),
        [compact] if compact.len() == 3 || compact.len() == 4 => {
            let (hour, minute) = compact.split_at(compact.len() - 2);
            (hour, minute, "0")
        }
        [hour, minute] => (*hour, *minute, "0"),
        [hour, minute, second] if seconds => (*hour, *minute, *second),
        _ => return None,
    };
    let mut hour = hour.parse::<u8>().ok()?;
    if let Some(pm) = meridiem {
        if !(1..=12).contains(&hour) {
            return None;
        }
        hour = hour % 12 + if pm { 12 } else { 0 };
    }
    {
        let minute = minute.parse::<u8>().ok()?;
        let second = second.parse::<u8>().ok()?;
        (hour < 24 && minute < 60 && second < 60).then_some(TimeOfDay {
            hour,
            minute,
            second,
        })
    }
}
