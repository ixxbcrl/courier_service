//! Vehicle scheduling and delivery time estimation.
//!
//! The rough idea: At each step the next available vehicle
//! loads the best possible combination of undelivered packages (maximise count,
//! then total weight, then minimise max distance) and departs for the task.

use crate::cost::calculate_cost;

/// Input description of a single package for scheduling.
#[derive(Debug, Clone)]
pub struct PackageInput {
    pub pkg_id: String,
    pub weight_kg: f64,
    pub distance_km: f64,
    pub offer_code: String,
}

/// The combined cost + delivery time result for a single package.
#[derive(Debug, Clone, PartialEq)]
pub struct PackageDeliveryResult {
    /// Package identifier.
    pub pkg_id: String,
    /// Absolute discount amount (zero when no offer applied).
    pub discount: f64,
    /// Final delivery cost after discount.
    pub total_cost: f64,
    /// Estimated delivery time in hours, truncated to 2 decimal places.
    pub delivery_time_hrs: f64,
}

/// Truncates a floating-point value to exactly two decimal places. Everything after
/// the second decimal place is discarded.
pub fn truncate_to_2dp(value: f64) -> f64 {
    (value * 100.0).floor() / 100.0
}

/// Returns all combinations of exactly `k` elements chosen from `items`.
/// Simple C(n, k)
fn combinations(items: &[usize], k: usize) -> Vec<Vec<usize>> {
    if k == 0 {
        return vec![vec![]];
    }
    if items.len() < k {
        return vec![];
    }
    let mut result = Vec::new();
    for (i, &item) in items.iter().enumerate() {
        for mut rest in combinations(&items[i + 1..], k - 1) {
            rest.insert(0, item);
            result.push(rest);
        }
    }
    result
}

/// Selects the best subset of `remaining` package indices to load onto one vehicle.
///
/// Two stages:
/// - Stage 1: sort by weight ascending, brute force count how many packages fit → max count (k).
/// - Stage 2: among all `k`-combinations, pick the heaviest valid subset;
///   break ties by smallest max distance.
fn best_subset(remaining: &[usize], packages: &[PackageInput], max_weight_kg: f64) -> Vec<usize> {
    // Stage 1: find the maximum number of packages that can fit (just a brute force count)
    let mut by_weight_asc = remaining.to_vec();
    by_weight_asc.sort_by(|&a, &b| packages[a].weight_kg.partial_cmp(&packages[b].weight_kg).unwrap());

    let mut k = 0;
    let mut running = 0.0_f64;

    // we're just accumulating (blindly) with the lightest package first until we hit the max weight
    for &idx in &by_weight_asc {
        let next = running + packages[idx].weight_kg;
        if next < max_weight_kg {
            running = next;
            k += 1;
        } else {
            break;
        }
    }

    if k == 0 {
        return vec![];
    }

    // Stage 2: among all k-combinations, pick the heaviest valid subset
    // Tiebreak by smallest max distance so the vehicle returns soonest
    let mut best: Vec<usize> = Vec::new();
    let mut best_weight = -1.0_f64;
    let mut best_max_dist = f64::MAX;

    // We're just doing a C(n, k) here bounded by k no. of packages
    for combo in combinations(&by_weight_asc, k) {
        let total_weight: f64 = combo.iter().map(|&i| packages[i].weight_kg).sum();
        if total_weight >= max_weight_kg {
            continue;
        }
        let max_dist = combo.iter().map(|&i| packages[i].distance_km).fold(0.0_f64, f64::max);

        // Primary goal here: maximize the total weight of the subset, break ties by max distance
        if total_weight > best_weight
            || (total_weight == best_weight && max_dist < best_max_dist)
        {
            best = combo;
            best_weight = total_weight;
            best_max_dist = max_dist;
        }
    }

    best
}

/// Schedules deliveries for a fleet of identical vehicles and returns one
/// `PackageDeliveryResult` per package, in the same order as `packages`.
///
/// Each iteration picks the vehicle that becomes available soonest, loads it
/// with the best feasible shipment of undelivered packages, records delivery
/// times, and advances the vehicle's availability clock by
/// `2 * truncate_to_2dp(max_distance / speed)` - for round trip
pub fn schedule_deliveries(
    packages: &[PackageInput],
    base_delivery_cost: f64,
    num_vehicles: usize,
    max_speed_kmhr: f64,
    max_weight_kg: f64,
) -> Vec<PackageDeliveryResult> {
    assert!(num_vehicles >= 1, "num_vehicles must be at least 1");
    for p in packages {
        assert!(
            p.weight_kg < max_weight_kg,
            "package '{}' weighs {}kg which meets or exceeds the vehicle limit of {}kg",
            p.pkg_id, p.weight_kg, max_weight_kg
        );
    }

    // Pre-compute costs for every package using the cost module
    let costs: Vec<_> = packages
        .iter()
        .map(|p| {
            calculate_cost(
                &p.pkg_id,
                p.weight_kg,
                p.distance_km,
                &p.offer_code,
                base_delivery_cost,
            )
        })
        .collect();

    // Remaining undelivered package indices.
    let mut undelivered: Vec<usize> = (0..packages.len()).collect();

    // When each vehicle next becomes available (all start at t = 0)
    let mut vehicle_available: Vec<f64> = vec![0.0; num_vehicles];

    // Delivery time for each package index; filled in as shipments depart
    let mut delivery_times: Vec<Option<f64>> = vec![None; packages.len()];

    while !undelivered.is_empty() {
        // Find the vehicle with the earliest availability time.
        let (vehicle_idx, &current_time) = vehicle_available
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .expect("at least one vehicle exists");

        // Choose the best subset from alist of undelivered packages
        let shipment = best_subset(&undelivered, packages, max_weight_kg);
        // Compute delivery times and find the max distance in this shipment
        let mut max_dist_in_shipment = 0.0_f64;
        for &pkg_idx in &shipment {
            let delivery_time =
                truncate_to_2dp(current_time + packages[pkg_idx].distance_km / max_speed_kmhr);
            delivery_times[pkg_idx] = Some(delivery_time);
            if packages[pkg_idx].distance_km > max_dist_in_shipment {
                max_dist_in_shipment = packages[pkg_idx].distance_km;
            }
        }

        // Vehicle return time uses the truncated one-way travel to the farthest
        let one_way_truncated = truncate_to_2dp(max_dist_in_shipment / max_speed_kmhr);
        vehicle_available[vehicle_idx] = current_time + 2.0 * one_way_truncated;

        // Remove shipped packages from the undelivered set.
        let shipment_set: std::collections::HashSet<usize> = shipment.into_iter().collect();
        undelivered.retain(|idx| !shipment_set.contains(idx));
    }

    // Assemble results in the original input order.
    packages
        .iter()
        .enumerate()
        .map(|(i, p)| PackageDeliveryResult {
            pkg_id: p.pkg_id.clone(),
            discount: costs[i].discount,
            total_cost: costs[i].total_cost,
            delivery_time_hrs: delivery_times[i]
                .expect("every package must have been delivered"),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: construct a PackageInput concisely.
    fn pkg(id: &str, weight: f64, distance: f64, offer: &str) -> PackageInput {
        PackageInput {
            pkg_id: id.to_string(),
            weight_kg: weight,
            distance_km: distance,
            offer_code: offer.to_string(),
        }
    }

    // Helper: find a result by pkg_id to allow order-independent assertions.
    fn find<'a>(results: &'a [PackageDeliveryResult], id: &str) -> &'a PackageDeliveryResult {
        results
            .iter()
            .find(|r| r.pkg_id == id)
            .unwrap_or_else(|| panic!("pkg_id '{}' not found in results", id))
    }

    #[test]
    fn test_truncate_floors_not_rounds_third_decimal() {
        // 3.456 should become 3.45, not 3.46
        assert_eq!(truncate_to_2dp(3.456), 3.45);
    }

    #[test]
    fn test_truncate_sample_2_pkg2_time() {
        // 125/70 = 1.78571... → 1.78
        assert_eq!(truncate_to_2dp(125.0 / 70.0), 1.78);
    }

    #[test]
    fn test_truncate_sample_2_pkg3_time() {
        // 100/70 = 1.42857... → 1.42
        assert_eq!(truncate_to_2dp(100.0 / 70.0), 1.42);
    }

    #[test]
    fn test_truncate_sample_2_pkg4_time() {
        // 60/70 = 0.85714... → 0.85
        assert_eq!(truncate_to_2dp(60.0 / 70.0), 0.85);
    }

    #[test]
    fn test_truncate_sample_2_pkg5_time() {
        // V2 available at 2.84 hrs; 95/70 = 1.35714...; 2.84 + 1.357... = 4.197... → 4.19
        let v2_available: f64 = 2.84;
        let travel = 95.0_f64 / 70.0;
        assert_eq!(truncate_to_2dp(v2_available + travel), 4.19);
    }

    #[test]
    fn test_truncate_sample_2_pkg1_time() {
        // V1 available at 3.56 hrs; 30/70 = 0.42857...; 3.56 + 0.428... = 3.988... → 3.98
        let v1_available: f64 = 3.56;
        let travel = 30.0_f64 / 70.0;
        assert_eq!(truncate_to_2dp(v1_available + travel), 3.98);
    }

    #[test]
    fn test_truncate_exact_two_decimal_places_unchanged() {
        assert_eq!(truncate_to_2dp(1.50), 1.50);
        assert_eq!(truncate_to_2dp(0.00), 0.00);
    }

    #[test]
    fn test_truncate_integer_value() {
        assert_eq!(truncate_to_2dp(5.0), 5.0);
    }

    // --- schedule_deliveries: greedy selection ---

    #[test]
    fn test_shipment_selection_prefers_more_packages_over_heavier_single() {
        // Two packages that fit together are preferred over one heavy single.
        // PKG_A=80kg, PKG_B=80kg (total=160, within 200 limit)
        // PKG_C=190kg (heavier single, but only 1 package)
        // Greedy must pick PKG_A + PKG_B first.
        let packages = vec![
            pkg("PKG_A", 80.0, 10.0, "NA"),
            pkg("PKG_B", 80.0, 20.0, "NA"),
            pkg("PKG_C", 190.0, 5.0, "NA"),
        ];
        let results = schedule_deliveries(&packages, 100.0, 1, 70.0, 200.0);
        // PKG_A and PKG_B must have earlier delivery times than PKG_C.
        let time_a = find(&results, "PKG_A").delivery_time_hrs;
        let time_b = find(&results, "PKG_B").delivery_time_hrs;
        let time_c = find(&results, "PKG_C").delivery_time_hrs;
        assert!(time_a < time_c, "PKG_A should be delivered before PKG_C");
        assert!(time_b < time_c, "PKG_B should be delivered before PKG_C");
    }

    #[test]
    fn test_shipment_tie_broken_by_heavier_total_weight() {
        // Two pairs both have 2 packages within the limit.
        // Pair 1: PKG_A(90kg) + PKG_B(90kg) = 180kg
        // Pair 2: PKG_C(70kg) + PKG_D(70kg) = 140kg
        // Greedy must pick Pair 1 (heavier) first.
        let packages = vec![
            pkg("PKG_A", 90.0, 10.0, "NA"),
            pkg("PKG_B", 90.0, 10.0, "NA"),
            pkg("PKG_C", 70.0, 10.0, "NA"),
            pkg("PKG_D", 70.0, 10.0, "NA"),
        ];
        // Single vehicle so sequencing is observable.
        let results = schedule_deliveries(&packages, 100.0, 1, 70.0, 200.0);
        let time_a = find(&results, "PKG_A").delivery_time_hrs;
        let time_c = find(&results, "PKG_C").delivery_time_hrs;
        // PKG_A (from heavier shipment) must be delivered no later than PKG_C.
        assert!(
            time_a <= time_c,
            "heavier shipment (PKG_A) should be scheduled first"
        );
    }

    #[test]
    fn test_vehicle_becomes_available_after_round_trip() {
        // Single vehicle, two sequential trips. Both packages weigh 100kg (equal),
        // so the tiebreaker fires: prefer the shorter distance first.
        // Trip 1: PKG_B at 35km → delivery = truncate(35/70) = 0.50hr; return at 2*0.50 = 1.0hr.
        // Trip 2: PKG_A at 70km → delivery = truncate(1.0 + 70/70) = 2.0hr.
        let packages = vec![
            pkg("PKG_A", 100.0, 70.0, "NA"),
            pkg("PKG_B", 100.0, 35.0, "NA"),
        ];
        let results = schedule_deliveries(&packages, 100.0, 1, 70.0, 200.0);
        assert_eq!(find(&results, "PKG_B").delivery_time_hrs, 0.50);
        assert_eq!(find(&results, "PKG_A").delivery_time_hrs, 2.0);
    }

    #[test]
    fn test_two_vehicles_depart_concurrently_at_time_zero() {
        // With 2 vehicles and 2 packages, both should depart at t=0.
        let packages = vec![
            pkg("PKG_A", 50.0, 70.0, "NA"),
            pkg("PKG_B", 50.0, 140.0, "NA"),
        ];
        let results = schedule_deliveries(&packages, 100.0, 2, 70.0, 200.0);
        // PKG_A: 70/70 = 1.0
        assert_eq!(find(&results, "PKG_A").delivery_time_hrs, 1.0);
        // PKG_B: 140/70 = 2.0
        assert_eq!(find(&results, "PKG_B").delivery_time_hrs, 2.0);
    }

    // --- schedule_deliveries: full Sample 2 integration ---

    #[test]
    fn test_schedule_deliveries_sample_2_delivery_times() {
        let packages = vec![
            pkg("PKG1", 50.0, 30.0, "OFR001"),
            pkg("PKG2", 75.0, 125.0, "OFR008"),
            pkg("PKG3", 175.0, 100.0, "OFR003"),
            pkg("PKG4", 110.0, 60.0, "OFR002"),
            pkg("PKG5", 155.0, 95.0, "NA"),
        ];
        let results = schedule_deliveries(&packages, 100.0, 2, 70.0, 200.0);

        assert_eq!(results.len(), 5);

        assert_eq!(find(&results, "PKG1").delivery_time_hrs, 3.98);
        assert_eq!(find(&results, "PKG2").delivery_time_hrs, 1.78);
        assert_eq!(find(&results, "PKG3").delivery_time_hrs, 1.42);
        assert_eq!(find(&results, "PKG4").delivery_time_hrs, 0.85);
        assert_eq!(find(&results, "PKG5").delivery_time_hrs, 4.19);
    }

    #[test]
    fn test_schedule_deliveries_sample_2_costs() {
        let packages = vec![
            pkg("PKG1", 50.0, 30.0, "OFR001"),
            pkg("PKG2", 75.0, 125.0, "OFR008"),
            pkg("PKG3", 175.0, 100.0, "OFR003"),
            pkg("PKG4", 110.0, 60.0, "OFR002"),
            pkg("PKG5", 155.0, 95.0, "NA"),
        ];
        let results = schedule_deliveries(&packages, 100.0, 2, 70.0, 200.0);

        let pkg1 = find(&results, "PKG1");
        assert!((pkg1.discount - 0.0).abs() < f64::EPSILON);
        assert!((pkg1.total_cost - 750.0).abs() < f64::EPSILON);

        let pkg2 = find(&results, "PKG2");
        assert!((pkg2.discount - 0.0).abs() < f64::EPSILON);
        assert!((pkg2.total_cost - 1475.0).abs() < f64::EPSILON);

        let pkg3 = find(&results, "PKG3");
        assert!((pkg3.discount - 0.0).abs() < f64::EPSILON);
        assert!((pkg3.total_cost - 2350.0).abs() < f64::EPSILON);

        let pkg4 = find(&results, "PKG4");
        assert!((pkg4.discount - 105.0).abs() < f64::EPSILON);
        assert!((pkg4.total_cost - 1395.0).abs() < f64::EPSILON);

        let pkg5 = find(&results, "PKG5");
        assert!((pkg5.discount - 0.0).abs() < f64::EPSILON);
        assert!((pkg5.total_cost - 2125.0).abs() < f64::EPSILON);
    }

    // --- single-package edge cases ---

    #[test]
    fn test_single_package_single_vehicle_delivery_time() {
        let packages = vec![pkg("PKG1", 5.0, 100.0, "NA")];
        let results = schedule_deliveries(&packages, 100.0, 1, 50.0, 200.0);
        // 100/50 = 2.0
        assert_eq!(results.len(), 1);
        assert_eq!(find(&results, "PKG1").delivery_time_hrs, 2.0);
    }

    #[test]
    fn test_package_exceeding_weight_limit_ships_alone() {
        // A package that is heavier than any other package but still within
        // the per-vehicle limit must not be combined and ships alone.
        let packages = vec![
            pkg("HEAVY", 195.0, 50.0, "NA"),
            pkg("LIGHT", 10.0, 10.0, "NA"),
        ];
        let results = schedule_deliveries(&packages, 100.0, 1, 50.0, 200.0);
        assert_eq!(results.len(), 2);
        // Both packages are eventually delivered.
        assert!(find(&results, "HEAVY").delivery_time_hrs >= 0.0);
        assert!(find(&results, "LIGHT").delivery_time_hrs >= 0.0);
    }

    // New tests to detect combination repetition
    // Suddenly I'm reminded of the exact same leetcode question I did before, which is Combinations!

    #[test]
    fn test_combinations_count_is_c_n_k() {
        // C(3,2) = 3. With i+0 this becomes C(4,2) = 6, which is wrong
        assert_eq!(combinations(&[0, 1, 2], 2).len(), 3);
    }

    #[test]
    fn test_combinations_no_repeated_elements_in_any_combo() {
        // With i+0, [0,0] [1,1] [2,2] would appear — each index used twice.
        for combo in combinations(&[0, 1, 2], 2) {
            let unique: std::collections::HashSet<_> = combo.iter().collect();
            assert_eq!(unique.len(), combo.len(), "duplicate index in combo: {:?}", combo);
        }
    }

    #[test]
    fn test_combinations_with_repetition_would_pick_wrong_shipment() {
        // With i+0, we get a fake combo [PKG_A, PKG_A] weighs 200 < 201 and wins
        let packages = vec![
            pkg("PKG_A", 100.0, 30.0, "NA"),
            pkg("PKG_B", 50.0, 10.0, "NA"),
        ];
        let results = schedule_deliveries(&packages, 0.0, 1, 70.0, 201.0);
        assert_eq!(find(&results, "PKG_B").delivery_time_hrs, 0.14);
    }

    // more edge cases

    #[test]
    #[should_panic(expected = "num_vehicles must be at least 1")]
    fn test_schedule_deliveries_panics_with_zero_vehicles() {
        // 0 vehicles = sad day.
        let packages = vec![pkg("PKG1", 10.0, 10.0, "NA")];
        schedule_deliveries(&packages, 0.0, 0, 70.0, 200.0);
    }

    #[test]
    #[should_panic(expected = "meets or exceeds the vehicle limit")]
    fn test_schedule_deliveries_panics_when_package_equals_max_weight() {
        // A package exactly at the limit can never be loaded (strict <).
        let packages = vec![pkg("PKG1", 200.0, 10.0, "NA")];
        schedule_deliveries(&packages, 0.0, 1, 70.0, 200.0);
    }

    #[test]
    #[should_panic(expected = "meets or exceeds the vehicle limit")]
    fn test_schedule_deliveries_panics_when_package_exceeds_max_weight() {
        // A package heavier than the limit would cause an infinite loop.
        let packages = vec![pkg("PKG1", 250.0, 10.0, "NA")];
        schedule_deliveries(&packages, 0.0, 1, 70.0, 200.0);
    }
}
