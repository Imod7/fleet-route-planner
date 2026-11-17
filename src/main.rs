use anyhow::Context;
use clap::Parser;
use std::fs;

use fleet_route_planner::{
    Route, RoutesFile, Vehicle, VehiclesFile, greedy_first_fit, print_comparison,
    print_optimal_assignment, print_worst_case, route_distance,
};

#[derive(Parser)]
pub struct Args {
    routes_path: String,
    vehicles_path: String,
}

// ------- main --------------------------------------------

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // load JSON
    let routes_content = fs::read_to_string(&args.routes_path)
        .with_context(|| format!("failed to read routes file: {0}", args.routes_path))?;
    let routes_raw: RoutesFile = serde_json::from_str(&routes_content)
        .with_context(|| format!("failed to parse routes content: {0}", routes_content))?;

    let vehicles_content = fs::read_to_string(&args.vehicles_path)
        .with_context(|| format!("failed to read vehicles file: {0}", args.vehicles_path))?;
    let vehicles_raw: VehiclesFile = serde_json::from_str(&vehicles_content)
        .with_context(|| format!("failed to parse vehicles content: {0}", vehicles_content))?;

    let routes: Vec<Route> = routes_raw.routes;
    let vehicles: Vec<Vehicle> = vehicles_raw.vehicles;

    // ------- precompute ----------------------------------
    let total_distance: f64 = routes.iter().map(route_distance).sum();

    // ------- Worst Case ----------------------------------
    // least efficient = highest consumption
    let least_efficient = vehicles
        .iter()
        .max_by(|a, b| a.kwh_per_100_km.partial_cmp(&b.kwh_per_100_km).unwrap())
        .unwrap();

    let worst_case = total_distance * least_efficient.kwh_per_100_km / 100.0;

    print_worst_case(least_efficient, total_distance, worst_case);

    // ------- Greedy assignment ---------------------------
    let assignment = greedy_first_fit(&routes, &vehicles);

    print_optimal_assignment(&assignment);

    print_comparison(worst_case, assignment.total_energy);

    Ok(())
}
