use hermes_core::hermes::Hermes;

fn main() {
    tracing_subscriber::fmt::init();

    let hermes = Hermes::from_osm_file("./data/osm/belgium-latest.osm.pbf");
    hermes.save("./data/");
}
