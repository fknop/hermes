use jiff::{SignedDuration, Timestamp};

use crate::problem::{
    job::ActivityId, time_window::TimeWindows, vehicle::VehicleIdx,
    vehicle_routing_problem::VehicleRoutingProblem,
};

pub(crate) fn compute_first_activity_arrival_time(
    problem: &VehicleRoutingProblem,
    vehicle_id: VehicleIdx,
    job_id: ActivityId,
) -> Timestamp {
    let task = problem.job_task(job_id);
    let vehicle = problem.vehicle(vehicle_id);
    let vehicle_depot_location_id = vehicle.depot_location_id();

    let earliest_start_time = vehicle.earliest_start_time().unwrap_or(Timestamp::MIN);
    let latest_start_time = vehicle.latest_start_time().unwrap_or(Timestamp::MAX);

    let travel_time = match vehicle_depot_location_id {
        Some(depot_location_id) => {
            problem.travel_time(vehicle, depot_location_id, task.location_id())
        }
        None => SignedDuration::ZERO,
    };

    // let time_window_start = task
    //     .time_windows()
    //     .iter()
    //     .filter(|tw| tw.is_satisfied(earliest_start_time + travel_time))
    //     .min_by_key(|tw| tw.start())
    //     .and_then(|tw| tw.start());

    // let latest_start = vehicle.latest_start_time();

    // let minimum_depot_departure_time = earliest_start_time + travel_time + depot_duration;
    // let maximum_depot_departure_time = latest_start
    //     .map(|latest| latest + travel_time + depot_duration)
    //     .unwrap_or(Timestamp::MAX);

    // match (latest_start, time_window_start) {
    //     (Some(_), Some(tw_start)) => {
    //         let ideal_depot_departure_time = tw_start - travel_time;

    //         let depot_departure_time = ideal_depot_departure_time
    //             .max(minimum_depot_departure_time)
    //             .min(maximum_depot_departure_time);

    //         depot_departure_time + travel_time
    //         // ((earliest_start_time + travel_time + depot_duration).max(tw_start)).min(latest_start)
    //     }
    //     (Some(latest_start), None) => earliest_start_time + travel_time + depot_duration,
    //     (None, Some(tw_start)) => tw_start,
    //     (None, None) => minimum_depot_departure_time + travel_time,
    // }

    compute_initial_arrival_time(
        earliest_start_time,
        latest_start_time,
        task.time_windows(),
        vehicle.depot_duration(),
        travel_time,
    )
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
    activity_id: ActivityId,
) -> Timestamp {
    let travel_time = problem.travel_time(
        problem.vehicle(vehicle_id),
        problem.job_task(previous_activity_id).location_id(),
        problem.job_task(activity_id).location_id(),
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
    activity_id: ActivityId,
) -> Timestamp {
    arrival_time + waiting_duration + problem.job_task(activity_id).duration()
}

pub(crate) fn compute_time_slack(
    problem: &VehicleRoutingProblem,
    job_id: ActivityId,
    arrival_time: Timestamp,
) -> SignedDuration {
    let task = problem.job_task(job_id);

    if let Some(max_end) = task.time_windows().end() {
        max_end.duration_since(arrival_time)
    } else {
        SignedDuration::MAX
    }
}

pub(crate) fn compute_waiting_time_slack(
    time_windows: &TimeWindows,
    arrival_time: Timestamp,
) -> SignedDuration {
    if let Some(start) = time_windows
        .iter()
        .filter(|tw| tw.is_satisfied(arrival_time))
        .filter_map(|tw| tw.start())
        .min()
    {
        arrival_time
            .duration_since(start)
            .clamp(SignedDuration::ZERO, SignedDuration::MAX)
    } else {
        SignedDuration::MAX
    }
}

fn compute_initial_arrival_time(
    earliest_start_time: Timestamp,
    latest_start_time: Timestamp,
    time_windows: &TimeWindows,
    depot_duration: SignedDuration,
    travel_time: SignedDuration,
) -> Timestamp {
    // Ignoring time windows, this is the window between which the vehicle can depart from the depot
    let minimum_depot_departure_time = earliest_start_time + depot_duration;
    let maximum_depot_departure_time = latest_start_time.saturating_add(depot_duration).unwrap();

    let time_window_start = time_windows
        .iter()
        .filter(|tw| tw.is_satisfied(earliest_start_time + depot_duration + travel_time))
        .min_by_key(|tw| tw.start())
        .and_then(|tw| tw.start());

    match time_window_start {
        Some(tw_start) => {
            let ideal_depot_departure_time = tw_start - travel_time;

            let depot_departure_time = ideal_depot_departure_time
                .clamp(minimum_depot_departure_time, maximum_depot_departure_time);

            depot_departure_time + travel_time
        }
        None => minimum_depot_departure_time + travel_time,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::problem::time_window::TimeWindow;

    #[test]
    fn test_compute_waiting_time_slack() {
        let time_windows = TimeWindows::from_vec(vec![TimeWindow::from_iso(
            Some("2026-01-16T15:00:00+01:00"),
            None,
        )]);

        assert_eq!(
            compute_waiting_time_slack(&time_windows, "2026-01-16T16:00:00+01:00".parse().unwrap()),
            SignedDuration::from_hours(1)
        );

        assert_eq!(
            compute_waiting_time_slack(&time_windows, "2026-01-16T14:00:00+01:00".parse().unwrap()),
            SignedDuration::ZERO
        );

        let time_windows = TimeWindows::from_vec(vec![TimeWindow::from_iso(
            Some("2026-01-16T15:00:00+01:00"),
            Some("2026-01-16T17:00:00+01:00"),
        )]);

        assert_eq!(
            compute_waiting_time_slack(&time_windows, "2026-01-16T16:00:00+01:00".parse().unwrap()),
            SignedDuration::from_hours(1)
        );

        assert_eq!(
            compute_waiting_time_slack(&time_windows, "2026-01-16T14:00:00+01:00".parse().unwrap()),
            SignedDuration::ZERO
        );

        let time_windows = TimeWindows::from_vec(vec![
            TimeWindow::from_iso(
                Some("2026-01-16T13:00:00+01:00"),
                Some("2026-01-16T15:00:00+01:00"),
            ),
            TimeWindow::from_iso(
                Some("2026-01-16T17:00:00+01:00"),
                Some("2026-01-16T19:00:00+01:00"),
            ),
        ]);

        assert_eq!(
            compute_waiting_time_slack(&time_windows, "2026-01-16T16:00:00+01:00".parse().unwrap()),
            SignedDuration::ZERO
        );

        assert_eq!(
            compute_waiting_time_slack(&time_windows, "2026-01-16T14:00:00+01:00".parse().unwrap()),
            SignedDuration::from_hours(1)
        );

        assert_eq!(
            compute_waiting_time_slack(&time_windows, "2026-01-16T20:00:00+01:00".parse().unwrap()),
            SignedDuration::MAX
        );
    }

    #[test]
    fn test_compute_initial_arrival_time() {
        let time_windows = TimeWindows::from_vec(vec![TimeWindow::from_iso(
            Some("2026-01-16T15:00:00+01:00"),
            None,
        )]);

        assert_eq!(
            compute_initial_arrival_time(
                "2026-01-16T10:00:00+01:00".parse().unwrap(),
                "2026-01-16T14:00:00+01:00".parse().unwrap(),
                &time_windows,
                SignedDuration::from_mins(10),
                SignedDuration::from_mins(20)
            ),
            "2026-01-16T14:30:00+01:00".parse().unwrap()
        );

        assert_eq!(
            compute_initial_arrival_time(
                "2026-01-16T13:00:00+01:00".parse().unwrap(),
                "2026-01-16T16:00:00+01:00".parse().unwrap(),
                &time_windows,
                SignedDuration::from_mins(10),
                SignedDuration::from_mins(20)
            ),
            "2026-01-16T15:00:00+01:00".parse().unwrap()
        );

        assert_eq!(
            compute_initial_arrival_time(
                "2026-01-16T15:00:00+01:00".parse().unwrap(),
                "2026-01-16T16:00:00+01:00".parse().unwrap(),
                &time_windows,
                SignedDuration::from_mins(10),
                SignedDuration::from_mins(20)
            ),
            "2026-01-16T15:30:00+01:00".parse().unwrap()
        );
    }
}
