use std::collections::HashSet;

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

/// Outcome of an assignment: the (route, vehicle) pairs, their total energy,
/// and the set of vehicles that went unused.
#[derive(Clone, Debug, Default)]
pub struct Assignment {
    /// (route_id, vehicle_id, distance_km, energy_kwh), in the solver's order.
    pub pairs: Vec<(String, String, f64, f64)>,
    /// Sum of the energy of all assigned pairs.
    pub total_energy: f64,
    /// Ids of vehicles that were not assigned to any route.
    pub unused: Vec<String>,
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

pub fn greedy_first_fit(routes: &[Route], vehicles: &[Vehicle]) -> Assignment {
    // routes: longest first
    let mut routes_sorted: Vec<&Route> = routes.iter().collect();
    routes_sorted.sort_by(|a, b| route_distance(b).partial_cmp(&route_distance(a)).unwrap());

    // println!("{:<8} {:>10}", "Route", "Distance (km)");
    // for route in &routes_sorted {
    //     println!("{:<8} {:>10.2}", route.route_id, route_distance(route));
    // }

    // vehicles: most efficient first
    let mut vehicles_sorted: Vec<&Vehicle> = vehicles.iter().collect();
    vehicles_sorted.sort_by(|a, b| a.kwh_per_100_km.partial_cmp(&b.kwh_per_100_km).unwrap());

    // println!("{:<8} {:>10}", "Vehicle", "Range");
    // for vehicle in &vehicles_sorted {
    //     println!("{:<8?} {:>10.2?}", vehicle.id, vehicle_range(vehicle));
    // }

    let mut used: HashSet<&str> = HashSet::with_capacity(vehicles.len());
    let mut pairs: Vec<(String, String, f64, f64)> = Vec::with_capacity(routes_sorted.len().min(vehicles_sorted.len()));
    let mut total_energy: f64 = 0.0;

    for route in &routes_sorted {
        let d = route_distance(route);
        for v in &vehicles_sorted {
            if used.contains(v.id.as_str()) {
                continue;
            }
            if vehicle_range(v) >= d {
                used.insert(v.id.as_str());
                let e = energy(d, v);
                total_energy += e;
                pairs.push((route.route_id.clone(), v.id.clone(), d, e));
                break;
            }
        }
    }

    let unused: Vec<String> = vehicles_sorted
        .iter()
        .map(|v| v.id.clone())
        .filter(|id| !used.contains(id.as_str()))
        .collect();

    Assignment {
        pairs,
        total_energy,
        unused,
    }
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

pub fn print_optimal_assignment(a: &Assignment) {
    println!("=== Optimal assignment ===");
    println!(
        "{:<12} {:<8} {:>10} {:>13}",
        "Route", "Vehicle", "Dist (km)", "Energy (kWh)"
    );
    println!("{}", "-".repeat(45));
    for (rid, vid, dist, e) in &a.pairs {
        println!("{:<12} {:<8} {:>10.1} {:>13.3}", rid, vid, dist, e);
    }

    if !a.unused.is_empty() {
        println!("\nUnused vehicles       : {}", a.unused.join(", "));
    }
    println!("\nOptimal total energy  : {:.3} kWh\n", a.total_energy);
}

pub fn print_comparison(worst_case: f64, total_optimal: f64) {
    let savings = worst_case - total_optimal;
    let pct = savings / worst_case * 100.0;

    println!("=== Comparison ===");
    println!("Worst case : {:.3} kWh", worst_case);
    println!("Optimal : {:.3} kWh", total_optimal);
    println!("Energy saved : {:.3} kWh  ({:.1} %)", savings, pct);
}
