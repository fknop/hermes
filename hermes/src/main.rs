use hermes_optimizer::{
    parsers::{parser::DatasetParser, solomon::SolomonParser},
    solver::{
        solver::Solver,
        solver_params::{SolverParams, SolverParamsDebugOptions, Termination, Threads},
    },
};
use hermes_routing::geopoint::GeoPoint;
use jiff::SignedDuration;
use mimalloc::MiMalloc;
use tracing::{Level, info, warn};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

struct SolomonDataset {
    file: &'static str,
    vehicles: usize,
    optimal_cost: f64,
}

fn create_solomon_dataset() -> Vec<SolomonDataset> {
    vec![
        SolomonDataset {
            file: "./data/solomon/c1/c101.txt",
            vehicles: 10,
            optimal_cost: 828.94,
        },
        SolomonDataset {
            file: "./data/solomon/c1/c102.txt",
            vehicles: 10,
            optimal_cost: 828.94,
        },
        SolomonDataset {
            file: "./data/solomon/c1/c103.txt",
            vehicles: 10,
            optimal_cost: 828.94,
        },
        SolomonDataset {
            file: "./data/solomon/c1/c104.txt",
            vehicles: 10,
            optimal_cost: 874.78,
        },
        SolomonDataset {
            file: "./data/solomon/c1/c105.txt",
            vehicles: 10,
            optimal_cost: 828.94,
        },
        SolomonDataset {
            file: "./data/solomon/c1/c106.txt",
            vehicles: 10,
            optimal_cost: 828.94,
        },
        SolomonDataset {
            file: "./data/solomon/c1/c107.txt",
            vehicles: 10,
            optimal_cost: 828.94,
        },
        SolomonDataset {
            file: "./data/solomon/c1/c108.txt",
            vehicles: 10,
            optimal_cost: 828.94,
        },
        SolomonDataset {
            file: "./data/solomon/c1/c109.txt",
            vehicles: 10,
            optimal_cost: 828.94,
        },
        // C2
        SolomonDataset {
            file: "./data/solomon/c2/c201.txt",
            vehicles: 3,
            optimal_cost: 591.56,
        },
        SolomonDataset {
            file: "./data/solomon/c2/c202.txt",
            vehicles: 3,
            optimal_cost: 591.56,
        },
        SolomonDataset {
            file: "./data/solomon/c2/c203.txt",
            vehicles: 3,
            optimal_cost: 591.17,
        },
        SolomonDataset {
            file: "./data/solomon/c2/c204.txt",
            vehicles: 3,
            optimal_cost: 590.60,
        },
        SolomonDataset {
            file: "./data/solomon/c2/c205.txt",
            vehicles: 3,
            optimal_cost: 588.88,
        },
        SolomonDataset {
            file: "./data/solomon/c2/c206.txt",
            vehicles: 3,
            optimal_cost: 588.49,
        },
        SolomonDataset {
            file: "./data/solomon/c2/c207.txt",
            vehicles: 3,
            optimal_cost: 588.29,
        },
        SolomonDataset {
            file: "./data/solomon/c2/c208.txt",
            vehicles: 3,
            optimal_cost: 588.32,
        },
        // r1
        SolomonDataset {
            file: "./data/solomon/r1/r101.txt",
            vehicles: 19,
            optimal_cost: 1650.80,
        },
        SolomonDataset {
            file: "./data/solomon/r1/r102.txt",
            vehicles: 17,
            optimal_cost: 1486.12,
        },
        // SolomonDataset {
        //     file: "./data/solomon/r1/r103.txt",
        //     vehicles: 13,
        //     optimal_cost: 1292.68,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r1/r104.txt",
        //     vehicles: 9,
        //     optimal_cost: 1007.31,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r1/r105.txt",
        //     vehicles: 14,
        //     optimal_cost: 1377.11,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r1/r106.txt",
        //     vehicles: 12,
        //     optimal_cost: 1252.03,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r1/r107.txt",
        //     vehicles: 10,
        //     optimal_cost: 1104.66,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r1/r108.txt",
        //     vehicles: 9,
        //     optimal_cost: 960.88,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r1/r109.txt",
        //     vehicles: 11,
        //     optimal_cost: 1194.84,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r1/r110.txt",
        //     vehicles: 10,
        //     optimal_cost: 1118.84,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r1/r111.txt",
        //     vehicles: 10,
        //     optimal_cost: 1096.72,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r1/r112.txt",
        //     vehicles: 9,
        //     optimal_cost: 982.14,
        // },
        // // r2
        // SolomonDataset {
        //     file: "./data/solomon/r2/r201.txt",
        //     vehicles: 4,
        //     optimal_cost: 1252.37,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r2/r202.txt",
        //     vehicles: 3,
        //     optimal_cost: 1191.70,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r2/r203.txt",
        //     vehicles: 3,
        //     optimal_cost: 939.50,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r2/r204.txt",
        //     vehicles: 2,
        //     optimal_cost: 825.52,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r2/r205.txt",
        //     vehicles: 3,
        //     optimal_cost: 994.43,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r2/r206.txt",
        //     vehicles: 3,
        //     optimal_cost: 906.14,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r2/r207.txt",
        //     vehicles: 2,
        //     optimal_cost: 890.61,
        // },
        // SolomonDataset {
        //     file: "./data/solomon/r2/r208.txt",
        //     vehicles: 2,
        //     optimal_cost: 726.82,
        // },
    ]
}

#[tokio::main]
async fn main() {
    dotenvy::from_filename("./.env.local").ok();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let datasets = create_solomon_dataset();

    for dataset in datasets {
        let parser = SolomonParser;
        let vrp = parser.parse(dataset.file).unwrap();

        let solver = Solver::new(
            vrp,
            SolverParams {
                terminations: vec![
                    // Termination::Iterations(20000),
                    Termination::VehiclesAndCosts {
                        vehicles: dataset.vehicles,
                        costs: dataset.optimal_cost + 0.5,
                    },
                    // Termination::IterationsWithoutImprovement(10000),
                    Termination::Duration(SignedDuration::from_secs(10)),
                ],
                run_intensify_search: true,
                insertion_threads: Threads::Multi(4),
                search_threads: Threads::Multi(2),
                debug_options: SolverParamsDebugOptions {
                    enable_local_search: true,
                },
                ..SolverParams::default()
            },
        );

        solver.solve();

        let best_solution = solver.current_best_solution().unwrap();
        if best_solution.solution.total_transport_costs() <= dataset.optimal_cost + 0.5
            && best_solution.solution.non_empty_routes_count() <= dataset.vehicles
            && best_solution.score.hard_score == 0.0
        {
            info!(
                "Found optimal solution for {} - Costs = {}, Vehicles = {}",
                dataset.file,
                best_solution.solution.total_transport_costs(),
                best_solution.solution.non_empty_routes_count()
            );
        } else {
            warn!(
                "Could not find optimal solution for {} - Costs = {:?}, Vehicles = {}",
                dataset.file,
                best_solution.solution.total_transport_costs(),
                best_solution.solution.non_empty_routes_count()
            );
        }
    }

    /*
    * Route #1: 20 22 24 27 30 29 6 32 33 31 35 37 38 39 36 34 28 26 23 18 19 16 14 12 15 17 13 25 9 11 10 8 21
    Route #2: 67 63 62 74 72 61 64 66 69 68 65 49 55 54 53 56 58 60 59 57 40 44 46 45 51 50 52 47 43 42 41 48
    Route #3: 93 5 75 2 1 99 100 97 92 94 95 98 7 3 4 89 91 88 84 86 83 82 85 76 71 70 73 80 79 81 78 77 96 87 90
    Cost 589.1
    */

    // solver.on_best_solution(|solution| {
    //     info!(
    //         thread = thread::current().name().unwrap_or("main"),
    //         "Score: {:?}", solution.score_analysis,
    //     );
    //     info!("Vehicles {:?}", solution.solution.routes().len());
    // });

    // solver.solve();

    // let hermes = Hermes::from_osm_file("./data/osm/united-kingdom-latest.osm.pbf");
    // let hermes = Hermes::from_osm_file("./data/osm/belgium-latest.osm.pbf");
    // hermes.save("./data/uk");
    //
    // let hermes = Hermes::from_directory("./data");

    // let brussels = GeoPoint::new(4.34878, 50.85045);
    // let liege = GeoPoint::new(5.56749, 50.63373);
    // let antwerp = GeoPoint::new(4.40346, 51.21989);

    // // let sources = vec![brussels, liege, antwerp];
    // let sources = create_belgium_coordinates();
    // let targets = sources.clone();

    // let result = hermes
    //     .matrix(MatrixRequest {
    //         sources: sources.clone(),
    //         targets: targets.clone(),
    //         profile: String::from("car"),
    //         options: None,
    //     })
    //     .unwrap();

    // let matrix = result.matrix;

    // println!("{:?}", result.duration);

    // for (source_index, source) in sources.iter().enumerate() {
    //     for (target_index, target) in targets.iter().enumerate() {
    //         let route_result = hermes
    //             .route(RoutingRequest {
    //                 start: *source,
    //                 end: *target,
    //                 options: Some(RoutingRequestOptions {
    //                     algorithm: Some(RoutingAlgorithm::ContractionHierarchies),
    //                     include_debug_info: None,
    //                 }),
    //                 profile: String::from("car"),
    //             })
    //             .unwrap();

    //         let time = route_result.path.time();
    //         let distance = route_result.path.distance();
    //         let matrix_entry = matrix.entry(source_index, target_index).unwrap();

    //         assert_eq!(
    //             distance,
    //             matrix_entry.distance(),
    //             "source_index {}, target_index {}",
    //             source_index,
    //             target_index
    //         );
    //         // assert_eq!(
    //         //     time,
    //         //     matrix_entry.time(),
    //         //     "source_index {}, target_index {}",
    //         //     source_index,
    //         //     target_index
    //         // );
    //     }
    // }
}

fn create_belgium_coordinates() -> Vec<GeoPoint> {
    vec![
        // Brussels region
        GeoPoint::new(4.34878, 50.85045),
        GeoPoint::new(4.35123, 50.84562),
        GeoPoint::new(4.36789, 50.85234),
        GeoPoint::new(4.33456, 50.86123),
        GeoPoint::new(4.37234, 50.84789),
        GeoPoint::new(4.32567, 50.85567),
        GeoPoint::new(4.38123, 50.83456),
        GeoPoint::new(4.31234, 50.86789),
        GeoPoint::new(4.39567, 50.82123),
        GeoPoint::new(4.30123, 50.87234),
        GeoPoint::new(4.40234, 50.81567),
        GeoPoint::new(4.29567, 50.87789),
        GeoPoint::new(4.41123, 50.80234),
        GeoPoint::new(4.28234, 50.88456),
        GeoPoint::new(4.42567, 50.79123),
        GeoPoint::new(4.27123, 50.89234),
        GeoPoint::new(4.43234, 50.78567),
        GeoPoint::new(4.26567, 50.89789),
        GeoPoint::new(4.44123, 50.77234),
        GeoPoint::new(4.25234, 50.90456),
        // Antwerp region
        GeoPoint::new(4.40346, 51.21989),
        GeoPoint::new(4.41234, 51.22456),
        GeoPoint::new(4.39567, 51.23123),
        GeoPoint::new(4.42123, 51.21234),
        GeoPoint::new(4.38234, 51.23789),
        GeoPoint::new(4.43567, 51.20456),
        GeoPoint::new(4.37123, 51.24234),
        GeoPoint::new(4.44234, 51.19567),
        GeoPoint::new(4.36567, 51.24789),
        GeoPoint::new(4.45123, 51.18234),
        GeoPoint::new(4.35234, 51.25456),
        GeoPoint::new(4.46567, 51.17123),
        GeoPoint::new(4.34123, 51.26234),
        GeoPoint::new(4.47234, 51.16567),
        GeoPoint::new(4.33567, 51.26789),
        GeoPoint::new(4.48123, 51.15234),
        GeoPoint::new(4.32234, 51.27456),
        GeoPoint::new(4.49567, 51.14123),
        GeoPoint::new(4.31123, 51.28234),
        GeoPoint::new(4.50234, 51.13567),
        // Ghent region
        GeoPoint::new(3.71947, 51.05),
        GeoPoint::new(3.72234, 51.05456),
        GeoPoint::new(3.71567, 51.06123),
        GeoPoint::new(3.73123, 51.04234),
        GeoPoint::new(3.70234, 51.06789),
        GeoPoint::new(3.74567, 51.03456),
        GeoPoint::new(3.69123, 51.07234),
        GeoPoint::new(3.75234, 51.02567),
        GeoPoint::new(3.68567, 51.07789),
        GeoPoint::new(3.76123, 51.01234),
        GeoPoint::new(3.67234, 51.08456),
        GeoPoint::new(3.77567, 51.00123),
        GeoPoint::new(3.66123, 51.09234),
        GeoPoint::new(3.78234, 50.99567),
        GeoPoint::new(3.65567, 51.09789),
        GeoPoint::new(3.79123, 50.98234),
        GeoPoint::new(3.64234, 51.10456),
        GeoPoint::new(3.80567, 50.97123),
        GeoPoint::new(3.63123, 51.11234),
        GeoPoint::new(3.81234, 50.96567),
        // Bruges region
        GeoPoint::new(3.22424, 51.20892),
        GeoPoint::new(3.23234, 51.21456),
        GeoPoint::new(3.21567, 51.22123),
        GeoPoint::new(3.24123, 51.20234),
        GeoPoint::new(3.20234, 51.22789),
        GeoPoint::new(3.25567, 51.19456),
        GeoPoint::new(3.19123, 51.23234),
        GeoPoint::new(3.26234, 51.18567),
        GeoPoint::new(3.18567, 51.23789),
        GeoPoint::new(3.27123, 51.17234),
        GeoPoint::new(3.17234, 51.24456),
        GeoPoint::new(3.28567, 51.16123),
        GeoPoint::new(3.16123, 51.25234),
        GeoPoint::new(3.29234, 51.15567),
        GeoPoint::new(3.15567, 51.25789),
        GeoPoint::new(3.30123, 51.14234),
        GeoPoint::new(3.14234, 51.26456),
        GeoPoint::new(3.31567, 51.13123),
        GeoPoint::new(3.13123, 51.27234),
        GeoPoint::new(3.32234, 51.12567),
        // Liège region
        GeoPoint::new(5.57978, 50.63373),
        GeoPoint::new(5.58234, 50.63789),
        GeoPoint::new(5.57567, 50.64456),
        GeoPoint::new(5.59123, 50.62567),
        GeoPoint::new(5.56234, 50.65123),
        GeoPoint::new(5.60567, 50.61234),
        GeoPoint::new(5.55123, 50.65789),
        GeoPoint::new(5.61234, 50.60456),
        GeoPoint::new(5.54567, 50.66234),
        GeoPoint::new(5.62123, 50.59567),
        GeoPoint::new(5.53234, 50.66789),
        GeoPoint::new(5.63567, 50.58234),
        GeoPoint::new(5.52123, 50.67456),
        GeoPoint::new(5.64234, 50.57123),
        GeoPoint::new(5.51567, 50.68234),
        GeoPoint::new(5.65123, 50.56567),
        GeoPoint::new(5.50234, 50.68789),
        GeoPoint::new(5.66567, 50.55234),
        GeoPoint::new(5.49123, 50.69456),
        GeoPoint::new(5.67234, 50.54123),
        // Charleroi region
        GeoPoint::new(4.44448, 50.41136),
        GeoPoint::new(4.45234, 50.41567),
        GeoPoint::new(4.43567, 50.42234),
        GeoPoint::new(4.46123, 50.40456),
        GeoPoint::new(4.42234, 50.42789),
        GeoPoint::new(4.47567, 50.39123),
        GeoPoint::new(4.41123, 50.43456),
        GeoPoint::new(4.48234, 50.38567),
        GeoPoint::new(4.40567, 50.44123),
        GeoPoint::new(4.49123, 50.37234),
        GeoPoint::new(4.39234, 50.44789),
        GeoPoint::new(4.50567, 50.36123),
        GeoPoint::new(4.38123, 50.45456),
        GeoPoint::new(4.51234, 50.35567),
        GeoPoint::new(4.37567, 50.46234),
        GeoPoint::new(4.52123, 50.34234),
        GeoPoint::new(4.36234, 50.46789),
        GeoPoint::new(4.53567, 50.33123),
        GeoPoint::new(4.35123, 50.47456),
        GeoPoint::new(4.54234, 50.32567),
        // Namur region
        GeoPoint::new(4.86746, 50.4669),
        GeoPoint::new(4.87234, 50.47123),
        GeoPoint::new(4.86567, 50.47789),
        GeoPoint::new(4.88123, 50.46234),
        GeoPoint::new(4.85234, 50.48456),
        GeoPoint::new(4.89567, 50.45123),
        GeoPoint::new(4.84123, 50.49234),
        GeoPoint::new(4.90234, 50.44567),
        GeoPoint::new(4.83567, 50.49789),
        GeoPoint::new(4.91123, 50.43234),
        GeoPoint::new(4.82234, 50.50456),
        GeoPoint::new(4.92567, 50.42123),
        GeoPoint::new(4.81123, 50.51234),
        GeoPoint::new(4.93234, 50.41567),
        GeoPoint::new(4.80567, 50.51789),
        GeoPoint::new(4.94123, 50.40234),
        GeoPoint::new(4.79234, 50.52456),
        GeoPoint::new(4.95567, 50.39123),
        GeoPoint::new(4.78123, 50.53234),
        GeoPoint::new(4.96234, 50.38567),
        // Mons region
        GeoPoint::new(3.95222, 50.45421),
        GeoPoint::new(3.95789, 50.45856),
        GeoPoint::new(3.94567, 50.46234),
        GeoPoint::new(3.96456, 50.44789),
        GeoPoint::new(3.93234, 50.46789),
        GeoPoint::new(3.97123, 50.44123),
        GeoPoint::new(3.92567, 50.47456),
        GeoPoint::new(3.98234, 50.43567),
        GeoPoint::new(3.91123, 50.48234),
        GeoPoint::new(3.99456, 50.42234),
        GeoPoint::new(3.90567, 50.48789),
        GeoPoint::new(4.00123, 50.41567),
        GeoPoint::new(3.89234, 50.49456),
        GeoPoint::new(4.01567, 50.40234),
        GeoPoint::new(3.88123, 50.50234),
        GeoPoint::new(4.02234, 50.39567),
        GeoPoint::new(3.87567, 50.50789),
        GeoPoint::new(4.03123, 50.38234),
        GeoPoint::new(3.86234, 50.51456),
        GeoPoint::new(4.04567, 50.37123),
        // Leuven region
        GeoPoint::new(4.70093, 50.87959),
        GeoPoint::new(4.70567, 50.88234),
        GeoPoint::new(4.69234, 50.88789),
        GeoPoint::new(4.71234, 50.87456),
        GeoPoint::new(4.68567, 50.89234),
        GeoPoint::new(4.72123, 50.86789),
        GeoPoint::new(4.67234, 50.89789),
        GeoPoint::new(4.73567, 50.86123),
        GeoPoint::new(4.66123, 50.90456),
        GeoPoint::new(4.74234, 50.85567),
        GeoPoint::new(4.65567, 50.91123),
        GeoPoint::new(4.75123, 50.84234),
        GeoPoint::new(4.64234, 50.91789),
        GeoPoint::new(4.76567, 50.83567),
        GeoPoint::new(4.63123, 50.92456),
        GeoPoint::new(4.77234, 50.82234),
        GeoPoint::new(4.62567, 50.93123),
        GeoPoint::new(4.78123, 50.81567),
        GeoPoint::new(4.61234, 50.93789),
        GeoPoint::new(4.79567, 50.80234),
        // Mechelen region
        GeoPoint::new(4.47762, 51.02574),
        GeoPoint::new(4.48234, 51.02987),
        GeoPoint::new(4.47123, 51.03456),
        GeoPoint::new(4.49567, 51.02123),
        GeoPoint::new(4.46567, 51.04234),
        GeoPoint::new(4.50234, 51.01567),
        GeoPoint::new(4.45234, 51.04789),
        GeoPoint::new(4.51123, 51.00234),
        GeoPoint::new(4.44123, 51.05456),
        GeoPoint::new(4.52567, 50.99567),
        GeoPoint::new(4.43567, 51.06123),
        GeoPoint::new(4.53234, 50.98234),
        GeoPoint::new(4.42234, 51.06789),
        GeoPoint::new(4.54123, 50.97123),
        GeoPoint::new(4.41123, 51.07456),
        GeoPoint::new(4.55567, 50.96567),
        GeoPoint::new(4.40567, 51.08234),
        GeoPoint::new(4.56234, 50.95234),
        GeoPoint::new(4.39234, 51.08789),
        GeoPoint::new(4.57123, 50.94123),
        // Hasselt region
        GeoPoint::new(5.33727, 50.93077),
        GeoPoint::new(5.34234, 50.93456),
        GeoPoint::new(5.33123, 50.94123),
        GeoPoint::new(5.35567, 50.92234),
        GeoPoint::new(5.32567, 50.94789),
        GeoPoint::new(5.36234, 50.91567),
        GeoPoint::new(5.31234, 50.95456),
        GeoPoint::new(5.37123, 50.90234),
        GeoPoint::new(5.30123, 50.96123),
        GeoPoint::new(5.38567, 50.89567),
        GeoPoint::new(5.29567, 50.96789),
        GeoPoint::new(5.39234, 50.88234),
        GeoPoint::new(5.28234, 50.97456),
        GeoPoint::new(5.40123, 50.87123),
        GeoPoint::new(5.27123, 50.98234),
        GeoPoint::new(5.41567, 50.86567),
        GeoPoint::new(5.26567, 50.98789),
        GeoPoint::new(5.42234, 50.85234),
        GeoPoint::new(5.25234, 50.99456),
        GeoPoint::new(5.43123, 50.84123),
        // Kortrijk region
        GeoPoint::new(3.26487, 50.82803),
        GeoPoint::new(3.26987, 50.83234),
        GeoPoint::new(3.25789, 50.83789),
        GeoPoint::new(3.27234, 50.82456),
        GeoPoint::new(3.25123, 50.84234),
        GeoPoint::new(3.28567, 50.81789),
        GeoPoint::new(3.24567, 50.84789),
        GeoPoint::new(3.29234, 50.81123),
        GeoPoint::new(3.23234, 50.85456),
        GeoPoint::new(3.30123, 50.80567),
        GeoPoint::new(3.22123, 50.86123),
        GeoPoint::new(3.31567, 50.79234),
        GeoPoint::new(3.21567, 50.86789),
        GeoPoint::new(3.32234, 50.78567),
        GeoPoint::new(3.20234, 50.87456),
        GeoPoint::new(3.33123, 50.77234),
        GeoPoint::new(3.19123, 50.88234),
        GeoPoint::new(3.34567, 50.76123),
        GeoPoint::new(3.18567, 50.88789),
        GeoPoint::new(3.35234, 50.75567),
        // Tournai region
        GeoPoint::new(3.38747, 50.60518),
        GeoPoint::new(3.39234, 50.60987),
        GeoPoint::new(3.38123, 50.61456),
        GeoPoint::new(3.40567, 50.60123),
        GeoPoint::new(3.37567, 50.62234),
        GeoPoint::new(3.41234, 50.59567),
        GeoPoint::new(3.36234, 50.62789),
        GeoPoint::new(3.42123, 50.58234),
        GeoPoint::new(3.35123, 50.63456),
        GeoPoint::new(3.43567, 50.57123),
        GeoPoint::new(3.34567, 50.64123),
        GeoPoint::new(3.44234, 50.56567),
        GeoPoint::new(3.33234, 50.64789),
        GeoPoint::new(3.45123, 50.55234),
        GeoPoint::new(3.32123, 50.65456),
        GeoPoint::new(3.46567, 50.54123),
        GeoPoint::new(3.31567, 50.66234),
        GeoPoint::new(3.47234, 50.53567),
        GeoPoint::new(3.30234, 50.66789),
        GeoPoint::new(3.48123, 50.52234),
        // Verviers region
        GeoPoint::new(5.86319, 50.59067),
        GeoPoint::new(5.86789, 50.59456),
        GeoPoint::new(5.85567, 50.60123),
        GeoPoint::new(5.87234, 50.58234),
        GeoPoint::new(5.84234, 50.60789),
        GeoPoint::new(5.88567, 50.57567),
        GeoPoint::new(5.83123, 50.61456),
        GeoPoint::new(5.89234, 50.56234),
        GeoPoint::new(5.82567, 50.62123),
        GeoPoint::new(5.90123, 50.55567),
        GeoPoint::new(5.81234, 50.62789),
        GeoPoint::new(5.91567, 50.54234),
        GeoPoint::new(5.80123, 50.63456),
        GeoPoint::new(5.92234, 50.53123),
        GeoPoint::new(5.79567, 50.64234),
        GeoPoint::new(5.93123, 50.52567),
        GeoPoint::new(5.78234, 50.64789),
        GeoPoint::new(5.94567, 50.51234),
        GeoPoint::new(5.77123, 50.65456),
        GeoPoint::new(5.95234, 50.50123),
        // Aalst region
        GeoPoint::new(4.03583, 50.93678),
        GeoPoint::new(4.04123, 50.94234),
        GeoPoint::new(4.02567, 50.94789),
        GeoPoint::new(4.05234, 50.93123),
        GeoPoint::new(4.01234, 50.95456),
        GeoPoint::new(4.06567, 50.92567),
        GeoPoint::new(4.00123, 50.96123),
        GeoPoint::new(4.07234, 50.91234),
        GeoPoint::new(3.99567, 50.96789),
        GeoPoint::new(4.08123, 50.90567),
        GeoPoint::new(3.98234, 50.97456),
        GeoPoint::new(4.09567, 50.89234),
        GeoPoint::new(3.97123, 50.98234),
        GeoPoint::new(4.10234, 50.88567),
        GeoPoint::new(3.96567, 50.98789),
        GeoPoint::new(4.11123, 50.87234),
        GeoPoint::new(3.95234, 50.99456),
        GeoPoint::new(4.12567, 50.86123),
        GeoPoint::new(3.94123, 51.00234),
        GeoPoint::new(4.13234, 50.85567),
        // La Louvière region
        GeoPoint::new(4.18836, 50.48565),
        GeoPoint::new(4.19234, 50.48987),
        GeoPoint::new(4.18123, 50.49456),
        GeoPoint::new(4.20567, 50.48123),
        GeoPoint::new(4.17567, 50.50234),
        GeoPoint::new(4.21234, 50.47567),
        GeoPoint::new(4.16234, 50.50789),
        GeoPoint::new(4.22123, 50.46234),
        GeoPoint::new(4.15123, 50.51456),
        GeoPoint::new(4.23567, 50.45123),
        GeoPoint::new(4.14567, 50.52123),
        GeoPoint::new(4.24234, 50.44567),
        GeoPoint::new(4.13234, 50.52789),
        GeoPoint::new(4.25123, 50.43234),
        GeoPoint::new(4.12123, 50.53456),
        GeoPoint::new(4.26567, 50.42123),
        GeoPoint::new(4.11567, 50.54234),
        GeoPoint::new(4.27234, 50.41567),
        GeoPoint::new(4.10234, 50.54789),
        GeoPoint::new(4.28123, 50.40234),
        // Mouscron region
        GeoPoint::new(3.20655, 50.74479),
        GeoPoint::new(3.21123, 50.74856),
        GeoPoint::new(3.19567, 50.75234),
        GeoPoint::new(3.22234, 50.74123),
        GeoPoint::new(3.18234, 50.75789),
        GeoPoint::new(3.23567, 50.73567),
        GeoPoint::new(3.17123, 50.76234),
        GeoPoint::new(3.24234, 50.72234),
        GeoPoint::new(3.16567, 50.76789),
        GeoPoint::new(3.25123, 50.71567),
        GeoPoint::new(3.15234, 50.77456),
        GeoPoint::new(3.26567, 50.70234),
        GeoPoint::new(3.14123, 50.78123),
        GeoPoint::new(3.27234, 50.69567),
        GeoPoint::new(3.13567, 50.78789),
        GeoPoint::new(3.28123, 50.68234),
        GeoPoint::new(3.12234, 50.79456),
        GeoPoint::new(3.29567, 50.67123),
        GeoPoint::new(3.11123, 50.80234),
        GeoPoint::new(3.30234, 50.66567),
        // Sint-Niklaas region
        GeoPoint::new(4.14342, 51.16636),
        GeoPoint::new(4.14789, 51.17123),
        GeoPoint::new(4.13567, 51.17567),
        GeoPoint::new(4.15234, 51.16234),
        GeoPoint::new(4.12234, 51.18234),
        GeoPoint::new(4.16567, 51.15567),
        GeoPoint::new(4.11123, 51.18789),
        GeoPoint::new(4.17234, 51.14234),
        GeoPoint::new(4.10567, 51.19456),
        GeoPoint::new(4.18123, 51.13567),
        GeoPoint::new(4.09234, 51.20123),
        GeoPoint::new(4.19567, 51.12234),
        GeoPoint::new(4.08123, 51.20789),
        GeoPoint::new(4.20234, 51.11567),
        GeoPoint::new(4.07567, 51.21456),
        GeoPoint::new(4.21123, 51.10234),
        GeoPoint::new(4.06234, 51.22123),
        GeoPoint::new(4.22567, 51.09567),
        GeoPoint::new(4.05123, 51.22789),
        GeoPoint::new(4.23234, 51.08234),
        // Genk region
        GeoPoint::new(5.50069, 50.96523),
        GeoPoint::new(5.50567, 50.96987),
        GeoPoint::new(5.49234, 50.97456),
        GeoPoint::new(5.51234, 50.96123),
        GeoPoint::new(5.48567, 50.98234),
        GeoPoint::new(5.52123, 50.95567),
        GeoPoint::new(5.47234, 50.98789),
        GeoPoint::new(5.53567, 50.94234),
        GeoPoint::new(5.46123, 50.99456),
        GeoPoint::new(5.54234, 50.93567),
        GeoPoint::new(5.45567, 51.00123),
        GeoPoint::new(5.55123, 50.92234),
        GeoPoint::new(5.44234, 51.00789),
        GeoPoint::new(5.56567, 50.91567),
        GeoPoint::new(5.43123, 51.01456),
        GeoPoint::new(5.57234, 50.90234),
        GeoPoint::new(5.42567, 51.02123),
        GeoPoint::new(5.58123, 50.89567),
        GeoPoint::new(5.41234, 51.02789),
        GeoPoint::new(5.59567, 50.88234),
        // Seraing region
        GeoPoint::new(5.49894, 50.61502),
        GeoPoint::new(5.50234, 50.61987),
        GeoPoint::new(5.49123, 50.62456),
        GeoPoint::new(5.51567, 50.61123),
        GeoPoint::new(5.48567, 50.63234),
        GeoPoint::new(5.52234, 50.60567),
        GeoPoint::new(5.47234, 50.63789),
        GeoPoint::new(5.53123, 50.59234),
        GeoPoint::new(5.46123, 50.64456),
        GeoPoint::new(5.54567, 50.58567),
        GeoPoint::new(5.45567, 50.65123),
        GeoPoint::new(5.55234, 50.57234),
        GeoPoint::new(5.44234, 50.65789),
        GeoPoint::new(5.56123, 50.56567),
        GeoPoint::new(5.43123, 50.66456),
        GeoPoint::new(5.57567, 50.55234),
        GeoPoint::new(5.42567, 50.67123),
        GeoPoint::new(5.58234, 50.54567),
        GeoPoint::new(5.41234, 50.67789),
        GeoPoint::new(5.59123, 50.53234),
        // Roeselare region
        GeoPoint::new(3.12527, 50.94653),
        GeoPoint::new(3.12987, 50.95123),
        GeoPoint::new(3.11789, 50.95567),
        GeoPoint::new(3.13456, 50.94234),
        GeoPoint::new(3.10567, 50.96234),
        GeoPoint::new(3.14234, 50.93567),
        GeoPoint::new(3.09234, 50.96789),
        GeoPoint::new(3.15123, 50.92234),
    ]
}
