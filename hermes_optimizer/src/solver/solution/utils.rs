use jiff::{SignedDuration, Timestamp};

use crate::problem::{
    job::JobId,
    service::{Service, ServiceId},
    vehicle::VehicleId,
    vehicle_routing_problem::VehicleRoutingProblem,
};

pub(crate) fn compute_first_activity_arrival_time(
    problem: &VehicleRoutingProblem,
    vehicle_id: VehicleId,
    job_id: JobId,
) -> Timestamp {
    let task = problem.job_task(job_id);

    let vehicle_depot_location = problem.vehicle_depot_location(vehicle_id);

    let vehicle = problem.vehicle(vehicle_id);
    let earliest_start_time = vehicle
        .earliest_start_time()
        .unwrap_or_else(|| Timestamp::from_second(0).unwrap());

    let travel_time = match vehicle_depot_location {
        Some(depot_location) => problem.travel_time(depot_location.id(), task.location_id()),
        None => SignedDuration::ZERO,
    };

    let depot_duration = vehicle.depot_duration();

    let time_window_start = task
        .time_windows()
        .iter()
        .filter(|tw| tw.is_satisfied(earliest_start_time + travel_time + depot_duration))
        .min_by_key(|tw| tw.start())
        .and_then(|tw| tw.start());

    match time_window_start {
        Some(start) => (earliest_start_time + travel_time + depot_duration).max(start),
        None => earliest_start_time + travel_time + depot_duration,
    }
}

pub(crate) fn compute_vehicle_start(
    problem: &VehicleRoutingProblem,
    vehicle_id: VehicleId,
    first_service_id: ServiceId,
    first_arrival_time: Timestamp,
) -> Timestamp {
    let vehicle = problem.vehicle(vehicle_id);
    let service = problem.service(first_service_id);

    if let Some(depot_location) = problem.vehicle_depot_location(vehicle_id) {
        let travel_time = problem.travel_time(depot_location.id(), service.location_id());

        first_arrival_time - travel_time - vehicle.depot_duration()
    } else {
        first_arrival_time
    }
}

pub(crate) fn compute_vehicle_end(
    problem: &VehicleRoutingProblem,
    vehicle_id: VehicleId,
    last_service_id: ServiceId,
    last_departure_time: Timestamp,
) -> Timestamp {
    let service = problem.service(last_service_id);
    let vehicle = problem.vehicle(vehicle_id);
    if let Some(depot_location_id) = vehicle.depot_location_id()
        && vehicle.should_return_to_depot()
    {
        let travel_time = problem.travel_time(service.location_id(), depot_location_id);

        last_departure_time + travel_time + vehicle.end_depot_duration()
    } else {
        last_departure_time
    }
}

pub(crate) fn compute_activity_arrival_time(
    problem: &VehicleRoutingProblem,
    previous_job_id: JobId,
    previous_activity_departure_time: Timestamp,
    id: JobId,
) -> Timestamp {
    let travel_time = problem.travel_time(
        problem.job_task(previous_job_id).location_id(),
        problem.job_task(id).location_id(),
    );

    previous_activity_departure_time + travel_time
}

pub(crate) fn compute_waiting_duration(
    problem: &VehicleRoutingProblem,
    job_id: JobId,
    arrival_time: Timestamp,
) -> SignedDuration {
    SignedDuration::from_secs(
        problem
            .job_task(job_id)
            .time_windows()
            .iter()
            .filter(|tw| tw.is_satisfied(arrival_time))
            .filter_map(|tw| tw.start())
            .map(|start| (start.as_second() - arrival_time.as_second()).max(0))
            .min()
            .unwrap_or(0),
    )
}

pub(crate) fn compute_departure_time(
    problem: &VehicleRoutingProblem,
    arrival_time: Timestamp,
    waiting_duration: SignedDuration,
    job_id: JobId,
) -> Timestamp {
    arrival_time + waiting_duration + problem.job_task(job_id).duration()
}

pub(crate) fn compute_time_slack(
    problem: &VehicleRoutingProblem,
    job_id: JobId,
    arrival_time: Timestamp,
) -> SignedDuration {
    let task = problem.job_task(job_id);

    if let Some(max_end) = task.time_windows().iter().filter_map(|tw| tw.end()).max() {
        max_end.duration_since(arrival_time)
    } else {
        SignedDuration::MAX
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::problem::{service::ServiceBuilder, time_window::TimeWindowBuilder};

    #[test]
    fn test_compute_waiting_duration() {
        let time_windows = vec![
            TimeWindowBuilder::default()
                .with_iso_start("2025-06-10T08:00:00+02:00")
                .with_iso_end("2025-06-10T10:00:00+02:00")
                .build(),
            TimeWindowBuilder::default()
                .with_iso_start("2025-06-10T14:00:00+02:00")
                .with_iso_end("2025-06-10T16:00:00+02:00")
                .build(),
        ];
        let mut builder = ServiceBuilder::default();

        builder
            .set_time_windows(time_windows)
            .set_external_id(String::from("0"))
            .set_location_id(0);

        let service = builder.build();

        // TODO: fix tests

        // let mut waiting_duration =
        //     compute_waiting_duration(&service, "2025-06-10T09:00:00+02:00".parse().unwrap());

        // assert_eq!(waiting_duration.as_secs(), 0);

        // waiting_duration =
        //     compute_waiting_duration(&service, "2025-06-10T07:00:00+02:00".parse().unwrap());
        // assert_eq!(waiting_duration.as_secs(), 3600); // 1 hour waiting time

        // waiting_duration =
        //     compute_waiting_duration(&service, "2025-06-10T11:00:00+02:00".parse().unwrap());
        // assert_eq!(waiting_duration.as_secs(), 10800); // 3 hours waiting time

        // waiting_duration =
        //     compute_waiting_duration(&service, "2025-06-10T15:00:00+02:00".parse().unwrap());
        // assert_eq!(waiting_duration.as_secs(), 0);
    }
}
