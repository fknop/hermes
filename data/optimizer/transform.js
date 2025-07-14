import data from "./data.json" with { type: "json" };

const locations = data.services.map((service) => {
  return {
    lon: service.location.lon,
    lat: service.location.lat,
  };
});

const services = data.services.map((service, index) => {
  return {
    ...service,
    location_id: index + 1,
    location: undefined,
  };
});

const final = {
  ...data,
  locations: [...data.locations, ...locations],
  services,
};

console.log(JSON.stringify(final, null, 2));
