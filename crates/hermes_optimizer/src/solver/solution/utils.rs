use jiff::{SignedDuration, Timestamp};

use crate::problem::{
    job::ActivityId, vehicle::VehicleIdx, vehicle_routing_problem::VehicleRoutingProblem,
};

pub(crate) fn compute_first_activity_arrival_time(
    problem: &VehicleRoutingProblem,
    vehicle_id: VehicleIdx,
    job_id: ActivityId,
) -> Timestamp {
    let task = problem.job_task(job_id);
    let vehicle = problem.vehicle(vehicle_id);
    let vehicle_depot_location_id = vehicle.depot_location_id();

    let earliest_start_time = vehicle
        .earliest_start_time()
        .unwrap_or_else(|| Timestamp::from_second(0).unwrap());

    let travel_time = match vehicle_depot_location_id {
        Some(depot_location_id) => {
            problem.travel_time(vehicle, depot_location_id, task.location_id())
        }
        None => SignedDuration::ZERO,
    };

    let depot_duration = vehicle.depot_duration();

    let time_window_start = task
        .time_windows()
        .iter()
        .filter(|tw| tw.is_satisfied(earliest_start_time + travel_time + depot_duration))
        .min_by_key(|tw| tw.start())
        .and_then(|tw| tw.start());

    let latest_start = vehicle.latest_start_time();

    // e.g. earliest = 13:30
    //  latest = 13:30
    //  time_window_start = 13:30

    let minimum_depot_departure_time = earliest_start_time + depot_duration;
    let maximum_depot_departure_time = latest_start
        .map(|latest| latest + depot_duration)
        .unwrap_or(Timestamp::MAX);

    match (latest_start, time_window_start) {
        (Some(_), Some(tw_start)) => {
            let ideal_depot_departure_time = tw_start - travel_time;

            let depot_departure_time = ideal_depot_departure_time
                .max(minimum_depot_departure_time)
                .min(maximum_depot_departure_time);

            depot_departure_time + travel_time
            // ((earliest_start_time + travel_time + depot_duration).max(tw_start)).min(latest_start)
        }
        (Some(latest_start), None) => earliest_start_time + travel_time + depot_duration,
        (None, Some(tw_start)) => tw_start,
        (None, None) => minimum_depot_departure_time + travel_time,
    }
}

pub(crate) fn compute_vehicle_start(
    problem: &VehicleRoutingProblem,
    vehicle_id: VehicleIdx,
    job_id: ActivityId,
    first_arrival_time: Timestamp,
) -> Timestamp {
    let vehicle = problem.vehicle(vehicle_id);
    let job_task = problem.job_task(job_id);

    if let Some(depot_location_id) = vehicle.depot_location_id() {
        let travel_time = problem.travel_time(vehicle, depot_location_id, job_task.location_id());

        first_arrival_time - travel_time - vehicle.depot_duration()
    } else {
        first_arrival_time
    }
}

pub(crate) fn compute_vehicle_end(
    problem: &VehicleRoutingProblem,
    vehicle_id: VehicleIdx,
    activity_id: ActivityId,
    last_departure_time: Timestamp,
) -> Timestamp {
    let job_task = problem.job_task(activity_id);
    let vehicle = problem.vehicle(vehicle_id);
    if let Some(depot_location_id) = vehicle.depot_location_id()
        && vehicle.should_return_to_depot()
    {
        let travel_time = problem.travel_time(vehicle, job_task.location_id(), depot_location_id);
        last_departure_time + travel_time + vehicle.end_depot_duration()
    } else {
        last_departure_time
    }
}

pub(crate) fn compute_activity_arrival_time(
    problem: &VehicleRoutingProblem,
    vehicle_id: VehicleIdx,
    previous_activity_id: ActivityId,
    previous_activity_departure_time: Timestamp,
    id: ActivityId,
) -> Timestamp {
    let travel_time = problem.travel_time(
        problem.vehicle(vehicle_id),
        problem.job_task(previous_activity_id).location_id(),
        problem.job_task(id).location_id(),
    );

    previous_activity_departure_time + travel_time
}

pub(crate) fn compute_waiting_duration(
    problem: &VehicleRoutingProblem,
    activity_id: ActivityId,
    arrival_time: Timestamp,
) -> SignedDuration {
    problem
        .job_task(activity_id)
        .time_windows()
        .waiting_duration(arrival_time)
}

pub(crate) fn compute_departure_time(
    problem: &VehicleRoutingProblem,
    arrival_time: Timestamp,
    waiting_duration: SignedDuration,
    job_id: ActivityId,
) -> Timestamp {
    arrival_time + waiting_duration + problem.job_task(job_id).duration()
}

pub(crate) fn compute_time_slack(
    problem: &VehicleRoutingProblem,
    job_id: ActivityId,
    arrival_time: Timestamp,
    waiting_duration: SignedDuration,
) -> SignedDuration {
    let task = problem.job_task(job_id);

    // TODO: we need to take into account waiting duration for time slacks
    // e.g., if we arrive at 10:00, but have to wait until 12:00 to start service,
    // // the time slack should be calculated from 12:00, not 10:00.
    if let Some(max_end) = task.time_windows().end() {
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
