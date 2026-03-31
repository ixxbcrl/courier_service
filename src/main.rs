use std::io::{self, BufRead};

use courier_service::cost::calculate_costs;

fn main() {
    let stdin = io::stdin();
    let lines: Vec<String> = stdin
        .lock()
        .lines()
        .filter_map(|l| l.ok())
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        eprintln!("No input provided.");
        std::process::exit(1);
    }

    // First line: base_delivery_cost n_packages
    let first: Vec<&str> = lines[0].split_whitespace().collect();
    let base_delivery_cost: f64 = first[0].parse().expect("base_delivery_cost must be a number");
    let n_packages: usize = first[1].parse().expect("n_packages must be an integer");

    // Next n lines: pkg_id weight distance offer_code
    let mut pkg_ids: Vec<String> = Vec::with_capacity(n_packages);
    let mut pkg_weights: Vec<f64> = Vec::with_capacity(n_packages);
    let mut pkg_distances: Vec<f64> = Vec::with_capacity(n_packages);
    let mut pkg_offers: Vec<String> = Vec::with_capacity(n_packages);

    for i in 0..n_packages {
        let parts: Vec<&str> = lines[1 + i].split_whitespace().collect();
        pkg_ids.push(parts[0].to_string());
        pkg_weights.push(parts[1].parse().expect("weight must be a number"));
        pkg_distances.push(parts[2].parse().expect("distance must be a number"));
        pkg_offers.push(parts[3].to_string());
    }

    let packages: Vec<(&str, f64, f64, &str)> = (0..n_packages)
        .map(|i| (pkg_ids[i].as_str(), pkg_weights[i], pkg_distances[i], pkg_offers[i].as_str()))
        .collect();

    let results = calculate_costs(&packages, base_delivery_cost);
    for r in &results {
        println!("{} {} {}", r.pkg_id, format_number(r.discount), format_number(r.total_cost));
    }
}

/// Formats a number as an integer when whole, otherwise two decimal places.
fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{:.2}", value)
    }
}
