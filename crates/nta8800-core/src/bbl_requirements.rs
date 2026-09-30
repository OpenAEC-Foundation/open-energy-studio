//! BENG requirements of the Besluit bouwwerken leefomgeving, article 4.149
//! with table 4.148A (consolidated version 2026-01-01, BWBR0041297,
//! retrieved 2026-10-01).
//!
//! Limits: energy need (BENG 1) depending on `A_ls/A_g`, primary fossil
//! energy (BENG 2) and minimum renewable share (BENG 3). Paragraph 4 raises
//! the energy-need limit by 5 kWh/m²·yr when the area-weighted specific
//! internal heat capacity is at most 180 kJ/m²K, for the rows where table
//! 4.148A marks paragraph 4 as applicable. Mixed-function weighting
//! (paragraph 2) is not implemented.

use serde::{Deserialize, Serialize};

pub const BBL_SOURCE: &str = "Bbl art. 4.149 met tabel 4.148A (BWBR0041297, versie 2026-01-01)";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BblFunction {
    /// 1a
    ResidentialBuilding,
    /// 1b
    Caravan,
    /// 1c
    FloatingBuildingAfter2018Berth,
    /// 1d
    FloatingBuildingOtherBerth,
    /// 1e, including ground-bound dwellings
    OtherResidential,
    /// 2a
    AssemblyChildCare,
    /// 2b
    OtherAssembly,
    /// 3
    Cell,
    /// 4a
    HealthcareWithBeds,
    /// 4b
    OtherHealthcare,
    /// 6
    Office,
    /// 7a
    LodgingInLodgingBuilding,
    /// 7b
    OtherLodging,
    /// 8
    Education,
    /// 9
    Sport,
    /// 10
    Retail,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BblLimits {
    pub energy_need_max_kwh_per_m2: f64,
    pub primary_fossil_max_kwh_per_m2: f64,
    pub renewable_share_min_percent: f64,
    pub light_construction_allowance_applied: bool,
}

/// Energy-need limit of table 4.148A for `x = A_ls/A_g`.
fn energy_need_limit(function: BblFunction, x: f64) -> f64 {
    use BblFunction::*;
    let dwelling = |low: f64| {
        // Footnotes (4)/(5)/(3) for 1e and 7b; (1)/(2)/(3) for 1a.
        if function == ResidentialBuilding {
            if x <= 1.83 {
                return 65.0;
            }
        } else if x <= 1.5 {
            return low;
        }
        if x <= 3.0 {
            55.0 + 30.0 * (x - 1.5)
        } else {
            100.0 + 50.0 * (x - 3.0)
        }
    };
    // Footnotes (6)/(7) for utility functions.
    let utility = |base: f64, slope: f64| {
        if x <= 1.8 {
            base
        } else {
            base + slope * (x - 1.8)
        }
    };
    match function {
        ResidentialBuilding => dwelling(65.0),
        OtherResidential | OtherLodging => dwelling(55.0),
        Caravan => 100.0 + 30.0 * (x - 2.0),
        FloatingBuildingAfter2018Berth | FloatingBuildingOtherBerth => 80.0 + 30.0 * (x - 1.5),
        AssemblyChildCare => utility(160.0, 30.0),
        OtherAssembly => utility(90.0, 30.0),
        Cell => utility(160.0, 35.0),
        HealthcareWithBeds => 350.0,
        OtherHealthcare => utility(90.0, 35.0),
        Office => utility(90.0, 30.0),
        LodgingInLodgingBuilding => utility(100.0, 35.0),
        Education => utility(190.0, 30.0),
        Sport => utility(40.0, 15.0),
        Retail => utility(70.0, 30.0),
    }
}

fn fixed_limits(function: BblFunction) -> (f64, f64, bool) {
    use BblFunction::*;
    // (primary fossil max, renewable min, paragraph 4 applicable)
    match function {
        ResidentialBuilding => (50.0, 40.0, true),
        Caravan => (60.0, 50.0, false),
        FloatingBuildingAfter2018Berth => (50.0, 50.0, false),
        FloatingBuildingOtherBerth => (70.0, 50.0, false),
        OtherResidential => (30.0, 50.0, true),
        AssemblyChildCare => (70.0, 40.0, false),
        OtherAssembly => (60.0, 30.0, false),
        Cell => (120.0, 30.0, false),
        HealthcareWithBeds => (130.0, 30.0, false),
        OtherHealthcare => (50.0, 40.0, false),
        Office => (40.0, 30.0, false),
        LodgingInLodgingBuilding => (130.0, 40.0, false),
        OtherLodging => (40.0, 50.0, true),
        Education => (70.0, 40.0, false),
        Sport => (90.0, 30.0, false),
        Retail => (60.0, 30.0, false),
    }
}

/// Limits for one use function; `None` for a non-positive or non-finite ratio.
pub fn bbl_limits(
    function: BblFunction,
    loss_area_ratio: f64,
    specific_heat_capacity_kj_per_m2k: f64,
) -> Option<BblLimits> {
    if !loss_area_ratio.is_finite() || loss_area_ratio <= 0.0 {
        return None;
    }
    let (fossil, renewable, paragraph_4) = fixed_limits(function);
    let allowance = paragraph_4 && specific_heat_capacity_kj_per_m2k <= 180.0;
    Some(BblLimits {
        energy_need_max_kwh_per_m2: energy_need_limit(function, loss_area_ratio)
            + if allowance { 5.0 } else { 0.0 },
        primary_fossil_max_kwh_per_m2: fossil,
        renewable_share_min_percent: renewable,
        light_construction_allowance_applied: allowance,
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BblCheck {
    pub source: &'static str,
    pub function: BblFunction,
    pub loss_area_ratio: f64,
    pub limits: BblLimits,
    /// `None` when the indicator is not available (BENG 1 without C1).
    pub energy_need_meets: Option<bool>,
    pub primary_fossil_meets: Option<bool>,
    pub renewable_share_meets: Option<bool>,
}

pub fn check(
    function: BblFunction,
    loss_area_ratio: f64,
    specific_heat_capacity_kj_per_m2k: f64,
    beng1: Option<f64>,
    beng2: Option<f64>,
    beng3: Option<f64>,
) -> Option<BblCheck> {
    let limits = bbl_limits(function, loss_area_ratio, specific_heat_capacity_kj_per_m2k)?;
    Some(BblCheck {
        source: BBL_SOURCE,
        function,
        loss_area_ratio,
        energy_need_meets: beng1.map(|value| value <= limits.energy_need_max_kwh_per_m2),
        primary_fossil_meets: beng2.map(|value| value <= limits.primary_fossil_max_kwh_per_m2),
        renewable_share_meets: beng3.map(|value| value >= limits.renewable_share_min_percent),
        limits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dwelling_limits_follow_table_4_148a() {
        let limit = |x, d_m| bbl_limits(BblFunction::OtherResidential, x, d_m).unwrap();
        assert_eq!(limit(1.2, 250.0).energy_need_max_kwh_per_m2, 55.0);
        assert!((limit(2.0, 250.0).energy_need_max_kwh_per_m2 - 70.0).abs() < 1e-12);
        assert!((limit(3.5, 250.0).energy_need_max_kwh_per_m2 - 125.0).abs() < 1e-12);
        // Paragraph 4: light construction (D_m ≤ 180) adds 5.
        let light = limit(2.0, 180.0);
        assert!((light.energy_need_max_kwh_per_m2 - 75.0).abs() < 1e-12);
        assert!(light.light_construction_allowance_applied);
        assert_eq!(light.primary_fossil_max_kwh_per_m2, 30.0);
        assert_eq!(light.renewable_share_min_percent, 50.0);
        let building = bbl_limits(BblFunction::ResidentialBuilding, 1.83, 250.0).unwrap();
        assert_eq!(building.energy_need_max_kwh_per_m2, 65.0);
        assert!(
            (bbl_limits(BblFunction::ResidentialBuilding, 2.0, 250.0)
                .unwrap()
                .energy_need_max_kwh_per_m2
                - 70.0)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn utility_limits_and_paragraph_4_scope() {
        let office = bbl_limits(BblFunction::Office, 2.3, 100.0).unwrap();
        assert!((office.energy_need_max_kwh_per_m2 - 105.0).abs() < 1e-12);
        assert!(!office.light_construction_allowance_applied);
        assert_eq!(office.primary_fossil_max_kwh_per_m2, 40.0);
        let beds = bbl_limits(BblFunction::HealthcareWithBeds, 5.0, 100.0).unwrap();
        assert_eq!(beds.energy_need_max_kwh_per_m2, 350.0);
        assert!(bbl_limits(BblFunction::Office, 0.0, 100.0).is_none());
    }

    #[test]
    fn check_reports_only_available_indicators() {
        let result = check(
            BblFunction::OtherResidential,
            2.0,
            250.0,
            None,
            Some(25.0),
            Some(49.9),
        )
        .unwrap();
        assert_eq!(result.energy_need_meets, None);
        assert_eq!(result.primary_fossil_meets, Some(true));
        assert_eq!(result.renewable_share_meets, Some(false));
    }
}
