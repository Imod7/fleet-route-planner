use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use clap::Parser;
use serde::Deserialize;

// ------- Data Structures ---------------------------------

#[derive(Deserialize, Debug)]
struct Stop {
    stop_id: String,
    distance_km: f64,
}

#[derive(Deserialize, Debug)]
pub struct Route {
    route_id: String,
    stops: Vec<Stop>,
}

#[derive(Deserialize)]
pub struct RoutesFile {
    routes: Vec<Route>,
}

#[derive(Deserialize, Debug)]
pub struct Vehicle {
    id: String,
    capacity_kwh: f64,
    kwh_per_100_km: f64,
}

#[derive(Deserialize)]
pub struct VehiclesFile {
    vehicles: Vec<Vehicle>,
}

#[derive(Parser)]
pub struct Args {
    routes_path: String,
    vehicles_path: String,
}

// ------- Helpers -----------------------------------------

fn route_distance(route: &Route) -> f64 {
    route.stops.iter().map(|s| s.distance_km).sum()
}

/// Maximum km the vehicle can travel on a full charge.
fn vehicle_range(v: &Vehicle) -> f64 {
    v.capacity_kwh * 100.0 / v.kwh_per_100_km
}

/// Energy (kWh) to complete a route of `dist` km with vehicle `v`.
fn energy(dist: f64, v: &Vehicle) -> f64 {
    dist * v.kwh_per_100_km / 100.0
}

// ------- Printing Helpers --------------------------------

fn print_worst_case(vehicle: &Vehicle, total_distance: f64, worst_case: f64) {
    println!("=== Worst case ===");
    println!(
        "Least efficient vehicle : {} ({} kWh/100km)",
        vehicle.id, vehicle.kwh_per_100_km
    );
    println!("Total distance          : {:.1} km", total_distance);
    println!("Worst-case energy       : {:.3} kWh\n", worst_case);
}

fn print_optimal_assignment(
    pairs: &Vec<(String, String, f64, f64)>,
    vehicles: &[Vehicle],
    used: &HashMap<&str, bool>,
    total_optimal: f64,
) {
    println!("=== Optimal assignment ===");
    println!(
        "{:<12} {:<8} {:>10} {:>13}",
        "Route", "Vehicle", "Dist (km)", "Energy (kWh)"
    );
    println!("{}", "-".repeat(45));
    for (rid, vid, dist, e) in pairs {
        println!("{:<12} {:<8} {:>10.1} {:>13.3}", rid, vid, dist, e);
    }

    let unused: Vec<&str> = vehicles
        .iter()
        .map(|v| v.id.as_str())
        .filter(|id| !used.get(id).copied().unwrap_or(false))
        .collect();
    if !unused.is_empty() {
        println!("\nUnused vehicles       : {}", unused.join(", "));
    }
    println!("\nOptimal total energy  : {:.3} kWh\n", total_optimal);
}

fn print_comparison(worst_case: f64, total_optimal: f64) {
    let savings = worst_case - total_optimal;
    let pct = savings / worst_case * 100.0;

    println!("=== Comparison ===");
    println!("Worst case : {:.3} kWh", worst_case);
    println!("Optimal : {:.3} kWh", total_optimal);
    println!("Energy saved : {:.3} kWh  ({:.1} %)", savings, pct);
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
