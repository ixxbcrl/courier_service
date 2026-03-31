use std::io::{self, BufRead};

use courier_service::cost::calculate_costs;
use courier_service::scheduler::{schedule_deliveries, PackageInput};

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
    if first.len() < 2 {
        eprintln!("Expected: base_cost n_packages");
        std::process::exit(1);
    }
    let base_delivery_cost: f64 = first[0].parse().expect("base_delivery_cost must be a number");
    let n_packages: usize = first[1].parse().expect("n_packages must be an integer");

    // Next n lines: pkg_id weight distance offer_code
    if lines.len() < 1 + n_packages {
        eprintln!("Not enough package lines in input.");
        std::process::exit(1);
    }
    let mut pkg_ids: Vec<String> = Vec::with_capacity(n_packages);
    let mut pkg_weights: Vec<f64> = Vec::with_capacity(n_packages);
    let mut pkg_distances: Vec<f64> = Vec::with_capacity(n_packages);
    let mut pkg_offers: Vec<String> = Vec::with_capacity(n_packages);

    for i in 0..n_packages {
        let parts: Vec<&str> = lines[1 + i].split_whitespace().collect();
        if parts.len() < 4 {
            eprintln!("Package line {} has fewer than 4 fields.", i + 1);
            std::process::exit(1);
        }
        pkg_ids.push(parts[0].to_string());
        pkg_weights.push(parts[1].parse().expect("weight must be a number"));
        pkg_distances.push(parts[2].parse().expect("distance must be a number"));
        pkg_offers.push(parts[3].to_string());
    }

    // Optional last line?: n_vehicles max_speed max_weight
    let vehicle_line_idx = 1 + n_packages;
    let has_vehicles = lines.len() > vehicle_line_idx;

    // Whether its a part 2 problem or not
    if has_vehicles {
        let vparts: Vec<&str> = lines[vehicle_line_idx].split_whitespace().collect();
        if vparts.len() < 3 {
            eprintln!("Vehicle line expects: n_vehicles speed max_weight");
            std::process::exit(1);
        }
        let num_vehicles: usize = vparts[0].parse().expect("num_vehicles must be an integer");
        let max_speed: f64 = vparts[1].parse().expect("max_speed must be a number");
        let max_weight: f64 = vparts[2].parse().expect("max_weight must be a number");

        let packages: Vec<PackageInput> = (0..n_packages)
            .map(|i| PackageInput {
                pkg_id: pkg_ids[i].clone(),
                weight_kg: pkg_weights[i],
                distance_km: pkg_distances[i],
                offer_code: pkg_offers[i].clone(),
            })
            .collect();

        let results =
            schedule_deliveries(&packages, base_delivery_cost, num_vehicles, max_speed, max_weight);

        for r in &results {
            println!(
                "{} {} {} {}",
                r.pkg_id,
                format_number(r.discount),
                format_number(r.total_cost),
                format_delivery_time(r.delivery_time_hrs),
            );
        }
    } else {
        let packages: Vec<(&str, f64, f64, &str)> = (0..n_packages)
            .map(|i| {
                (
                    pkg_ids[i].as_str(),
                    pkg_weights[i],
                    pkg_distances[i],
                    pkg_offers[i].as_str(),
                )
            })
            .collect();

        let results = calculate_costs(&packages, base_delivery_cost);
        for r in &results {
            println!(
                "{} {} {}",
                r.pkg_id,
                format_number(r.discount),
                format_number(r.total_cost),
            );
        }
    }
}

/// Formats a cost value: integer display when the value is whole, otherwise
/// two decimal places.
fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{:.2}", value)
    }
}

/// Formats a delivery time always with two decimal places.
fn format_delivery_time(value: f64) -> String {
    format!("{:.2}", value)
}
