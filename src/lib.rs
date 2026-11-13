use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ------- Data Structures ---------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Stop {
    pub stop_id: String,
    pub distance_km: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Route {
    pub route_id: String,
    pub stops: Vec<Stop>,
}

#[derive(Serialize, Deserialize)]
pub struct RoutesFile {
    pub routes: Vec<Route>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vehicle {
    pub id: String,
    pub capacity_kwh: f64,
    pub kwh_per_100_km: f64,
}

#[derive(Serialize, Deserialize)]
pub struct VehiclesFile {
    pub vehicles: Vec<Vehicle>,
}

// ------- Helpers -----------------------------------------

pub fn route_distance(route: &Route) -> f64 {
    route.stops.iter().map(|s| s.distance_km).sum()
}

/// Maximum km the vehicle can travel on a full charge.
pub fn vehicle_range(v: &Vehicle) -> f64 {
    v.capacity_kwh * 100.0 / v.kwh_per_100_km
}

/// Energy (kWh) to complete a route of `dist` km with vehicle `v`.
pub fn energy(dist: f64, v: &Vehicle) -> f64 {
    dist * v.kwh_per_100_km / 100.0
}

// ------- Printing Helpers --------------------------------

pub fn print_worst_case(vehicle: &Vehicle, total_distance: f64, worst_case: f64) {
    println!("=== Worst case ===");
    println!(
        "Least efficient vehicle : {} ({} kWh/100km)",
        vehicle.id, vehicle.kwh_per_100_km
    );
    println!("Total distance          : {:.1} km", total_distance);
    println!("Worst-case energy       : {:.3} kWh\n", worst_case);
}

pub fn print_optimal_assignment(
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

pub fn print_comparison(worst_case: f64, total_optimal: f64) {
    let savings = worst_case - total_optimal;
    let pct = savings / worst_case * 100.0;

    println!("=== Comparison ===");
    println!("Worst case : {:.3} kWh", worst_case);
    println!("Optimal : {:.3} kWh", total_optimal);
    println!("Energy saved : {:.3} kWh  ({:.1} %)", savings, pct);
}
