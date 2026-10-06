use super::*;

#[test]
fn default_edition_is_2025_and_only_it_is_registrable() {
    assert_eq!(NormVersion::default(), NormVersion::V2025C1);
    assert_eq!(current(), NormVersion::V2025C1);
    for version in NormVersion::ALL {
        assert_eq!(
            version.registration_eligible(),
            version == NormVersion::V2025C1
        );
    }
    assert!(NormVersion::V2024 < NormVersion::V2025C1);
    assert!(NormVersion::V2020A1 < NormVersion::V2022);
}

#[test]
fn identifiers_round_trip_through_serde() {
    for version in NormVersion::ALL {
        let json = serde_json::to_value(version).unwrap();
        assert_eq!(json, serde_json::json!(version.id()));
        let back: NormVersion = serde_json::from_value(json).unwrap();
        assert_eq!(back, version);
    }
    assert!(serde_json::from_value::<NormVersion>(serde_json::json!("2019")).is_err());
}

#[test]
fn guard_restores_the_previous_edition() {
    let inner = with_version(NormVersion::V2024, || {
        assert_eq!(current(), NormVersion::V2024);
        with_version(NormVersion::V2025C1, current)
    });
    assert_eq!(inner, NormVersion::V2025C1);
    assert_eq!(current(), NormVersion::V2025C1);
    let caught = std::panic::catch_unwind(|| {
        with_version(NormVersion::V2024, || panic!("run failed"));
    });
    assert!(caught.is_err());
    assert_eq!(current(), NormVersion::V2025C1);
}

#[test]
fn other_threads_keep_the_default() {
    with_version(NormVersion::V2024, || {
        let other = std::thread::spawn(current).join().unwrap();
        assert_eq!(other, NormVersion::V2025C1);
    });
}

/// Every profile field against a hand-written expectation, with the page it
/// comes from, as a guard against copy errors between the edition files.
#[test]
fn profile_values_per_edition() {
    let p25 = NormVersion::V2025C1.profile();
    let p24 = NormVersion::V2024.profile();
    // Table 5.3: 2025 p. 96–98, 2024 p. 94–95.
    assert_eq!(
        (p25.k_co2_electricity, p24.k_co2_electricity),
        (0.268, 0.34)
    );
    assert_eq!((p25.k_co2_gas, p24.k_co2_gas), (0.218, 0.183));
    assert_eq!((p25.k_co2_oil, p24.k_co2_oil), (0.326, 0.260));
    assert_eq!((p25.k_co2_biomass, p24.k_co2_biomass), (0.104, 0.372));
    assert_eq!(
        (
            p25.k_co2_district_heat_forfait,
            p24.k_co2_district_heat_forfait
        ),
        (0.09, 0.17)
    );
    // Table 5.6: 2025 p. 125, 2024 p. 117.
    assert_eq!(
        (p25.k_co2_waste_incineration, p24.k_co2_waste_incineration),
        (0.138, 0.113)
    );
    // Table 9.16: 2025 p. 310, 2024 p. 292.
    assert_eq!(
        (
            p25.psi_collective_combined_small,
            p24.psi_collective_combined_small
        ),
        (1.0, 2.0)
    );
    // (I.3): 2025 p. 833, 2024 p. 814.
    assert_eq!(
        (p25.reed_thatch_divisor, p24.reed_thatch_divisor),
        (0.105, 0.2)
    );
    assert!(p25.bacs_factor_route && !p24.bacs_factor_route);
    assert!(p25.storage_credit && !p24.storage_credit);
    assert!(p25.indicators_2025 && !p24.indicators_2025);
    assert!(p25.collective_source_correction && !p24.collective_source_correction);
    assert!(p25.roof_edge_obstruction && !p24.roof_edge_obstruction);
    assert!(p25.permanent_shading_routes && !p24.permanent_shading_routes);
    assert!(p25.hot_water_building_share && !p24.hot_water_building_share);
    assert!(p25.motor_efficiency_free_choice && !p24.motor_efficiency_free_choice);
    assert!(p25.annex_aa_requirement_floor && !p24.annex_aa_requirement_floor);
    assert!(p25.bio_based_ageing_one && !p24.bio_based_ageing_one);
    assert!(p25.bio_based_lambda_2025 && !p24.bio_based_lambda_2025);
    assert!(p25.booster_bounds && !p24.booster_bounds);
    assert!(
        p25.wide_collective_low_temperature_sources && !p24.wide_collective_low_temperature_sources
    );
    assert_eq!(p25.table_13_18, Table1318Lookup::Interpolate);
    assert_eq!(p24.table_13_18, Table1318Lookup::LowerColumn);
    assert_eq!(p25.annex_aa, AnnexAaVariant::Hourly2025);
    assert_eq!(p24.annex_aa, AnnexAaVariant::Monthly2024);
    assert_eq!(
        p25.heat_pump_source_route,
        HeatPumpSourceRoute::CollectiveOnly2025
    );
    assert_eq!(
        p24.heat_pump_source_route,
        HeatPumpSourceRoute::AnySourceFrom15C2024
    );
    assert_eq!(p24.biomass_threshold_kw, 500.0);
    assert_eq!(p25.version, NormVersion::V2025C1);
    assert_eq!(p24.version, NormVersion::V2024);
}

mod switch_points {
    use crate::norm_versions::{with_version, NormVersion};

    fn legacy<R>(body: impl FnOnce() -> R) -> R {
        with_version(NormVersion::V2024, body)
    }

    #[test]
    fn table_5_3_and_5_6_co2_factors() {
        use crate::annex_p::{cold_forfait, heat_forfait, k_co2_el, SystemCarrier};
        // 2025+C1 p. 96–98 and 125.
        assert_eq!(k_co2_el(), 0.268);
        assert_eq!(heat_forfait().co2_kg_per_kwh, 0.09);
        assert_eq!(SystemCarrier::NaturalGas.co2(), 0.218);
        assert_eq!(SystemCarrier::WasteIncineration.co2(), 0.138);
        // 2024 p. 94–95 and 117.
        legacy(|| {
            assert_eq!(k_co2_el(), 0.34);
            assert_eq!(heat_forfait().co2_kg_per_kwh, 0.17);
            assert_eq!(heat_forfait().primary_factor, 0.9);
            assert!((cold_forfait().co2_kg_per_kwh - 0.34 / 3.0).abs() < 1e-12);
            assert_eq!(SystemCarrier::NaturalGas.co2(), 0.183);
            assert_eq!(SystemCarrier::Oil.co2(), 0.260);
            assert_eq!(SystemCarrier::WasteIncineration.co2(), 0.113);
        });
    }

    #[test]
    fn table_9_16_combined_collective_pipes() {
        use crate::heating_distribution::{PipeInsulation, PipeTransmittance};
        let pipe = PipeTransmittance::Forfait {
            insulation: PipeInsulation::Unknown,
        };
        // 2025+C1 p. 310: 1,0 up to 500 m²; 2024 p. 292: 2,0.
        assert_eq!(pipe.value(300.0, true), Some(1.0));
        assert_eq!(legacy(|| pipe.value(300.0, true)), Some(2.0));
        // Above 500 m² and other rows are equal.
        assert_eq!(legacy(|| pipe.value(800.0, true)), pipe.value(800.0, true));
        assert_eq!(
            legacy(|| pipe.value(300.0, false)),
            pipe.value(300.0, false)
        );
    }

    #[test]
    fn collective_source_factors_2024() {
        use crate::annex_p::{source_factors, CollectiveHeatPumpSource, SourceTemperatureClass};
        let source = CollectiveHeatPumpSource {
            temperature_class: SourceTemperatureClass::Below20C,
            supplier_reference: "invoice".into(),
            annex_p: None,
            realised_from_2013: None,
        };
        // 2025+C1 9.6.8.1.1.2.3 (p. 362–363): f_P;el/23, f_Pren 0,95.
        let (current, _) = source_factors(&source, "s").unwrap();
        assert!((current.declared.primary_factor - 1.45 / 23.0).abs() < 1e-12);
        assert_eq!(current.declared.renewable_factor, 0.95);
        // 2024 9.6.3.1.3 (p. 323): table 5.2 external heat, 0,9 (p. 93).
        let (old, _) = legacy(|| source_factors(&source, "s")).unwrap();
        assert_eq!(old.declared.primary_factor, 0.9);
        assert_eq!(old.declared.renewable_factor, 0.0);
        assert_eq!(old.declared.co2_kg_per_kwh, 0.17);
        assert_eq!(old.forfait, old.declared);
    }

    #[test]
    fn roof_edge_obstruction_is_2025_only() {
        use crate::climate::Orientation;
        use crate::solar_shading::{collector_obstruction_factor, CollectorObstruction};
        let edge = CollectorObstruction::RoofEdge {
            height_m: 2.0,
            distance_m: 1.0,
        };
        // 17.3.2 f) (2025+C1 p. 702); 2024 has no situation f (p. 678–687).
        assert!(collector_obstruction_factor(&edge, Orientation::South, 30.0, 6).is_some());
        assert!(
            legacy(|| collector_obstruction_factor(&edge, Orientation::South, 30.0, 6)).is_none()
        );
    }

    #[test]
    fn bio_based_lambda_and_ageing_2024() {
        use crate::materials::{ForfaitMaterial, InSituProduct};
        // Table E.11 (2025+C1 p. 810) against 2024 p. 791.
        assert_eq!(ForfaitMaterial::HempBoard.lambda(), 0.045);
        assert_eq!(legacy(|| ForfaitMaterial::HempBoard.lambda()), 0.100);
        assert_eq!(legacy(|| ForfaitMaterial::Straw.lambda()), 0.060);
        assert_eq!(legacy(|| ForfaitMaterial::FlaxBoard.lambda()), 0.050);
        assert_eq!(legacy(|| ForfaitMaterial::Coconut.lambda()), 0.055);
        // Reed: insulation at 0,100 in 2025+C1, table E.12 0,200 in 2024.
        assert!(ForfaitMaterial::Reed.is_insulation());
        assert!(!legacy(|| ForfaitMaterial::Reed.is_insulation()));
        // Unchanged rows.
        assert_eq!(legacy(|| ForfaitMaterial::WoodFibre.lambda()), 0.045);
        assert_eq!(legacy(|| ForfaitMaterial::GlassWool.lambda()), 0.040);
        // Table E.5 (2025+C1 p. 803; 2024 p. 783): cellulose 1,00 → overig 1,30.
        assert_eq!(
            ForfaitMaterial::CelluloseLoose.in_situ_product(),
            Some(InSituProduct::FibresAndFlakes)
        );
        assert_eq!(
            legacy(|| ForfaitMaterial::CelluloseLoose.in_situ_product()),
            Some(InSituProduct::Other)
        );
        assert_eq!(
            legacy(|| ForfaitMaterial::MineralWoolFlakes.in_situ_product()),
            Some(InSituProduct::FibresAndFlakes)
        );
    }

    #[test]
    fn table_13_18_column_or_interpolation() {
        use crate::domestic_hot_water::{european_profile_correction, TappingProfile};
        // 2025+C1 p. 630–631: linear between 765 (0,43) and 2 130 kWh (0,74).
        let interpolated = european_profile_correction(TappingProfile::L, 1500.0).unwrap();
        let expected = 0.43 + (1500.0 - 765.0) / (2130.0 - 765.0) * (0.74 - 0.43);
        assert!((interpolated - expected).abs() < 1e-12);
        // 2024 p. 614: no interpolation rule, the lower column.
        assert_eq!(
            legacy(|| european_profile_correction(TappingProfile::L, 1500.0)),
            Some(0.43)
        );
        assert_eq!(
            legacy(|| european_profile_correction(TappingProfile::L, 500.0)),
            Some(0.43)
        );
        assert_eq!(
            legacy(|| european_profile_correction(TappingProfile::L, 4250.0)),
            Some(1.0)
        );
        assert_eq!(
            legacy(|| european_profile_correction(TappingProfile::L, 5000.0)),
            None
        );
    }
}

/// NTA 8800:2023 against 2024: each value with the page of both editions.
mod switch_points_2023 {
    use crate::norm_versions::{with_version, NormVersion};

    fn v2023<R>(body: impl FnOnce() -> R) -> R {
        with_version(NormVersion::V2023, body)
    }

    fn v2024<R>(body: impl FnOnce() -> R) -> R {
        with_version(NormVersion::V2024, body)
    }

    #[test]
    fn profile_2023_is_2024_plus_its_own_differences() {
        let p23 = NormVersion::V2023.profile();
        let p24 = NormVersion::V2024.profile();
        assert!(NormVersion::V2023.implemented());
        assert!(!NormVersion::V2023.registration_eligible());
        assert_eq!(p23.version, NormVersion::V2023);
        // Table 5.3 (2023 p. 92–93, 2024 p. 94–95) and 5.6 (p. 114 / 117).
        assert_eq!(p23.k_co2_electricity, p24.k_co2_electricity);
        assert_eq!(p23.k_co2_gas, p24.k_co2_gas);
        assert_eq!(p23.k_co2_district_heat_forfait, 0.17);
        assert_eq!(p23.k_co2_waste_incineration, 0.113);
        // Tables 5.2/5.3: 100 kW (2023 p. 90–93) against 500 kW (2024 p. 92–95).
        assert_eq!(
            (p23.biomass_threshold_kw, p24.biomass_threshold_kw),
            (100.0, 500.0)
        );
        // (I.3): d/0,2 in both (2023 p. 815, 2024 p. 814).
        assert_eq!(p23.reed_thatch_divisor, p24.reed_thatch_divisor);
        // Table 11.7 τ_argII May–November (2023 p. 454, 2024 p. 449).
        assert_eq!(
            p23.tau_ventilative_cooling[4..11],
            [0.12, 0.18, 0.28, 0.25, 0.17, 0.06, 0.01]
        );
        assert_eq!(
            p24.tau_ventilative_cooling[4..11],
            [0.46, 0.75, 0.81, 0.79, 0.75, 0.26, 0.05]
        );
        // Table 17.1 θ_e;argII May–September (2023 p. 676, 2024 p. 674).
        let may_sep = |p: &crate::norm_versions::NormProfile| {
            p.argii_temperature_c[4..9]
                .iter()
                .map(|value| value.unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(may_sep(p23), [14.56, 15.62, 16.17, 16.90, 15.11]);
        assert_eq!(may_sep(p24), [16.42, 16.76, 17.51, 18.24, 16.74]);
        assert_eq!(p23.argii_temperature_c[9], p24.argii_temperature_c[9]);
        assert_eq!(p23.argii_temperature_c[10], p24.argii_temperature_c[10]);
        // 11.2.3.3.1 f_argII (2023 p. 466, 2024 p. 461).
        assert_eq!(p23.ventilative_cooling_operation, [0.5, 0.9]);
        assert_eq!(p24.ventilative_cooling_operation, [0.35, 0.50]);
        // (11.71a): 2024 p. 460–461 only.
        assert!(!p23.discharge_opening_route && p24.discharge_opening_route);
        // Table 11.8 f_τ of dwellings (2023 p. 460, 2024 p. 455).
        assert_eq!(p23.dwelling_occupancy_factor, Some(0.80));
        assert_eq!(p24.dwelling_occupancy_factor, None);
        // 11.3.2.7 ΔT_fan (2023 p. 496, 2024 p. 491).
        assert_eq!(p23.fan_temperature_rise_k, [1.0, 0.7, 1.5]);
        assert_eq!(p24.fan_temperature_rise_k, [0.7, 0.4, 0.7]);
        // (P.25) (2023 p. 950, 2024 p. 948) and P.6.5.4.8 (p. 960 / 958).
        assert_eq!(p23.reference_power_divisor, 8800.0);
        assert_eq!(p24.reference_power_divisor, 5400.0);
        assert!(p23.geothermal_efficiency_fixed && !p24.geothermal_efficiency_fixed);
        // 5.7.1 (2023 p. 103–104) without annex AA; table 5.7 (2024 p. 74)
        // absent in 2023.
        assert!(!p23.annex_aa_route && p24.annex_aa_route);
        assert!(!p23.renovation_standard && p24.renovation_standard);
        assert!(p23.kitchen_diameter_rows && !p24.kitchen_diameter_rows);
        // Table 14.3 (2023 p. 648, 2024 p. 646) and (8.17)/(8.18) (2023
        // p. 214–218, 2024 p. 219–222).
        assert!(!p23.led_2017_column && p24.led_2017_column);
        assert!(!p23.glazing_bar_term && p24.glazing_bar_term);
    }

    #[test]
    fn table_13_2_kitchen_rows_by_inner_diameter() {
        use crate::domestic_hot_water::{kitchen_emission_for, KitchenPipeDiameter::*};
        // 2023 p. 533: band 6–8 m gives 0,67 (≤ 8 mm), 0,55 (≤ 10 mm) and
        // 0,43 (overig); 2024 p. 527 has the "overig" row only.
        assert_eq!(v2023(|| kitchen_emission_for(7.0, Some(UpTo8Mm))), 0.67);
        assert_eq!(v2023(|| kitchen_emission_for(7.0, Some(UpTo10Mm))), 0.55);
        assert_eq!(v2023(|| kitchen_emission_for(7.0, Some(Other))), 0.43);
        assert_eq!(v2023(|| kitchen_emission_for(7.0, None)), 0.43);
        assert_eq!(v2023(|| kitchen_emission_for(15.0, Some(UpTo10Mm))), 0.35);
        assert_eq!(v2023(|| kitchen_emission_for(1.0, Some(UpTo8Mm))), 1.00);
        assert_eq!(v2024(|| kitchen_emission_for(7.0, Some(UpTo10Mm))), 0.43);
    }

    #[test]
    fn table_e_10_wood_fibre_and_cellulose() {
        use crate::materials::ForfaitMaterial;
        // 2023 p. 791: 0,050; 2024 p. 790: 0,045.
        assert_eq!(v2023(|| ForfaitMaterial::WoodFibre.lambda()), 0.050);
        assert_eq!(v2023(|| ForfaitMaterial::CelluloseLoose.lambda()), 0.050);
        assert_eq!(v2024(|| ForfaitMaterial::WoodFibre.lambda()), 0.045);
        assert_eq!(v2024(|| ForfaitMaterial::CelluloseLoose.lambda()), 0.045);
        // Bio-based rows of 2024 hold in 2023 too (2023 p. 792).
        assert_eq!(v2023(|| ForfaitMaterial::SheepWool.lambda()), 0.050);
    }

    #[test]
    fn table_i_1_detail_17() {
        use crate::forfait_envelope::{forfait_psi, PsiColumn};
        // 2023 p. 804: 0,60 / 0,90; 2024 p. 803: 0,06 / 0,09.
        assert_eq!(v2023(|| forfait_psi(17, 0, PsiColumn::A)), Some(0.60));
        assert_eq!(v2023(|| forfait_psi(17, 0, PsiColumn::B)), Some(0.90));
        assert_eq!(v2024(|| forfait_psi(17, 0, PsiColumn::A)), Some(0.06));
        assert_eq!(v2024(|| forfait_psi(17, 0, PsiColumn::B)), Some(0.09));
        // Neighbouring details are unchanged.
        assert_eq!(
            v2023(|| forfait_psi(16, 0, PsiColumn::A)),
            v2024(|| forfait_psi(16, 0, PsiColumn::A))
        );
    }

    #[test]
    fn f_sh_with_of_dwellings_on_the_heating_balance() {
        use crate::climate::Orientation::South;
        use crate::solar_shading::Balance::*;
        use crate::solar_shading::{shading_fraction, ShadingControl::*};
        // Table 7.7 July, south vertical 0,59 (2023 p. 181, 2024 p. 184).
        // 2024 p. 181 case 1: 0 on the heating balance of dwellings; 2023
        // p. 179 has no such case.
        assert_eq!(
            v2024(|| shading_fraction(ManualResidential, South, 90.0, 7, Heating)),
            0.0
        );
        assert_eq!(
            v2023(|| shading_fraction(ManualResidential, South, 90.0, 7, Heating)),
            0.59
        );
        // Automatic shading of a dwelling: table 7.7 in 2023 (p. 181).
        assert_eq!(
            v2023(|| shading_fraction(AutomaticResidentialIso52016, South, 90.0, 7, Cooling)),
            0.59
        );
        assert_eq!(
            v2023(|| shading_fraction(AutomaticResidentialIso52016, South, 90.0, 7, Heating)),
            0.59
        );
        assert_eq!(
            v2024(|| shading_fraction(AutomaticResidentialIso52016, South, 90.0, 7, Heating)),
            0.0
        );
        // The generic automatic control (table 7.9) is for utility buildings
        // only in 2023 (p. 183).
        assert!(v2024(|| Automatic.in_edition(true)));
        assert!(!v2023(|| Automatic.in_edition(true)));
        assert!(v2023(|| Automatic.in_edition(false)));
    }

    #[test]
    fn tables_i_13_and_i_14_panels_by_build_year() {
        use crate::forfait_envelope::{forfait_panel_u_in_edition, PanelInsulation::*};
        use crate::window_u::FrameGroup::*;
        // 2023 p. 819–820: 1992–2012 metal without thermal break 4,2
        // (outside) and 3,0 (not outside); from 2013 1,65 and 1,4.
        let u = |year, frame, exterior| {
            forfait_panel_u_in_edition(AbsentOrUnknown, false, frame, exterior, Some(year))
        };
        assert_eq!(v2023(|| u(2000, MetalWithoutThermalBreak, true)), Some(4.2));
        assert_eq!(
            v2023(|| u(2000, MetalWithoutThermalBreak, false)),
            Some(3.0)
        );
        assert_eq!(v2023(|| u(2015, WoodOrPlastic, true)), Some(1.65));
        assert_eq!(v2023(|| u(2015, MetalWithThermalBreak, false)), Some(1.4));
        assert_eq!(v2023(|| u(1970, WoodOrPlastic, true)), Some(3.7));
        // Before 1965 table I.11 (2023 p. 818): 3,7 without cavity.
        assert_eq!(v2023(|| u(1960, WoodOrPlastic, true)), Some(3.7));
        // 2024 p. 817 takes table I.11 whatever the year: insulation
        // unknown, no cavity, metal without break 4,9.
        assert_eq!(v2024(|| u(2015, MetalWithoutThermalBreak, true)), Some(4.9));
        // A known thickness keeps tables I.15/I.16 in both editions.
        let known = |year| {
            forfait_panel_u_in_edition(
                KnownThickness { thickness_mm: 20.0 },
                false,
                WoodOrPlastic,
                true,
                Some(year),
            )
        };
        assert_eq!(v2023(|| known(2015)), v2024(|| known(2015)));
    }

    #[test]
    fn table_13_4_rows_for_an_unknown_diameter() {
        use crate::domestic_hot_water::{table_13_4_system_psi, PipeInsulation::*};
        // 2023 p. 542 rows "klein" and "overig"; 2024 p. 537 replaced them
        // by table 13.29 and the 35/80 mm rule (p. 538).
        assert_eq!(table_13_4_system_psi(true, Unknown), 0.4);
        assert_eq!(table_13_4_system_psi(true, Mm20), 0.25);
        assert_eq!(table_13_4_system_psi(false, None), 2.0);
        assert_eq!(table_13_4_system_psi(false, Mm15), 0.56);
        let p23 = NormVersion::V2023.profile();
        let p24 = NormVersion::V2024.profile();
        assert!(p23.table_13_4_system_rows && !p24.table_13_4_system_rows);
        assert!(!p23.hot_water_series_routes && p24.hot_water_series_routes);
    }

    #[test]
    fn stage_two_profile_switches() {
        use crate::norm_versions::HeatPumpSourceRoute;
        let p23 = NormVersion::V2023.profile();
        let p24 = NormVersion::V2024.profile();
        let p25 = NormVersion::V2025C1.profile();
        // (11.71): NEN 1087 opening without screen data, 0,5 (2023 p. 466)
        // against 0,3 (2024 p. 461, 2025+C1 unchanged).
        assert_eq!(p23.unspecified_screen_factor, 0.5);
        assert_eq!(p24.unspecified_screen_factor, 0.3);
        assert_eq!(p25.unspecified_screen_factor, 0.3);
        // (11.77) 2023 p. 469 against (11.77a/b) 2024 p. 464.
        assert!(!p23.cross_area_roof_all_sectors && p24.cross_area_roof_all_sectors);
        // Note 2 of 8.2.2.1 e): 2024 p. 210 only.
        assert!(!p23.rooflight_route && p24.rooflight_route);
        // (5.47)/(5.55): 2023 p. 116/121 against 2024 p. 119/124.
        assert!(p23.residual_heat_pren_primary && !p24.residual_heat_pren_primary);
        // 9.4: 2023 p. 290–299 against 2024 p. 284–295.
        assert!(!p23.distribution_2024_rules && p24.distribution_2024_rules);
        assert!(!p23.table_9_16_combined_rows && p24.table_9_16_combined_rows);
        assert!(p23.distribution_half_recoverable_route);
        assert!(!p24.distribution_half_recoverable_route);
        // Source heat ≥ 20 °C (2023 p. 326, 343) against ≥ 15 °C (2024 p. 323).
        assert_eq!(p23.heat_pump_source_route, HeatPumpSourceRoute::From20C2023);
        assert!(p23.heat_pump_source_route.books_table_sources());
        assert!(p24.heat_pump_source_route.books_table_sources());
        assert!(!p25.heat_pump_source_route.books_table_sources());
        // 2024 and 2025+C1 share these values.
        assert_eq!(
            p24.dwelling_shading_heating_off,
            p25.dwelling_shading_heating_off
        );
        assert_eq!(p24.panel_build_year_tables, p25.panel_build_year_tables);
    }

    #[test]
    fn source_heat_from_20_c_in_2023() {
        use crate::space_heating_chain::SystemHeatPump;
        let pump = |from_15: bool, from_15_to_20: bool| SystemHeatPump {
            heat_pump_output_kwh: vec![100.0; 12],
            generator_electricity_kwh: vec![25.0; 12],
            generation_efficiency: Some(4.0),
            collective_source: false,
            ground_storage_source: false,
            source_from_15_c: from_15,
            source_15_to_20_c: from_15_to_20,
        };
        // A 15–20 °C (ground)water source: external heat in 2024 (p. 323),
        // not in 2023 (≥ 20 °C, p. 326).
        assert!(v2024(|| pump(true, true).source_heat_booked()));
        assert!(!v2023(|| pump(true, true).source_heat_booked()));
        // A 20–40 °C source books its heat in both.
        assert!(v2024(|| pump(true, false).source_heat_booked()));
        assert!(v2023(|| pump(true, false).source_heat_booked()));
        assert!(!v2023(|| pump(false, false).source_heat_booked()));
    }

    #[test]
    fn heating_emission_tables_9_2_to_9_10() {
        use crate::heating_emission::*;
        let close = |a: f64, b: f64| assert!((a - b).abs() < 1e-9, "{a} != {b}");
        let with = |kind, certified, room_automation, pipe_system, balancing| Emission2023 {
            kind,
            certified_control: certified,
            room_automation,
            pipe_system,
            balancing,
        };
        // Table 9.3 (2023 p. 277–278): Δθ_str = (0,7 + 1,3)/2, Δθ_ctr,1 2,5,
        // Δθ_im;emt −0,3; table 9.2 two-pipe static 0,4; Δθ_roomaut −0,5.
        let radiators = with(
            Emission2023Kind::Radiators {
                control: RoomControl2023::Room,
                over_temperature: OverTemperature2023::TwoPipe42K,
                position: RadiatorPosition2023::InnerWall,
            },
            false,
            RoomAutomation2023::IndividualPerRoom,
            PipeSystem2023::TwoPipe,
            BalancingRow2023::Static,
        );
        close(radiators.increment_k(), 1.0 + 2.5 - 0.3 + 0.4 - 0.5);
        // Table 9.4 (p. 279–280): wet floor, minimal insulation (9.18):
        // Δθ_emb (0,7 + 0,5)/2, Δθ_ctr,2 1,5, Δθ_im;emt −0,2, one-pipe 0,7.
        let floor = with(
            Emission2023Kind::Surface {
                control: RoomControl2023::Room,
                system: SurfaceSystem2023::FloorWetOrUnknown,
                insulation: SurfaceInsulation2023::MinimalInsulation,
            },
            true,
            RoomAutomation2023::Unknown,
            PipeSystem2023::OnePipe,
            BalancingRow2023::NoneOrUnknown,
        );
        close(floor.increment_k(), 0.6 + 1.5 - 0.2 + 0.7);
        // Table 9.6 (p. 282): unknown wall and control, variation b: the
        // highest 3,1; Δθ_im −0,3. Certified PI at an outer wall: 0,7.
        let electric = |wall, control, certified| {
            with(
                Emission2023Kind::ElectricAir { wall, control },
                certified,
                RoomAutomation2023::Unknown,
                PipeSystem2023::NotHydronic,
                BalancingRow2023::NoneOrUnknown,
            )
            .increment_k()
        };
        close(
            electric(
                WallArea2023::Unknown,
                ElectricAirControl2023::Unknown,
                false,
            ),
            3.1 - 0.3,
        );
        close(
            electric(
                WallArea2023::OuterWall,
                ElectricAirControl2023::PiPerRoom,
                true,
            ),
            0.7 - 0.3,
        );
        // Table 9.7 (p. 283): recirculation, high quality 0,7.
        close(
            with(
                Emission2023Kind::VentilationAir {
                    configuration: VentilationAirHeating2023::Recirculation,
                },
                true,
                RoomAutomation2023::Unknown,
                PipeSystem2023::NotHydronic,
                BalancingRow2023::NoneOrUnknown,
            )
            .increment_k(),
            0.7,
        );
        // 9.19 (p. 283) with table 9.8 0,60 K/m at 8 m: 10·0,60/16·2,9;
        // table 9.10 Δθ_ctr,1 2,5.
        let high = |height_m, emitter, radiant, certified| {
            with(
                Emission2023Kind::HighRoom {
                    height_m,
                    emitter,
                    control: HighRoomControl2023::Controlled,
                    radiant,
                },
                certified,
                RoomAutomation2023::Unknown,
                PipeSystem2023::NotHydronic,
                BalancingRow2023::NoneOrUnknown,
            )
        };
        close(
            high(8.0, HighRoomEmitter2023::WarmAirFromCeiling, None, false).increment_k(),
            10.0 * 0.60 / 16.0 * 2.9 + 2.5,
        );
        // 9.20 (p. 284) dark radiators, 10 m, RF 0,55, p_h 70 W/m²:
        // 10·(0,36/0,75 + 0,354 − 0,9) = −0,66; Δθ_str 10·0,20/16·3,9;
        // certified controlled Δθ_ctr,2 0,7 (table 9.10, p. 285).
        let product = RadiantProduct2023 {
            radiation_factor: None,
            specific_power_w_per_m2: 70.0,
        };
        close(
            high(
                10.0,
                HighRoomEmitter2023::DarkRadiators,
                Some(product),
                true,
            )
            .increment_k(),
            10.0 * 0.20 / 16.0 * 3.9 - 0.66 + 0.7,
        );
        assert!(!high(10.0, HighRoomEmitter2023::DarkRadiators, None, true).valid());
        assert!(!high(4.0, HighRoomEmitter2023::WarmAirFromCeiling, None, true).valid());
        assert!(!high(
            7.0,
            HighRoomEmitter2023::WarmAirHorizontalLowTemperature,
            None,
            true
        )
        .valid());

        // A 2024 input under 2023: the unknown values of each category.
        let input = |system, balancing, control| EmissionInput {
            system,
            balancing,
            control,
            source_reference: "test".into(),
            fans: None,
            air_heaters: None,
            edition2023: None,
        };
        let floor_2024 = input(
            EmissionSystem::FloorHeating,
            HydronicBalancing::NoneOrUnknown,
            EmissionControl::IndividualRoomThermostats,
        );
        // 2024 p. 279–280: 0,3 + 0,7 + 1,5. 2023: 0 + 2,5 + 0,7 (9.18a)
        // − 0,2 + 0,7 − 0,5.
        close(v2024(|| temperature_increment_k(&floor_2024)), 2.5);
        close(v2023(|| temperature_increment_k(&floor_2024)), 3.2);
        let radiators_2024 = input(
            EmissionSystem::RadiatorsOrConvectors,
            HydronicBalancing::NoneOrUnknown,
            EmissionControl::MainRoomThermostat,
        );
        // 2023: (1,6 + 1,7)/2 + 2,5 − 0,3 + 0,7.
        close(v2024(|| temperature_increment_k(&radiators_2024)), 3.55);
        close(v2023(|| temperature_increment_k(&radiators_2024)), 4.55);
        // An explicit 2023 description wins under 2023.
        let mut described = radiators_2024.clone();
        described.edition2023 = Some(radiators);
        close(v2023(|| temperature_increment_k(&described)), 3.1);
        close(v2024(|| temperature_increment_k(&described)), 3.55);
    }

    #[test]
    fn cooling_emission_tables_10_2_to_10_5() {
        use crate::space_cooling::*;
        let close = |a: f64, b: f64| assert!((a - b).abs() < 1e-9, "{a} != {b}");
        let emission = |emitter, balancing, control| CoolingEmission {
            emitter,
            balancing,
            control,
            fan_coil_count: 0,
            source_reference: "test".into(),
            edition2023: None,
        };
        // Floor cooling without balancing, standalone per room:
        // 2024 p. 359 table 10.35 −1,7, table 10.4 −0,6, table 10.5 −1,25;
        // 2023 p. 362–364: Δϑ_str −0,7, Δϑ_ctr,1 −2,5, Δϑ_emb −0,7,
        // Δϑ_hydr −0,6, Δϑ_roomaut +0,5.
        let floor = emission(
            CoolingEmitter::FloorCooling,
            CoolingBalancing::NoneOrUnknown,
            CoolingControl::StandalonePerRoom,
        );
        close(v2024(|| floor.delta_internal()), -1.7 - 0.6 - 1.25);
        close(
            v2023(|| floor.delta_internal()),
            -0.7 - 2.5 - 0.7 - 0.6 + 0.5,
        );
        // Certified P control from before 1988 (Δϑ_ctr,2 −1,5), fan coil on
        // the ceiling (0/0), direct expansion (0) and a network (+1,2).
        let mut fan_coil = emission(
            CoolingEmitter::FanCoilOrRacOnCeiling,
            CoolingBalancing::NotApplicable,
            CoolingControl::CentralWithRoomControl,
        );
        fan_coil.edition2023 = Some(CoolingEmission2023 {
            control: CoolingRoomControl2023::PBefore1988,
            certified_control: true,
            balancing: CoolingBalancingRow2023::DynamicOrDirectExpansion,
            room_automation: CoolingRoomAutomation2023::NetworkWithOverrideAndAdaptive,
        });
        close(v2023(|| fan_coil.delta_internal()), -1.5 + 1.2);
        close(v2024(|| fan_coil.delta_internal()), -0.5 - 0.75);
        // Central control stays −2,5 with a certified product.
        fan_coil.edition2023.as_mut().unwrap().control = CoolingRoomControl2023::Central;
        close(v2023(|| fan_coil.delta_internal()), -2.5 + 1.2);
    }
}

/// NTA 8800:2022 against 2023: each value with the page of both editions.
mod switch_points_2022 {
    use crate::norm_versions::{with_version, NormVersion};

    fn v2022<R>(body: impl FnOnce() -> R) -> R {
        with_version(NormVersion::V2022, body)
    }

    fn v2023<R>(body: impl FnOnce() -> R) -> R {
        with_version(NormVersion::V2023, body)
    }

    #[test]
    fn profile_2022_is_2023_plus_its_own_differences() {
        let p22 = NormVersion::V2022.profile();
        let p23 = NormVersion::V2023.profile();
        assert!(NormVersion::V2022.implemented());
        assert!(!NormVersion::V2022.registration_eligible());
        assert!(NormVersion::V2020A1.implemented());
        assert_eq!(p22.version, NormVersion::V2022);
        assert_eq!(NormVersion::V2022.label(), "NTA 8800:2022");
        // Everything 2023 has against 2024 holds in 2022 too.
        assert_eq!(p22.tau_ventilative_cooling, p23.tau_ventilative_cooling);
        assert_eq!(p22.fan_temperature_rise_k, p23.fan_temperature_rise_k);
        assert_eq!(p22.reference_power_divisor, 8800.0);
        assert!(p22.emission_tables_2023 && p22.cooling_emission_tables_2023);
        assert!(p22.kitchen_diameter_rows && p22.panel_build_year_tables);
        // Tables 9.27/9.29/P.5 (2022 p. 314–317, 941; 2023 p. 319–324, 955).
        assert!(p22.heat_pump_tables_2022 && !p23.heat_pump_tables_2022);
        assert_eq!(
            p22.heat_pump_source_route,
            crate::norm_versions::HeatPumpSourceRoute::None2022
        );
        assert!(!p22.heat_pump_source_route.books_table_sources());
        // Tables 5.2/5.4: no kW limit (2022 p. 88–92), 100 kW (2023 p. 90–94).
        assert!(p22.biomass_threshold_kw.is_infinite());
        assert_eq!(p23.biomass_threshold_kw, 100.0);
        // Flex mode (2023 p. 111, 963).
        assert!(!p22.flex_mode_route && p23.flex_mode_route);
        // (9.58) (2022 p. 305, 2023 p. 309).
        assert!(!p22.preference_beta_building_share && p23.preference_beta_building_share);
        // Table E.5 (2022 p. 775, 2023 p. 785) and (I.2) (p. 805 / 814).
        assert_eq!(
            (
                p22.mineral_wool_flakes_ageing,
                p23.mineral_wool_flakes_ageing
            ),
            (1.05, 1.00)
        );
        assert_eq!((p22.lambda_equi_ntr, p23.lambda_equi_ntr), (0.06, 0.045));
        assert!(!p22.lambda_equi_known_route && p23.lambda_equi_known_route);
        // (8.47) (2022 p. 236, 2023 p. 240).
        assert!(!p22.crawl_wall_height_fixed && p23.crawl_wall_height_fixed);
        // Chapters 11, 13 and 14.
        assert!(!p22.swimming_pool_route && !p22.en_13053_route && !p22.mixed_air_route);
        assert!(p23.swimming_pool_route && p23.en_13053_route && p23.mixed_air_route);
        assert!(p22.electric_boiler_insulated_pipe_factor);
        assert!(!p23.electric_boiler_insulated_pipe_factor);
        assert!(!p22.storage_ambient_exhaust_air && p23.storage_ambient_exhaust_air);
        assert!(p22.lighting_maintenance_factor && !p23.lighting_maintenance_factor);
    }

    #[test]
    fn table_9_14_without_60_50_and_70_60() {
        use crate::heating_distribution::DesignTemperatureClass::*;
        // 2022 p. 290 against 2023 p. 294.
        assert!(!v2022(|| C60.in_edition()) && !v2022(|| C70.in_edition()));
        assert!(v2022(|| C65.in_edition()) && v2022(|| C55.in_edition()));
        assert!(v2023(|| C60.in_edition()) && v2023(|| C70.in_edition()));
    }

    #[test]
    fn table_7_5_without_unknown_colour() {
        use crate::solar_shading::{ShadeColour, ShadingDevice};
        // 2022 p. 176 against 2023 p. 180.
        let unknown = ShadingDevice::ExternalScreen {
            colour: ShadeColour::Unknown,
        };
        let white = ShadingDevice::ExternalScreen {
            colour: ShadeColour::White,
        };
        assert!(!v2022(|| unknown.in_edition()));
        assert!(v2022(|| white.in_edition()));
        assert!(v2023(|| unknown.in_edition()));
    }

    #[test]
    fn table_7_10_by_mass_per_m2() {
        use crate::monthly_demand::{specific_heat_capacity_by_mass, CeilingColumn::*};
        // 2022 p. 181–182: < 250, 250–500, 500–750, > 750 kg/m².
        assert_eq!(specific_heat_capacity_by_mass(200.0, OpenOrNone), 80.0);
        assert_eq!(
            specific_heat_capacity_by_mass(250.0, ClosedOrSuspended),
            110.0
        );
        assert_eq!(specific_heat_capacity_by_mass(750.0, OpenOrNone), 360.0);
        assert_eq!(
            specific_heat_capacity_by_mass(751.0, ClosedOrSuspended),
            250.0
        );
    }

    #[test]
    fn table_e_5_mineral_wool_flakes() {
        use crate::materials::{Ageing, InSituProduct, InSituSituation};
        let flakes = Ageing::InSitu {
            product: InSituProduct::FibresAndFlakes,
            situation: InSituSituation::A,
            practice_tested: false,
        };
        // 2022 p. 775: 1,05; 2023 p. 785: 1,00.
        assert_eq!(v2022(|| flakes.factor()), 1.05);
        assert_eq!(v2023(|| flakes.factor()), 1.00);
    }

    #[test]
    fn table_14_4_maintenance_factor() {
        use crate::lighting::ConstantIlluminance;
        // 2022 p. 639 with (14.15) and F_CC = 1: 1 − ½·(1 − MF).
        assert!((ConstantIlluminance::LinearFluorescent.compensation_factor() - 0.9).abs() < 1e-12);
        assert!((ConstantIlluminance::LedL80.compensation_factor() - 0.85).abs() < 1e-12);
    }
}

/// NTA 8800:2020+A1 against 2022: each value with the page of both editions.
mod switch_points_2020 {
    use crate::forfait_envelope::{forfait_psi, PsiColumn};
    use crate::norm_versions::{with_version, NormVersion};
    use crate::pv::PeakPower;

    fn v2020<R>(body: impl FnOnce() -> R) -> R {
        with_version(NormVersion::V2020A1, body)
    }

    fn v2022<R>(body: impl FnOnce() -> R) -> R {
        with_version(NormVersion::V2022, body)
    }

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    #[test]
    fn profile_2020_is_2022_plus_its_own_differences() {
        let p20 = NormVersion::V2020A1.profile();
        let p22 = NormVersion::V2022.profile();
        assert!(NormVersion::V2020A1.implemented());
        assert!(!NormVersion::V2020A1.registration_eligible());
        assert_eq!(p20.version, NormVersion::V2020A1);
        assert_eq!(NormVersion::V2020A1.label(), "NTA 8800:2020+A1:2020");
        // Everything 2022 has against 2023 holds in 2020+A1 too.
        assert!(p20.heat_pump_tables_2022 && p20.thermal_mass_by_kg_per_m2);
        assert!(!p20.crawl_wall_height_fixed && p20.lighting_maintenance_factor);
        assert_eq!(
            p20.mineral_wool_flakes_ageing,
            p22.mineral_wool_flakes_ageing
        );
        // (I.2) 2020 p. 796: 0,045; 2022 p. 805: 0,06.
        assert_eq!((p20.lambda_equi_ntr, p22.lambda_equi_ntr), (0.045, 0.06));
        assert!(!p20.lambda_equi_known_route);
        // (P.25) 2020 p. 925: 4 000; 2022 p. 937: 8 800.
        assert_eq!(
            (p20.reference_power_divisor, p22.reference_power_divisor),
            (4000.0, 8800.0)
        );
        // 9.85 forfait A from 2015: 2020 p. 336 13,0; 2022 p. 338 43,8.
        assert_eq!(
            (
                p20.device_aux_a_from_2015_kwh,
                p22.device_aux_a_from_2015_kwh
            ),
            (13.0, 43.8)
        );
        // (11.142) 2020 p. 495: f_systype 1,5 for E1.
        assert_eq!(p20.fan_systype_combined, Some(1.5));
        assert_eq!(p22.fan_systype_combined, None);
        // 2020 p. 786: Ψ detail 14 0,70; 2022 p. 793: 0,03.
        assert_eq!((p20.psi_detail_14, p22.psi_detail_14), (0.70, 0.03));
        for (own, later) in [
            (p20.pv_kpk_per_m2_floor, !p22.pv_kpk_per_m2_floor),
            (
                p20.residual_heat_fixed_factors,
                !p22.residual_heat_fixed_factors,
            ),
            (
                !p20.small_system_forfait_route,
                p22.small_system_forfait_route,
            ),
            (!p20.unknown_beta_route, p22.unknown_beta_route),
            (!p20.heat_pump_aux_constants, p22.heat_pump_aux_constants),
            (!p20.psi_columns_and_default, p22.psi_columns_and_default),
            (
                !p20.residential_actual_pipe_length,
                p22.residential_actual_pipe_length,
            ),
            (
                !p20.declared_exhaust_air_flow_route,
                p22.declared_exhaust_air_flow_route,
            ),
            (!p20.cold_recovery_route, p22.cold_recovery_route),
        ] {
            assert!(own && later);
        }
    }

    #[test]
    fn pv_peak_power_per_m2_is_floored_to_5_w() {
        // 2020 p. 651: K_pk 203 → 200 W/m², area as given; 2022 p. 657:
        // two decimals for both.
        let declared = PeakPower::DeclaredSpecific {
            peak_power_w_per_m2: 203.0,
            panel_area_m2: 1.234,
        };
        close(v2020(|| declared.kw()), 200.0 * 1.234 / 1000.0);
        close(v2022(|| declared.kw()), 203.0 * 1.23 / 1000.0);
    }

    #[test]
    fn table_i1_has_one_column_and_detail_14_is_0_70() {
        // 2020 p. 786: detail 14 0,70; 2022 p. 793: 0,03 / 0,13.
        assert_eq!(v2020(|| forfait_psi(14, 0, PsiColumn::A)), Some(0.70));
        assert_eq!(v2022(|| forfait_psi(14, 0, PsiColumn::A)), Some(0.03));
        assert_eq!(v2020(|| forfait_psi(1, 0, PsiColumn::A)), Some(0.27));
        assert_eq!(v2020(|| forfait_psi(1, 0, PsiColumn::B)), None);
        assert_eq!(v2022(|| forfait_psi(1, 0, PsiColumn::B)), Some(0.41));
    }

    #[test]
    fn heat_pumps_take_the_device_aux_forfait() {
        // 2020 p. 334–336: A 87,6, B 0,132, C 1,44/3,6, B_nom 24; 2022
        // p. 338: A 43,8, B 0,132, C 0,7, B_nom 3.
        let aux = crate::space_heating_chain::heat_pump_forfait_auxiliary_kwh;
        close(
            v2020(|| aux(100.0)),
            87.6 / 12.0 + 0.132 * 100.0 / (0.4 * 24.0),
        );
        close(
            v2022(|| aux(100.0)),
            43.8 / 12.0 + 0.132 * 100.0 / (0.7 * 3.0),
        );
    }
}
