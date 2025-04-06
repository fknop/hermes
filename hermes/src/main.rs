use hermes_core::hermes::Hermes;

fn main() {
    let hermes = Hermes::from_osm_file("./data/osm/united-kingdom-latest.osm.pbf");
    hermes.save("./data/");
}
