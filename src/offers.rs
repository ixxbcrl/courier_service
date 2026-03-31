//! Offer code definitions and discount eligibility logic.
//!
//! An `Offer` describes a named discount that applies when a package's weight
//! and distance both fall within the offer's inclusive ranges. The discount is
//! expressed as a percentage of the raw delivery cost (before any discount).

/// A named promotional offer with eligibility criteria.
#[derive(Debug, Clone, PartialEq)]
pub struct Offer {
    /// Unique offer identifier, e.g. `"OFR001"`.
    pub code: &'static str,
    /// Discount percentage in the range `0.0..=100.0`.
    pub discount_pct: f64,
    /// Inclusive minimum package weight in kg.
    pub min_weight_kg: f64,
    /// Inclusive maximum package weight in kg.
    pub max_weight_kg: f64,
    /// Inclusive minimum delivery distance in km.
    pub min_distance_km: f64,
    /// Inclusive maximum delivery distance in km.
    pub max_distance_km: f64,
}

/// Returns the static offer registry — all known offer codes.
pub fn all_offers() -> &'static [Offer] {
    static OFFERS: &[Offer] = &[
        // 10% discount for packages of weight between 70 and 200 kg,
        Offer {
            code: "OFR001",
            discount_pct: 10.0,
            min_weight_kg: 70.0,
            max_weight_kg: 200.0,
            min_distance_km: 50.0,
            max_distance_km: 150.0,
        },
        // 7% discount for packages of weight between 100 and 250 kg,
        Offer {
            code: "OFR002",
            discount_pct: 7.0,
            min_weight_kg: 100.0,
            max_weight_kg: 250.0,
            min_distance_km: 50.0,
            max_distance_km: 150.0,
        },
        // 5% discount for packages of weight between 10 and 150 kg,
        Offer {
            code: "OFR003",
            discount_pct: 5.0,
            min_weight_kg: 10.0,
            max_weight_kg: 150.0,
            min_distance_km: 50.0,
            max_distance_km: 250.0,
        },
    ];
    OFFERS
}

/// Looks up an offer by its code string (case-sensitive).
pub fn find_offer(code: &str) -> Option<&'static Offer> {
    all_offers().iter().find(|o| o.code == code)
}

/// Computes the discount amount for a given offer and package.
///
/// Returns `raw_cost * discount_pct / 100.0` when both `weight_kg` and
/// `distance_km` are within the offer's inclusive bounds, otherwise `0.0`.
pub fn applicable_discount(
    offer: &Offer,
    weight_kg: f64,
    distance_km: f64,
    raw_cost: f64,
) -> f64 {
    let weight_ok = weight_kg >= offer.min_weight_kg && weight_kg <= offer.max_weight_kg;
    let distance_ok = distance_km >= offer.min_distance_km && distance_km <= offer.max_distance_km;
    if weight_ok && distance_ok {
        raw_cost * offer.discount_pct / 100.0
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_offer_returns_ofr001_for_known_code() {
        let offer = find_offer("OFR001");
        assert!(offer.is_some());
        assert_eq!(offer.unwrap().code, "OFR001");
    }

    #[test]
    fn test_find_offer_returns_ofr002_for_known_code() {
        let offer = find_offer("OFR002");
        assert!(offer.is_some());
        assert_eq!(offer.unwrap().code, "OFR002");
    }

    #[test]
    fn test_find_offer_returns_ofr003_for_known_code() {
        let offer = find_offer("OFR003");
        assert!(offer.is_some());
        assert_eq!(offer.unwrap().code, "OFR003");
    }

    #[test]
    fn test_find_offer_returns_none_for_unknown_code() {
        assert!(find_offer("OFR999").is_none());
    }

    #[test]
    fn test_find_offer_returns_none_for_na_placeholder() {
        // "NA" is the conventional no-offer sentinel used in input files.
        assert!(find_offer("NA").is_none());
    }

    #[test]
    fn test_find_offer_returns_none_for_empty_string() {
        assert!(find_offer("").is_none());
    }

    #[test]
    fn test_find_offer_is_case_sensitive() {
        // Lowercase variant must not match.
        assert!(find_offer("ofr001").is_none());
    }

    #[test]
    fn test_ofr001_applies_when_weight_and_distance_in_range() {
        let offer = find_offer("OFR001").unwrap();
        // weight=100 (in 70–200), distance=100 (in 50–150), raw_cost=1000
        let discount = applicable_discount(offer, 100.0, 100.0, 1000.0);
        assert!((discount - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr001_applies_at_minimum_boundary_weight_and_distance() {
        let offer = find_offer("OFR001").unwrap();
        // Exact lower bounds: weight=70, distance=50
        let discount = applicable_discount(offer, 70.0, 50.0, 1000.0);
        assert!((discount - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr001_applies_at_maximum_boundary_weight_and_distance() {
        let offer = find_offer("OFR001").unwrap();
        // Exact upper bounds: weight=200, distance=150
        let discount = applicable_discount(offer, 200.0, 150.0, 1000.0);
        assert!((discount - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr001_does_not_apply_when_weight_below_minimum() {
        let offer = find_offer("OFR001").unwrap();
        // weight=69 is just below the 70 minimum
        let discount = applicable_discount(offer, 69.0, 100.0, 1000.0);
        assert!((discount - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr001_does_not_apply_when_weight_above_maximum() {
        let offer = find_offer("OFR001").unwrap();
        let discount = applicable_discount(offer, 201.0, 100.0, 1000.0);
        assert!((discount - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr001_does_not_apply_when_distance_below_minimum() {
        let offer = find_offer("OFR001").unwrap();
        // distance=49 is just below the 50 minimum
        let discount = applicable_discount(offer, 100.0, 49.0, 1000.0);
        assert!((discount - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr001_does_not_apply_when_distance_above_maximum() {
        let offer = find_offer("OFR001").unwrap();
        let discount = applicable_discount(offer, 100.0, 151.0, 1000.0);
        assert!((discount - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr001_does_not_apply_to_pkg1_from_sample_1() {
        // PKG1: weight=5 (below 70 minimum) — from Sample 1
        let offer = find_offer("OFR001").unwrap();
        let discount = applicable_discount(offer, 5.0, 5.0, 175.0);
        assert!((discount - 0.0).abs() < f64::EPSILON);
    }

    // --- applicable_discount: OFR002 (7%, weight 100–250, distance 50–150) ---

    #[test]
    fn test_ofr002_does_not_apply_to_pkg2_from_sample_1() {
        // PKG2: weight=15 (below 100 minimum) — from Sample 1
        let offer = find_offer("OFR002").unwrap();
        let discount = applicable_discount(offer, 15.0, 5.0, 275.0);
        assert!((discount - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr002_applies_when_weight_and_distance_in_range() {
        let offer = find_offer("OFR002").unwrap();
        // weight=150, distance=100, raw_cost=1000 → 7% = 70
        let discount = applicable_discount(offer, 150.0, 100.0, 1000.0);
        assert!((discount - 70.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr002_applies_at_exact_lower_boundaries() {
        let offer = find_offer("OFR002").unwrap();
        let discount = applicable_discount(offer, 100.0, 50.0, 1000.0);
        assert!((discount - 70.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr002_applies_at_exact_upper_boundaries() {
        let offer = find_offer("OFR002").unwrap();
        let discount = applicable_discount(offer, 250.0, 150.0, 1000.0);
        assert!((discount - 70.0).abs() < f64::EPSILON);
    }

    // --- applicable_discount: OFR003 (5%, weight 10–150, distance 50–250) ---

    #[test]
    fn test_ofr003_applies_to_pkg3_from_sample_1() {
        // PKG3: weight=10, distance=100, raw_cost=700 → 5% = 35
        let offer = find_offer("OFR003").unwrap();
        let discount = applicable_discount(offer, 10.0, 100.0, 700.0);
        assert!((discount - 35.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr003_applies_at_exact_lower_boundaries() {
        let offer = find_offer("OFR003").unwrap();
        let discount = applicable_discount(offer, 10.0, 50.0, 1000.0);
        assert!((discount - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr003_applies_at_exact_upper_boundaries() {
        let offer = find_offer("OFR003").unwrap();
        let discount = applicable_discount(offer, 150.0, 250.0, 1000.0);
        assert!((discount - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr003_does_not_apply_when_weight_below_minimum() {
        let offer = find_offer("OFR003").unwrap();
        let discount = applicable_discount(offer, 9.0, 100.0, 1000.0);
        assert!((discount - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ofr003_does_not_apply_when_distance_above_maximum() {
        let offer = find_offer("OFR003").unwrap();
        let discount = applicable_discount(offer, 100.0, 251.0, 1000.0);
        assert!((discount - 0.0).abs() < f64::EPSILON);
    }
}
