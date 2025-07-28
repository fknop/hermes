import data from "./sample-2.json" with { type: "json" };
import { writeFileSync } from "node:fs";

const transformDuration = (seconds) => {
  return `PT${seconds}S`;
};

const vehicle_location = data.vehicles[0].warehouse;

const locations = [
  {
    id: 0,
    lat: vehicle_location.latitude,
    lon: vehicle_location.longitude,
  },
  ...data.services.map((service, index) => {
    return {
      id: index + 1,
      lat: service.location.latitude,
      lon: service.location.longitude,
    };
  }),
];

const vehicles = data.vehicles.map((vehicle, index) => {
  return {
    external_id: index.toString(),
    capacity: vehicle.capacity,
    depot_location_id: 0,
    depot_duration: transformDuration(vehicle.warehouseDuration),
    end_depot_duration: transformDuration(vehicle.warehouseReturnDuration),
    should_return_to_depot: vehicle.shouldReturnToWarehouse,
    shift: {
      earliest_start: vehicle.shift.earliestStartTime,
      latest_end: vehicle.shift.latestEndTime,
      maximum_working_duration: transformDuration(
        vehicle.shift.maximumShiftDuration,
      ),
      maximum_transport_duration: transformDuration(
        vehicle.shift.maximumTransportDuration,
      ),
    },
  };
});

const services = data.services.map((service, index) => {
  return {
    external_id: service.id,
    service_duration: transformDuration(service.duration),
    demand: service.demand,
    time_windows: service.timeWindows.map((timeWindow) => {
      return {
        start: timeWindow.start,
        end: timeWindow.end,
      };
    }),
    location_id: index + 1,
  };
});

writeFileSync(
  "output.json",
  JSON.stringify({ locations, vehicles, services }, null, 2),
);

const matrixPayload = {
  from_points: locations.map((location) => [location.lon, location.lat]),
  to_points: locations.map((location) => [location.lon, location.lat]),
  out_arrays: ["weights", "times", "distances"],
  profile: "truck",
};

writeFileSync("matrix-payload.json", JSON.stringify(matrixPayload, null, 2));
