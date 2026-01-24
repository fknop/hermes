#[macro_export]
macro_rules! timer_debug {
    ($msg:literal,$block:expr) => {{
        let now = jiff::Timestamp::now();
        let result = $block;
        let elapsed = jiff::Timestamp::now().duration_since(now);

        tracing::debug!("{}: Took {:?}", $msg, elapsed);

        result
    }};
}
