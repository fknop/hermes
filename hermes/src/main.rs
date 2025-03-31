use hermes_core::hermes::Hermes;

fn main() {
    let hermes = Hermes::from_osm_file("./data/osm/france-latest.osm.pbf");
    hermes.save("./data/");
}
