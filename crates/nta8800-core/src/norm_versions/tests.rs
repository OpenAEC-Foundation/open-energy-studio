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
}
