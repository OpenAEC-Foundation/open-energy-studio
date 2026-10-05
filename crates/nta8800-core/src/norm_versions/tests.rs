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
