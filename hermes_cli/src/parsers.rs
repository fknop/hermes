use jiff::SpanRelativeTo;

pub fn parse_duration(input: &str) -> Result<jiff::SignedDuration, String> {
    if let Ok(duration) = input.parse::<jiff::SignedDuration>() {
        return Ok(duration);
    }

    input
        .parse::<jiff::Span>()
        .and_then(|span| span.to_duration(SpanRelativeTo::days_are_24_hours()))
        .map_err(|e| format!("invalid duration {e}"))
}
