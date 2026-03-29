//! Vehicle scheduling and delivery time estimation.
//!
//! Thinking of using a greedy shipment strategy for now.


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

/// Truncates a floating-point value to exactly two decimal places.
pub fn truncate_to_2dp(value: f64) -> f64 {
    todo!()
}

/// Schedules deliveries for a fleet of identical vehicles and returns one
/// `PackageDeliveryResult` per package, in the same order as `packages`.
pub fn schedule_deliveries(
    packages: &[PackageInput],
    base_delivery_cost: f64,
    num_vehicles: usize,
    max_speed_kmhr: f64,
    max_weight_kg: f64,
) -> Vec<PackageDeliveryResult> {
    todo!()
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
        // Single vehicle, two sequential trips.
        // Trip 1: one package at 70km → delivery=1.0hr; return at 2.0hrs.
        // Trip 2: one package at 35km → delivery = 2.0 + 0.5 = 2.5hrs.
        let packages = vec![
            pkg("PKG_A", 100.0, 70.0, "NA"),
            pkg("PKG_B", 100.0, 35.0, "NA"),
        ];
        let results = schedule_deliveries(&packages, 100.0, 1, 70.0, 200.0);
        // PKG_A: 70/70 = 1.0
        assert_eq!(find(&results, "PKG_A").delivery_time_hrs, 1.0);
        // PKG_B: vehicle returns after 2*(70/70)=2.0hrs; 2.0 + 35/70 = 2.5
        assert_eq!(find(&results, "PKG_B").delivery_time_hrs, 2.5);
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
}
