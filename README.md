## EV Fleet Route Planner

A small Rust CLI that reads a set of delivery routes and a fleet of electric vehicles, then reports three
numbers:

1. **Worst-case energy**: what it would cost to run *every* route with the *least* efficient vehicle.
2. **Greedy assignment**: a fast heuristic that pairs each route with a vehicle so total energy is kept low.
3. **Comparison**: the absolute and percentage energy saving the greedy plan achieves over the worst case.

---

### Build & Run

```bash
# build
cargo build --release

# run
cargo run --release -- routes.json vehicles.json
```

---

### Input

1. `routes.json`: an array of routes. Each route has an identifier and an ordered list of stops; every stop records its distance (km) from the preceding one.

2. `vehicles.json`: an array of vehicles. Each vehicle has an identifier, a battery capacity (kWh), and a consumption rate (kWh/100 km).

---

### Key formulas

- **Route distance** `route_distance(r)` = sum of each stop's `distance_km`. function `route_distance`
- **Vehicle range** `vehicle_range(v)` = `capacity_kwh × 100 / kwh_per_100_km` (km on a full charge). function `vehicle_range`
- **Energy** `energy(d, v)` = `d × kwh_per_100_km / 100` (kWh to travel `d` km with vehicle `v`). function `energy`

---

### Assignment (greedy)

The greedy heuristic works in two passes:

1. **Sort routes** by total distance, **longest first**.
2. **Sort vehicles** by `kwh_per_100_km`, **lowest first** (most efficient).
3. For each route(longest first), pick the first most-efficient **unused** vehicle whose range covers the route (range >= the route distance). Mark it used.
4. Vehicles not assigned to any route are reported as *unused*.

Complexity: **O(R log R + V log V + R·V)** — fast for small fleets.

This is **O(R log R + V log V + R·V)** and is fast for small fleets, but it is *not* a minimum-cost matching.
A different ordering or a different set of routes/vehicles can make the greedy choice sub-optimal.

> This is a greedy heuristic, not a provably optimal matching. It happens to be optimal for the provided
input, but a different set of routes or vehicles could make the choice sub-optimal. For a guaranteed optimum
on small inputs, use the Hungarian algorithm or an ILP solver (e.g. `gluon` / HiGHS). The greedy result serves
as a quick upper bound.

---

### Example output

```
=== Worst case ===
Least efficient vehicle : v-00007 (23 kWh/100km)
Total distance          : 271.0 km
Worst-case energy       : 62.330 kWh

=== Optimal assignment ===
Route        Vehicle   Dist (km)  Energy (kWh)
---------------------------------------------
r08          v-00006        51.3         5.130
r07          v-00004        48.7         5.844
r06          v-00008        43.6         5.232
r04          v-00001        33.4         4.676
r02          v-00005        25.4         4.572
r01          v-00002        24.8         4.712
r05          v-00009        24.2         5.324
r03          v-00007        19.6         4.508

Unused vehicles       : v-00003

Optimal total energy  : 39.998 kWh

=== Comparison ===
Worst case : 62.330 kWh
Optimal : 39.998 kWh
Energy saved : 22.332 kWh  (35.8 %)
```

---

### Project layout

```
fleet-route-planner/
├── Cargo.toml
├── README.md
├── routes.json          < input
├── vehicles.json        < input
└── src/
    └── main.rs          < solution
```
