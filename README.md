# Courier Service

A command-line application for estimating delivery costs for Kiki's courier (delivery sounds better!) service.

## Problem 1: Delivery Cost Estimation

Given a base delivery cost and a list of packages, the app calculates the total delivery cost for each package — applying a discount if a valid offer code is provided and the package meets the criteria.


## Usage

First create a file for the std input. I've placed the sample input values in input.txt.

Build and run via stdin:

```bash
cargo build --release
target\release\courier_service.exe < input.txt
```

Or pipe directly without build:

```bash
echo "100 3
PKG1 5 5 OFR001
PKG2 15 5 OFR002
PKG3 10 100 OFR003" | cargo run --quiet
```

## Problem 2: Delivery Time Estimation

Extends Problem 1 with vehicle scheduling. Each vehicle has a max carriable weight and the same speed. The app simulates dispatching vehicles and estimates the delivery time for each package.

Add a vehicle line at the end of the input:

```
no_of_vehicles max_speed_kmhr max_carriable_weight_kg
```

Output gains a delivery time column:

```
pkg_id discount total_cost estimated_delivery_time_hrs
```

### Shipment selection rules (per vehicle trip)

1. Maximise number of packages
2. Among equal count — prefer heavier total weight
3. Among equal count and weight — prefer the shipment whose farthest package is closest (vehicle returns sooner)

### Example

Input:
```
100 5
PKG1 50 30 OFR001
PKG2 75 125 OFR008
PKG3 175 100 OFR003
PKG4 110 60 OFR002
PKG5 155 95 NA
2 70 200
```

Output:
```
PKG1 0 750 3.98
PKG2 0 1475 1.78
PKG3 0 2350 1.42
PKG4 105 1395 0.85
PKG5 0 2125 4.19
```

## Project Structure

```
src/
  main.rs       — CLI entry point, reads stdin and prints results
  lib.rs        — module declarations
  offers.rs     — offer code registry and discount eligibility logic
  cost.rs       — delivery cost formula and batch calculation
  scheduler.rs  — vehicle scheduling and delivery time estimation
```

## Running Tests

```bash
cargo test            # run all tests
cargo test offers     # offer code criteria tests
cargo test cost       # cost formula tests
cargo test scheduler  # delivery time / scheduling tests
```
