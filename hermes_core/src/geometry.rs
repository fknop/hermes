use crate::{
    distance::{Distance, Meters},
    geopoint::GeoPoint,
    meters,
};

pub fn compute_geometry_distance(geometry: &[GeoPoint]) -> Distance<Meters> {
    let mut distance = meters!(0);
    for i in 0..geometry.len() - 1 {
        distance = distance + geometry[i].haversine_distance(&geometry[i + 1]);
    }

    distance
}

pub fn closest_point_index(points: &[GeoPoint], point: &GeoPoint) -> Option<usize> {
    points
        .iter()
        .enumerate()
        .min_by(|(_, p), (_, p2)| {
            point
                .haversine_distance(p)
                .cmp(&point.haversine_distance(p2))
        })
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
    geometry: &[GeoPoint],
    point: &GeoPoint,
) -> (Vec<GeoPoint>, Vec<GeoPoint>) {
    let mut first_geometry = Vec::new();
    let mut second_geometry = Vec::new();

    let (first_segment, second_segment) = split_geometry(geometry, point);

    first_geometry.extend_from_slice(first_segment);
    first_geometry.push(*point);

    second_geometry.push(*point);
    second_geometry.extend_from_slice(second_segment);
    (first_geometry, second_geometry)
}

pub fn create_virtual_geometry_between_points(
    geometry: &[GeoPoint],
    points: (&GeoPoint, &GeoPoint),
) -> Vec<GeoPoint> {
    let mut sorted_points = vec![points.0, points.1];

    sorted_points.sort_by(|a, b| {
        a.haversine_distance(&geometry[0])
            .cmp(&b.haversine_distance(&geometry[0]))
    });

    sorted_points.into_iter().cloned().collect::<Vec<_>>()
}
