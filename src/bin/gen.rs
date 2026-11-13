use clap::Parser;
use rand::RngExt;
use std::fs::File;
use std::io::Write;

use fleet_route_planner::{Route, RoutesFile, Stop, Vehicle, VehiclesFile};

#[derive(Parser)]
pub struct Args {
    #[arg(short = 'r', default_value_t = 50_000)]
    n_routes: u32,

    #[arg(short = 'v', default_value_t = 200)]
    n_vehicles: u32,

    #[arg(short = 'o', default_value = "routes_big.json")]
    routes_file: String,

    #[arg(short = 'f', default_value = "vehicles_big.json")]
    vehicles_file: String,
}

// ------- main --------------------------------------------

fn main() {
    let args = Args::parse();
    let mut rng = rand::rng();

    // ------- generate routes -----------------------------
    let routes: Vec<Route> = (0..args.n_routes)
        .map(|i| {
            let route_id = format!("r{:02}", i + 1);

            // random 3–10 stops per route
            let n_stops = rng.random_range(3..=10) as u32;
            let stops: Vec<Stop> = (0..n_stops)
                .map(|j| Stop {
                    stop_id: format!("s{:02}{:02}", i + 1, j + 1),
                    distance_km: (rng.random_range(0.5_f64..=8.0) * 10.0).round() / 10.0,
                })
                .collect();

            Route { route_id, stops }
        })
        .collect();

    let vehicles: Vec<Vehicle> = (0..args.n_vehicles)
        .map(|i| Vehicle {
            id: format!("r{:03}", i + 1),
            capacity_kwh: rng.random_range(50.0_f64..=150.0),
            kwh_per_100_km: rng.random_range(12.0_f64..=22.0_f64),
        })
        .collect();

    // ------- write files ---------------------------------
    let routes_data = serde_json::to_vec_pretty(&RoutesFile { routes }).unwrap();
    let vehicles_data = serde_json::to_vec_pretty(&VehiclesFile { vehicles }).unwrap();

    File::create(&args.routes_file)
        .unwrap()
        .write_all(&routes_data)
        .unwrap();
    File::create(&args.vehicles_file)
        .unwrap()
        .write_all(&vehicles_data)
        .unwrap();

    eprintln!("wrote {} routes   → {}", args.n_routes, args.routes_file);
    eprintln!(
        "wrote {} vehicles → {}",
        args.n_vehicles, args.vehicles_file
    );
}
