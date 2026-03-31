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

## Project Structure

```
src/
  main.rs       — CLI entry point, reads stdin and prints results
  lib.rs        — module declarations
  offers.rs     — offer code registry and discount eligibility logic
  cost.rs       — delivery cost formula and batch calculation
  scheduler.rs  — vehicle scheduling stubs (Problem 2, not yet implemented)
```

## Running Tests

```bash
cargo test          # run all tests
cargo test offers   # offer code criteria tests
cargo test cost     # cost formula tests
```
