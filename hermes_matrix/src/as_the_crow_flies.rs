use crate::travel_matrices::TravelMatrices;

const EARTH_RADIUS_METERS: f64 = 6_371_000.0;
fn haversine_distance<P>(from: P, to: P) -> f64
where
    P: Into<geo_types::Point>,
{
    let from: geo_types::Point = from.into();
    let to: geo_types::Point = to.into();

    let lat1_rad = from.y().to_radians();
    let lon1_rad = from.x().to_radians();
    let lat2_rad = to.y().to_radians();
    let lon2_rad = to.x().to_radians();

    let delta_lat = lat2_rad - lat1_rad;
    let delta_lon = lon2_rad - lon1_rad;

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    // Calculate distance
    EARTH_RADIUS_METERS * c
}

pub fn as_the_crow_flies_matrices<P>(points: &[P], speed_kmh: f64) -> TravelMatrices
where
    for<'a> &'a P: Into<geo_types::Point>,
{
    let num_points = points.len();
    let mut distances: Vec<f64> = vec![0.0; num_points * num_points];
    let mut times: Vec<f64> = vec![0.0; num_points * num_points];

    for (i, from) in points.iter().enumerate() {
        for (j, to) in points.iter().enumerate() {
            distances[i * num_points + j] = haversine_distance(from, to);
            times[i * num_points + j] = (distances[i * num_points + j]) / speed_kmh;
        }
    }

    TravelMatrices {
        distances,
        times,
        costs: None,
    }
}
