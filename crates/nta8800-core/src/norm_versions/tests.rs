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
