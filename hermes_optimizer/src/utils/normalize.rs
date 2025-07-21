pub fn normalize(value: f64, min: f64, max: f64) -> f64 {
    if min == max {
        return 0.0; // Avoid division by zero
    }

    (value - min) / (max - min)
}
