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
    let first_index = closest_point_index(geometry, points.0);
    let second_index = closest_point_index(geometry, points.1);
    // let mut virtual_geometry = Vec::new();

    let mut sorted_points = vec![points.0, points.1];

    sorted_points.sort_by(|a, b| {
        a.haversine_distance(&geometry[0])
            .cmp(&b.haversine_distance(&geometry[0]))
    });

    sorted_points.into_iter().cloned().collect::<Vec<_>>()

    // if let (Some(first_index), Some(second_index)) = (first_index, second_index) {
    //     if first_index <= second_index {
    //         virtual_geometry.push(*points.0);
    //         // virtual_geometry.extend_from_slice(&geometry[first_index..=second_index]);
    //         virtual_geometry.push(*points.1);
    //     } else {
    //         virtual_geometry.push(*points.1);
    //         // virtual_geometry.extend_from_slice(&geometry[second_index..=first_index]);
    //         virtual_geometry.push(*points.0);
    //     }
    // }

    // virtual_geometry
}

pub fn generate_intermediate_points_on_line(
    start: &GeoPoint,
    end: &GeoPoint,
    interval: Distance<Meters>,
) -> Vec<GeoPoint> {
    let distance = start.haversine_distance(end);

    let num_points = (distance / interval).ceil() as usize;

    let mut points = Vec::with_capacity(num_points);

    for i in 1..num_points {
        let fraction = i as f64 / num_points as f64;

        points.push(GeoPoint::new(
            start.lon() + fraction * (end.lon() - start.lon()),
            start.lat() + fraction * (end.lat() - start.lat()),
        ))
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
