//! Delivery cost calculation.

use crate::offers::{applicable_discount, find_offer};

/// The computed cost outcome for a single package.
#[derive(Debug, Clone, PartialEq)]
pub struct PackageCostResult {
    /// Package identifier, copied from input.
    pub pkg_id: String,
    /// Absolute discount amount (not a percentage). Zero when no offer applies.
    pub discount: f64,
    /// Final delivery cost after subtracting the discount.
    pub total_cost: f64,
}

/// Calculates the delivery cost for a single package.
///
/// Formula: `base_delivery_cost + (weight_kg * 10) + (distance_km * 5)`.
/// If `offer_code` matches a known offer whose eligibility criteria are met, the
/// discount is subtracted from the raw cost to produce `total_cost`.
pub fn calculate_cost(
    pkg_id: &str,
    weight_kg: f64,
    distance_km: f64,
    offer_code: &str,
    base_delivery_cost: f64,
) -> PackageCostResult {
    let raw_cost = base_delivery_cost + (weight_kg * 10.0) + (distance_km * 5.0);
    let discount = find_offer(offer_code)
        .map(|offer| applicable_discount(offer, weight_kg, distance_km, raw_cost))
        .unwrap_or(0.0);
    PackageCostResult {
        pkg_id: pkg_id.to_string(),
        discount,
        total_cost: raw_cost - discount,
    }
}

/// Calculates costs for a batch of packages and returns results in input order.
///
/// Each tuple is `(pkg_id, weight_kg, distance_km, offer_code)`.
pub fn calculate_costs(
    packages: &[(&str, f64, f64, &str)],
    base_delivery_cost: f64,
) -> Vec<PackageCostResult> {
    packages
        .iter()
        .map(|&(pkg_id, weight_kg, distance_km, offer_code)| {
            calculate_cost(pkg_id, weight_kg, distance_km, offer_code, base_delivery_cost)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_formula_no_offer_simple_values() {
        // raw = 100 + (5*10) + (5*5) = 100 + 50 + 25 = 175
        let result = calculate_cost("PKG1", 5.0, 5.0, "OFR001", 100.0);
        assert_eq!(result.pkg_id, "PKG1");
        assert!((result.discount - 0.0).abs() < f64::EPSILON);
        assert!((result.total_cost - 175.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_formula_no_offer_pkg2_from_sample_1() {
        // raw = 100 + (15*10) + (5*5) = 100 + 150 + 25 = 275
        let result = calculate_cost("PKG2", 15.0, 5.0, "OFR002", 100.0);
        assert_eq!(result.pkg_id, "PKG2");
        assert!((result.discount - 0.0).abs() < f64::EPSILON);
        assert!((result.total_cost - 275.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_formula_with_applicable_offer_pkg3_from_sample_1() {
        // raw = 100 + (10*10) + (100*5) = 100 + 100 + 500 = 700
        // OFR003 applies (weight=10 in 10–150, distance=100 in 50–250)
        // discount = 5% of 700 = 35; total = 665
        let result = calculate_cost("PKG3", 10.0, 100.0, "OFR003", 100.0);
        assert_eq!(result.pkg_id, "PKG3");
        assert!((result.discount - 35.0).abs() < f64::EPSILON);
        assert!((result.total_cost - 665.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_with_na_offer_code_gives_zero_discount() {
        let result = calculate_cost("PKG_X", 50.0, 50.0, "NA", 100.0);
        assert!((result.discount - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_with_empty_offer_code_gives_zero_discount() {
        let result = calculate_cost("PKG_X", 50.0, 50.0, "", 100.0);
        assert!((result.discount - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_with_unknown_offer_code_gives_zero_discount() {
        let result = calculate_cost("PKG_X", 100.0, 100.0, "OFR999", 100.0);
        assert!((result.discount - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_with_valid_offer_code_but_criteria_not_met_gives_zero_discount() {
        // OFR001 requires weight 70–200, but we pass weight=5
        let result = calculate_cost("PKG_X", 5.0, 100.0, "OFR001", 100.0);
        assert!((result.discount - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_base_delivery_cost_is_included_in_total() {
        // With zero weight and zero distance the total equals the base cost.
        let result = calculate_cost("PKG_X", 0.0, 0.0, "NA", 250.0);
        assert!((result.total_cost - 250.0).abs() < f64::EPSILON);
    }

    // --- OFR002 applied from Sample 2 ---

    #[test]
    fn test_ofr002_applied_to_pkg4_from_sample_2() {
        // PKG4: weight=110, distance=60
        // raw = 100 + (110*10) + (60*5) = 100 + 1100 + 300 = 1500
        // OFR002: weight 100–250 ✓, distance 50–150 ✓ → 7% of 1500 = 105
        // total = 1395
        let result = calculate_cost("PKG4", 110.0, 60.0, "OFR002", 100.0);
        assert!((result.discount - 105.0).abs() < f64::EPSILON);
        assert!((result.total_cost - 1395.0).abs() < f64::EPSILON);
    }

    // --- Batch calculation ---

    #[test]
    fn test_calculate_costs_returns_results_in_input_order() {
        let packages = vec![
            ("PKG1", 5.0, 5.0, "OFR001"),
            ("PKG2", 15.0, 5.0, "OFR002"),
            ("PKG3", 10.0, 100.0, "OFR003"),
        ];
        let results = calculate_costs(&packages, 100.0);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].pkg_id, "PKG1");
        assert_eq!(results[1].pkg_id, "PKG2");
        assert_eq!(results[2].pkg_id, "PKG3");
    }

    #[test]
    fn test_calculate_costs_sample_1_full() {
        // Full Sample 1 validation
        let packages = vec![
            ("PKG1", 5.0, 5.0, "OFR001"),
            ("PKG2", 15.0, 5.0, "OFR002"),
            ("PKG3", 10.0, 100.0, "OFR003"),
        ];
        let results = calculate_costs(&packages, 100.0);

        // PKG1: discount=0, total=175
        assert!((results[0].discount - 0.0).abs() < f64::EPSILON);
        assert!((results[0].total_cost - 175.0).abs() < f64::EPSILON);

        // PKG2: discount=0, total=275
        assert!((results[1].discount - 0.0).abs() < f64::EPSILON);
        assert!((results[1].total_cost - 275.0).abs() < f64::EPSILON);

        // PKG3: discount=35, total=665
        assert!((results[2].discount - 35.0).abs() < f64::EPSILON);
        assert!((results[2].total_cost - 665.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_costs_sample_2_cost_portion() {
        // Cost figures only from Sample 2 (delivery times tested in scheduler)
        let packages = vec![
            ("PKG1", 50.0, 30.0, "OFR001"),
            ("PKG2", 75.0, 125.0, "OFR008"),
            ("PKG3", 175.0, 100.0, "OFR003"),
            ("PKG4", 110.0, 60.0, "OFR002"),
            ("PKG5", 155.0, 95.0, "NA"),
        ];
        let results = calculate_costs(&packages, 100.0);

        // PKG1: raw=100+500+150=750; OFR001 weight=50 not in 70-200 → 0 discount
        assert!((results[0].discount - 0.0).abs() < f64::EPSILON);
        assert!((results[0].total_cost - 750.0).abs() < f64::EPSILON);

        // PKG2: raw=100+750+625=1475; OFR008 unknown → 0 discount
        assert!((results[1].discount - 0.0).abs() < f64::EPSILON);
        assert!((results[1].total_cost - 1475.0).abs() < f64::EPSILON);

        // PKG3: raw=100+1750+500=2350; OFR003 weight=175 > 150 max → 0 discount
        assert!((results[2].discount - 0.0).abs() < f64::EPSILON);
        assert!((results[2].total_cost - 2350.0).abs() < f64::EPSILON);

        // PKG4: raw=100+1100+300=1500; OFR002 weight=110 in 100-250, dist=60 in 50-150 → 7%=105
        assert!((results[3].discount - 105.0).abs() < f64::EPSILON);
        assert!((results[3].total_cost - 1395.0).abs() < f64::EPSILON);

        // PKG5: raw=100+1550+475=2125; NA → 0 discount
        assert!((results[4].discount - 0.0).abs() < f64::EPSILON);
        assert!((results[4].total_cost - 2125.0).abs() < f64::EPSILON);
    }
}
