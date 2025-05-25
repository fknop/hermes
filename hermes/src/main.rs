use hermes_core::{geopoint::GeoPoint, hermes::Hermes, matrix::matrix_request::MatrixRequest};

fn main() {
    tracing_subscriber::fmt::init();

    // let hermes = Hermes::from_osm_file("./data/osm/united-kingdom-latest.osm.pbf");
    // let hermes = Hermes::from_osm_file("./data/osm/belgium-latest.osm.pbf");
    // hermes.save("./data/uk");
    //
    let hermes = Hermes::from_directory("./data");

    let brussels = GeoPoint::new(4.34878, 50.85045);
    let liege = GeoPoint::new(5.56749, 50.63373);
    let antwerp = GeoPoint::new(4.40346, 51.21989);

    let sources = vec![brussels, liege, antwerp];
    let targets = sources.clone();

    let result = hermes.matrix(MatrixRequest {
        sources,
        targets,
        profile: String::from("car"),
        options: None,
    });

    let matrix = result.unwrap().matrix;
    println!("BXL -> Liège {:?}", matrix.entry(0, 1));
    println!("BXL -> Antwerp {:?}", matrix.entry(0, 2));

    println!("Liège -> BXL {:?}", matrix.entry(1, 0));
    println!("Liège -> Antwerp {:?}", matrix.entry(1, 2));

    println!("Antwerp -> BXL {:?}", matrix.entry(2, 0));
    println!("Antwerp -> Liège {:?}", matrix.entry(2, 1));
}
