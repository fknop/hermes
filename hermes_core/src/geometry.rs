use crate::geopoint::{GeoPoint, haversine_distance};

pub fn compute_geometry_distance(geometry: &[GeoPoint]) -> f64 {
    let mut distance = 0.0;
    for i in 0..geometry.len() - 1 {
        distance += geometry[i].distance(&geometry[i + 1])
    }

    distance
}

pub fn closest_point_index(points: &[GeoPoint], point: &GeoPoint) -> Option<usize> {
    points
        .iter()
        .enumerate()
        .min_by(|(_, p), (_, p2)| point.distance(p).total_cmp(&point.distance(p2)))
        .map(|v| v.0)
}

pub fn split_geometry<'a>(
    points: &'a [GeoPoint],
    point: &'a GeoPoint,
) -> (&'a [GeoPoint], &'a [GeoPoint]) {
    let index = closest_point_index(points, point);

    match index {
        None => (points, &[]),
        Some(index) => (&points[..=index], &points[index + 1..]),
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
    first_geometry.push(point.clone());
    first_geometry.reverse();

    second_geometry.push(point.clone());
    second_geometry.extend_from_slice(second_segment);
    (first_geometry, second_geometry)
}
