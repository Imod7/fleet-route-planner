use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use clap::Parser;

use fleet_route_planner::{
    print_comparison, print_optimal_assignment, print_worst_case,
    route_distance, vehicle_range, energy,
    Route, RoutesFile, Vehicle, VehiclesFile};

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

    // ------- Optimal assignment (greedy) -----------------
    // routes: longest first
    let mut routes_sorted = routes;
    routes_sorted.sort_by(|a, b| route_distance(b).partial_cmp(&route_distance(a)).unwrap());

    // println!("{:<8} {:>10}", "Route", "Distance (km)");
    // for route in &routes_sorted {
    //     println!("{:<8} {:>10.2}", route.route_id, route_distance(route));
    // }

    // vehicles: most efficient first
    let mut vehicles_sorted = vehicles;
    vehicles_sorted.sort_by(|a, b| a.kwh_per_100_km.partial_cmp(&b.kwh_per_100_km).unwrap());

    // println!("{:<8} {:>10}", "Vehicle", "Range");
    // for vehicle in &vehicles_sorted {
    //     println!("{:<8?} {:>10.2?}", vehicle.id, vehicle_range(vehicle));
    // }

    let mut used: HashMap<&str, bool> = HashMap::new();
    let mut pairs: Vec<(String, String, f64, f64)> = Vec::new();
    let mut total_optimal: f64 = 0.0;

    for route in &routes_sorted {
        let d = route_distance(route);
        for v in &vehicles_sorted {
            if used.get(v.id.as_str()).copied().unwrap_or(false) {
                continue;
            }
            if vehicle_range(v) >= d {
                used.insert(v.id.as_str(), true);
                let e = energy(d, v);
                total_optimal += e;
                pairs.push((route.route_id.clone(), v.id.clone(), d, e));
                break;
            }
        }
    }

    print_optimal_assignment(&pairs, &vehicles_sorted, &used, total_optimal);

    print_comparison(worst_case, total_optimal);

    Ok(())
}
