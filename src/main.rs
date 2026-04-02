use std::io::{self, BufRead};

use courier_service::cost::calculate_costs;
use courier_service::scheduler::{schedule_deliveries, PackageInput};

/// Informative usage message in the case of a panic.
fn usage() -> &'static str {
    "Expected input format:

  base_delivery_cost  no_of_packages
  pkg_id  weight_kg  distance_km  offer_code
  ...
  (optional) no_of_vehicles  max_speed_kmhr  max_carriable_weight_kg

Example (Problem 1 — cost only):
  100 3
  PKG1 5 5 OFR001
  PKG2 15 5 OFR002
  PKG3 10 100 OFR003

Example (Problem 2 — cost + delivery time):
  100 5
  PKG1 50 30 OFR001
  PKG2 75 125 NA
  2 70 200

Notes:
  - offer_code is required per package; use NA if none applies
  - The vehicle line is optional; omit it for Problem 1"
}

fn bail(msg: &str) -> ! {
    eprintln!("Error: {}\n", msg);
    eprintln!("{}", usage());
    std::process::exit(1);
}

fn parse_f64(s: &str, field: &str, line: usize) -> f64 {
    s.parse().unwrap_or_else(|_| {
        bail(&format!(
            "line {}: '{}' is not a valid number for field '{}'",
            line, s, field
        ))
    })
}

fn parse_usize(s: &str, field: &str, line: usize) -> usize {
    s.parse().unwrap_or_else(|_| {
        bail(&format!(
            "line {}: '{}' is not a valid integer for field '{}'",
            line, s, field
        ))
    })
}

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
        bail("no input provided");
    }

    // Line 1: base_delivery_cost n_packages
    let first: Vec<&str> = lines[0].split_whitespace().collect();
    if first.len() < 2 {
        bail("line 1: expected 'base_delivery_cost no_of_packages'");
    }
    let base_delivery_cost = parse_f64(first[0], "base_delivery_cost", 1);
    if base_delivery_cost < 0.0 {
        bail("line 1: base_delivery_cost must not be negative");
    }
    let n_packages = parse_usize(first[1], "no_of_packages", 1);

    if lines.len() < 1 + n_packages {
        bail(&format!(
            "expected {} package line(s) but only found {}",
            n_packages,
            lines.len() - 1
        ));
    }

    let mut pkg_ids: Vec<String> = Vec::with_capacity(n_packages);
    let mut pkg_weights: Vec<f64> = Vec::with_capacity(n_packages);
    let mut pkg_distances: Vec<f64> = Vec::with_capacity(n_packages);
    let mut pkg_offers: Vec<String> = Vec::with_capacity(n_packages);

    for i in 0..n_packages {
        let line_no = i + 2;
        let parts: Vec<&str> = lines[1 + i].split_whitespace().collect();
        if parts.len() != 4 {
            bail(&format!(
                "line {}: expected exactly 4 fields (pkg_id weight_kg distance_km offer_code), got {} in '{}'",
                line_no,
                parts.len(),
                lines[1 + i]
            ));
        }
        pkg_ids.push(parts[0].to_string());
        let weight = parse_f64(parts[1], "weight_kg", line_no);
        if weight <= 0.0 {
            bail(&format!("line {}: weight_kg must be positive, got '{}'", line_no, parts[1]));
        }
        let distance = parse_f64(parts[2], "distance_km", line_no);
        if distance <= 0.0 {
            bail(&format!("line {}: distance_km must be positive, got '{}'", line_no, parts[2]));
        }
        pkg_weights.push(weight);
        pkg_distances.push(distance);
        pkg_offers.push(parts[3].to_string());
    }

    // Optional vehicle line: no_of_vehicles max_speed max_carriable_weight
    let vehicle_line_idx = 1 + n_packages;
    let has_vehicles = lines.len() > vehicle_line_idx;

    if has_vehicles {
        let line_no = vehicle_line_idx + 1;
        let vparts: Vec<&str> = lines[vehicle_line_idx].split_whitespace().collect();
        if vparts.len() < 3 {
            bail(&format!(
                "line {}: expected 'no_of_vehicles max_speed_kmhr max_carriable_weight_kg', got '{}' ({} field(s))",
                line_no,
                lines[vehicle_line_idx],
                vparts.len()
            ));
        }
        let num_vehicles = parse_usize(vparts[0], "no_of_vehicles", line_no);
        if num_vehicles == 0 {
            bail(&format!("line {}: no_of_vehicles must be at least 1", line_no));
        }
        let max_speed = parse_f64(vparts[1], "max_speed_kmhr", line_no);
        let max_weight = parse_f64(vparts[2], "max_carriable_weight_kg", line_no);
        for i in 0..n_packages {
            if pkg_weights[i] >= max_weight {
                bail(&format!(
                    "package '{}' weighs {}kg which meets or exceeds the vehicle limit of {}kg",
                    pkg_ids[i], pkg_weights[i], max_weight
                ));
            }
        }

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
            .map(|i| (pkg_ids[i].as_str(), pkg_weights[i], pkg_distances[i], pkg_offers[i].as_str()))
            .collect();

        let results = calculate_costs(&packages, base_delivery_cost);
        for r in &results {
            println!("{} {} {}", r.pkg_id, format_number(r.discount), format_number(r.total_cost));
        }
    }
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{:.2}", value)
    }
}

fn format_delivery_time(value: f64) -> String {
    format!("{:.2}", value)
}
