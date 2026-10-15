//! Review 72: edition switch points that the two example projects of
//! review 71 do not reach, checked against the text of each edition.
//!
//! The expected values were read from the edition PDFs (page numbers in the
//! comments; no norm text is reproduced) and computed by hand, independently
//! of the kernel's own tables. Each test runs the kernel in the edition where
//! the switch point differs and in the edition next to it.

use nta8800_core::domestic_hot_water::{kitchen_emission_for, KitchenPipeDiameter};
use nta8800_core::forfait_envelope::{
    forfait_panel_u_in_edition, forfait_psi, PanelInsulation, PsiColumn,
};
use nta8800_core::heating_distribution::{PipeInsulation, PipeTransmittance};
use nta8800_core::materials::{
    Ageing, Conductivity, ForfaitMaterial, InSituProduct, InSituSituation, InsulationMoisture,
};
use nta8800_core::norm_versions::{with_version, NormVersion};
use nta8800_core::space_heating_chain::heat_pump_forfait_auxiliary_kwh;
use nta8800_core::window_u::FrameGroup;

const V2020: NormVersion = NormVersion::V2020A1;
const V2022: NormVersion = NormVersion::V2022;
const V2023: NormVersion = NormVersion::V2023;
const V2024: NormVersion = NormVersion::V2024;
const V2025: NormVersion = NormVersion::V2025C1;

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-9 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}

/// Switch point 6: table 9.16, uninsulated pipes of a collective
/// installation for heating and hot water. 2024 p. 292: 2,0 for any area;
/// 2025+C1 p. 310: 1,0 up to 500 m², 2,0 above. Insulated: 0,4 in both.
#[test]
fn switch_6_table_9_16_combined_collective_pipes() {
    let uninsulated = PipeTransmittance::Forfait {
        insulation: PipeInsulation::Uninsulated,
    };
    let psi = |version, area| with_version(version, || uninsulated.value(area, true).unwrap());
    close(psi(V2024, 300.0), 2.0);
    close(psi(V2024, 800.0), 2.0);
    close(psi(V2025, 300.0), 1.0);
    close(psi(V2025, 500.0), 1.0);
    close(psi(V2025, 800.0), 2.0);
}

/// Switch points 15 and 60: table E.5 F_A;iso of in-situ products, with
/// F_A;appl 1,00 (situation A). Mineral-wool flakes 1,05 in 2020+A1 (p. 768)
/// and 2022 (p. 775), 1,00 from 2023 (p. 785). Cellulose and sprayed flax
/// fall under "overig" 1,30 up to 2024 (p. 783) and under the fibre row
/// 1,00 in 2025+C1 (p. 803).
#[test]
fn switches_15_and_60_table_e_5_ageing() {
    let factor = |version, material: ForfaitMaterial| {
        with_version(version, || {
            let product = material.in_situ_product().unwrap();
            Ageing::InSitu {
                product,
                situation: InSituSituation::A,
                practice_tested: false,
            }
            .factor()
        })
    };
    close(factor(V2020, ForfaitMaterial::MineralWoolFlakes), 1.05);
    close(factor(V2022, ForfaitMaterial::MineralWoolFlakes), 1.05);
    close(factor(V2023, ForfaitMaterial::MineralWoolFlakes), 1.00);
    close(factor(V2024, ForfaitMaterial::CelluloseLoose), 1.30);
    close(factor(V2024, ForfaitMaterial::FlaxSprayed), 1.30);
    close(factor(V2025, ForfaitMaterial::CelluloseLoose), 1.00);
    close(factor(V2025, ForfaitMaterial::FlaxSprayed), 1.00);
    with_version(V2024, || {
        assert_eq!(
            ForfaitMaterial::CelluloseLoose.in_situ_product(),
            Some(InSituProduct::Other)
        )
    });
}

/// Switch points 16 and 27: tables E.10–E.12 λ_for. Wood fibre and
/// cellulose 0,050 in 2023 (p. 791), 0,045 from 2024 (p. 790). Bio-based
/// materials up to 2024 (p. 791): flax, sheep wool and cotton 0,050,
/// coconut 0,055, straw 0,060, hemp and peat 0,100, reed 0,200 (table E.12);
/// 2025+C1 (p. 810): flax boards 0,045, sprayed flax 0,050, sheep wool and
/// cotton 0,045, coconut 0,050, straw 0,055, hemp boards 0,045, other hemp
/// 0,060, peat 0,100, miscanthus 0,060, grass wool 0,045, reed 0,100.
#[test]
fn switches_16_and_27_tables_e_10_to_e_12() {
    use ForfaitMaterial::*;
    let lambda = |version, material: ForfaitMaterial| with_version(version, || material.lambda());
    close(lambda(V2023, WoodFibre), 0.050);
    close(lambda(V2023, CelluloseLoose), 0.050);
    close(lambda(V2024, WoodFibre), 0.045);
    close(lambda(V2024, CelluloseLoose), 0.045);
    for (material, old, new) in [
        (FlaxBoard, 0.050, 0.045),
        (FlaxSprayed, 0.050, 0.050),
        (SheepWool, 0.050, 0.045),
        (Cotton, 0.050, 0.045),
        (Coconut, 0.055, 0.050),
        (Straw, 0.060, 0.055),
        (HempBoard, 0.100, 0.045),
        (HempOther, 0.100, 0.060),
        (Peat, 0.100, 0.100),
        (Reed, 0.200, 0.100),
    ] {
        for version in [V2020, V2022, V2023, V2024] {
            close(lambda(version, material), old);
        }
        close(lambda(V2025, material), new);
    }
    close(lambda(V2025, Miscanthus), 0.060);
    close(lambda(V2025, GrassWool), 0.045);
}

/// Miscanthus and grass wool have no row in table E.11 before 2025+C1
/// (2020+A1 p. 775, 2022 p. 782, 2023 p. 792, 2024 p. 791), so a forfait
/// λ for them is refused in those editions instead of taking the 2025+C1
/// value.
#[test]
fn miscanthus_and_grass_wool_are_2025_rows() {
    for material in [ForfaitMaterial::Miscanthus, ForfaitMaterial::GrassWool] {
        let input = Conductivity::ForfaitInsulation {
            material,
            moisture: InsulationMoisture::Regular,
            ageing: Ageing::FactoryMade,
        };
        for version in [V2020, V2022, V2023, V2024] {
            let issues = with_version(version, || input.validate());
            assert!(
                issues
                    .iter()
                    .any(|issue| issue.code == "route_not_in_edition" && issue.field == "material"),
                "{material:?} in {version:?}: {issues:?}"
            );
        }
        assert!(with_version(V2025, || input.validate()).is_empty());
    }
}

/// Switch point 19: table 13.2 of 2023 (p. 533) with kitchen rows for an
/// inner diameter of at most 8 or 10 mm; 2024 (p. 527) has only "overig".
#[test]
fn switch_19_table_13_2_kitchen_rows() {
    let rows = [
        (
            Some(KitchenPipeDiameter::UpTo8Mm),
            [1.00, 0.86, 0.75, 0.67, 0.60, 0.55, 0.50, 0.46],
        ),
        (
            Some(KitchenPipeDiameter::UpTo10Mm),
            [1.00, 0.79, 0.65, 0.55, 0.48, 0.43, 0.38, 0.35],
        ),
        (None, [1.00, 0.69, 0.53, 0.43, 0.36, 0.31, 0.27, 0.24]),
    ];
    let other = [1.00, 0.69, 0.53, 0.43, 0.36, 0.31, 0.27, 0.24];
    for (diameter, values) in rows {
        for (band, expected) in values.iter().enumerate() {
            let length = band as f64 * 2.0 + 1.0;
            close(
                with_version(V2023, || kitchen_emission_for(length, diameter)),
                *expected,
            );
            close(
                with_version(V2024, || kitchen_emission_for(length, diameter)),
                other[band],
            );
        }
    }
}

/// Switch points 26 and 76: tables I.1/I.2. Detail 17 is 0,60/0,90 in
/// 2022 (p. 793) and 2023 (p. 803), 0,06/0,09 from 2024 (p. 802). 2020+A1
/// (p. 786–787) has one column: detail 14 is 0,70, detail 17 0,60, the other
/// details equal column A of 2022, and there is no column B.
#[test]
fn switches_26_and_76_tables_i_1_and_i_2() {
    let psi =
        |version, position, column| with_version(version, || forfait_psi(position, 0, column));
    assert_eq!(psi(V2023, 17, PsiColumn::A), Some(0.60));
    assert_eq!(psi(V2023, 17, PsiColumn::B), Some(0.90));
    assert_eq!(psi(V2024, 17, PsiColumn::A), Some(0.06));
    assert_eq!(psi(V2024, 17, PsiColumn::B), Some(0.09));
    assert_eq!(psi(V2022, 14, PsiColumn::A), Some(0.03));
    assert_eq!(psi(V2022, 14, PsiColumn::B), Some(0.13));
    assert_eq!(psi(V2020, 14, PsiColumn::A), Some(0.70));
    assert_eq!(psi(V2020, 17, PsiColumn::A), Some(0.60));
    assert_eq!(psi(V2020, 1, PsiColumn::B), None);
    // A sample of the single 2020+A1 column (p. 786–787).
    for (position, value) in [
        (1, 0.27),
        (3, 0.60),
        (13, 0.16),
        (18, 0.50),
        (23, 0.24),
        (50, 0.61),
        (62, 0.39),
        (67, 0.78),
        (72, 0.44),
    ] {
        assert_eq!(
            psi(V2020, position, PsiColumn::A),
            Some(value),
            "detail {position}"
        );
    }
}

/// Switch point 38: tables I.13/I.14 of 2023 (p. 819–820), panel U by
/// build-year class from 1965 when the insulation thickness is unknown;
/// 2024 has no such tables.
#[test]
fn switch_38_panel_u_by_build_year_2023() {
    let u = |version, year, frame, exterior| {
        with_version(version, || {
            forfait_panel_u_in_edition(
                PanelInsulation::PresentUnknownThickness,
                false,
                frame,
                exterior,
                Some(year),
            )
        })
    };
    use FrameGroup::*;
    assert_eq!(u(V2023, 1970, WoodOrPlastic, true), Some(3.7));
    assert_eq!(u(V2023, 1980, MetalWithThermalBreak, true), Some(4.1));
    assert_eq!(u(V2023, 1991, MetalWithoutThermalBreak, true), Some(4.9));
    assert_eq!(u(V2023, 2000, MetalWithoutThermalBreak, true), Some(4.2));
    assert_eq!(u(V2023, 2013, WoodOrPlastic, true), Some(1.65));
    assert_eq!(u(V2023, 1970, MetalWithoutThermalBreak, false), Some(3.4));
    assert_eq!(u(V2023, 2005, MetalWithoutThermalBreak, false), Some(3.0));
    assert_eq!(u(V2023, 2020, MetalWithThermalBreak, false), Some(1.4));
    // 2024 takes tables I.11/I.12 whatever the build year.
    assert_eq!(
        u(V2024, 1970, WoodOrPlastic, true),
        with_version(V2024, || forfait_panel_u_in_edition(
            PanelInsulation::PresentUnknownThickness,
            false,
            WoodOrPlastic,
            true,
            None
        ))
    );
    assert_ne!(u(V2024, 2013, WoodOrPlastic, true), Some(1.65));
}

/// Switch point 74: (9.85) for an individual electric heat pump. 2020+A1
/// (p. 335–336) has one set for all devices: A 87,6 kWh (before 2015 or
/// unknown) or 13,0 kWh (from 2015), B 0,132 kW, C 1,44/3,6, B_nom 24 kW.
/// With E = 1 200 kWh in a month, a device from 2016 gives
/// 13,0/12 + 0,132 × 1 200/(0,4 × 24) = 1,083 33 + 16,5 kWh, and one from
/// 2010 gives 87,6/12 + 16,5 = 7,3 + 16,5 kWh.
#[test]
fn switch_74_device_forfait_9_85_in_2020() {
    let aux = |year| with_version(V2020, || heat_pump_forfait_auxiliary_kwh(1200.0, year));
    close(aux(Some(2016)), 13.0 / 12.0 + 16.5);
    close(aux(Some(2010)), 7.3 + 16.5);
    close(aux(None), 7.3 + 16.5);
    // 2022 (p. 338): the electric heat-pump set A 43,8 kWh, B 0,132, C 0,7,
    // B_nom 3 kW: 3,65 + 0,132 × 1 200/(0,7 × 3).
    close(
        with_version(V2022, || {
            heat_pump_forfait_auxiliary_kwh(1200.0, Some(2016))
        }),
        3.65 + 158.4 / 2.1,
    );
}

/// Switch point 12: table 13.18 c_W,EU;gen. 2025+C1 (p. 630–631) says to
/// interpolate linearly between the columns; 2024 (p. 614–615) has columns
/// "≤ 765", 2 130, 4 250 and "≥ 6 960" without an interpolation rule, and the
/// kernel takes the column of the largest quantity not above the demand.
/// Profile L at 3 000 kWh: 2024 0,74; 2025+C1
/// 0,74 + (3 000 − 2 130)/(4 250 − 2 130) × (1 − 0,74).
#[test]
fn switch_12_table_13_18_interpolation() {
    use nta8800_core::domestic_hot_water::{european_profile_correction, TappingProfile};
    let c = |version, profile, kwh| {
        with_version(version, || {
            european_profile_correction(profile, kwh).unwrap()
        })
    };
    close(c(V2024, TappingProfile::L, 3000.0), 0.74);
    close(
        c(V2025, TappingProfile::L, 3000.0),
        0.74 + 870.0 / 2120.0 * 0.26,
    );
    close(c(V2024, TappingProfile::Xl, 5000.0), 0.79);
    close(
        c(V2025, TappingProfile::Xl, 5000.0),
        0.79 + 750.0 / 2710.0 * 0.21,
    );
    close(c(V2024, TappingProfile::M, 500.0), 0.56);
    close(c(V2025, TappingProfile::M, 500.0), 0.56);
    // A use in a higher class than measured is not allowed (both editions).
    assert!(with_version(V2024, || european_profile_correction(
        TappingProfile::M,
        3000.0
    ))
    .is_none());
}

/// Review 73, switch point 44: NTA 8800:2023 table 13.4 (p. 542) rows for an
/// unknown diameter by type of system. "klein": 1,0 / 0,4 / 0,4 / 0,3 /
/// 0,25 / 0,2; "overig": 2,0 / 0,74 / 0,74 / 0,56 / 0,46 / 0,4 for no,
/// unknown, 10, 15, 20 and 25 mm insulation.
#[test]
fn switch_44_table_13_4_unknown_diameter_rows_2023() {
    use nta8800_core::domestic_hot_water::{table_13_4_system_psi, PipeInsulation as Ins};
    let columns = [
        Ins::None,
        Ins::Unknown,
        Ins::Mm10,
        Ins::Mm15,
        Ins::Mm20,
        Ins::Mm25,
    ];
    let small = [1.0, 0.4, 0.4, 0.3, 0.25, 0.2];
    let other = [2.0, 0.74, 0.74, 0.56, 0.46, 0.4];
    for (index, insulation) in columns.into_iter().enumerate() {
        close(table_13_4_system_psi(true, insulation), small[index]);
        close(table_13_4_system_psi(false, insulation), other[index]);
    }
}

/// Review 73, switch point 58: NTA 8800:2022 table 7.10 (p. 181–182) by the
/// mass of the construction per m² usable floor area: below 250 kg/m² 55/80,
/// 250 to 500 110/180, 500 to 750 180/360, above 750 250/450 kJ/(m²K) for a
/// closed or suspended ceiling / no or open ceiling.
#[test]
fn switch_58_table_7_10_by_mass_2022() {
    use nta8800_core::monthly_demand::{specific_heat_capacity_by_mass, CeilingColumn};
    let check = |mass, closed: f64, open: f64| {
        close(
            specific_heat_capacity_by_mass(mass, CeilingColumn::ClosedOrSuspended),
            closed,
        );
        close(
            specific_heat_capacity_by_mass(mass, CeilingColumn::OpenOrNone),
            open,
        );
    };
    check(200.0, 55.0, 80.0);
    check(250.0, 110.0, 180.0);
    check(499.0, 110.0, 180.0);
    check(500.0, 180.0, 360.0);
    check(750.0, 180.0, 360.0);
    check(751.0, 250.0, 450.0);
}

/// Review 73: switch points whose kernel value is a per-edition constant,
/// each checked against the text of the editions on both sides.
/// 7: (9.62) has no f_cor.bron.col in 2024 (p. 314) and 0,022/0,009 in
///    2025+C1 (p. 331–332).
/// 13: 11.136 measured motor efficiency only when higher than table 11.20
///    in 2024 (p. 499), free choice in 2025+C1 (p. 515).
/// 29: P.6.5.4.8 geothermal efficiency is the forfait 20 in 2023 (p. 960)
///    and temperature-dependent in 2024 (p. 958).
/// 30 and 53: the bmA biomass limit names no kW value in 2022 (p. 96),
///    100 kW in 2023 (p. 90, 92) and 500 kW in 2024 (p. 99, 103) and
///    2025+C1 (p. 105).
/// 37: U_rc of rooflights appears from 2024 (p. 210), not in 2023.
/// 54: (9.58) has no f_gebouw;si;H in 2022 (p. 305) and has it in 2023
///    (p. 309).
/// 59: 8.47 h is fixed at 0,125 m from 2023 (p. 240), not in 2022 (p. 236).
/// 62: swimming pools in (11.57) from 2023 (p. 459), not in 2022.
/// 73: an unknown β may be taken as 0,5 in 2022 (p. 936, opmerking 2), with
///    no such rule in 2020+A1 (p. 921–926).
/// 75: (11.139) of 2020+A1 (p. 495) has one f_systype term; 2022 (p. 499)
///    splits it over (11.140)–(11.142).
/// 77: actual pipe length only for utility buildings in 2020+A1 (p. 292,
///    363), for dwellings too in 2022 (p. 294, 367).
#[test]
fn review73_edition_constants_match_the_text() {
    let p = |version: NormVersion| version.profile();
    assert!(!p(V2024).collective_source_correction && p(V2025).collective_source_correction);
    assert!(!p(V2024).motor_efficiency_free_choice && p(V2025).motor_efficiency_free_choice);
    assert!(p(V2023).geothermal_efficiency_fixed && !p(V2024).geothermal_efficiency_fixed);
    assert!(p(V2022).biomass_threshold_kw.is_infinite());
    close(p(V2023).biomass_threshold_kw, 100.0);
    close(p(V2024).biomass_threshold_kw, 500.0);
    close(p(V2025).biomass_threshold_kw, 500.0);
    assert!(!p(V2023).rooflight_route && p(V2024).rooflight_route);
    assert!(!p(V2022).preference_beta_building_share && p(V2023).preference_beta_building_share);
    assert!(!p(V2022).crawl_wall_height_fixed && p(V2023).crawl_wall_height_fixed);
    assert!(!p(V2022).swimming_pool_route && p(V2023).swimming_pool_route);
    assert!(!p(V2020).unknown_beta_route && p(V2022).unknown_beta_route);
    assert_eq!(p(V2020).fan_systype_combined, Some(1.5));
    assert_eq!(p(V2022).fan_systype_combined, None);
    assert!(!p(V2020).residential_actual_pipe_length && p(V2022).residential_actual_pipe_length);
}
