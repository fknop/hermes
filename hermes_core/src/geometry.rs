use crate::{
    distance::{Distance, Meters, meters},
    geopoint::GeoPoint,
};

pub fn compute_geometry_distance(geometry: &[GeoPoint]) -> Distance<Meters> {
    let mut distance = meters!(0);
    for i in 0..geometry.len() - 1 {
        distance = distance + geometry[i].distance(&geometry[i + 1]);
    }

    distance
}

pub fn closest_point_index(points: &[GeoPoint], point: &GeoPoint) -> Option<usize> {
    points
        .iter()
        .enumerate()
        .min_by(|(_, p), (_, p2)| point.distance(p).cmp(&point.distance(p2)))
        .map(|v| v.0)
}

pub fn split_geometry<'a>(
    points: &'a [GeoPoint],
    point: &'a GeoPoint,
) -> (&'a [GeoPoint], &'a [GeoPoint]) {
    let index = closest_point_index(points, point);

    match index {
        None => (points, &[]),
        Some(index) => (&points[..index], &points[index..]),
    }
}

pub fn create_virtual_geometries(
    points: &[GeoPoint],
    point: &GeoPoint,
) -> (Vec<GeoPoint>, Vec<GeoPoint>) {
    let mut first_geometry = Vec::new();
    let mut second_geometry = Vec::new();

    let (first_segment, second_segment) = split_geometry(points, point);

    first_geometry.extend_from_slice(first_segment);
    first_geometry.push(*point);

    second_geometry.push(*point);
    second_geometry.extend_from_slice(second_segment);
    (first_geometry, second_geometry)
}

pub fn generate_intermediate_points_on_line(
    start: &GeoPoint,
    end: &GeoPoint,
    interval: Distance<Meters>,
) -> Vec<GeoPoint> {
    let distance = start.distance(end);

    let num_points = (distance / interval).ceil() as usize;

    let mut points = Vec::with_capacity(num_points);

    for i in 1..num_points {
        let fraction = i as f64 / num_points as f64;

        points.push(GeoPoint {
            lat: start.lat + fraction * (end.lat - start.lat),
            lon: start.lon + fraction * (end.lon - start.lon),
        })
    }

    points
}

pub fn interpolate_geometry(points: &[GeoPoint], interval: Distance<Meters>) -> Vec<GeoPoint> {
    let mut interpolated_line = Vec::new();

    interpolated_line.push(points[0]);

    for window in points.windows(2) {
        interpolated_line.extend(generate_intermediate_points_on_line(
            &window[0], &window[1], interval,
        ));
    }

    interpolated_line.push(points[points.len() - 1]);

    interpolated_line
}
