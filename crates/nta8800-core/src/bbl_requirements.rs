//! BENG requirements of the Besluit bouwwerken leefomgeving, article 4.149
//! with table 4.148A (consolidated version 2026-01-01, BWBR0041297,
//! retrieved 2026-10-01). Every row was checked on 2026-10-05 against the
//! official XML of the versions 2026-01-01 and 2026-09-24; table 4.148A and
//! articles 4.149–4.149b are unchanged between them.
//!
//! Limits: energy need (BENG 1) depending on `A_ls/A_g`, primary fossil
//! energy (BENG 2) and minimum renewable share (BENG 3). Paragraph 4 raises
//! the energy-need limit by 5 kWh/m²·yr when the area-weighted specific
//! internal heat capacity is at most 180 kJ/m²K, for the rows where table
//! 4.148A marks paragraph 4 as applicable. Paragraph 2: for several use
//! functions of different kinds the limits of each function are weighted by
//! usable floor area ([`check_mixed`]); paragraph 3 (an accessory function
//! of a dwelling takes the dwelling limits) is an input choice.

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

/// One use function with its usable floor area for paragraph 2.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BblFunctionArea {
    pub function: BblFunction,
    pub area_m2: f64,
}

/// Paragraph 2: limits weighted by usable floor area; paragraph 4 applies
/// per function. `None` for an empty list, a non-positive area or ratio.
pub fn bbl_limits_mixed(
    functions: &[BblFunctionArea],
    loss_area_ratio: f64,
    specific_heat_capacity_kj_per_m2k: f64,
) -> Option<BblLimits> {
    let total: f64 = functions.iter().map(|part| part.area_m2).sum();
    if functions.is_empty()
        || !total.is_finite()
        || total <= 0.0
        || functions
            .iter()
            .any(|part| !part.area_m2.is_finite() || part.area_m2 <= 0.0)
    {
        return None;
    }
    let mut weighted = BblLimits {
        energy_need_max_kwh_per_m2: 0.0,
        primary_fossil_max_kwh_per_m2: 0.0,
        renewable_share_min_percent: 0.0,
        light_construction_allowance_applied: false,
    };
    for part in functions {
        let share = part.area_m2 / total;
        let limits = bbl_limits(
            part.function,
            loss_area_ratio,
            specific_heat_capacity_kj_per_m2k,
        )?;
        weighted.energy_need_max_kwh_per_m2 += share * limits.energy_need_max_kwh_per_m2;
        weighted.primary_fossil_max_kwh_per_m2 += share * limits.primary_fossil_max_kwh_per_m2;
        weighted.renewable_share_min_percent += share * limits.renewable_share_min_percent;
        weighted.light_construction_allowance_applied |=
            limits.light_construction_allowance_applied;
    }
    Some(weighted)
}

pub const A0_SOURCE: &str = "Omgevingsregeling art. 5.11/5.12 lid 5, bijlagen IXa/Xa (Stcrt. 2026, 18123; in werking mei 2026)";

/// Annexes IXa/Xa: maximum primary fossil energy for the A0 designation.
pub fn a0_primary_fossil_max(function: BblFunction) -> f64 {
    use BblFunction::*;
    match function {
        ResidentialBuilding => 45.0,
        Caravan => 54.0,
        FloatingBuildingAfter2018Berth => 45.0,
        FloatingBuildingOtherBerth => 63.0,
        OtherResidential => 27.0,
        OtherLodging => 36.0,
        AssemblyChildCare => 63.0,
        OtherAssembly => 54.0,
        Cell => 108.0,
        HealthcareWithBeds => 117.0,
        OtherHealthcare => 45.0,
        Office => 36.0,
        LodgingInLodgingBuilding => 117.0,
        Education => 64.0,
        Sport => 81.0,
        Retail => 54.0,
    }
}

/// A0 designation for emission-free new buildings with a permit application
/// after 29 May 2026 (Omgevingsregeling art. 5.11/5.12 lid 5).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct A0Check {
    pub source: &'static str,
    pub primary_fossil_max_kwh_per_m2: f64,
    /// a: energy need within the table 4.148A limit.
    pub energy_need_meets: Option<bool>,
    /// b: primary fossil energy within annex IXa/Xa.
    pub primary_fossil_meets: Option<bool>,
    /// c: renewable share at least the table 4.148A value.
    pub renewable_share_meets: Option<bool>,
    /// d: no on-site carbon emissions from fossil fuels.
    pub no_on_site_fossil_combustion: bool,
    /// All four conditions; `None` when one of them cannot be checked.
    pub eligible: Option<bool>,
}

pub fn a0_check(check: &BblCheck, beng2: Option<f64>, on_site_fossil_use: bool) -> A0Check {
    // Mixed functions: area-weighted like article 4.149 paragraph 2
    // (interpretation; the Omgevingsregeling gives no mixed rule).
    let total: f64 = check.functions.iter().map(|part| part.area_m2).sum();
    let max = if check.functions.len() > 1 && total > 0.0 {
        check
            .functions
            .iter()
            .map(|part| part.area_m2 / total * a0_primary_fossil_max(part.function))
            .sum()
    } else {
        a0_primary_fossil_max(check.function)
    };
    let fossil = beng2.map(|value| value <= max);
    let parts = [check.energy_need_meets, fossil, check.renewable_share_meets];
    let eligible = if on_site_fossil_use || parts.contains(&Some(false)) {
        Some(false)
    } else if parts.iter().all(Option::is_some) {
        Some(true)
    } else {
        None
    };
    A0Check {
        source: A0_SOURCE,
        primary_fossil_max_kwh_per_m2: max,
        energy_need_meets: check.energy_need_meets,
        primary_fossil_meets: fossil,
        renewable_share_meets: check.renewable_share_meets,
        no_on_site_fossil_combustion: !on_site_fossil_use,
        eligible,
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BblCheck {
    pub source: &'static str,
    /// The function, or the largest one for mixed functions.
    pub function: BblFunction,
    /// Paragraph 2 functions with their areas; one entry for a single
    /// function.
    pub functions: Vec<BblFunctionArea>,
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
    check_mixed(
        &[BblFunctionArea {
            function,
            area_m2: 1.0,
        }],
        loss_area_ratio,
        specific_heat_capacity_kj_per_m2k,
        beng1,
        beng2,
        beng3,
    )
}

/// Article 4.149 paragraph 2 for one or more use functions.
pub fn check_mixed(
    functions: &[BblFunctionArea],
    loss_area_ratio: f64,
    specific_heat_capacity_kj_per_m2k: f64,
    beng1: Option<f64>,
    beng2: Option<f64>,
    beng3: Option<f64>,
) -> Option<BblCheck> {
    let limits = bbl_limits_mixed(
        functions,
        loss_area_ratio,
        specific_heat_capacity_kj_per_m2k,
    )?;
    let function = functions
        .iter()
        .fold(None::<&BblFunctionArea>, |best, part| match best {
            Some(best) if best.area_m2 >= part.area_m2 => Some(best),
            _ => Some(part),
        })?
        .function;
    Some(BblCheck {
        source: BBL_SOURCE,
        function,
        functions: functions.to_vec(),
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
    fn mixed_functions_weight_the_limits_by_area() {
        let parts = [
            BblFunctionArea {
                function: BblFunction::Office,
                area_m2: 750.0,
            },
            BblFunctionArea {
                function: BblFunction::Retail,
                area_m2: 250.0,
            },
        ];
        let mixed = check_mixed(&parts, 1.5, 250.0, Some(80.0), Some(45.0), Some(30.0)).unwrap();
        // x ≤ 1,8: office 90 / 40 / 30, retail 70 / 60 / 30.
        assert!((mixed.limits.energy_need_max_kwh_per_m2 - 85.0).abs() < 1e-12);
        assert!((mixed.limits.primary_fossil_max_kwh_per_m2 - 45.0).abs() < 1e-12);
        assert!((mixed.limits.renewable_share_min_percent - 30.0).abs() < 1e-12);
        assert_eq!(mixed.function, BblFunction::Office);
        assert_eq!(mixed.primary_fossil_meets, Some(true));
        // A0: 0,75·36 + 0,25·54 = 40,5.
        let a0 = a0_check(&mixed, Some(45.0), false);
        assert!((a0.primary_fossil_max_kwh_per_m2 - 40.5).abs() < 1e-12);
        assert!(check_mixed(&[], 1.5, 250.0, None, None, None).is_none());
    }

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
    fn education_limits_follow_table_4_148a_at_shape_boundary() {
        // Bbl table 4.148A, education row: 190 through A_ls/A_g = 1.8;
        // above that, 190 + 30 × (A_ls/A_g − 1.8). The same row gives
        // BENG 2 = 70 and BENG 3 = 40 (IPLO 2026 working text, p. 150).
        let at_boundary = bbl_limits(BblFunction::Education, 1.8, 250.0).unwrap();
        assert_eq!(at_boundary.energy_need_max_kwh_per_m2, 190.0);
        assert_eq!(at_boundary.primary_fossil_max_kwh_per_m2, 70.0);
        assert_eq!(at_boundary.renewable_share_min_percent, 40.0);
        assert_eq!(
            bbl_limits(BblFunction::Education, 2.0, 250.0)
                .unwrap()
                .energy_need_max_kwh_per_m2,
            196.0
        );
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

    #[test]
    fn a0_requires_all_four_conditions() {
        let base = check(
            BblFunction::OtherResidential,
            2.0,
            250.0,
            Some(60.0),
            Some(20.0),
            Some(80.0),
        )
        .unwrap();
        let a0 = a0_check(&base, Some(20.0), false);
        assert_eq!(a0.primary_fossil_max_kwh_per_m2, 27.0);
        assert_eq!(a0.eligible, Some(true));
        assert_eq!(a0_check(&base, Some(27.01), false).eligible, Some(false));
        assert_eq!(a0_check(&base, Some(20.0), true).eligible, Some(false));
        let no_need = check(
            BblFunction::OtherResidential,
            2.0,
            250.0,
            None,
            Some(20.0),
            Some(80.0),
        )
        .unwrap();
        assert_eq!(a0_check(&no_need, Some(20.0), false).eligible, None);
        assert_eq!(a0_primary_fossil_max(BblFunction::Education), 64.0);
        assert_eq!(
            a0_primary_fossil_max(BblFunction::HealthcareWithBeds),
            117.0
        );
    }
}
