use jiff::SpanRelativeTo;

pub fn parse_duration(input: &str) -> Result<jiff::SignedDuration, String> {
    if let Ok(duration) = input.parse::<jiff::SignedDuration>() {
        return Ok(duration);
    }

    if let Ok(duration) = input
        .parse::<jiff::Span>()
        .and_then(|span| span.to_duration(SpanRelativeTo::days_are_24_hours()))
    {
        return Ok(duration);
    }

    if let Ok(seconds) = input.parse::<i64>() {
        return Ok(jiff::SignedDuration::from_secs(seconds.abs()));
    }

    Err(String::from("Invalid duration"))
}
