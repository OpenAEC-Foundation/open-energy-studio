//! Review 77: coverage sweep of NTA 8800:2025+C1:2026 chapters 11 to 17 and annexes S to Z. Each part
//! pins provisions that earlier reviews did not pin with an independent value: part A chapters 11 and 12,
//! part B chapter 13 and annexes T, U and W, part C chapters 14 to 17 and annexes V, X and Y. The expected
//! values come from separate implementations written from the norm pages before the kernel's formula code
//! was read; only numbers and page references are kept here. Tolerance 1e-6 relative. The classification
//! is in docs/nta8800-dekking-ch11-17.md.

mod part_a {
    // Review 77 part A: chapter 11 and 12 provisions of NTA 8800:2025+C1:2026 that were not yet
    // pinned by an independent value (systems A, B and E.1 through the airflow model of 11.2.1, the
    // lower bounds 11.64/11.65, installed capacity 11.55-11.62 with recirculation 11.110-11.113,
    // tau_sysC of 11.46-11.54, cold recovery 11.106a, 11.109, overventilation 11.23/11.24, tables
    // 11.13/11.14, 11.20-11.23 and chapter 12). The expected values come from a separate
    // implementation written from the norm pages (p. 426-524, table 17.1 p. 690); only numbers and
    // page references are kept here.

    use nta8800_core::humidification::{calculate_humidity, HumidityInput};
    use nta8800_core::space_heating_chain::{assess_space_heating_chain, SpaceHeatingChainInput};
    use nta8800_core::ventilation::{assess_ventilation, VentilationInput};
    use serde_json::Value;

    fn actual(input: &str) -> Value {
        let input: VentilationInput = serde_json::from_str(input).expect("valid ventilation input");
        let assessment = assess_ventilation(&input);
        assert_eq!(
            assessment.status, "calculated_unverified",
            "{:?}",
            assessment.issues
        );
        serde_json::to_value(assessment.actual.expect("actual run")).expect("serializable")
    }

    fn close_f64(label: &str, kernel: f64, expected: f64) {
        let diff = (kernel - expected).abs();
        let scale = kernel.abs().max(expected.abs()).max(1e-12);
        assert!(
            diff <= 1e-9 || diff / scale <= 1e-6,
            "{label}: kernel {kernel} independent {expected}"
        );
    }

    fn close(label: &str, kernel: &Value, expected: f64) {
        let kernel = kernel
            .as_f64()
            .unwrap_or_else(|| panic!("{label}: missing"));
        close_f64(label, kernel, expected);
    }

    fn check(a: &Value, rows: &[(&str, f64)]) {
        for (pointer, expected) in rows {
            close(
                pointer,
                a.pointer(pointer).unwrap_or(&Value::Null),
                *expected,
            );
        }
    }

    /// System A.2b (table 11.5 p. 460, 11.25/11.26 p. 452) in a zone of 40 m2 of a 60 m2 dwelling: lower
    /// bound 11.65 (p. 469) and f_tau of table 11.8 on the whole dwelling; reference infiltration 11.86 with
    /// tables 11.13/11.14 (p. 486-487); airflow model 11.1-11.21 (p. 439-447), 11.20 summed over all
    /// natural paths of a kind.
    #[test]
    fn natural_a2b_split_dwelling_with_reference_infiltration() {
        let a = actual(
            r#"{"zoneId": "z1", "usableFloorAreaM2": 40.0, "category": "residential", "functions": [{"function": "residential", "areaM2": 40.0, "swimmingPool": false}], "dwellingCount": 1, "wholeDwellingAreaM2": 60.0, "apartmentBuilding": false, "buildingHeightM": 9.0, "constructionYear": 1975, "floorAboveCrawlspace": false, "heatingSetpointC": 20.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "reference", "buildingType": "pitched_roof_end_or_corner"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "forfait", "current": "ac", "manufactureYear": null}, "sourceReference": "r77 synthetic", "system": {"kind": "single", "unit": {"variant": "a2b", "heatRecovery": null, "ducts": "no_ducts", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#,
        );
        close("q-v1", &a["infiltrationQV1M3PerH"], 92.35996226649642);
        check(
            &a,
            &[
                (
                    "/months/0/heating/requiredOutdoorAirM3PerH",
                    58.88842105263158,
                ),
                ("/months/0/heatingReferencePressurePa/0", -4.667849359058096),
                ("/months/0/heating/infiltrationM3PerH", 73.49100307857663),
                ("/months/0/heating/naturalSupplyM3PerH", 98.38598118316739),
                ("/months/0/heating/purgeM3PerH", 10.104506175568545),
                (
                    "/months/0/cooling/requiredOutdoorAirM3PerH",
                    58.88842105263158,
                ),
                ("/months/0/coolingReferencePressurePa/0", -5.415526631785369),
                ("/months/0/cooling/infiltrationM3PerH", 74.04124458196821),
                ("/months/0/cooling/naturalSupplyM3PerH", 98.93518830559891),
                ("/months/0/cooling/purgeM3PerH", 10.160911231385837),
                (
                    "/months/3/heating/requiredOutdoorAirM3PerH",
                    58.88842105263158,
                ),
                ("/months/3/heatingReferencePressurePa/0", -3.249847959673417),
                ("/months/3/heating/infiltrationM3PerH", 71.89425987188277),
                ("/months/3/heating/naturalSupplyM3PerH", 96.78629148089084),
                ("/months/3/heating/purgeM3PerH", 9.94021371965906),
                (
                    "/months/3/cooling/requiredOutdoorAirM3PerH",
                    58.88842105263158,
                ),
                (
                    "/months/3/coolingReferencePressurePa/0",
                    -3.9194002324006902,
                ),
                ("/months/3/cooling/infiltrationM3PerH", 71.0559708682285),
                ("/months/3/cooling/naturalSupplyM3PerH", 95.94285242314476),
                ("/months/3/cooling/purgeM3PerH", 39.41436099545407),
                (
                    "/months/6/heating/requiredOutdoorAirM3PerH",
                    58.88842105263158,
                ),
                (
                    "/months/6/heatingReferencePressurePa/0",
                    -1.1786467612738392,
                ),
                ("/months/6/heating/infiltrationM3PerH", 56.21428460044538),
                ("/months/6/heating/naturalSupplyM3PerH", 80.55210380391797),
                ("/months/6/heating/purgeM3PerH", 8.272918769051037),
                (
                    "/months/6/cooling/requiredOutdoorAirM3PerH",
                    58.88842105263158,
                ),
                (
                    "/months/6/coolingReferencePressurePa/0",
                    -1.8481990340011123,
                ),
                ("/months/6/cooling/infiltrationM3PerH", 55.26678185240298),
                ("/months/6/cooling/naturalSupplyM3PerH", 79.53669681161364),
                ("/months/6/cooling/purgeM3PerH", 65.34906981278526),
                (
                    "/months/9/heating/requiredOutdoorAirM3PerH",
                    58.88842105263158,
                ),
                ("/months/9/heatingReferencePressurePa/0", -2.830236354181366),
                ("/months/9/heating/infiltrationM3PerH", 63.26237710792297),
                ("/months/9/heating/naturalSupplyM3PerH", 87.9750632907937),
                ("/months/9/heating/purgeM3PerH", 9.035276770405842),
                (
                    "/months/9/cooling/requiredOutdoorAirM3PerH",
                    58.88842105263158,
                ),
                (
                    "/months/9/coolingReferencePressurePa/0",
                    -3.5154136269086393,
                ),
                ("/months/9/cooling/infiltrationM3PerH", 62.66751069604319),
                ("/months/9/cooling/naturalSupplyM3PerH", 87.35697779416024),
                ("/months/9/cooling/purgeM3PerH", 35.88719087760097),
            ],
        );
        close("fan-year", &a["annualFanElectricityKwh"], 0.0);
    }

    /// System B.1 in an apartment zone with three dwellings: lower bound 11.64 (p. 469), q_SUP;eff =
    /// q_ODA;req (11.2.2.2.2, p. 452-453) with C_vent;out of 11.27, supply at theta_e plus the fan rise of
    /// 11.3.2.7 (p. 507) and no frost jump for SUPPLY_OP (p. 495); forfait fans 11.142 with table 11.23.
    #[test]
    fn mechanical_supply_b1_apartment_zone() {
        let a = actual(
            r#"{"zoneId": "z1", "usableFloorAreaM2": 150.0, "category": "residential", "functions": [{"function": "residential", "areaM2": 150.0, "swimmingPool": false}], "dwellingCount": 3, "wholeDwellingAreaM2": null, "apartmentBuilding": true, "buildingHeightM": 12.0, "constructionYear": 2015, "floorAboveCrawlspace": false, "heatingSetpointC": 20.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.6, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2012}, "sourceReference": "r77 synthetic", "system": {"kind": "single", "unit": {"variant": "b1", "heatRecovery": null, "ducts": "luka_a_b_c", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#,
        );
        check(
            &a,
            &[
                (
                    "/months/0/heating/requiredOutdoorAirM3PerH",
                    284.0968421052632,
                ),
                (
                    "/months/0/heatingReferencePressurePa/0",
                    -3.5633972977214086,
                ),
                ("/months/0/heating/infiltrationM3PerH", 18.968980890512775),
                ("/months/0/heating/naturalSupplyM3PerH", 107.06060478129683),
                ("/months/0/heating/purgeM3PerH", 17.093710007097812),
                (
                    "/months/0/heating/mechanicalSupplyM3PerH",
                    284.0968421052632,
                ),
                (
                    "/months/0/heating/mechanicalSupplyTemperatureC",
                    3.3099999999999996,
                ),
                ("/months/0/fanElectricityKwh", 29.356673684210527),
                (
                    "/months/0/cooling/requiredOutdoorAirM3PerH",
                    284.0968421052632,
                ),
                ("/months/0/coolingReferencePressurePa/0", -4.503008661357782),
                ("/months/0/cooling/infiltrationM3PerH", 18.61779004157393),
                ("/months/0/cooling/naturalSupplyM3PerH", 105.57791289413672),
                ("/months/0/cooling/purgeM3PerH", 16.856977688979814),
                (
                    "/months/0/cooling/mechanicalSupplyM3PerH",
                    284.0968421052632,
                ),
                ("/months/0/cooling/mechanicalSupplyTemperatureC", 3.01),
                (
                    "/months/3/heating/requiredOutdoorAirM3PerH",
                    284.0968421052632,
                ),
                (
                    "/months/3/heatingReferencePressurePa/0",
                    -1.7858605818751556,
                ),
                ("/months/3/heating/infiltrationM3PerH", 18.93443915934897),
                ("/months/3/heating/naturalSupplyM3PerH", 106.91508387559797),
                ("/months/3/heating/purgeM3PerH", 17.070475576776147),
                (
                    "/months/3/heating/mechanicalSupplyM3PerH",
                    284.0968421052632,
                ),
                ("/months/3/heating/mechanicalSupplyTemperatureC", 10.02),
                ("/months/3/fanElectricityKwh", 28.409684210526322),
                (
                    "/months/3/cooling/requiredOutdoorAirM3PerH",
                    284.0968421052632,
                ),
                (
                    "/months/3/coolingReferencePressurePa/0",
                    -3.0535969455115293,
                ),
                ("/months/3/cooling/infiltrationM3PerH", 25.41439669950059),
                ("/months/3/cooling/naturalSupplyM3PerH", 133.17801343027486),
                ("/months/3/cooling/purgeM3PerH", 85.05486572017554),
                (
                    "/months/3/cooling/mechanicalSupplyM3PerH",
                    284.0968421052632,
                ),
                ("/months/3/cooling/mechanicalSupplyTemperatureC", 9.72),
                (
                    "/months/6/heating/requiredOutdoorAirM3PerH",
                    284.0968421052632,
                ),
                (
                    "/months/6/heatingReferencePressurePa/0",
                    0.35549160842209915,
                ),
                ("/months/6/heating/infiltrationM3PerH", 9.990099015040242),
                ("/months/6/heating/naturalSupplyM3PerH", 66.34575192028723),
                ("/months/6/heating/purgeM3PerH", 10.593019214163506),
                (
                    "/months/6/heating/mechanicalSupplyM3PerH",
                    284.0968421052632,
                ),
                ("/months/6/heating/mechanicalSupplyTemperatureC", 18.75),
                ("/months/6/fanElectricityKwh", 29.356673684210527),
                (
                    "/months/6/cooling/requiredOutdoorAirM3PerH",
                    284.0968421052632,
                ),
                (
                    "/months/6/coolingReferencePressurePa/0",
                    -1.0958385052142745,
                ),
                ("/months/6/cooling/infiltrationM3PerH", 22.11473575017643),
                ("/months/6/cooling/naturalSupplyM3PerH", 120.04922493867916),
                ("/months/6/cooling/purgeM3PerH", 153.34018647629608),
                (
                    "/months/6/cooling/mechanicalSupplyM3PerH",
                    284.0968421052632,
                ),
                ("/months/6/cooling/mechanicalSupplyTemperatureC", 18.45),
                (
                    "/months/9/heating/requiredOutdoorAirM3PerH",
                    284.0968421052632,
                ),
                ("/months/9/heatingReferencePressurePa/0", -1.538399334950597),
                ("/months/9/heating/infiltrationM3PerH", 13.431626575189961),
                ("/months/9/heating/naturalSupplyM3PerH", 82.74705303460098),
                ("/months/9/heating/purgeM3PerH", 13.211714350062342),
                (
                    "/months/9/heating/mechanicalSupplyM3PerH",
                    284.0968421052632,
                ),
                ("/months/9/heating/mechanicalSupplyTemperatureC", 11.1),
                ("/months/9/fanElectricityKwh", 29.356673684210527),
                (
                    "/months/9/cooling/requiredOutdoorAirM3PerH",
                    284.0968421052632,
                ),
                (
                    "/months/9/coolingReferencePressurePa/0",
                    -2.7748856985869708,
                ),
                ("/months/9/cooling/infiltrationM3PerH", 20.127191179857828),
                ("/months/9/cooling/naturalSupplyM3PerH", 111.90204679499202),
                ("/months/9/cooling/purgeM3PerH", 71.46685341528902),
                (
                    "/months/9/cooling/mechanicalSupplyM3PerH",
                    284.0968421052632,
                ),
                ("/months/9/cooling/mechanicalSupplyTemperatureC", 10.8),
            ],
        );
        close("frost-year", &a["annualFrostProtectionElectricityKwh"], 0.0);
    }

    /// System E.1: D.5b part with decentral heat recovery and C.1 part (11.29-11.45, 11.50/11.51, p. 455-463;
    /// 11.87/11.88, 11.93-11.96, p. 489-492), declared efficiency with table 11.17 and 11.109 with a
    /// specified insulation (p. 500-501), f_prac;hr capped at 0,9, partial bypass in cooling (p. 502),
    /// forfait fans 11.139-11.141 (p. 517-518) and frost protection on the D.5b part (11.105/11.106, p. 497-498).
    /// 11.32 with 11.93 is read with f_BAL-DEC applied once (a literal reading applies it twice).
    #[test]
    fn combined_e1_decentral_d5b_with_c1() {
        let a = actual(
            r#"{"zoneId": "z1", "usableFloorAreaM2": 120.0, "category": "residential", "functions": [{"function": "residential", "areaM2": 120.0, "swimmingPool": false}], "dwellingCount": 1, "wholeDwellingAreaM2": null, "apartmentBuilding": false, "buildingHeightM": 9.0, "constructionYear": 2018, "floorAboveCrawlspace": false, "heatingSetpointC": 20.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.5, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2018}, "sourceReference": "r77 synthetic", "system": {"kind": "combined", "decentralAreaM2": 30.0, "totalResidenceAreaM2": 80.0, "decentral": {"variant": "d5b", "heatRecovery": {"efficiency": {"method": "declared", "value": 0.86, "standard": "en13141_8", "sourceReference": "r77 synthetic"}, "bypass": {"kind": "partial", "fraction": 0.6}, "layout": "decentral", "constantVolumeControl": false, "supplyDuctLengthM": 0.5, "supplyDuctInsulation": {"kind": "specified", "thicknessM": 0.02, "conductivityWPerMK": 0.04}, "manufactureYear": 2018, "equipmentReference": "r77 synthetic"}, "ducts": "no_ducts", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}, "other": {"variant": "c1", "heatRecovery": null, "ducts": "luka_a_b_c", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#,
        );
        check(
            &a,
            &[
                (
                    "/months/0/heating/requiredOutdoorAirM3PerH",
                    156.61136842105265,
                ),
                ("/months/0/heatingReferencePressurePa/0", -5.745974359058096),
                ("/months/0/heating/infiltrationM3PerH", 45.73375957917289),
                ("/months/0/heating/naturalSupplyM3PerH", 96.2712389895571),
                ("/months/0/heating/purgeM3PerH", 35.690800796128485),
                (
                    "/months/0/heating/mechanicalSupplyM3PerH",
                    55.93263157894738,
                ),
                (
                    "/months/0/heating/mechanicalExtractM3PerH",
                    156.61136842105265,
                ),
                ("/months/0/heating/mechanicalSupplyTemperatureC", 16.11506),
                ("/months/0/fanElectricityKwh", 22.2518652631579),
                (
                    "/months/0/cooling/requiredOutdoorAirM3PerH",
                    156.61136842105265,
                ),
                ("/months/0/coolingReferencePressurePa/0", -6.462401631785369),
                ("/months/0/cooling/infiltrationM3PerH", 45.73375957917289),
                ("/months/0/cooling/naturalSupplyM3PerH", 96.2712389895571),
                ("/months/0/cooling/purgeM3PerH", 35.690800796128485),
                (
                    "/months/0/cooling/mechanicalSupplyM3PerH",
                    55.93263157894738,
                ),
                (
                    "/months/0/cooling/mechanicalExtractM3PerH",
                    156.61136842105265,
                ),
                (
                    "/months/0/cooling/mechanicalSupplyTemperatureC",
                    19.51265364,
                ),
                (
                    "/months/3/heating/requiredOutdoorAirM3PerH",
                    156.61136842105265,
                ),
                ("/months/3/heatingReferencePressurePa/0", -4.374847959673417),
                ("/months/3/heating/infiltrationM3PerH", 45.391375976448046),
                ("/months/3/heating/naturalSupplyM3PerH", 95.73286831393561),
                ("/months/3/heating/purgeM3PerH", 35.49120971638588),
                (
                    "/months/3/heating/mechanicalSupplyM3PerH",
                    55.93263157894738,
                ),
                (
                    "/months/3/heating/mechanicalExtractM3PerH",
                    156.61136842105265,
                ),
                ("/months/3/heating/mechanicalSupplyTemperatureC", 17.58632),
                ("/months/3/fanElectricityKwh", 21.53406315789474),
                (
                    "/months/3/cooling/requiredOutdoorAirM3PerH",
                    156.61136842105265,
                ),
                ("/months/3/coolingReferencePressurePa/0", -4.52877523240069),
                ("/months/3/cooling/infiltrationM3PerH", 40.80428515592969),
                ("/months/3/cooling/naturalSupplyM3PerH", 88.4164413145367),
                ("/months/3/cooling/purgeM3PerH", 131.11511297375196),
                (
                    "/months/3/cooling/mechanicalSupplyM3PerH",
                    55.93263157894738,
                ),
                (
                    "/months/3/cooling/mechanicalExtractM3PerH",
                    156.61136842105265,
                ),
                ("/months/3/cooling/mechanicalSupplyTemperatureC", 19.0371024),
                (
                    "/months/6/heating/requiredOutdoorAirM3PerH",
                    156.61136842105265,
                ),
                ("/months/6/heatingReferencePressurePa/0", -2.186459261273839),
                ("/months/6/heating/infiltrationM3PerH", 37.51418267854994),
                ("/months/6/heating/naturalSupplyM3PerH", 83.03984877347092),
                ("/months/6/heating/purgeM3PerH", 30.78550491114044),
                (
                    "/months/6/heating/mechanicalSupplyM3PerH",
                    55.93263157894738,
                ),
                (
                    "/months/6/heating/mechanicalExtractM3PerH",
                    156.61136842105265,
                ),
                ("/months/6/heating/mechanicalSupplyTemperatureC", 19.5593),
                ("/months/6/fanElectricityKwh", 22.2518652631579),
                (
                    "/months/6/cooling/requiredOutdoorAirM3PerH",
                    156.61136842105265,
                ),
                (
                    "/months/6/coolingReferencePressurePa/0",
                    -2.2153865340011123,
                ),
                ("/months/6/cooling/infiltrationM3PerH", 31.246197353058157),
                ("/months/6/cooling/naturalSupplyM3PerH", 72.44928001263105),
                ("/months/6/cooling/purgeM3PerH", 214.87396218380331),
                (
                    "/months/6/cooling/mechanicalSupplyM3PerH",
                    55.93263157894738,
                ),
                (
                    "/months/6/cooling/mechanicalExtractM3PerH",
                    156.61136842105265,
                ),
                ("/months/6/cooling/mechanicalSupplyTemperatureC", 20.29212),
                (
                    "/months/9/heating/requiredOutdoorAirM3PerH",
                    156.61136842105265,
                ),
                ("/months/9/heatingReferencePressurePa/0", -3.853673854181366),
                ("/months/9/heating/infiltrationM3PerH", 40.732964006697266),
                ("/months/9/heating/naturalSupplyM3PerH", 88.30108612045784),
                ("/months/9/heating/purgeM3PerH", 32.73601241538925),
                (
                    "/months/9/heating/mechanicalSupplyM3PerH",
                    55.93263157894738,
                ),
                (
                    "/months/9/heating/mechanicalExtractM3PerH",
                    156.61136842105265,
                ),
                ("/months/9/heating/mechanicalSupplyTemperatureC", 17.8756),
                ("/months/9/fanElectricityKwh", 22.2518652631579),
                (
                    "/months/9/cooling/requiredOutdoorAirM3PerH",
                    156.61136842105265,
                ),
                ("/months/9/coolingReferencePressurePa/0", -4.077913626908639),
                ("/months/9/cooling/infiltrationM3PerH", 36.49609601072952),
                ("/months/9/cooling/naturalSupplyM3PerH", 81.35220465222125),
                ("/months/9/cooling/purgeM3PerH", 120.6393668989037),
                (
                    "/months/9/cooling/mechanicalSupplyM3PerH",
                    55.93263157894738,
                ),
                (
                    "/months/9/cooling/mechanicalExtractM3PerH",
                    156.61136842105265,
                ),
                (
                    "/months/9/cooling/mechanicalSupplyTemperatureC",
                    17.886742399999996,
                ),
            ],
        );
        close(
            "frost-1",
            &a["months"][0]["frostProtectionElectricityKwh"],
            2.942659486686316,
        );
        close(
            "frost-2",
            &a["months"][1]["frostProtectionElectricityKwh"],
            2.6578859879747374,
        );
        close(
            "frost-11",
            &a["months"][10]["frostProtectionElectricityKwh"],
            2.84773498711579,
        );
    }

    /// The same E.1 zone with maximum use of the installed capacity for cooling: 11.54 (p. 465) with table 11.6
    /// and tau_sysC of table 11.7 in 11.48 (p. 459).
    #[test]
    fn combined_e1_with_maximum_capacity_for_cooling() {
        let a = actual(
            r#"{"zoneId": "z1", "usableFloorAreaM2": 120.0, "category": "residential", "functions": [{"function": "residential", "areaM2": 120.0, "swimmingPool": false}], "dwellingCount": 1, "wholeDwellingAreaM2": null, "apartmentBuilding": false, "buildingHeightM": 9.0, "constructionYear": 2018, "floorAboveCrawlspace": false, "heatingSetpointC": 20.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": "r77 synthetic", "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.5, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2018}, "sourceReference": "r77 synthetic", "system": {"kind": "combined", "decentralAreaM2": 30.0, "totalResidenceAreaM2": 80.0, "decentral": {"variant": "d5b", "heatRecovery": {"efficiency": {"method": "declared", "value": 0.86, "standard": "en13141_8", "sourceReference": "r77 synthetic"}, "bypass": {"kind": "partial", "fraction": 0.6}, "layout": "decentral", "constantVolumeControl": false, "supplyDuctLengthM": 0.5, "supplyDuctInsulation": {"kind": "specified", "thicknessM": 0.02, "conductivityWPerMK": 0.04}, "manufactureYear": 2018, "equipmentReference": "r77 synthetic"}, "ducts": "no_ducts", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}, "other": {"variant": "c1", "heatRecovery": null, "ducts": "luka_a_b_c", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#,
        );
        check(
            &a,
            &[
                ("/months/3/cooling/requiredOutdoorAirM3PerH", 166.9248),
                ("/months/3/coolingReferencePressurePa/0", -4.56002523240069),
                ("/months/3/cooling/mechanicalExtractM3PerH", 166.9248),
                ("/months/3/cooling/naturalSupplyM3PerH", 94.68907326655719),
                (
                    "/months/5/cooling/requiredOutdoorAirM3PerH",
                    186.86410105263158,
                ),
                ("/months/5/coolingReferencePressurePa/0", -2.741650902275552),
                (
                    "/months/5/cooling/mechanicalExtractM3PerH",
                    186.86410105263158,
                ),
                ("/months/5/cooling/naturalSupplyM3PerH", 92.16543597561001),
                (
                    "/months/6/cooling/requiredOutdoorAirM3PerH",
                    190.98947368421054,
                ),
                (
                    "/months/6/coolingReferencePressurePa/0",
                    -2.2935115340011123,
                ),
                (
                    "/months/6/cooling/mechanicalExtractM3PerH",
                    190.98947368421054,
                ),
                ("/months/6/cooling/naturalSupplyM3PerH", 89.9139093087999),
                (
                    "/months/8/cooling/requiredOutdoorAirM3PerH",
                    187.2078821052632,
                ),
                (
                    "/months/8/coolingReferencePressurePa/0",
                    -2.9173236636732867,
                ),
                (
                    "/months/8/cooling/mechanicalExtractM3PerH",
                    187.2078821052632,
                ),
                ("/months/8/cooling/naturalSupplyM3PerH", 92.84132015698373),
            ],
        );
        check(
            &a,
            &[(
                "/months/0/heating/requiredOutdoorAirM3PerH",
                156.61136842105265,
            )],
        );
    }

    /// System E.1 with the other part natural (A.2a: 11.34-11.36), mechanical supply (B.2: 11.37-11.39, no
    /// division by f_lea;du) or balanced without heat recovery (D.1: 11.43-11.45 with 11.97/11.98, f_lea;du
    /// 1,10 of table 11.9 as the highest value, p. 490), f_ctrl;overig in 11.51 and f_systype per part in
    /// 11.140/11.141 (p. 518).
    #[test]
    fn combined_e1_with_natural_supply_or_balanced_other_part() {
        // natural
        let a = actual(
            r#"{"zoneId": "z1", "usableFloorAreaM2": 120.0, "category": "residential", "functions": [{"function": "residential", "areaM2": 120.0, "swimmingPool": false}], "dwellingCount": 1, "wholeDwellingAreaM2": null, "apartmentBuilding": false, "buildingHeightM": 9.0, "constructionYear": 2018, "floorAboveCrawlspace": false, "heatingSetpointC": 20.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.5, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2018}, "sourceReference": "r77 synthetic", "system": {"kind": "combined", "decentralAreaM2": 30.0, "totalResidenceAreaM2": 80.0, "decentral": {"variant": "d5b", "heatRecovery": {"efficiency": {"method": "declared", "value": 0.86, "standard": "en13141_8", "sourceReference": "r77 synthetic"}, "bypass": {"kind": "partial", "fraction": 0.6}, "layout": "decentral", "constantVolumeControl": false, "supplyDuctLengthM": 0.5, "supplyDuctInsulation": {"kind": "specified", "thicknessM": 0.02, "conductivityWPerMK": 0.04}, "manufactureYear": 2018, "equipmentReference": "r77 synthetic"}, "ducts": "no_ducts", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}, "other": {"variant": "a2a", "heatRecovery": null, "ducts": "no_ducts", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#,
        );
        check(
            &a,
            &[
                (
                    "/months/0/heating/requiredOutdoorAirM3PerH",
                    132.10105263157897,
                ),
                ("/months/0/heatingReferencePressurePa/0", -4.308474359058096),
                ("/months/0/heating/naturalSupplyM3PerH", 128.75421207651323),
                (
                    "/months/0/heating/mechanicalSupplyM3PerH",
                    49.53789473684211,
                ),
                (
                    "/months/0/heating/mechanicalExtractM3PerH",
                    49.53789473684211,
                ),
                ("/months/0/heating/mechanicalSupplyTemperatureC", 16.11506),
                ("/months/0/fanElectricityKwh", 10.237831578947372),
                (
                    "/months/0/cooling/requiredOutdoorAirM3PerH",
                    132.10105263157897,
                ),
                ("/months/0/coolingReferencePressurePa/0", -5.009276631785369),
                ("/months/0/cooling/naturalSupplyM3PerH", 128.33992589961827),
                (
                    "/months/0/cooling/mechanicalSupplyM3PerH",
                    49.53789473684211,
                ),
                (
                    "/months/0/cooling/mechanicalExtractM3PerH",
                    49.53789473684211,
                ),
                (
                    "/months/0/cooling/mechanicalSupplyTemperatureC",
                    19.51265364,
                ),
                (
                    "/months/6/heating/requiredOutdoorAirM3PerH",
                    132.10105263157897,
                ),
                (
                    "/months/6/heatingReferencePressurePa/0",
                    -1.0223967612738392,
                ),
                ("/months/6/heating/naturalSupplyM3PerH", 108.11792546677628),
                (
                    "/months/6/heating/mechanicalSupplyM3PerH",
                    49.53789473684211,
                ),
                (
                    "/months/6/heating/mechanicalExtractM3PerH",
                    49.53789473684211,
                ),
                ("/months/6/heating/mechanicalSupplyTemperatureC", 19.5593),
                ("/months/6/fanElectricityKwh", 10.237831578947372),
                (
                    "/months/6/cooling/requiredOutdoorAirM3PerH",
                    132.10105263157897,
                ),
                (
                    "/months/6/coolingReferencePressurePa/0",
                    -1.6606990340011123,
                ),
                ("/months/6/cooling/naturalSupplyM3PerH", 105.6263843441505),
                (
                    "/months/6/cooling/mechanicalSupplyM3PerH",
                    49.53789473684211,
                ),
                (
                    "/months/6/cooling/mechanicalExtractM3PerH",
                    49.53789473684211,
                ),
                ("/months/6/cooling/mechanicalSupplyTemperatureC", 20.29212),
            ],
        );
        // supply
        let a = actual(
            r#"{"zoneId": "z1", "usableFloorAreaM2": 120.0, "category": "residential", "functions": [{"function": "residential", "areaM2": 120.0, "swimmingPool": false}], "dwellingCount": 1, "wholeDwellingAreaM2": null, "apartmentBuilding": false, "buildingHeightM": 9.0, "constructionYear": 2018, "floorAboveCrawlspace": false, "heatingSetpointC": 20.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.5, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2018}, "sourceReference": "r77 synthetic", "system": {"kind": "combined", "decentralAreaM2": 30.0, "totalResidenceAreaM2": 80.0, "decentral": {"variant": "d5b", "heatRecovery": {"efficiency": {"method": "declared", "value": 0.86, "standard": "en13141_8", "sourceReference": "r77 synthetic"}, "bypass": {"kind": "partial", "fraction": 0.6}, "layout": "decentral", "constantVolumeControl": false, "supplyDuctLengthM": 0.5, "supplyDuctInsulation": {"kind": "specified", "thicknessM": 0.02, "conductivityWPerMK": 0.04}, "manufactureYear": 2018, "equipmentReference": "r77 synthetic"}, "ducts": "no_ducts", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}, "other": {"variant": "b2", "heatRecovery": null, "ducts": "luka_a_b_c", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#,
        );
        check(
            &a,
            &[
                (
                    "/months/0/heating/requiredOutdoorAirM3PerH",
                    138.70610526315792,
                ),
                (
                    "/months/0/heatingReferencePressurePa/0",
                    -3.1053493590580956,
                ),
                ("/months/0/heating/naturalSupplyM3PerH", 48.04921774156509),
                (
                    "/months/0/heating/mechanicalSupplyM3PerH",
                    136.22921052631582,
                ),
                (
                    "/months/0/heating/mechanicalExtractM3PerH",
                    52.01478947368422,
                ),
                (
                    "/months/0/heating/mechanicalSupplyTemperatureC",
                    7.9663854545454535,
                ),
                ("/months/0/fanElectricityKwh", 19.707825789473684),
                (
                    "/months/0/cooling/requiredOutdoorAirM3PerH",
                    138.70610526315792,
                ),
                (
                    "/months/0/coolingReferencePressurePa/0",
                    -3.8217766317853687,
                ),
                ("/months/0/cooling/naturalSupplyM3PerH", 48.04921774156509),
                (
                    "/months/0/cooling/mechanicalSupplyM3PerH",
                    136.22921052631582,
                ),
                (
                    "/months/0/cooling/mechanicalExtractM3PerH",
                    52.01478947368422,
                ),
                (
                    "/months/0/cooling/mechanicalSupplyTemperatureC",
                    9.010964959999999,
                ),
                (
                    "/months/6/heating/requiredOutdoorAirM3PerH",
                    138.70610526315792,
                ),
                (
                    "/months/6/heatingReferencePressurePa/0",
                    0.00885323872616084,
                ),
                ("/months/6/heating/naturalSupplyM3PerH", 35.83789234832397),
                (
                    "/months/6/heating/mechanicalSupplyM3PerH",
                    136.22921052631582,
                ),
                (
                    "/months/6/heating/mechanicalExtractM3PerH",
                    52.01478947368422,
                ),
                (
                    "/months/6/heating/mechanicalSupplyTemperatureC",
                    19.044290909090908,
                ),
                ("/months/6/fanElectricityKwh", 19.707825789473684),
                (
                    "/months/6/cooling/requiredOutdoorAirM3PerH",
                    138.70610526315792,
                ),
                (
                    "/months/6/coolingReferencePressurePa/0",
                    -1.1919490340011123,
                ),
                ("/months/6/cooling/naturalSupplyM3PerH", 46.84464310326712),
                (
                    "/months/6/cooling/mechanicalSupplyM3PerH",
                    136.22921052631582,
                ),
                (
                    "/months/6/cooling/mechanicalExtractM3PerH",
                    52.01478947368422,
                ),
                (
                    "/months/6/cooling/mechanicalSupplyTemperatureC",
                    19.119861818181818,
                ),
            ],
        );
        // balanced
        let a = actual(
            r#"{"zoneId": "z1", "usableFloorAreaM2": 120.0, "category": "residential", "functions": [{"function": "residential", "areaM2": 120.0, "swimmingPool": false}], "dwellingCount": 1, "wholeDwellingAreaM2": null, "apartmentBuilding": false, "buildingHeightM": 9.0, "constructionYear": 2018, "floorAboveCrawlspace": false, "heatingSetpointC": 20.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.5, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2018}, "sourceReference": "r77 synthetic", "system": {"kind": "combined", "decentralAreaM2": 30.0, "totalResidenceAreaM2": 80.0, "decentral": {"variant": "d5b", "heatRecovery": {"efficiency": {"method": "declared", "value": 0.86, "standard": "en13141_8", "sourceReference": "r77 synthetic"}, "bypass": {"kind": "partial", "fraction": 0.6}, "layout": "decentral", "constantVolumeControl": false, "supplyDuctLengthM": 0.5, "supplyDuctInsulation": {"kind": "specified", "thicknessM": 0.02, "conductivityWPerMK": 0.04}, "manufactureYear": 2018, "equipmentReference": "r77 synthetic"}, "ducts": "no_ducts", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}, "other": {"variant": "d1", "heatRecovery": null, "ducts": "unknown", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#,
        );
        check(
            &a,
            &[
                (
                    "/months/0/heating/requiredOutdoorAirM3PerH",
                    164.06905263157896,
                ),
                ("/months/0/heatingReferencePressurePa/0", -5.105349359058096),
                ("/months/0/heating/naturalSupplyM3PerH", 0.0),
                (
                    "/months/0/heating/mechanicalSupplyM3PerH",
                    149.1536842105263,
                ),
                (
                    "/months/0/heating/mechanicalExtractM3PerH",
                    164.06905263157896,
                ),
                ("/months/0/heating/mechanicalSupplyTemperatureC", 8.1118975),
                ("/months/0/fanElectricityKwh", 33.907604210526316),
                (
                    "/months/0/cooling/requiredOutdoorAirM3PerH",
                    164.06905263157896,
                ),
                ("/months/0/coolingReferencePressurePa/0", -5.821776631785369),
                ("/months/0/cooling/naturalSupplyM3PerH", 0.0),
                (
                    "/months/0/cooling/mechanicalSupplyM3PerH",
                    149.1536842105263,
                ),
                (
                    "/months/0/cooling/mechanicalExtractM3PerH",
                    164.06905263157896,
                ),
                (
                    "/months/0/cooling/mechanicalSupplyTemperatureC",
                    9.198495115,
                ),
                (
                    "/months/6/heating/requiredOutdoorAirM3PerH",
                    164.06905263157896,
                ),
                (
                    "/months/6/heatingReferencePressurePa/0",
                    -1.6161467612738392,
                ),
                ("/months/6/heating/naturalSupplyM3PerH", 0.0),
                (
                    "/months/6/heating/mechanicalSupplyM3PerH",
                    149.1536842105263,
                ),
                (
                    "/months/6/heating/mechanicalExtractM3PerH",
                    164.06905263157896,
                ),
                ("/months/6/heating/mechanicalSupplyTemperatureC", 19.0534875),
                ("/months/6/fanElectricityKwh", 33.907604210526316),
                (
                    "/months/6/cooling/requiredOutdoorAirM3PerH",
                    164.06905263157896,
                ),
                (
                    "/months/6/coolingReferencePressurePa/0",
                    -1.7935115340011123,
                ),
                ("/months/6/cooling/naturalSupplyM3PerH", 0.0),
                (
                    "/months/6/cooling/mechanicalSupplyM3PerH",
                    149.1536842105263,
                ),
                (
                    "/months/6/cooling/mechanicalExtractM3PerH",
                    164.06905263157896,
                ),
                ("/months/6/cooling/mechanicalSupplyTemperatureC", 19.140795),
            ],
        );
    }

    /// System E.1 in a utility zone: f_ctrlE.1 of 11.52 (p. 463) with 0,67 and f_ctrl 1,32 of C.1 (table 11.5).
    #[test]
    fn combined_e1_utility_control_factor() {
        let a = actual(
            r#"{"zoneId": "z1", "usableFloorAreaM2": 500.0, "category": "utility", "functions": [{"function": "office", "areaM2": 500.0, "swimmingPool": false}], "dwellingCount": 0, "wholeDwellingAreaM2": null, "apartmentBuilding": false, "buildingHeightM": 9.0, "constructionYear": 2020, "floorAboveCrawlspace": false, "heatingSetpointC": 21.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2020}, "sourceReference": "r77 synthetic", "system": {"kind": "combined", "decentralAreaM2": 100.0, "totalResidenceAreaM2": 400.0, "decentral": {"variant": "d5b", "heatRecovery": {"efficiency": {"method": "declared", "value": 0.86, "standard": "en13141_8", "sourceReference": "r77 synthetic"}, "bypass": {"kind": "partial", "fraction": 0.6}, "layout": "decentral", "constantVolumeControl": false, "supplyDuctLengthM": 0.5, "supplyDuctInsulation": {"kind": "specified", "thicknessM": 0.02, "conductivityWPerMK": 0.04}, "manufactureYear": 2018, "equipmentReference": "r77 synthetic"}, "ducts": "no_ducts", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}, "other": {"variant": "c1", "heatRecovery": null, "ducts": "luka_a_b_c", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#,
        );
        for m in [0, 6] {
            for balance in ["heating", "cooling"] {
                close(
                    balance,
                    &a["months"][m][balance]["requiredOutdoorAirM3PerH"],
                    766.837657894737,
                );
            }
        }
    }

    /// Utility D.5a with installed capacity (11.55/11.58, p. 466), recirculation and flow control (11.59-11.61,
    /// p. 467-468), tau_sysC (11.48, table 11.7), AHU in the zone (f_lea;ahu 1,02, p. 473), recirculation jump
    /// 11.110-11.113 (p. 503) in the heating balance, partial bypass, and declared fans 11.133/11.135 with
    /// tables 11.20 and 11.22 times f_tau (p. 513-517).
    #[test]
    fn utility_installed_capacity_recirculation_and_declared_fans() {
        let a = actual(
            r#"{"zoneId": "z1", "usableFloorAreaM2": 500.0, "category": "utility", "functions": [{"function": "office", "areaM2": 500.0, "swimmingPool": false}], "dwellingCount": 0, "wholeDwellingAreaM2": null, "apartmentBuilding": false, "buildingHeightM": 12.0, "constructionYear": 2010, "floorAboveCrawlspace": false, "heatingSetpointC": 21.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": "r77 synthetic", "installedCapacity": {"totalDm3PerS": 2000.0, "naturalSupplyDm3PerS": 0.0, "sourceReference": "r77 synthetic"}, "flowReduction": {"collective": true, "recirculationPercent": 30, "flowControlPercent": 60, "evidenceReference": "r77 synthetic"}, "infiltration": {"method": "measured", "qv10DmPerSM2": 1.0, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "declared", "fans": [{"id": "f1", "power": {"method": "motor", "motorPowerW": 2500.0, "manufactureYear": 2010}}, {"id": "f2", "power": {"method": "nominal", "nominalPowerW": 400.0}}], "control": {"method": "flow_control", "control": "speed_control"}, "buildingShare": 0.8, "sourceReference": "r77 synthetic"}, "sourceReference": "r77 synthetic", "system": {"kind": "single", "unit": {"variant": "d5a", "heatRecovery": {"efficiency": {"method": "table", "exchanger": "rotary"}, "bypass": {"kind": "partial", "fraction": 0.5}, "layout": "central", "constantVolumeControl": true, "supplyDuctLengthM": null, "supplyDuctInsulation": {"kind": "insulated"}, "manufactureYear": 2010, "equipmentReference": "r77 synthetic"}, "ducts": "luka_d", "airHandlingUnit": {"insideThermalZone": true, "supplyDuctsOutside": "none", "heatingCoil": false, "coolingCoil": false}, "equipmentReference": "r77 synthetic"}}}"#,
        );
        check(
            &a,
            &[
                (
                    "/months/0/heating/requiredOutdoorAirM3PerH",
                    639.8147368421054,
                ),
                (
                    "/months/0/heating/mechanicalSupplyM3PerH",
                    627.2693498452013,
                ),
                (
                    "/months/0/heating/mechanicalExtractM3PerH",
                    627.2693498452013,
                ),
                (
                    "/months/0/heating/mechanicalSupplyTemperatureC",
                    15.65686909090909,
                ),
                ("/months/0/fanElectricityKwh", 258.91200000000003),
                (
                    "/months/0/cooling/requiredOutdoorAirM3PerH",
                    1530.8715789473686,
                ),
                (
                    "/months/0/cooling/mechanicalSupplyM3PerH",
                    1500.8544891640868,
                ),
                (
                    "/months/0/cooling/mechanicalExtractM3PerH",
                    1500.8544891640868,
                ),
                ("/months/0/cooling/mechanicalSupplyTemperatureC", 16.5884842),
                (
                    "/months/3/heating/requiredOutdoorAirM3PerH",
                    639.8147368421054,
                ),
                (
                    "/months/3/heating/mechanicalSupplyM3PerH",
                    627.2693498452013,
                ),
                (
                    "/months/3/heating/mechanicalExtractM3PerH",
                    627.2693498452013,
                ),
                (
                    "/months/3/heating/mechanicalSupplyTemperatureC",
                    17.60642909090909,
                ),
                ("/months/3/fanElectricityKwh", 250.56000000000003),
                (
                    "/months/3/cooling/requiredOutdoorAirM3PerH",
                    1748.4631578947367,
                ),
                (
                    "/months/3/cooling/mechanicalSupplyM3PerH",
                    1714.1795665634672,
                ),
                (
                    "/months/3/cooling/mechanicalExtractM3PerH",
                    1714.1795665634672,
                ),
                ("/months/3/cooling/mechanicalSupplyTemperatureC", 17.762232),
                (
                    "/months/6/heating/requiredOutdoorAirM3PerH",
                    639.8147368421054,
                ),
                (
                    "/months/6/heating/mechanicalSupplyM3PerH",
                    627.2693498452013,
                ),
                (
                    "/months/6/heating/mechanicalExtractM3PerH",
                    627.2693498452013,
                ),
                (
                    "/months/6/heating/mechanicalSupplyTemperatureC",
                    20.14289090909091,
                ),
                ("/months/6/fanElectricityKwh", 258.91200000000003),
                (
                    "/months/6/cooling/requiredOutdoorAirM3PerH",
                    2273.684210526316,
                ),
                (
                    "/months/6/cooling/mechanicalSupplyM3PerH",
                    2229.1021671826625,
                ),
                (
                    "/months/6/cooling/mechanicalExtractM3PerH",
                    2229.1021671826625,
                ),
                ("/months/6/cooling/mechanicalSupplyTemperatureC", 20.5469),
                (
                    "/months/9/heating/requiredOutdoorAirM3PerH",
                    639.8147368421054,
                ),
                (
                    "/months/9/heating/mechanicalSupplyM3PerH",
                    627.2693498452013,
                ),
                (
                    "/months/9/heating/mechanicalExtractM3PerH",
                    627.2693498452013,
                ),
                (
                    "/months/9/heating/mechanicalSupplyTemperatureC",
                    17.920218181818182,
                ),
                ("/months/9/fanElectricityKwh", 258.91200000000003),
                (
                    "/months/9/cooling/requiredOutdoorAirM3PerH",
                    1943.5452631578949,
                ),
                ("/months/9/cooling/mechanicalSupplyM3PerH", 1905.43653250774),
                (
                    "/months/9/cooling/mechanicalExtractM3PerH",
                    1905.43653250774,
                ),
                ("/months/9/cooling/mechanicalSupplyTemperatureC", 17.134048),
            ],
        );
    }

    /// Dwelling D.2 with full bypass and cold recovery (11.106a, p. 498, tau_C of table 11.7 and the WTWC
    /// column of table 17.1) and the duct length 0,5 H of an apartment building in 11.109 (p. 500).
    #[test]
    fn cold_recovery_with_full_bypass_and_apartment_duct_length() {
        let a = actual(
            r#"{"zoneId": "z1", "usableFloorAreaM2": 100.0, "category": "residential", "functions": [{"function": "residential", "areaM2": 100.0, "swimmingPool": false}], "dwellingCount": 1, "wholeDwellingAreaM2": null, "apartmentBuilding": true, "buildingHeightM": 12.0, "constructionYear": 2020, "floorAboveCrawlspace": false, "heatingSetpointC": 20.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2020}, "sourceReference": "r77 synthetic", "system": {"kind": "single", "unit": {"variant": "d2", "heatRecovery": {"efficiency": {"method": "table", "exchanger": "counter_flow_plastic"}, "bypass": {"kind": "full", "coldRecoveryEvidence": "r77 synthetic"}, "layout": "central", "constantVolumeControl": false, "supplyDuctLengthM": null, "supplyDuctInsulation": {"kind": "insulated"}, "manufactureYear": 2020, "equipmentReference": "r77 synthetic"}, "ducts": "luka_a_b_c", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#,
        );
        check(
            &a,
            &[
                ("/months/0/cooling/mechanicalSupplyTemperatureC", 3.21),
                (
                    "/months/1/cooling/mechanicalSupplyTemperatureC",
                    5.420000000000001,
                ),
                (
                    "/months/2/cooling/mechanicalSupplyTemperatureC",
                    6.3100000000000005,
                ),
                ("/months/3/cooling/mechanicalSupplyTemperatureC", 9.72),
                ("/months/4/cooling/mechanicalSupplyTemperatureC", 14.9344),
                ("/months/5/cooling/mechanicalSupplyTemperatureC", 16.29664),
                ("/months/6/cooling/mechanicalSupplyTemperatureC", 18.13176),
                ("/months/7/cooling/mechanicalSupplyTemperatureC", 18.43256),
                (
                    "/months/8/cooling/mechanicalSupplyTemperatureC",
                    15.946800000000001,
                ),
                ("/months/9/cooling/mechanicalSupplyTemperatureC", 11.0),
                ("/months/10/cooling/mechanicalSupplyTemperatureC", 8.59),
                (
                    "/months/11/cooling/mechanicalSupplyTemperatureC",
                    4.6000000000000005,
                ),
                (
                    "/months/0/heating/mechanicalSupplyTemperatureC",
                    14.224160000000001,
                ),
                ("/months/3/heating/mechanicalSupplyTemperatureC", 16.41152),
                ("/months/10/heating/mechanicalSupplyTemperatureC", 16.03184),
            ],
        );
    }

    /// Overventilation 11.23/11.23a (p. 449) and f_buitenlucht 11.24 (p. 451) for a C.1 dwelling with an
    /// exhaust-air heat pump for heating.
    #[test]
    fn overventilation_and_outdoor_air_fraction() {
        let a = actual(
            r#"{"zoneId": "z1", "usableFloorAreaM2": 100.0, "category": "residential", "functions": [{"function": "residential", "areaM2": 100.0, "swimmingPool": false}], "dwellingCount": 1, "wholeDwellingAreaM2": null, "apartmentBuilding": false, "buildingHeightM": 9.0, "constructionYear": 2020, "floorAboveCrawlspace": false, "heatingSetpointC": 20.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": {"heatingTimeFraction": [0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3], "hotWaterTimeFraction": [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], "heatingFlowM3PerH": 230.0, "heatingAreaShare": 1.0, "hotWaterFlowM3PerH": [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], "sourceReference": "r77 synthetic"}, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2020}, "sourceReference": "r77 synthetic", "system": {"kind": "single", "unit": {"variant": "c1", "heatRecovery": null, "ducts": "luka_a_b_c", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#,
        );
        for m in [0, 5, 11] {
            close(
                "req-H",
                &a["months"][m]["heating"]["requiredOutdoorAirM3PerH"],
                180.4105263157895,
            );
            close(
                "f-outdoor",
                &a["months"][m]["outdoorAirFraction"],
                0.30800915331807766,
            );
        }
    }

    /// 11.85/11.86 (p. 485) with spot rows of tables 11.13 and 11.14 (p. 486-489), including a renovation year.
    #[test]
    fn reference_infiltration_rows_of_tables_11_13_and_11_14() {
        let cases: [(&str, i32, Option<i32>, f64); 7] = [
            ("flat-roof-detached", 1969, None, 226.28190755291618),
            ("storey-end-top", 1985, None, 107.75328931091249),
            (
                "multi-storey-whole-bottom-layer",
                2005,
                None,
                42.33164937214419,
            ),
            (
                "pitched-roof-detached-partly-flat",
                1960,
                Some(1995),
                138.53994339974463,
            ),
            (
                "multi-storey-whole-top-layer",
                2012,
                None,
                35.019819026046555,
            ),
            (
                "storey-middle-lower-or-intermediate",
                1970,
                None,
                96.20829402760043,
            ),
            ("flat-roof-end-or-corner", 2010, None, 45.256381510583246),
        ];
        let template: Value = serde_json::from_str(r#"{"zoneId": "z1", "usableFloorAreaM2": 100.0, "category": "residential", "functions": [{"function": "residential", "areaM2": 100.0, "swimmingPool": false}], "dwellingCount": 1, "wholeDwellingAreaM2": null, "apartmentBuilding": false, "buildingHeightM": 9.0, "constructionYear": 2020, "floorAboveCrawlspace": false, "heatingSetpointC": 20.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2020}, "sourceReference": "r77 synthetic", "system": {"kind": "single", "unit": {"variant": "c1", "heatRecovery": null, "ducts": "luka_a_b_c", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#).expect("template");
        for (kind, year, renovation, expected) in cases {
            let mut input = template.clone();
            input["constructionYear"] = serde_json::json!(year);
            input["infiltration"] = serde_json::json!({
                "method": "reference",
                "buildingType": kind.replace('-', "_"),
                "renovationYear": renovation,
            });
            let a = actual(&input.to_string());
            close(kind, &a["infiltrationQV1M3PerH"], expected);
        }
    }

    /// 11.133 with a declared monthly f_regfan and f_gebouw;si;V 0,5, 11.135 with rows of table 11.20 (p. 516)
    /// and 11.136 with a measured efficiency of exactly 0,75 (rounded down to a multiple of 0,025, p. 515).
    #[test]
    fn declared_fans_motor_rows_and_monthly_regulation() {
        let a = actual(
            r#"{"zoneId": "z1", "usableFloorAreaM2": 100.0, "category": "residential", "functions": [{"function": "residential", "areaM2": 100.0, "swimmingPool": false}], "dwellingCount": 1, "wholeDwellingAreaM2": null, "apartmentBuilding": false, "buildingHeightM": 9.0, "constructionYear": 2020, "floorAboveCrawlspace": false, "heatingSetpointC": 20.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "declared", "fans": [{"id": "m1", "power": {"method": "motor", "motorPowerW": 800.0}}, {"id": "m2", "power": {"method": "motor", "motorPowerW": 15000.0, "manufactureYear": 2000}}, {"id": "m3", "power": {"method": "motor", "motorPowerW": 150000.0, "manufactureYear": 2010}}, {"id": "m4", "power": {"method": "motor", "motorPowerW": 45000.0, "manufactureYear": 2005}}, {"id": "m5", "power": {"method": "motor", "motorPowerW": 3000.0, "manufactureYear": 2010, "electricalInputW": 4000.0}}], "control": {"method": "declared", "monthly": [0.3, 0.3, 0.35, 0.4, 0.45, 0.5, 0.5, 0.5, 0.45, 0.4, 0.35, 0.3], "sourceReference": "r77 synthetic"}, "buildingShare": 0.5, "sourceReference": "r77 synthetic"}, "sourceReference": "r77 synthetic", "system": {"kind": "single", "unit": {"variant": "c1", "heatRecovery": null, "ducts": "luka_a_b_c", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#,
        );
        let expected = [
            22892.638437723268,
            20677.221814717792,
            26708.07817734381,
            29538.888306739707,
            34338.957656584906,
            36923.610383424624,
            38154.397396205444,
            38154.397396205444,
            33231.249345082164,
            30523.517916964363,
            25846.527268397236,
            22892.638437723268,
        ];
        for (m, value) in expected.iter().enumerate() {
            close("fan", &a["months"][m]["fanElectricityKwh"], *value);
        }
    }

    /// 11.142 (p. 518) with rows of table 11.23 (p. 519) for a C.1 dwelling (f_systype 1), January.
    #[test]
    fn forfait_fan_rows_of_table_11_23() {
        let template: Value = serde_json::from_str(r#"{"zoneId": "z1", "usableFloorAreaM2": 100.0, "category": "residential", "functions": [{"function": "residential", "areaM2": 100.0, "swimmingPool": false}], "dwellingCount": 1, "wholeDwellingAreaM2": null, "apartmentBuilding": false, "buildingHeightM": 9.0, "constructionYear": 2020, "floorAboveCrawlspace": false, "heatingSetpointC": 20.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2020}, "sourceReference": "r77 synthetic", "system": {"kind": "single", "unit": {"variant": "c1", "heatRecovery": null, "ducts": "luka_a_b_c", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#).expect("template");
        let cases: [(&str, Option<i32>, f64); 6] = [
            ("ac", None, 146.18947368421058),
            ("ac", Some(1983), 102.33263157894737),
            ("dc", Some(1990), 47.51157894736843),
            ("ac", Some(1998), 73.09473684210529),
            ("dc", Some(2006), 32.89263157894737),
            ("dc", Some(1980), 80.40421052631581),
        ];
        for (current, year, expected) in cases {
            let mut input = template.clone();
            input["fans"] = serde_json::json!({
                "method": "forfait",
                "current": current,
                "manufactureYear": year,
            });
            let a = actual(&input.to_string());
            close(current, &a["months"][0]["fanElectricityKwh"], expected);
        }
    }

    /// Table 11.22 (p. 517) times f_tau of table 11.8 for a utility zone: throttle at 70 %, inlet vane at 40 %
    /// and speed control without flow control.
    #[test]
    fn utility_fan_regulation_rows_of_table_11_22() {
        let template: Value = serde_json::from_str(r#"{"zoneId": "z1", "usableFloorAreaM2": 500.0, "category": "utility", "functions": [{"function": "office", "areaM2": 500.0, "swimmingPool": false}], "dwellingCount": 0, "wholeDwellingAreaM2": null, "apartmentBuilding": false, "buildingHeightM": 9.0, "constructionYear": 2020, "floorAboveCrawlspace": false, "heatingSetpointC": 20.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "declared", "fans": [{"id": "f", "power": {"method": "nominal", "nominalPowerW": 1000.0}}], "control": {"method": "flow_control", "control": "throttle"}, "buildingShare": 1.0, "sourceReference": "r77 synthetic"}, "sourceReference": "r77 synthetic", "system": {"kind": "single", "unit": {"variant": "c1", "heatRecovery": null, "ducts": "luka_a_b_c", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#).expect("template");
        let cases: [(&str, Option<u32>, f64); 3] = [
            ("throttle", Some(70), 235.6),
            ("inlet-vane-or-blade-pitch", Some(40), 148.8),
            ("speed-control", None, 248.0),
        ];
        for (method, percent, expected) in cases {
            let mut input = template.clone();
            input["fans"]["control"]["control"] = serde_json::json!(method.replace('-', "_"));
            input["flowReduction"] = serde_json::json!({
                "collective": true,
                "recirculationPercent": null,
                "flowControlPercent": percent,
                "evidenceReference": "r77 synthetic",
            });
            let a = actual(&input.to_string());
            close(method, &a["months"][0]["fanElectricityKwh"], expected);
        }
    }

    /// Chapter 12 (p. 520-524): 12.1 with the healthcare column of table 12.1 and eta_HU;rvd 0,55 of a heat
    /// wheel, 12.3 for an electric steam generator, 12.4 zero above 500 m2, 12.5 with the 6/12 column of table
    /// 12.2; then the general column, a gas-fired generator serving 400 m2 (12.4 applies) and the 12/18 column.
    #[test]
    fn humidification_and_dehumidification_spot_rows() {
        let large: HumidityInput = serde_json::from_str(r#"{"zoneId": "z1", "usableFloorAreaM2": 600.0, "installationAreaM2": null, "functions": [{"function": "healthcare", "areaM2": 600.0}], "humidification": {"humidifier": {"kind": "steam", "carrier": "electricity"}, "rotaryWheel": true, "equipmentReference": "r77 synthetic"}, "supplyFlowM3PerH": [2000.0, 2010.0, 2020.0, 2030.0, 2040.0, 2050.0, 2060.0, 2070.0, 2080.0, 2090.0, 2100.0, 2110.0], "coolingDesign": "from6_to12", "coolingNeedKwh": [50.0, 100.0, 150.0, 200.0, 250.0, 300.0, 350.0, 400.0, 450.0, 500.0, 550.0, 600.0]}"#).expect("large");
        let small: HumidityInput = serde_json::from_str(r#"{"zoneId": "z1", "usableFloorAreaM2": 300.0, "installationAreaM2": 400.0, "functions": [{"function": "general", "areaM2": 300.0}], "humidification": {"humidifier": {"kind": "steam", "carrier": "gas"}, "rotaryWheel": false, "equipmentReference": "r77 synthetic"}, "supplyFlowM3PerH": [2000.0, 2010.0, 2020.0, 2030.0, 2040.0, 2050.0, 2060.0, 2070.0, 2080.0, 2090.0, 2100.0, 2110.0], "coolingDesign": "from12_to18", "coolingNeedKwh": [50.0, 100.0, 150.0, 200.0, 250.0, 300.0, 350.0, 400.0, 450.0, 500.0, 550.0, 600.0]}"#).expect("small");
        let large = calculate_humidity(&large).expect("large run");
        let small = calculate_humidity(&small).expect("small run");
        let q_large = [
            1292.2293475,
            968.2544513249998,
            1166.1414070249998,
            682.9699722999999,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            297.0598233749999,
            1177.7638248249998,
        ];
        let dhum_large = [
            0.0, 0.0, 0.0, 0.0, 50.0, 345.0, 644.0, 860.0, 859.5, 0.0, 0.0, 0.0,
        ];
        let q_small = [
            1393.330788888889,
            802.6095099166666,
            1098.3524657777778,
            68.98686588888889,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1057.657011638889,
        ];
        let dhum_small = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0];
        for m in 0..12 {
            // 12.3 with eta 0,8; 12.4 zero above 500 m2.
            close_f64("q-large", large[m].humidification_need_kwh, q_large[m]);
            close_f64("el-large", large[m].steam_electricity_kwh, q_large[m] / 0.8);
            close_f64("rbl-large", large[m].recoverable_kwh, 0.0);
            close_f64(
                "dhum-large",
                large[m].dehumidification_need_kwh,
                dhum_large[m],
            );
            // 12.3 with eta 0,6; 12.4 with (1 - 0,6).
            close_f64("q-small", small[m].humidification_need_kwh, q_small[m]);
            close_f64("gas-small", small[m].steam_fuel_kwh, q_small[m] / 0.6);
            close_f64("rbl-small", small[m].recoverable_kwh, 0.4 * q_small[m]);
            close_f64(
                "dhum-small",
                small[m].dehumidification_need_kwh,
                dhum_small[m],
            );
        }
    }

    /// 12.1 (p. 520-521) takes q_V;SUP;dis;out of 11.3.1.1, which is 11.87 (p. 489) for SUPPLY_OP as well:
    /// q_ODA;req/(f_lea;du x f_lea;ahu). For a utility B.1 zone (f_ctrl 1,02, table 11.5) with an electric
    /// steam humidifier the January electricity follows 12.1 and 12.3 with that flow.
    #[test]
    fn humidification_uses_the_supply_flow_of_11_87_for_a_supply_system() {
        let mut chain: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-space-heating-chain-synthetic.json"
        ))
        .expect("chain");
        let demand = &mut chain["demand"];
        demand["usageFunction"] = serde_json::json!("office");
        demand["dwellingType"] = Value::Null;
        demand["setpoints"]["heatingC"] = serde_json::json!(21.0);
        demand["internalGains"] = serde_json::json!({"method": "declared", "heatFluxWPerM2": 5.5, "sourceReference": "tables 7.2/7.3"});
        demand["ventilationFlows"] = serde_json::json!([]);
        demand["ventilation"] = serde_json::from_str(r#"{"zoneId": "rz-1", "usableFloorAreaM2": 100.0, "category": "utility", "functions": [{"function": "office", "areaM2": 100.0, "swimmingPool": false}], "dwellingCount": 0, "wholeDwellingAreaM2": null, "apartmentBuilding": false, "buildingHeightM": 9.0, "constructionYear": 2020, "floorAboveCrawlspace": false, "heatingSetpointC": 21.0, "coolingSetpointC": 24.0, "maximumCapacityForCooling": null, "installedCapacity": null, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "r77 synthetic"}, "combustionAppliances": [], "overventilation": null, "ventilativeCooling": null, "grillePreheating": null, "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2020}, "sourceReference": "r77 synthetic", "system": {"kind": "single", "unit": {"variant": "b1", "heatRecovery": null, "ducts": "luka_a_b_c", "airHandlingUnit": null, "equipmentReference": "r77 synthetic"}}}"#).expect("ventilation");
        chain["humidifiers"] = serde_json::json!([{"zoneId": "rz-1", "humidification": {"humidifier": {"kind": "steam", "carrier": "electricity"}, "rotaryWheel": false, "equipmentReference": "r77 synthetic"}}]);
        let input: SpaceHeatingChainInput = serde_json::from_value(chain).expect("chain input");
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        close_f64(
            "humidification-1",
            result.monthly[0].humidification_electricity_kwh,
            112.0875953101579,
        );
    }
}

mod part_b {
    // Review 77 part B: chapter 13 (domestic hot water) and annexes T, U and W of NTA 8800:2025+C1:2026,
    // provisions that earlier reviews did not pin with an independent value. The expected values come from a
    // separate implementation written from the norm pages before the kernel's formula code was read; only
    // numbers and page references are kept here. Tolerance 1e-6 relative.

    use nta8800_core::climate::Orientation;
    use nta8800_core::domestic_hot_water::{
        assess_hot_water, assess_hot_water_with, bathroom_emission, en16147_corrected_input,
        european_profile_correction, gas_class_correction, heat_pump_class_correction,
        kitchen_emission, residential_need_fractions, shower_share, table_13_4_psi,
        utility_specific_need, validate_hot_water, winter_gas_consumption_kwh,
        AdditionalHotWaterGenerator, ApplicationClass, BiomassStoreInsulation, Circulation,
        CirculationPump, CollectiveHotWater, ConnectedTaps, DeclaredGeneratorShare, DeliverySets,
        ExhaustAirUse, GasAppliance, HotWaterAssessment, HotWaterContext, HotWaterEmission,
        HotWaterExtras, HotWaterGenerator, HotWaterMonth, HotWaterNeed, HotWaterSystem,
        IndirectBoiler, MixedAirCorrection, MixedAirVentilation, PfhrdTest, PipeInsulation,
        ProfileTest, PumpControl, SeriesArrangement, ServedTaps, SharePoint, ShowerConnection,
        ShowerUnit, StorageLabel, StorageLoss, StorageVessel, TappingProfile, TestProfile,
        TwoProfileStandard, TwoProfileTest, UtilityArea,
    };
    use nta8800_core::heating_distribution::PipeGeometry;
    use nta8800_core::hot_water_tests::{
        water_density, water_enthalpy, AnnexTTest, AnnexUTest, ShowerRun, ShowerSample,
        ShowerTestClass, TestFuel,
    };
    use nta8800_core::label_class::LabelFunction;
    use nta8800_core::solar_shading::CollectorObstruction;
    use nta8800_core::solar_thermal::{
        CollectorEfficiency, CollectorField, CollectorType, LoopPipes, PvtCover, SolarMethod,
        SolarStorage, SolarType, SolarUse, SolarWaterHeater,
    };

    fn close(label: &str, kernel: f64, expected: f64) {
        let diff = (kernel - expected).abs();
        let scale = kernel.abs().max(expected.abs()).max(1e-12);
        assert!(
            diff <= 1e-9 || diff / scale <= 1e-6,
            "{label}: kernel {kernel} independent {expected}"
        );
    }

    fn months(label: &str, kernel: impl Fn(usize) -> f64, expected: &[f64; 12]) {
        for (index, value) in expected.iter().enumerate() {
            close(&format!("{label} m{}", index + 1), kernel(index), *value);
        }
    }

    fn context(residential: bool, area: f64) -> HotWaterContext {
        HotWaterContext {
            residential,
            usable_floor_area_m2: area,
            heated_ambient_c: 20.0,
            space_heating: None,
            standard_setpoint_c: None,
            levelled_setpoint_c: None,
            need_fraction: None,
        }
    }

    fn residential_emission() -> HotWaterEmission {
        // Table 13.2 (p. 543): kitchen 3 m and bathroom 5 m.
        HotWaterEmission::Residential {
            served: ServedTaps::KitchenAndBathroom,
            kitchen_length_m: Some(3.0),
            kitchen_pipe_diameter: None,
            bathroom_length_m: Some(5.0),
            source_reference: "r77 synthetic".into(),
        }
    }

    fn indirect_hr107() -> HotWaterGenerator {
        HotWaterGenerator::IndirectBoiler {
            boiler: IndirectBoiler::Hr107,
            oil: false,
            inside_boundary: true,
            also_space_heating: true,
            declared: None,
            pilot_flame: false,
        }
    }

    fn system(
        need: HotWaterNeed,
        emission: HotWaterEmission,
        generator: HotWaterGenerator,
    ) -> HotWaterSystem {
        HotWaterSystem {
            need,
            emission,
            shower_heat_recovery: None,
            circulation: None,
            storage: Vec::new(),
            delivery_sets: None,
            boiling_water_tap: false,
            generator,
            nominal_power_kw: Some(100.0),
            exhaust_air: None,
            declared_share: None,
            additional_generators: Vec::new(),
            series: None,
            solar: Vec::new(),
            collective: None,
            unheated_reduction_factor: None,
            connected_taps: None,
            equipment_reference: "r77 synthetic".into(),
        }
    }

    fn run(system: &HotWaterSystem, context: HotWaterContext) -> HotWaterAssessment {
        let issues = validate_hot_water(system, context, "hotWater");
        assert!(issues.is_empty(), "{issues:?}");
        assess_hot_water(system, context).expect("assessment")
    }

    fn vessel(volume_l: f64, loss: StorageLoss, connection_factor: u8) -> StorageVessel {
        StorageVessel {
            id: "vat".into(),
            volume_l,
            loss,
            connection_factor,
            electric_boiler_insulated_pipe: false,
            in_heated_zone: true,
            unheated_ambient_c: None,
            not_in_appliance_test: false,
            source_reference: "r77 synthetic".into(),
        }
    }

    fn collective(building_m2: f64) -> Option<CollectiveHotWater> {
        Some(CollectiveHotWater {
            building_usable_floor_area_m2: building_m2,
            source_reference: "r77 synthetic".into(),
        })
    }

    fn circulation(
        insulation: PipeInsulation,
        fittings_insulated: bool,
        floors: u32,
        pump: CirculationPump,
    ) -> Circulation {
        Circulation {
            outer_diameter_mm: None,
            insulation,
            declared_psi_w_per_mk: None,
            calculated_psi: None,
            fittings_insulated,
            length_m: None,
            unheated_length_m: None,
            unheated_ambient_c: None,
            floor_count: floors,
            sport_hall_area_m2: 0.0,
            connected_dwellings: None,
            pump,
            source_reference: "r77 synthetic".into(),
        }
    }

    #[rustfmt::skip]
const CIRC_A_NEED: [f64; 12] = [4175.135812133072, 3771.0904109589037, 4175.135812133072, 4040.4540117416827, 4175.135812133072, 4040.4540117416827, 4175.135812133072, 4175.135812133072, 4040.4540117416827, 4175.135812133072, 4040.4540117416827, 4175.135812133072];
    #[rustfmt::skip]
const CIRC_A_EM: [f64; 12] = [4921.416126379081, 4445.150049632718, 4921.416126379081, 4762.660767463627, 4921.416126379081, 4762.660767463627, 4921.416126379081, 4921.416126379081, 4762.660767463627, 4921.416126379081, 4762.660767463627, 4921.416126379081];
    #[rustfmt::skip]
const CIRC_A_LOSS: [f64; 12] = [13944.959028, 12571.053767999998, 13890.945744, 13390.5798, 13782.919175999998, 13286.03796, 13701.89925, 13701.89925, 13312.17342, 13809.925818, 13416.715259999999, 13917.952385999997];
    #[rustfmt::skip]
const CIRC_A_ETA: [f64; 12] = [0.265861537412173, 0.2662499788860425, 0.26663955709866966, 0.26742214375815493, 0.26820933772117334, 0.2690011797945454, 0.2693988567567101, 0.2693988567567101, 0.2686046751748424, 0.26781516228754093, 0.2670302770472106, 0.26624997888604257];
    #[rustfmt::skip]
const CIRC_A_STO: [f64; 12] = [320.51923703260724, 289.50124635203235, 320.51923703260724, 310.1799068057489, 320.51923703260724, 310.1799068057489, 320.51923703260724, 320.51923703260724, 310.1799068057489, 320.51923703260724, 310.1799068057489, 320.51923703260724];
    #[rustfmt::skip]
const CIRC_A_DIS: [f64; 12] = [18831.71936881122, 16984.901817764974, 18777.706084811223, 18119.702710462472, 18669.67951681122, 18015.160870462474, 18588.659590811225, 18588.659590811225, 18041.29633046247, 18696.686158811222, 18145.838170462473, 18804.71272681122];
    #[rustfmt::skip]
const CIRC_A_GAS: [f64; 12] = [20924.132632012468, 18872.11313084997, 20864.11787201247, 20133.00301162497, 20744.088352012466, 20016.84541162497, 20654.066212012473, 20654.066212012473, 20045.88481162497, 20774.09573201247, 20162.04241162497, 20894.125252012465];
    #[rustfmt::skip]
const CIRC_A_AUX: [f64; 12] = [464.6836695562739, 419.6874497742603, 464.62425494387395, 449.5788777401361, 464.5054257190739, 449.4638817161361, 464.41630380047394, 464.41630380047394, 449.4926307221361, 464.5351330252739, 449.6076267461361, 464.6539622500739];
    #[rustfmt::skip]
const CIRC_A_RBL: [f64; 12] = [11942.008929450116, 10765.596514354942, 11896.097638050116, 11467.922270951725, 11804.275055250115, 11379.061706951725, 11735.408118150117, 11735.408118150117, 11401.276847951725, 11827.230700950116, 11490.137411951724, 11919.053283750114];
    #[rustfmt::skip]
const CIRC_B_NEED: [f64; 12] = [139.17119373776907, 125.70301369863013, 139.17119373776907, 134.68180039138943, 139.17119373776907, 134.68180039138943, 139.17119373776907, 139.17119373776907, 134.68180039138943, 139.17119373776907, 134.68180039138943, 139.17119373776907];
    #[rustfmt::skip]
const CIRC_B_EM: [f64; 12] = [164.04720421263602, 148.1716683210906, 164.04720421263602, 158.75535891545422, 164.04720421263602, 158.75535891545422, 164.04720421263602, 164.04720421263602, 158.75535891545422, 164.04720421263602, 158.75535891545422, 164.04720421263602];
    #[rustfmt::skip]
const CIRC_B_LOSS: [f64; 12] = [366.619578, 331.140264, 366.619578, 354.79314, 366.619578, 354.79314, 366.619578, 366.619578, 354.79314, 366.619578, 354.79314, 366.619578];
    #[rustfmt::skip]
const CIRC_B_ETA: [f64; 12] = [0.31404836446290657, 0.3140483644629066, 0.31404836446290657, 0.31404836446290657, 0.31404836446290657, 0.31404836446290657, 0.31404836446290657, 0.31404836446290657, 0.31404836446290657, 0.31404836446290657, 0.31404836446290657, 0.31404836446290657];
    #[rustfmt::skip]
const CIRC_B_STO: [f64; 12] = [2.9803971215510168, 2.691971593658983, 2.9803971215510168, 2.884255278920339, 2.9803971215510168, 2.884255278920339, 2.9803971215510168, 2.9803971215510168, 2.884255278920339, 2.9803971215510168, 2.884255278920339, 2.9803971215510168];
    #[rustfmt::skip]
const CIRC_B_DIS: [f64; 12] = [525.3432646792079, 474.5035939038006, 525.3432646792079, 508.3967077540722, 525.3432646792079, 508.3967077540722, 525.3432646792079, 525.3432646792079, 508.3967077540722, 525.3432646792079, 508.3967077540722, 525.3432646792079];
    #[rustfmt::skip]
const CIRC_B_GAS: [f64; 12] = [583.7147385324532, 527.2262154486673, 583.7147385324532, 564.885230837858, 583.7147385324532, 564.885230837858, 583.7147385324532, 583.7147385324532, 564.885230837858, 583.7147385324532, 564.885230837858, 583.7147385324532];
    #[rustfmt::skip]
const CIRC_B_AUX: [f64; 12] = [10.95777090987108, 9.897341466980329, 10.95777090987108, 10.604294428907496, 10.95777090987108, 10.604294428907496, 10.95777090987108, 10.95777090987108, 10.604294428907496, 10.95777090987108, 10.604294428907496, 10.95777090987108];
    #[rustfmt::skip]
const CIRC_B_RBL: [f64; 12] = [313.7026199637447, 283.3443019027372, 313.7026199637447, 303.5831806100756, 313.7026199637447, 303.5831806100756, 313.7026199637447, 313.7026199637447, 303.5831806100756, 313.7026199637447, 303.5831806100756, 313.7026199637447];
    #[rustfmt::skip]
const CIRC_C_NEED: [f64; 12] = [461.17808219178085, 416.54794520547944, 461.17808219178085, 446.3013698630137, 461.17808219178085, 446.3013698630137, 461.17808219178085, 461.17808219178085, 446.3013698630137, 461.17808219178085, 446.3013698630137, 461.17808219178085];
    #[rustfmt::skip]
const CIRC_C_EM: [f64; 12] = [576.472602739726, 520.6849315068492, 576.472602739726, 557.8767123287671, 576.472602739726, 557.8767123287671, 576.472602739726, 576.472602739726, 557.8767123287671, 576.472602739726, 557.8767123287671, 576.472602739726];
    #[rustfmt::skip]
const CIRC_C_LOSS: [f64; 12] = [5759.68344, 5202.294720000001, 5759.68344, 5573.8872, 5759.68344, 5573.8872, 5759.68344, 5759.68344, 5573.8872, 5759.68344, 5573.8872, 5759.68344];
    #[rustfmt::skip]
const CIRC_C_ETA: [f64; 12] = [0.0914032014281138, 0.09140320142811376, 0.0914032014281138, 0.09140320142811378, 0.0914032014281138, 0.09140320142811378, 0.0914032014281138, 0.0914032014281138, 0.09140320142811378, 0.0914032014281138, 0.09140320142811378, 0.0914032014281138];
    #[rustfmt::skip]
const CIRC_C_STO: [f64; 12] = [284.2069984773695, 256.7030953989144, 284.2069984773695, 275.03903078455113, 284.2069984773695, 275.03903078455113, 284.2069984773695, 284.2069984773695, 275.03903078455113, 284.2069984773695, 275.03903078455113, 284.2069984773695];
    #[rustfmt::skip]
const CIRC_C_DIS: [f64; 12] = [6591.126162497095, 5953.275243545764, 6591.126162497095, 6378.509189513318, 6591.126162497095, 6378.509189513318, 6591.126162497095, 6591.126162497095, 6378.509189513318, 6591.126162497095, 6378.509189513318, 6591.126162497095];
    #[rustfmt::skip]
const CIRC_C_AUX: [f64; 12] = [43.796337178746796, 39.55798196790033, 43.796337178746796, 42.38355210846464, 43.796337178746796, 42.38355210846464, 43.796337178746796, 43.796337178746796, 42.38355210846464, 43.796337178746796, 42.38355210846464, 43.796337178746796];
    #[rustfmt::skip]
const CIRC_C_RBL: [f64; 12] = [4903.040143679999, 4428.55238784, 4903.040143679999, 4744.8775583999995, 4903.040143679999, 4744.8775583999995, 4903.040143679999, 4903.040143679999, 4744.8775583999995, 4903.040143679999, 4744.8775583999995, 4903.040143679999];
    #[rustfmt::skip]
const CIRC_E_LOSS: [f64; 12] = [164610.0372, 148680.0336, 164610.0372, 159300.036, 164610.0372, 159300.036, 164610.0372, 164610.0372, 159300.036, 164610.0372, 159300.036, 164610.0372];
    #[rustfmt::skip]
const CIRC_E_ETA: [f64; 12] = [0.00755685197767756, 0.007556851977677561, 0.00755685197767756, 0.007556851977677561, 0.00755685197767756, 0.007556851977677561, 0.00755685197767756, 0.00755685197767756, 0.007556851977677561, 0.00755685197767756, 0.007556851977677561, 0.00755685197767756];
    #[rustfmt::skip]
const CIRC_E_DIS: [f64; 12] = [157710.8820109432, 142448.53859052932, 157710.8820109432, 152623.43420413855, 157710.8820109432, 152623.43420413855, 157710.8820109432, 157710.8820109432, 152623.43420413855, 157710.8820109432, 152623.43420413855, 157710.8820109432];
    #[rustfmt::skip]
const CIRC_E_AUX: [f64; 12] = [10739.71990495272, 9700.392172215363, 10739.71990495272, 10393.277327373604, 10739.71990495272, 10393.277327373604, 10739.71990495272, 10739.71990495272, 10393.277327373604, 10739.71990495272, 10393.277327373604, 10739.71990495272];

    /// Whole apartment building (30 dwellings, 2 400 m², 5 floors) with the forfait circulation: 13.15/13.17
    /// (p. 537), table 13.2 with 13.23 (p. 542-543), table 13.29 at 30 dwellings and table 13.4 "unknown"
    /// (p. 553), L = 0,3·A + 10 with 15 % at 13 °C (13.31, 13.26, 13.26a with 0,15 for uninsulated fittings,
    /// p. 549-554), the levelled setpoint as ambient, pump 13.32-13.41/13.43/13.44 with table 13.6 "uncontrolled"
    /// and EEI 0,23 (p. 554-559), 13.50/13.25 (p. 548, 561), a 500 l label D vessel with f_sto;dis;ls 4 and
    /// S_conn of label C (13.59, table 13.9, p. 567-570), table 13.28 HR 107 inside (p. 645), 13.181 with 1 W/kW
    /// (p. 647-648) and the recoverable loss 13.13/13.14/13.47/13.49 with the 500 m² rule (p. 534, 560-561).
    #[test]
    fn collective_circulation_forfait_whole_building() {
        let mut input = system(
            HotWaterNeed::Residential {
                dwelling_count: 30,
                source_reference: "r77 synthetic".into(),
            },
            residential_emission(),
            indirect_hr107(),
        );
        let mut circ = circulation(
            PipeInsulation::Unknown,
            false,
            5,
            CirculationPump {
                control: PumpControl::UncontrolledOrUnknown,
                label_power_kw: None,
                energy_efficiency_index: None,
            },
        );
        circ.connected_dwellings = Some(30);
        input.circulation = Some(circ);
        input.storage = vec![vessel(
            500.0,
            StorageLoss::Label {
                label: StorageLabel::D,
            },
            4,
        )];
        // The whole building is assessed: f_gebouw = 1, and the vessel and generator losses are not
        // recoverable above 500 m² (13.13, p. 534).
        input.collective = collective(2400.0);
        let mut ctx = context(true, 2400.0);
        ctx.levelled_setpoint_c = Some([
            19.6, 19.7, 19.8, 20.0, 20.2, 20.4, 20.5, 20.5, 20.3, 20.1, 19.9, 19.7,
        ]);
        let result = run(&input, ctx);
        let m = &result.months;
        months("Q_W;nd 13.15", |i| m[i].net_need_kwh, &CIRC_A_NEED);
        months("Q_W;em 13.9", |i| m[i].emission_input_kwh, &CIRC_A_EM);
        months(
            "Q_W;dis;ls 13.26",
            |i| m[i].circulation_loss_kwh,
            &CIRC_A_LOSS,
        );
        months(
            "eta_W;dis 13.25",
            |i| m[i].distribution_efficiency,
            &CIRC_A_ETA,
        );
        months("Q_W;sto;ls 13.59", |i| m[i].storage_loss_kwh, &CIRC_A_STO);
        months(
            "Q_W;dis;nren 13.7",
            |i| m[i].generator_output_kwh,
            &CIRC_A_DIS,
        );
        months("E_W gas 13.3", |i| m[i].natural_gas_kwh, &CIRC_A_GAS);
        months(
            "W_W;aux 13.2",
            |i| m[i].auxiliary_electricity_kwh,
            &CIRC_A_AUX,
        );
        months(
            "Q_W;ls;rbl 13.13",
            |i| m[i].recoverable_loss_kwh,
            &CIRC_A_RBL,
        );
    }

    /// One 80 m² dwelling on a collective system of a 2 400 m² building (13.1a, p. 529): the number of dwellings
    /// from A_g;gebouw/A_g;tot for table 13.29 (p. 553), f_gebouw in 13.26 and 13.43 and its inverse in 13.38
    /// (p. 549, 557, 559). Reading kept (kernel and recalculation): L and L_max of 13.31/13.32 (p. 554) from the
    /// area of the whole collective system, which 13.26 then scales by f_gebouw; the legend's A_g;si;W of 13.19b
    /// (the assessed dwelling) would give 34 m instead of 730 m and a January loss of 17,08 instead of 366,62 kWh.
    /// 10 mm insulation and
    /// insulated fittings (0,03), table 13.6 constant pressure with a known EEI 0,20 (p. 558), a 300 l label B vessel
    /// with f_sto;dis;ls 2 times f_gebouw (13.59, p. 567-569).
    #[test]
    fn collective_circulation_share_of_one_dwelling() {
        let mut input = system(
            HotWaterNeed::Residential {
                dwelling_count: 1,
                source_reference: "r77 synthetic".into(),
            },
            residential_emission(),
            indirect_hr107(),
        );
        input.circulation = Some(circulation(
            PipeInsulation::Mm10,
            true,
            5,
            CirculationPump {
                control: PumpControl::ConstantPressure,
                label_power_kw: None,
                energy_efficiency_index: Some(0.20),
            },
        ));
        input.collective = collective(2400.0);
        input.storage = vec![vessel(
            300.0,
            StorageLoss::Label {
                label: StorageLabel::B,
            },
            2,
        )];
        let result = run(&input, context(true, 80.0));
        let m = &result.months;
        months("Q_W;nd 13.15", |i| m[i].net_need_kwh, &CIRC_B_NEED);
        months("Q_W;em 13.9", |i| m[i].emission_input_kwh, &CIRC_B_EM);
        months(
            "Q_W;dis;ls 13.26",
            |i| m[i].circulation_loss_kwh,
            &CIRC_B_LOSS,
        );
        months(
            "eta_W;dis 13.25",
            |i| m[i].distribution_efficiency,
            &CIRC_B_ETA,
        );
        months("Q_W;sto;ls 13.59", |i| m[i].storage_loss_kwh, &CIRC_B_STO);
        months(
            "Q_W;dis;nren 13.7",
            |i| m[i].generator_output_kwh,
            &CIRC_B_DIS,
        );
        months("E_W gas 13.3", |i| m[i].natural_gas_kwh, &CIRC_B_GAS);
        months(
            "W_W;aux 13.2",
            |i| m[i].auxiliary_electricity_kwh,
            &CIRC_B_AUX,
        );
        months(
            "Q_W;ls;rbl 13.13",
            |i| m[i].recoverable_loss_kwh,
            &CIRC_B_RBL,
        );
    }

    /// Utility building, office 1 200 m² and sport 300 m² (table 13.1, p. 539; 13.19, p. 538), table 13.3 with a
    /// mean draw-off length of 4 m (p. 544), the default 80 mm above 500 m² with 20 mm insulation (p. 553-554),
    /// A_g;si;red without the sport hall (13.32a, p. 554), 3 floors, a pump with a label power of 0,045 kW (13.39,
    /// p. 558) and EEI 0,21, and a 1 000 l vessel of unknown label produced from 2018, label C (p. 569-570).
    #[test]
    fn utility_circulation_with_sport_hall_and_label_power() {
        let mut input = system(
            HotWaterNeed::Utility {
                areas: vec![
                    UtilityArea {
                        function: LabelFunction::Office,
                        area_m2: 1200.0,
                    },
                    UtilityArea {
                        function: LabelFunction::Sport,
                        area_m2: 300.0,
                    },
                ],
                source_reference: "r77 synthetic".into(),
            },
            HotWaterEmission::Utility {
                mean_length_m: 4.0,
                source_reference: "r77 synthetic".into(),
            },
            indirect_hr107(),
        );
        let mut circ = circulation(
            PipeInsulation::Mm20,
            true,
            3,
            CirculationPump {
                control: PumpControl::ConstantPressure,
                label_power_kw: Some(0.045),
                energy_efficiency_index: Some(0.21),
            },
        );
        circ.sport_hall_area_m2 = 300.0;
        input.circulation = Some(circ);
        input.storage = vec![vessel(
            1000.0,
            StorageLoss::UnknownLabel {
                produced_from_2018: true,
            },
            3,
        )];
        let mut ctx = context(false, 1500.0);
        ctx.heated_ambient_c = 21.0;
        let result = run(&input, ctx);
        let m = &result.months;
        months("Q_W;nd 13.19", |i| m[i].net_need_kwh, &CIRC_C_NEED);
        months("Q_W;em 13.9a", |i| m[i].emission_input_kwh, &CIRC_C_EM);
        months(
            "Q_W;dis;ls 13.26",
            |i| m[i].circulation_loss_kwh,
            &CIRC_C_LOSS,
        );
        months(
            "eta_W;dis 13.25",
            |i| m[i].distribution_efficiency,
            &CIRC_C_ETA,
        );
        months("Q_W;sto;ls 13.59", |i| m[i].storage_loss_kwh, &CIRC_C_STO);
        months(
            "Q_W;dis;nren 13.7",
            |i| m[i].generator_output_kwh,
            &CIRC_C_DIS,
        );
        months(
            "W_W;aux 13.2",
            |i| m[i].auxiliary_electricity_kwh,
            &CIRC_C_AUX,
        );
        months(
            "Q_W;ls;rbl 13.13",
            |i| m[i].recoverable_loss_kwh,
            &CIRC_C_RBL,
        );
    }

    /// Large utility system (office 8 000 m², 2 floors) with an uninsulated 80 mm circulation (Ψ 2,0 of table
    /// 13.4, c_ls;kb 0,15): P_W;hydr;dis above 2,5 kW, so 13.42 with b = 2 and EEI 0,25 (p. 558-559) replace
    /// 13.40/13.41; a 2 000 l label C vessel (13.59) and P_nom 1 000 kW so that f_func 0,6 (13.141) does not bind.
    #[test]
    fn large_circulation_pump_above_2_5_kw() {
        let mut input = system(
            HotWaterNeed::Utility {
                areas: vec![UtilityArea {
                    function: LabelFunction::Office,
                    area_m2: 8000.0,
                }],
                source_reference: "r77 synthetic".into(),
            },
            HotWaterEmission::Utility {
                mean_length_m: 4.0,
                source_reference: "r77 synthetic".into(),
            },
            indirect_hr107(),
        );
        input.nominal_power_kw = Some(1000.0);
        input.circulation = Some(circulation(
            PipeInsulation::None,
            false,
            2,
            CirculationPump {
                control: PumpControl::UncontrolledOrUnknown,
                label_power_kw: None,
                energy_efficiency_index: None,
            },
        ));
        input.storage = vec![vessel(
            2000.0,
            StorageLoss::Label {
                label: StorageLabel::C,
            },
            3,
        )];
        let mut ctx = context(false, 8000.0);
        ctx.heated_ambient_c = 21.0;
        let result = run(&input, ctx);
        let m = &result.months;
        months(
            "Q_W;dis;ls 13.26",
            |i| m[i].circulation_loss_kwh,
            &CIRC_E_LOSS,
        );
        months(
            "eta_W;dis 13.25",
            |i| m[i].distribution_efficiency,
            &CIRC_E_ETA,
        );
        months(
            "Q_W;dis;nren 13.7",
            |i| m[i].generator_output_kwh,
            &CIRC_E_DIS,
        );
        months(
            "W_W;aux 13.42",
            |i| m[i].auxiliary_electricity_kwh,
            &CIRC_E_AUX,
        );
    }
    #[rustfmt::skip]
const DISP_D_DIS: [f64; 12] = [1029.8487915510032, 930.1860052718739, 1029.8487915510032, 996.6278627912933, 1029.8487915510032, 996.6278627912933, 1029.8487915510032, 1029.8487915510032, 996.6278627912933, 1029.8487915510032, 996.6278627912933, 1029.8487915510032];
    #[rustfmt::skip]
const DISP_D_STO: [f64; 12] = [76.90592767700382, 69.4634185469712, 76.90592767700382, 74.42509130032627, 76.90592767700382, 74.42509130032627, 76.90592767700382, 76.90592767700382, 74.42509130032627, 76.90592767700382, 74.42509130032627, 76.90592767700382];
    #[rustfmt::skip]
const DISP_D_HP: [f64; 12] = [446.4, 403.2, 446.4, 432.0, 446.4, 432.0, 446.4, 446.4, 432.0, 446.4, 432.0, 446.4];
    #[rustfmt::skip]
const DISP_D_EL: [f64; 12] = [223.2, 201.6, 223.2, 216.0, 223.2, 216.0, 223.2, 223.2, 216.0, 223.2, 216.0, 223.2];
    #[rustfmt::skip]
const DISP_D_BOILER: [f64; 12] = [360.2487915510032, 325.38600527187384, 360.2487915510032, 348.6278627912933, 360.2487915510032, 348.6278627912933, 360.2487915510032, 360.2487915510032, 348.6278627912933, 360.2487915510032, 348.6278627912933, 360.2487915510032];
    #[rustfmt::skip]
const DISP_D_SHARE_HP: [f64; 12] = [0.43346169230115766, 0.4334616923011576, 0.43346169230115766, 0.4334616923011577, 0.43346169230115766, 0.4334616923011577, 0.43346169230115766, 0.43346169230115766, 0.4334616923011577, 0.43346169230115766, 0.4334616923011577, 0.43346169230115766];
    #[rustfmt::skip]
const DISP_D_SHARE_EL: [f64; 12] = [0.21673084615057883, 0.2167308461505788, 0.21673084615057883, 0.21673084615057886, 0.21673084615057883, 0.21673084615057886, 0.21673084615057883, 0.21673084615057883, 0.21673084615057886, 0.21673084615057883, 0.21673084615057886, 0.21673084615057883];
    #[rustfmt::skip]
const DISP_D_SHARE_BOILER: [f64; 12] = [0.34980746154826353, 0.3498074615482635, 0.34980746154826353, 0.3498074615482634, 0.34980746154826353, 0.3498074615482634, 0.34980746154826353, 0.34980746154826353, 0.3498074615482634, 0.34980746154826353, 0.3498074615482634, 0.34980746154826353];
    #[rustfmt::skip]
const DISP_D_ELEC: [f64; 12] = [553.8045112781955, 500.2105263157895, 553.8045112781955, 535.9398496240602, 553.8045112781955, 535.9398496240602, 553.8045112781955, 553.8045112781955, 535.9398496240602, 553.8045112781955, 535.9398496240602, 553.8045112781955];
    #[rustfmt::skip]
const DISP_D_GAS: [f64; 12] = [400.2764350566702, 361.5400058576376, 400.2764350566702, 387.36429199032585, 400.2764350566702, 387.36429199032585, 400.2764350566702, 400.2764350566702, 387.36429199032585, 400.2764350566702, 387.36429199032585, 400.2764350566702];
    #[rustfmt::skip]
const DISP_D_AUX: [f64; 12] = [7.836273670706104, 7.077924605799061, 7.836273670706104, 7.583490649070423, 7.836273670706104, 7.583490649070423, 7.836273670706104, 7.836273670706104, 7.583490649070423, 7.836273670706104, 7.583490649070423, 7.836273670706104];
    #[rustfmt::skip]
const DISP_E_DIS: [f64; 12] = [366.78016703064003, 331.28531215670705, 366.78016703064003, 354.94854873932906, 366.78016703064003, 354.94854873932906, 366.78016703064003, 366.78016703064003, 354.94854873932906, 366.78016703064003, 354.94854873932906, 366.78016703064003];
    #[rustfmt::skip]
const DISP_E_HP: [f64; 12] = [119.04000000000002, 134.4, 178.56, 230.4, 297.6, 288.0, 297.6, 297.6, 288.0, 238.08000000000004, 172.79999999999998, 133.92];
    #[rustfmt::skip]
const DISP_E_TFRAC: [f64; 12] = [0.4, 0.5, 0.6, 0.8, 1.0, 1.0, 1.0, 1.0, 1.0, 0.8, 0.6, 0.4499999999999999];
    #[rustfmt::skip]
const DISP_E_ELEC: [f64; 12] = [345.80769461871887, 303.2476970070601, 325.6693487540572, 295.6751640865118, 285.3926570247339, 276.18644228200054, 285.3926570247339, 285.3926570247339, 276.18644228200054, 305.5310028893955, 315.1638858910231, 340.77310815255345];
    #[rustfmt::skip]
const DISP_E_RBL: [f64; 12] = [13.038956159507393, 10.36238485035301, 9.90632458056001, 6.55518677575418, 3.641061422665274, 3.5236078283857495, 3.641061422665274, 3.641061422665274, 3.5236078283857495, 6.773693001612628, 9.586765723122596, 12.255798264770533];

    /// Three generators on one collective system of 600 m² (4 dwellings, f_func 0,6): order of 13.8.2.1 (heat pump
    /// of category b first, then within category c the higher efficiency, p. 610-611), 13.141 with P_nom, the
    /// cascade 13.143a/13.144b/13.145 and the shares 13.150 (p. 611-616), table 13.28 HR 107 0,90 and the indirect
    /// heat pump 1,4 (p. 645-646), the electric instantaneous heater 0,95 (table 13.25, p. 641) with f_prac 1
    /// (13.152, p. 616-617), a 150 l label A vessel (13.59), and 13.181 with 10 W stand-by for the electric
    /// instantaneous heater and 1 W/kW for the gas boiler (p. 638, 647-648).
    #[test]
    fn three_generators_dispatch_by_category_and_efficiency() {
        let mut input = system(
            HotWaterNeed::Residential {
                dwelling_count: 4,
                source_reference: "r77 synthetic".into(),
            },
            residential_emission(),
            indirect_hr107(),
        );
        input.nominal_power_kw = Some(20.0);
        input.collective = collective(600.0);
        input.storage = vec![vessel(
            150.0,
            StorageLoss::Label {
                label: StorageLabel::A,
            },
            3,
        )];
        input.additional_generators = vec![
            AdditionalHotWaterGenerator {
                generator: HotWaterGenerator::IndirectHeatPump {
                    also_space_heating: true,
                },
                nominal_power_kw: Some(1.0),
                exhaust_air: None,
                declared_share: None,
                equipment_reference: "r77 synthetic".into(),
            },
            AdditionalHotWaterGenerator {
                generator: HotWaterGenerator::ElectricInstantaneous,
                nominal_power_kw: Some(0.5),
                exhaust_air: None,
                declared_share: None,
                equipment_reference: "r77 synthetic".into(),
            },
        ];
        let result = run(&input, context(true, 600.0));
        let m = &result.months;
        months("Q_W;sto;ls 13.59", |i| m[i].storage_loss_kwh, &DISP_D_STO);
        months(
            "Q_W;dis;nren 13.7",
            |i| m[i].generator_output_kwh,
            &DISP_D_DIS,
        );
        let by_index = |index: usize| {
            result
                .generators
                .iter()
                .find(|item| item.index == index)
                .expect("generator")
        };
        assert_eq!(by_index(1).order, 0, "heat pump first");
        assert_eq!(by_index(2).order, 1, "electric 0,95 before boiler 0,90");
        assert_eq!(by_index(0).order, 2, "boiler last");
        months(
            "Q_W;gen;out heat pump 13.145",
            |i| by_index(1).monthly_output_kwh[i],
            &DISP_D_HP,
        );
        months(
            "Q_W;gen;out electric 13.145",
            |i| by_index(2).monthly_output_kwh[i],
            &DISP_D_EL,
        );
        months(
            "Q_W;gen;out boiler 13.145",
            |i| by_index(0).monthly_output_kwh[i],
            &DISP_D_BOILER,
        );
        months(
            "F_W;gen heat pump 13.150",
            |i| by_index(1).monthly_share[i],
            &DISP_D_SHARE_HP,
        );
        months(
            "F_W;gen electric 13.150",
            |i| by_index(2).monthly_share[i],
            &DISP_D_SHARE_EL,
        );
        months(
            "F_W;gen boiler 13.150",
            |i| by_index(0).monthly_share[i],
            &DISP_D_SHARE_BOILER,
        );
        months("E_W gas 13.3", |i| m[i].natural_gas_kwh, &DISP_D_GAS);
        months(
            "W_W;aux 13.181",
            |i| m[i].auxiliary_electricity_kwh,
            &DISP_D_AUX,
        );
        months(
            "E_W electricity 13.3",
            |i| m[i].electricity_kwh,
            &DISP_D_ELEC,
        );
        months(
            "Q_W;ls;rbl 13.13",
            |i| m[i].recoverable_loss_kwh,
            &[0.0; 12],
        );
    }

    /// Exhaust-air heat pump for hot water and heating in a 300 m² dwelling: 13.141 with P_nom 0,4 kW, 13.144a
    /// with f_combi per month (p. 613-614), the uncovered rest on an extra electric instantaneous heater (p. 614,
    /// 0,95 of table 13.25 with 13.179 as recoverable loss, p. 639-641) and the time fraction 13.149 (p. 615).
    /// Table 13.25 1,4 with c_W;gen class 4 (kernel reading, NEN question 15) is 1 above 3 890 kWh.
    #[test]
    fn exhaust_air_heat_pump_with_f_combi_and_extra_electric_heater() {
        let mut input = system(
            HotWaterNeed::Residential {
                dwelling_count: 1,
                source_reference: "r77 synthetic".into(),
            },
            residential_emission(),
            HotWaterGenerator::HeatPump {
                exhaust_air_source: true,
                source_correction: None,
                measured_class: None,
                outdoor_air_fraction: None,
                same_ground_source: false,
                declared: None,
            },
        );
        input.nominal_power_kw = Some(0.4);
        input.exhaust_air = Some(ExhaustAirUse {
            ventilation_suitable: true,
            heating_time_fraction: vec![
                0.6, 0.5, 0.4, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.2, 0.4, 0.55,
            ],
            declared_flow_m3_per_h: None,
        });
        let result = run(&input, context(true, 300.0));
        let m = &result.months;
        months(
            "Q_W;dis;nren 13.4",
            |i| m[i].generator_output_kwh,
            &DISP_E_DIS,
        );
        let main = result
            .generators
            .iter()
            .find(|item| item.index == 0)
            .expect("heat pump");
        months(
            "Q_W;gen;out heat pump 13.144a",
            |i| main.monthly_output_kwh[i],
            &DISP_E_HP,
        );
        months(
            "E_W electricity 13.3",
            |i| m[i].electricity_kwh,
            &DISP_E_ELEC,
        );
        months(
            "Q_W;gen;ls;rbl 13.179",
            |i| m[i].recoverable_loss_kwh,
            &DISP_E_RBL,
        );
        let exhaust = result.exhaust_air.as_ref().expect("13.149");
        months(
            "f_W;t;hp-on 13.149",
            |i| exhaust.time_fraction[i],
            &DISP_E_TFRAC,
        );
    }
    #[rustfmt::skip]
const TEST_F1_DIS: [f64; 12] = [195.38756561445308, 176.4790915227318, 195.38756561445308, 189.08474091721266, 195.38756561445308, 189.08474091721266, 195.38756561445308, 195.38756561445308, 189.08474091721266, 195.38756561445308, 189.08474091721266, 195.38756561445308];
    #[rustfmt::skip]
const TEST_F1_GAS: [f64; 12] = [283.6843057923093, 256.23098587692454, 283.6843057923093, 274.53319915384776, 283.6843057923093, 274.53319915384776, 283.6843057923093, 283.6843057923093, 274.53319915384776, 283.6843057923093, 274.53319915384776, 283.6843057923093];
    #[rustfmt::skip]
const TEST_F1_AUX: [f64; 12] = [3.1488556475540554, 2.8441276816617274, 3.1488556475540554, 3.0472796589232796, 3.1488556475540554, 3.0472796589232796, 3.1488556475540554, 3.1488556475540554, 3.0472796589232796, 3.1488556475540554, 3.0472796589232796, 3.1488556475540554];
    #[rustfmt::skip]
const TEST_F1_ETA: [f64; 12] = [0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001];
    #[rustfmt::skip]
const TEST_F2_DIS: [f64; 12] = [195.38756561445308, 176.4790915227318, 195.38756561445308, 189.08474091721266, 195.38756561445308, 189.08474091721266, 195.38756561445308, 195.38756561445308, 189.08474091721266, 195.38756561445308, 189.08474091721266, 195.38756561445308];
    #[rustfmt::skip]
const TEST_F2_ELEC: [f64; 12] = [79.10427757670166, 71.44902490798859, 79.10427757670166, 76.55252668713064, 79.10427757670166, 76.55252668713064, 79.10427757670166, 79.10427757670166, 76.55252668713064, 79.10427757670166, 76.55252668713064, 79.10427757670166];
    #[rustfmt::skip]
const TEST_F2_RBL: [f64; 12] = [36.788304113284035, 33.228145650708164, 36.788304113284035, 35.60158462575875, 36.788304113284035, 35.60158462575875, 36.788304113284035, 36.788304113284035, 35.60158462575875, 36.788304113284035, 35.60158462575875, 36.788304113284035];
    #[rustfmt::skip]
const TEST_F2_TFRAC: [f64; 12] = [0.17507846381223394, 0.17507846381223394, 0.17507846381223394, 0.17507846381223394, 0.17507846381223394, 0.17507846381223394, 0.17507846381223394, 0.17507846381223394, 0.17507846381223394, 0.17507846381223394, 0.17507846381223394, 0.17507846381223394];
    #[rustfmt::skip]
const TEST_F2_ETA: [f64; 12] = [2.4699999999999998, 2.4699999999999998, 2.4699999999999998, 2.4699999999999998, 2.4699999999999998, 2.4699999999999998, 2.4699999999999998, 2.4699999999999998, 2.4699999999999998, 2.4699999999999998, 2.4699999999999998, 2.4699999999999998];
    #[rustfmt::skip]
const TEST_F3_DIS: [f64; 12] = [267.0606358515309, 241.2160581884795, 267.0606358515309, 258.44577663051376, 267.0606358515309, 258.44577663051376, 267.0606358515309, 267.0606358515309, 258.44577663051376, 267.0606358515309, 258.44577663051376, 267.0606358515309];
    #[rustfmt::skip]
const TEST_F3_HP: [f64; 12] = [164.09336777423266, 148.2133644412424, 164.09336777423266, 158.8000333299026, 164.09336777423266, 158.8000333299026, 164.09336777423266, 164.09336777423266, 158.8000333299026, 164.09336777423266, 158.8000333299026, 164.09336777423266];
    #[rustfmt::skip]
const TEST_F3_BOILER: [f64; 12] = [102.96726807729823, 93.00269374723712, 102.96726807729823, 99.64574330061117, 102.96726807729823, 99.64574330061117, 102.96726807729823, 102.96726807729823, 99.64574330061117, 102.96726807729823, 99.64574330061117, 102.96726807729823];
    #[rustfmt::skip]
const TEST_F3_ELEC: [f64; 12] = [117.20954841016619, 105.86668888660171, 117.20954841016619, 113.42859523564472, 117.20954841016619, 113.42859523564472, 117.20954841016619, 117.20954841016619, 113.42859523564472, 117.20954841016619, 113.42859523564472, 117.20954841016619];
    #[rustfmt::skip]
const TEST_F3_GAS: [f64; 12] = [114.40807564144248, 103.33632638581902, 114.40807564144248, 110.71749255623463, 114.40807564144248, 110.71749255623463, 114.40807564144248, 114.40807564144248, 110.71749255623463, 114.40807564144248, 110.71749255623463, 114.40807564144248];
    #[rustfmt::skip]
const TEST_F3_SHARE: [f64; 12] = [0.6144423615671251, 0.6144423615671251, 0.6144423615671251, 0.6144423615671251, 0.6144423615671251, 0.6144423615671251, 0.6144423615671251, 0.6144423615671251, 0.6144423615671251, 0.6144423615671251, 0.6144423615671251, 0.6144423615671251];
    #[rustfmt::skip]
const TEST_F4_GAS: [f64; 12] = [265.3820925153862, 239.69995453002625, 265.3820925153862, 256.8213798535995, 265.3820925153862, 256.8213798535995, 265.3820925153862, 265.3820925153862, 256.8213798535995, 265.3820925153862, 256.8213798535995, 265.3820925153862];
    #[rustfmt::skip]
const TEST_F4_RBL: [f64; 12] = [19.74121084337353, 17.830771084337382, 19.74121084337353, 19.10439759036148, 19.74121084337353, 19.10439759036148, 19.74121084337353, 19.74121084337353, 19.10439759036148, 19.74121084337353, 19.10439759036148, 19.74121084337353];
    #[rustfmt::skip]
const TEST_F4_ETA: [f64; 12] = [0.73625, 0.73625, 0.73625, 0.73625, 0.73625, 0.73625, 0.73625, 0.73625, 0.73625, 0.73625, 0.73625, 0.73625];
    #[rustfmt::skip]
const TEST_F5_GAS: [f64; 12] = [169.92618818135816, 153.48171835735576, 169.92618818135816, 164.44469824002405, 169.92618818135816, 164.44469824002405, 169.92618818135816, 169.92618818135816, 164.44469824002405, 169.92618818135816, 164.44469824002405, 169.92618818135816];
    #[rustfmt::skip]
const TEST_F5_AUX: [f64; 12] = [2.879145136350811, 2.6005181876717, 2.879145136350811, 2.7862694867911078, 2.879145136350811, 2.7862694867911078, 2.879145136350811, 2.879145136350811, 2.7862694867911078, 2.879145136350811, 2.7862694867911078, 2.879145136350811];
    #[rustfmt::skip]
const TEST_F5_ETA: [f64; 12] = [0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001, 0.6887500000000001];

    fn profile(
        profile: TestProfile,
        delivered: f64,
        input: f64,
        auxiliary: Option<f64>,
        max_test_c: Option<f64>,
    ) -> ProfileTest {
        ProfileTest {
            profile,
            delivered_kwh_per_day: delivered,
            input_kwh_per_day: input,
            auxiliary_kwh_per_day: auxiliary,
            max_test_temperature_c: max_test_c,
        }
    }

    fn dwelling_100(generator: HotWaterGenerator) -> HotWaterSystem {
        system(
            HotWaterNeed::Residential {
                dwelling_count: 1,
                source_reference: "r77 synthetic".into(),
            },
            residential_emission(),
            generator,
        )
    }

    /// Instantaneous gas water heater tested per NEN-EN 13203-2 at M and L (p. 625-626): 13.153a (f_Hs/Hi 1,11,
    /// p. 621), 13.157 (p. 627), 13.154, 13.158 rounded down to 0,025 (p. 625-628), f_prac 0,95 (13.152, p. 617)
    /// and the auxiliary energy 13.159/13.160 (p. 628).
    #[test]
    fn gas_appliance_tested_at_two_profiles() {
        let input = dwelling_100(HotWaterGenerator::MeasuredTwoProfiles(Box::new(
            TwoProfileTest {
                standard: TwoProfileStandard::En13203Gas,
                storage_appliance: false,
                low: profile(TestProfile::M, 5.845, 7.2, Some(0.10), None),
                high: profile(TestProfile::L, 11.655, 13.6, Some(0.12), None),
                combi: false,
                integrated_vessel: false,
                exhaust_air_source: false,
                outdoor_air_fraction: None,
                smart_control_factor: None,
                design_set_temperature_c: None,
                legionella_cycle_tested: false,
                mixed_air: None,
                pfhrd: None,
                source_reference: "r77 synthetic".into(),
            },
        )));
        let result = run(&input, context(true, 100.0));
        let m = &result.months;
        months("Q_W;dis;nren", |i| m[i].generator_output_kwh, &TEST_F1_DIS);
        months(
            "eta_W;gen;prac 13.158/13.152",
            |i| m[i].generation_efficiency,
            &TEST_F1_ETA,
        );
        months("E_W gas 13.3", |i| m[i].natural_gas_kwh, &TEST_F1_GAS);
        months(
            "W_W;aux;gen 13.160",
            |i| m[i].auxiliary_electricity_kwh,
            &TEST_F1_AUX,
        );
        months("Q_W;ls;rbl", |i| m[i].recoverable_loss_kwh, &[0.0; 12]);
    }

    /// Exhaust-air heat pump tested per NEN-EN 16147 at L and XL as a storage appliance with the legionella cycle
    /// in the test (f_prac 0,95, p. 616-617): SCF 0,10 (smart 1, 13.153b), T_max;test 53/54 °C (13.153c, p. 622),
    /// 13.154 below i1 with the 13.154a check (p. 625), 13.158 rounded down to 0,05 (p. 627-628), the recoverable
    /// loss 13.160a with the uncorrected test energy (p. 629) and 13.149 with P_nom 1,5 kW (p. 611, 615).
    #[test]
    fn exhaust_air_heat_pump_tested_at_two_profiles() {
        let mut input = dwelling_100(HotWaterGenerator::MeasuredTwoProfiles(Box::new(
            TwoProfileTest {
                standard: TwoProfileStandard::En16147HeatPump,
                storage_appliance: true,
                low: profile(TestProfile::L, 11.655, 4.2, None, Some(53.0)),
                high: profile(TestProfile::Xl, 19.07, 6.6, None, Some(54.0)),
                combi: false,
                integrated_vessel: false,
                exhaust_air_source: true,
                outdoor_air_fraction: None,
                smart_control_factor: Some(0.10),
                design_set_temperature_c: None,
                legionella_cycle_tested: true,
                mixed_air: None,
                pfhrd: None,
                source_reference: "r77 synthetic".into(),
            },
        )));
        input.nominal_power_kw = Some(1.5);
        input.exhaust_air = Some(ExhaustAirUse {
            ventilation_suitable: true,
            heating_time_fraction: Vec::new(),
            declared_flow_m3_per_h: Some(150.0),
        });
        let result = run(&input, context(true, 100.0));
        let m = &result.months;
        months("Q_W;dis;nren", |i| m[i].generator_output_kwh, &TEST_F2_DIS);
        months(
            "eta_W;gen;prac 13.158/13.152",
            |i| m[i].generation_efficiency,
            &TEST_F2_ETA,
        );
        months(
            "E_W electricity 13.3",
            |i| m[i].electricity_kwh,
            &TEST_F2_ELEC,
        );
        months(
            "W_W;aux;gen",
            |i| m[i].auxiliary_electricity_kwh,
            &[0.0; 12],
        );
        months(
            "Q_W;gen;ls;rbl 13.160a",
            |i| m[i].recoverable_loss_kwh,
            &TEST_F2_RBL,
        );
        let exhaust = result.exhaust_air.as_ref().expect("13.149");
        months(
            "f_W;t;hp-on 13.149",
            |i| exhaust.time_fraction[i],
            &TEST_F2_TFRAC,
        );
    }

    /// A declared energetic contribution F_W;gen of an indirect heat pump at two classes, interpolated over
    /// Q_W;dis;nren;an (13.146, p. 614); the boiler (table 13.28 HR 107, p. 645) takes the rest (13.143a/13.145).
    #[test]
    fn declared_generator_share_is_interpolated() {
        let mut input = dwelling_100(indirect_hr107());
        input.storage = vec![vessel(
            120.0,
            StorageLoss::Label {
                label: StorageLabel::A,
            },
            3,
        )];
        input.additional_generators = vec![AdditionalHotWaterGenerator {
            generator: HotWaterGenerator::IndirectHeatPump {
                also_space_heating: true,
            },
            nominal_power_kw: Some(5.0),
            exhaust_air: None,
            declared_share: Some(DeclaredGeneratorShare {
                points: vec![
                    SharePoint {
                        annual_kwh: 2000.0,
                        share: 0.5,
                    },
                    SharePoint {
                        annual_kwh: 4000.0,
                        share: 0.7,
                    },
                ],
                source_reference: "r77 synthetic".into(),
            }),
            equipment_reference: "r77 synthetic".into(),
        }];
        let result = run(&input, context(true, 100.0));
        let m = &result.months;
        months("Q_W;dis;nren", |i| m[i].generator_output_kwh, &TEST_F3_DIS);
        let by_index = |index: usize| {
            result
                .generators
                .iter()
                .find(|item| item.index == index)
                .expect("generator")
        };
        months(
            "F_W;gen heat pump 13.146",
            |i| by_index(1).monthly_share[i],
            &TEST_F3_SHARE,
        );
        months(
            "Q_W;gen;out heat pump 13.146",
            |i| by_index(1).monthly_output_kwh[i],
            &TEST_F3_HP,
        );
        months(
            "Q_W;gen;out boiler 13.145",
            |i| by_index(0).monthly_output_kwh[i],
            &TEST_F3_BOILER,
        );
        months(
            "E_W electricity 13.3",
            |i| m[i].electricity_kwh,
            &TEST_F3_ELEC,
        );
        months("E_W gas 13.3", |i| m[i].natural_gas_kwh, &TEST_F3_GAS);
    }

    /// Gas combi appliance with an integrated vessel tested per NEN-EN 13203-2 at M and L, with a PFHRD
    /// (13.156a/b, p. 627) on Σ E_H;gen;gi;in of 8 000 kWh: 13.154 minus E_W;gen;in;PFHRD (p. 625), 13.158, the
    /// recoverable loss 13.160a of a combi with an integrated vessel (p. 629) and no auxiliary energy for a combi
    /// (p. 629).
    #[test]
    fn gas_combi_with_vessel_and_pfhrd() {
        let input = dwelling_100(HotWaterGenerator::MeasuredTwoProfiles(Box::new(
            TwoProfileTest {
                standard: TwoProfileStandard::En13203Gas,
                storage_appliance: true,
                low: profile(TestProfile::M, 5.845, 7.0, Some(0.05), None),
                high: profile(TestProfile::L, 11.655, 13.2, Some(0.06), None),
                combi: true,
                integrated_vessel: true,
                exhaust_air_source: false,
                outdoor_air_fraction: None,
                smart_control_factor: None,
                design_set_temperature_c: None,
                legionella_cycle_tested: false,
                mixed_air: None,
                pfhrd: Some(PfhrdTest {
                    indirect_gas_kwh_per_day: 0.3,
                    heating_gas_kwh_per_day: 30.0,
                    source_reference: "r77 synthetic".into(),
                }),
                source_reference: "r77 synthetic".into(),
            },
        )));
        let ctx = context(true, 100.0);
        assert!(validate_hot_water(&input, ctx, "hotWater").is_empty());
        let extras = HotWaterExtras {
            combi_space_heating_gas_kwh: Some(8000.0),
            ..HotWaterExtras::default()
        };
        let result = assess_hot_water_with(&input, ctx, &extras).expect("assessment");
        let m = &result.months;
        months(
            "eta_W;gen;prac 13.158/13.152",
            |i| m[i].generation_efficiency,
            &TEST_F4_ETA,
        );
        months("E_W gas 13.3", |i| m[i].natural_gas_kwh, &TEST_F4_GAS);
        months(
            "W_W;aux combi",
            |i| m[i].auxiliary_electricity_kwh,
            &[0.0; 12],
        );
        months(
            "Q_W;gen;ls;rbl 13.160a",
            |i| m[i].recoverable_loss_kwh,
            &TEST_F4_RBL,
        );
    }

    /// The instantaneous gas heater with a steeper test line in a 50 m² dwelling: Q_W;b;d below Q_test;M, where
    /// the linear extrapolation would raise the efficiency above that of the M test, so 13.154a (p. 625) caps it;
    /// 13.159/13.160 extrapolated (p. 628).
    #[test]
    fn gas_appliance_below_the_lowest_profile_13_154a() {
        let input = system(
            HotWaterNeed::Residential {
                dwelling_count: 1,
                source_reference: "r77 synthetic".into(),
            },
            residential_emission(),
            HotWaterGenerator::MeasuredTwoProfiles(Box::new(TwoProfileTest {
                standard: TwoProfileStandard::En13203Gas,
                storage_appliance: false,
                low: profile(TestProfile::M, 5.845, 7.2, Some(0.10), None),
                high: profile(TestProfile::L, 11.655, 15.5, Some(0.12), None),
                combi: false,
                integrated_vessel: false,
                exhaust_air_source: false,
                outdoor_air_fraction: None,
                smart_control_factor: None,
                design_set_temperature_c: None,
                legionella_cycle_tested: false,
                mixed_air: None,
                pfhrd: None,
                source_reference: "r77 synthetic".into(),
            })),
        );
        let result = run(&input, context(true, 50.0));
        let m = &result.months;
        months(
            "eta_W;gen;prac 13.154a/13.158",
            |i| m[i].generation_efficiency,
            &TEST_F5_ETA,
        );
        months("E_W gas 13.3", |i| m[i].natural_gas_kwh, &TEST_F5_GAS);
        months(
            "W_W;aux;gen 13.159/13.160",
            |i| m[i].auxiliary_electricity_kwh,
            &TEST_F5_AUX,
        );
    }
    #[rustfmt::skip]
const GAS_G1_DIS: [f64; 12] = [195.38756561445308, 176.4790915227318, 195.38756561445308, 189.08474091721266, 195.38756561445308, 189.08474091721266, 195.38756561445308, 195.38756561445308, 189.08474091721266, 195.38756561445308, 189.08474091721266, 195.38756561445308];
    #[rustfmt::skip]
const GAS_G1_ETA: [f64; 12] = [0.375, 0.375, 0.375, 0.375, 0.4, 0.4, 0.4, 0.4, 0.4, 0.375, 0.375, 0.375];
    #[rustfmt::skip]
const GAS_G1_GAS: [f64; 12] = [521.0335083052082, 470.6109107272848, 521.0335083052082, 504.22597577923375, 488.4689140361327, 472.7118522930316, 488.4689140361327, 488.4689140361327, 472.7118522930316, 521.0335083052082, 504.22597577923375, 521.0335083052082];
    #[rustfmt::skip]
const GAS_G1_AUX: [f64; 12] = [7.654926322175898, 6.914127000675005, 7.654926322175898, 7.407993215008934, 7.654926322175898, 7.407993215008934, 7.654926322175898, 7.654926322175898, 7.407993215008934, 7.654926322175898, 7.407993215008934, 7.654926322175898];
    #[rustfmt::skip]
const GAS_G1_RBL: [f64; 12] = [233.5490767082125, 210.4254057470033, 232.39289316015208, 223.77746091492736, 230.08052606403112, 221.53968630577808, 228.34625074194042, 228.34625074194042, 222.0991299580654, 230.65861783806133, 224.33690456721467, 232.97098493418227];
    #[rustfmt::skip]
const GAS_G2_ETA: [f64; 12] = [0.59375, 0.59375, 0.59375, 0.59375, 0.59375, 0.59375, 0.59375, 0.59375, 0.59375, 0.59375, 0.59375, 0.59375];
    #[rustfmt::skip]
const GAS_G2_GAS: [f64; 12] = [329.0737947190789, 297.22794361723254, 329.0737947190789, 318.4585110184634, 329.0737947190789, 318.4585110184634, 329.0737947190789, 329.0737947190789, 318.4585110184634, 329.0737947190789, 318.4585110184634, 329.0737947190789];
    #[rustfmt::skip]
const GAS_G2_AUX: [f64; 12] = [7.654926322175898, 6.914127000675005, 7.654926322175898, 7.407993215008934, 7.654926322175898, 7.407993215008934, 7.654926322175898, 7.654926322175898, 7.407993215008934, 7.654926322175898, 7.407993215008934, 7.654926322175898];
    #[rustfmt::skip]
const GAS_G2_RBL: [f64; 12] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    #[rustfmt::skip]
const GAS_G3_ETA: [f64; 12] = [0.375, 0.375, 0.375, 0.375, 0.375, 0.375, 0.375, 0.375, 0.375, 0.375, 0.375, 0.375];
    #[rustfmt::skip]
const GAS_G3_GAS: [f64; 12] = [521.0335083052082, 470.6109107272848, 521.0335083052082, 504.22597577923375, 521.0335083052082, 504.22597577923375, 521.0335083052082, 521.0335083052082, 504.22597577923375, 521.0335083052082, 504.22597577923375, 521.0335083052082];
    #[rustfmt::skip]
const GAS_G3_AUX: [f64; 12] = [7.654926322175898, 6.914127000675005, 7.654926322175898, 7.407993215008934, 7.654926322175898, 7.407993215008934, 7.654926322175898, 7.654926322175898, 7.407993215008934, 7.654926322175898, 7.407993215008934, 7.654926322175898];
    #[rustfmt::skip]
const GAS_G3_RBL: [f64; 12] = [237.35764719689934, 213.85689004869144, 236.18260933948898, 227.42668207943083, 233.83253362466812, 225.15241525863655, 232.06997683855252, 232.06997683855252, 225.72098196383513, 234.42005255337332, 227.9952487846294, 236.7701282681941];

    fn gas_storage_case(
        volume_l: f64,
        measured: Option<f64>,
        before_1985: bool,
        in_heated_zone: bool,
    ) -> HotWaterAssessment {
        let input = dwelling_100(HotWaterGenerator::GasStorageHeater {
            volume_l,
            measured_standby_kwh_per_day: measured,
            before_1985,
            in_heated_zone,
        });
        let mut ctx = context(true, 100.0);
        ctx.levelled_setpoint_c = Some([
            19.6, 19.7, 19.8, 20.0, 20.2, 20.4, 20.5, 20.5, 20.3, 20.1, 19.9, 19.7,
        ]);
        run(&input, ctx)
    }

    fn check_gas_storage(
        result: &HotWaterAssessment,
        eta: &[f64; 12],
        gas: &[f64; 12],
        aux: &[f64; 12],
        rbl: &[f64; 12],
    ) {
        let m = &result.months;
        months("Q_W;dis;nren", |i| m[i].generator_output_kwh, &GAS_G1_DIS);
        months("Q_W;sto;ls", |i| m[i].storage_loss_kwh, &[0.0; 12]);
        months(
            "eta_W;gen;prac 13.165/13.152",
            |i| m[i].generation_efficiency,
            eta,
        );
        months("E_W gas 13.3", |i| m[i].natural_gas_kwh, gas);
        months(
            "W_W;aux;gen 13.181",
            |i| m[i].auxiliary_electricity_kwh,
            aux,
        );
        months(
            "Q_W;ls;rbl 13.63 with 13.168",
            |i| m[i].recoverable_loss_kwh,
            rbl,
        );
    }

    /// Kernel deviation (src/domestic_hot_water.rs:3393): 13.168 (p. 634) takes (ϑ_Wc;mn + 5 − ϑ_W;amb)/45, the
    /// kernel (ϑ_Wc;mn − ϑ_W;amb)/45, so the stand-by loss is 35/40 of the norm value in a heated zone at 20 °C.
    ///
    /// Gas storage heater of 120 l in a heated zone without a measured stand-by loss: 13.169 (p. 635), 13.168 with
    /// 55 °C (no circulation) and the levelled setpoint (p. 634-635), 13.174/13.175 with η_100% 0,84 and 1,11
    /// (p. 636), 13.165-13.167 rounded down to 0,025 per month (p. 634), f_prac 1 (13.152), no vessel loss in
    /// 13.7 (13.6.2 note 1, p. 566), 13.181 with 10 W and 1 W/kW (p. 647-648) and the recoverable loss of 13.6.5
    /// with Q_W;S;ls (p. 636).
    #[test]
    fn gas_storage_heater_forfait_stand_by_loss() {
        let result = gas_storage_case(120.0, None, false, true);
        check_gas_storage(&result, &GAS_G1_ETA, &GAS_G1_GAS, &GAS_G1_AUX, &GAS_G1_RBL);
    }

    /// The same heater with a stand-by loss of 1,4 kWh/day measured per NEN-EN 89 in an unheated space (13 °C):
    /// f_prac 0,95 (note 2, p. 617) and no recoverable loss outside the heated zone.
    #[test]
    fn gas_storage_heater_measured_stand_by_loss_unheated() {
        let result = gas_storage_case(120.0, Some(1.4), false, false);
        check_gas_storage(&result, &GAS_G2_ETA, &GAS_G2_GAS, &GAS_G2_AUX, &GAS_G2_RBL);
    }

    /// An 80 l gas storage heater from before 1985: 13.170 (p. 635).
    #[test]
    fn gas_storage_heater_before_1985() {
        let result = gas_storage_case(80.0, None, true, true);
        check_gas_storage(&result, &GAS_G3_ETA, &GAS_G3_GAS, &GAS_G3_AUX, &GAS_G3_RBL);
    }
    #[rustfmt::skip]
const MIX_M_C: [f64; 12] = [1.0001059434262105, 1.0247503256621797, 1.0469531887709833, 1.0957682914372209, 1.1739307778783687, 1.2033571625246027, 1.2401471361008933, 1.35632183908046, 1.1888330294085052, 1.108553713185213, 1.0698872085010698, 1.0150537634408603];

    /// Tables 13.1 (p. 539) and 13.7 (p. 564), every row, read from the rendered pages.
    #[test]
    fn tables_13_1_and_13_7_per_function() {
        use LabelFunction::*;
        let rows = [
            (AssemblyWithDayCare, Some(2.8), 0.0),
            (AssemblyWithoutDayCare, Some(2.8), 0.0),
            (Cell, Some(4.2), 0.40),
            (HealthcareWithBeds, Some(15.3), 0.40),
            (HealthcareWithoutBeds, Some(2.8), 0.40),
            (Office, Some(1.4), 0.0),
            (Lodging, Some(12.5), 0.60),
            (Education, Some(1.4), 0.0),
            (Sport, Some(12.5), 0.80),
            (Retail, Some(1.4), 0.0),
            (Residential, None, 0.80),
        ];
        for (function, need, shower) in rows {
            assert_eq!(
                utility_specific_need(function),
                need,
                "table 13.1 {function:?}"
            );
            close(
                &format!("table 13.7 {function:?}"),
                shower_share(function),
                shower,
            );
        }
    }

    /// Table 13.2 (p. 543): bands "from a to b" with 14 m or more as the last band.
    #[test]
    fn table_13_2_bands() {
        let kitchen = [1.00, 0.69, 0.53, 0.43, 0.36, 0.31, 0.27, 0.24];
        let bathroom = [1.00, 0.95, 0.90, 0.86, 0.82, 0.78, 0.75, 0.72];
        for band in 0..8 {
            for offset in [0.0, 1.0, 1.99] {
                let length = 2.0 * band as f64 + offset;
                close(
                    &format!("eta_W;em;k {length}"),
                    kitchen_emission(length),
                    kitchen[band],
                );
                close(
                    &format!("eta_W;em;b {length}"),
                    bathroom_emission(length),
                    bathroom[band],
                );
            }
        }
        close("eta_W;em;k 30 m", kitchen_emission(30.0), 0.24);
        close("eta_W;em;b 30 m", bathroom_emission(30.0), 0.72);
    }

    /// Table 13.4 (p. 553) per row, and the nearest outer diameter with the larger diameter on a tie (p. 552).
    #[test]
    fn table_13_4_rows_and_nearest_diameter() {
        use PipeInsulation::*;
        let rows = [
            (10.0, [0.407, 0.165, 0.165, 0.136, 0.114, 0.106]),
            (12.0, [0.453, 0.184, 0.184, 0.154, 0.136, 0.124]),
            (15.0, [0.539, 0.211, 0.211, 0.174, 0.154, 0.138]),
            (22.0, [0.728, 0.271, 0.271, 0.219, 0.189, 0.169]),
            (28.0, [0.880, 0.321, 0.321, 0.256, 0.219, 0.194]),
            (35.0, [1.049, 0.378, 0.378, 0.299, 0.253, 0.223]),
            (42.0, [1.211, 0.435, 0.435, 0.341, 0.287, 0.251]),
            (54.0, [1.477, 0.531, 0.531, 0.412, 0.343, 0.299]),
            (67.0, [1.753, 0.635, 0.635, 0.488, 0.404, 0.349]),
            (80.0, [2.0, 0.74, 0.74, 0.56, 0.46, 0.4]),
        ];
        let columns = [None, Unknown, Mm10, Mm15, Mm20, Mm25];
        for (diameter, values) in rows {
            for (column, expected) in columns.iter().zip(values) {
                close(
                    &format!("psi {diameter} {column:?}"),
                    table_13_4_psi(diameter, *column),
                    expected,
                );
            }
        }
        // Nearest diameter: 20 mm takes 22 mm; ties 25 mm (22/28) and 18,5 mm (15/22) take the larger.
        close("psi 20 mm", table_13_4_psi(20.0, Mm15), 0.219);
        close("psi 25 mm tie", table_13_4_psi(25.0, Mm15), 0.256);
        close("psi 18,5 mm tie", table_13_4_psi(18.5, None), 0.728);
        close("psi 100 mm", table_13_4_psi(100.0, Mm20), 0.46);
    }

    /// Table 13.9 (p. 570) at 200 l for every label, and the shower tables 13.8 (p. 565) and the defaults of
    /// §13.5.3 (p. 565) with a declared annex U value rounded down to 0,025 (p. 1113).
    #[test]
    fn tables_13_8_and_13_9_and_shower_defaults() {
        use StorageLabel::*;
        let labels = [
            (APlus, 31.808681353899193),
            (A, 37.80446867386931),
            (B, 52.626958256755344),
            (C, 73.69104368775356),
            (D, 96.50721425059477),
            (E, 123.3231295674846),
            (F, 154.71506624212395),
            (G, 169.70336435315207),
        ];
        for (label, expected) in labels {
            close(
                &format!("S_sto;ls {label:?}"),
                label.standing_loss_w(200.0),
                expected,
            );
        }
        use ShowerConnection::*;
        for (connection, factor) in [
            (MixerAndHeater, 1.00),
            (MixerOnly, 0.85),
            (HeaterOnly, 0.75),
            (SharedUnits, 0.75),
            (Unknown, 0.75),
        ] {
            close(
                &format!("C_W;sh;rcd;conf {connection:?}"),
                connection.factor(),
                factor,
            );
        }
        close("vertical", ShowerUnit::Vertical.efficiency(), 0.40);
        close("horizontal", ShowerUnit::Horizontal.efficiency(), 0.20);
        close("unknown", ShowerUnit::Unknown.efficiency(), 0.20);
        close("none", ShowerUnit::None.efficiency(), 0.0);
        let declared = ShowerUnit::Declared {
            efficiency: 0.587,
            test_class: None,
            source_reference: "r77 synthetic".into(),
        };
        close("declared 0,587", declared.efficiency(), 0.575);
    }

    /// Tables 13.26 and 13.27 (p. 643-644) and 13.18 (p. 630-631) with linear interpolation; heat pumps above
    /// their class are refused (p. 631, 644); the worked examples of p. 642-643 (class 2 and 4 at 2 150 kWh).
    #[test]
    fn class_correction_tables() {
        use ApplicationClass::*;
        close(
            "13.26 c2 2150",
            gas_class_correction(Class2, 2150.0),
            0.9496402877697842,
        );
        close(
            "13.26 c4 2150",
            gas_class_correction(Class4, 2150.0),
            0.8332589928057554,
        );
        close(
            "13.26 c3 3000",
            gas_class_correction(Class3, 3000.0),
            0.9789568345323741,
        );
        close(
            "13.26 c4 1000",
            gas_class_correction(Class4, 1000.0),
            0.72272,
        );
        close("13.26 c1 555", gas_class_correction(Class1, 555.0), 1.0);
        close("13.26 c4 3890", gas_class_correction(Class4, 3890.0), 1.0);
        let hp = |class, annual| heat_pump_class_correction(class, annual).expect("in range");
        close("13.27 c3 2847,5", hp(Class3, 2847.5), 0.905);
        close("13.27 c4 3500", hp(Class4, 3500.0), 0.9551079136690648);
        close("13.27 c2 2000", hp(Class2, 2000.0), 0.7122302158273381);
        close("13.27 c1 1805", hp(Class1, 1805.0), 1.0);
        assert!(heat_pump_class_correction(Class2, 2600.0).is_none());
        assert!(heat_pump_class_correction(Class3, 3300.0).is_none());
        use TappingProfile::*;
        let eu = |profile, annual| european_profile_correction(profile, annual).expect("in range");
        close("13.18 L 3190", eu(L, 3190.0), 0.87);
        close("13.18 XL 5000", eu(Xl, 5000.0), 0.8481180811808119);
        close("13.18 M 1500", eu(M, 1500.0), 0.796923076923077);
        close("13.18 S 765", eu(S, 765.0), 1.0);
        assert!(european_profile_correction(M, 2200.0).is_none());
        assert!(european_profile_correction(Xl, 7000.0).is_none());
    }

    /// Note 5 of §13.8.4.2 (p. 625), 13.153 (p. 621) and 13.153b/c (p. 622).
    #[test]
    fn test_profile_energies_and_corrections() {
        use TestProfile::*;
        for (profile, q) in [
            (S, 2.1),
            (M, 5.845),
            (L, 11.655),
            (Xl, 19.07),
            (Xxl, 24.53),
            (ThreeXl, 46.76),
            (FourXl, 93.52),
        ] {
            close(
                &format!("Q_ref {profile:?}"),
                profile.reference_kwh_per_day(),
                q,
            );
        }
        close(
            "13.153",
            winter_gas_consumption_kwh(6.0, 0.08, 0.88).expect("valid"),
            4.515,
        );
        // SCF 0,12 (smart 1), T_max;test 52 °C, T_set;design 50 °C.
        close(
            "13.153b/c",
            en16147_corrected_input(3.0, Some(0.12), Some(52.0), Some(50.0)),
            2.544,
        );
        // SCF below 0,07: smart 0.
        close(
            "13.153b smart 0",
            en16147_corrected_input(3.0, Some(0.06), None, None),
            3.0,
        );
    }

    /// 13.153d-13.153i (p. 623-624) per month with θ_e;avg of table 17.1, the outdoor-air share bounded at 0 in
    /// August (q_V;ODA;req above q_V;hp;W).
    #[test]
    fn mixed_air_correction_per_month() {
        let correction = MixedAirCorrection::En14511 {
            cop_condition_2: 3.2,
            condenser_out_c: 55.0,
            evaporator_in_c: 7.0,
            evaporator_out_c: 3.0,
            minimum_air_flow_m3_per_h: 400.0,
            source_reference: "r77 synthetic".into(),
        };
        let ventilation = MixedAirVentilation {
            required_outdoor_air_m3_per_h: [
                150.0, 150.0, 160.0, 170.0, 180.0, 190.0, 200.0, 420.0, 180.0, 170.0, 160.0, 150.0,
            ],
            extract_air_c: [
                20.5, 20.6, 20.8, 21.0, 21.5, 22.0, 22.5, 22.5, 21.8, 21.0, 20.7, 20.5,
            ],
        };
        months(
            "C_W;mixed air 13.153d",
            |i| correction.factor(i, Some(&ventilation)),
            &MIX_M_C,
        );
    }

    /// 13.19a (p. 540) with C_W;nd;b 0,8 and C_W;nd;k 0,2 (p. 539): two bathrooms and one kitchen on system 1, one
    /// bathroom on system 2 and one kitchen on system 3.
    #[test]
    fn need_fractions_13_19a() {
        let taps = [
            Some(ConnectedTaps {
                bathrooms: 2,
                kitchens: 1,
            }),
            Some(ConnectedTaps {
                bathrooms: 1,
                kitchens: 0,
            }),
            Some(ConnectedTaps {
                bathrooms: 0,
                kitchens: 1,
            }),
        ];
        let fractions = residential_need_fractions(&taps).expect("fractions");
        let expected = [2.0 * 0.8 / 3.0 + 0.2 / 2.0, 0.8 / 3.0, 0.2 / 2.0];
        for (index, value) in expected.iter().enumerate() {
            close(&format!("F_W;si {index}"), fractions[index], *value);
        }
    }

    /// 13.15-13.18 (p. 537-538) with 856 kWh per occupant (p. 538): N_P;woon 1 up to 30 m², 13.17 between 30 and
    /// 100 m² and 13.18 above, per dwelling of A_g;zi/N_woon;zi.
    #[test]
    fn residential_need_13_15_to_13_18() {
        for (area, dwellings, occupants) in [
            (50.0, 2, 1.0),
            (60.0, 2, 1.0),
            (130.0, 2, 2.28 - 1.28 / 70.0 * (100.0 - 65.0)),
            (200.0, 2, 2.28),
            (360.0, 2, 1.28 + 0.01 * 180.0),
        ] {
            let input = system(
                HotWaterNeed::Residential {
                    dwelling_count: dwellings,
                    source_reference: "r77 synthetic".into(),
                },
                residential_emission(),
                HotWaterGenerator::ElectricInstantaneous,
            );
            let result = run(&input, context(true, area));
            let annual = 856.0 * f64::from(dwellings) * occupants;
            close(
                &format!("Q_W;nd {area} m² / {dwellings}"),
                result.annual_net_need_kwh,
                annual,
            );
            close(
                &format!("Q_W;nd January {area} m²"),
                result.months[0].net_need_kwh,
                annual * 744.0 / 8760.0,
            );
        }
    }

    /// 13.27-13.29 (p. 551) with h_a 8,0 W/(m²·K): an insulated pipe in air, a pipe embedded at 5 cm in a covering
    /// of λ 1,0 and an uninsulated copper pipe (λ_P 380).
    #[test]
    fn calculated_psi_13_27_to_13_29() {
        let insulated = PipeGeometry::InsulatedInAir {
            pipe_outer_diameter_m: 0.022,
            insulated_diameter_m: 0.062,
            insulation_lambda: 0.035,
            surface_coefficient: None,
        };
        close("13.27", insulated.psi().expect("psi"), 0.18680561465229706);
        let embedded = PipeGeometry::InsulatedEmbedded {
            pipe_outer_diameter_m: 0.022,
            insulated_diameter_m: 0.042,
            insulation_lambda: 0.035,
            embedding_lambda: 1.0,
            depth_m: 0.05,
        };
        close("13.28", embedded.psi().expect("psi"), 0.3135993399582573);
        let bare = PipeGeometry::Uninsulated {
            inner_diameter_m: 0.020,
            outer_diameter_m: 0.022,
            pipe_lambda: 380.0,
            surface_coefficient: None,
        };
        close("13.29", bare.psi().expect("psi"), 0.5529081033375259);
    }
    #[rustfmt::skip]
const SER_H1_DIS: [f64; 12] = [240.16315900476502, 216.92156297204582, 240.16315900476502, 232.41596032719195, 240.16315900476502, 232.41596032719195, 240.16315900476502, 240.16315900476502, 232.41596032719195, 240.16315900476502, 232.41596032719195, 240.16315900476502];
    #[rustfmt::skip]
const SER_H1_STO: [f64; 12] = [44.77559339031195, 40.44247144931402, 44.77559339031195, 43.331219409979305, 44.77559339031195, 43.331219409979305, 44.77559339031195, 44.77559339031195, 43.331219409979305, 44.77559339031195, 43.331219409979305, 44.77559339031195];
    #[rustfmt::skip]
const SER_H1_FIRST: [f64; 12] = [192.13052720381202, 173.53725037763667, 192.13052720381202, 185.93276826175358, 192.13052720381202, 185.93276826175358, 192.13052720381202, 192.13052720381202, 185.93276826175358, 192.13052720381202, 185.93276826175358, 192.13052720381202];
    #[rustfmt::skip]
const SER_H1_SECOND: [f64; 12] = [48.032631800953, 43.38431259440915, 48.032631800953, 46.483192065438374, 48.032631800953, 46.483192065438374, 48.032631800953, 48.032631800953, 46.483192065438374, 48.032631800953, 46.483192065438374, 48.032631800953];
    #[rustfmt::skip]
const SER_H1_GAS: [f64; 12] = [337.21916815158954, 304.58505510466154, 337.21916815158954, 326.34113046928024, 337.21916815158954, 326.34113046928024, 337.21916815158954, 337.21916815158954, 326.34113046928024, 337.21916815158954, 326.34113046928024, 337.21916815158954];
    #[rustfmt::skip]
const SER_H1_ELEC: [f64; 12] = [48.032631800953, 43.38431259440915, 48.032631800953, 46.483192065438374, 48.032631800953, 46.483192065438374, 48.032631800953, 48.032631800953, 46.483192065438374, 48.032631800953, 46.483192065438374, 48.032631800953];
    #[rustfmt::skip]
const SER_H1_RBL: [f64; 12] = [44.77559339031195, 40.44247144931402, 44.77559339031195, 43.331219409979305, 44.77559339031195, 43.331219409979305, 44.77559339031195, 44.77559339031195, 43.331219409979305, 44.77559339031195, 43.331219409979305, 44.77559339031195];
    #[rustfmt::skip]
const SER_H2_DIS: [f64; 12] = [2744.91506166691, 2479.2781202152732, 2744.91506166691, 2656.3694145163645, 2744.91506166691, 2656.3694145163645, 2744.91506166691, 2744.91506166691, 2656.3694145163645, 2744.91506166691, 2656.3694145163645, 2744.91506166691];
    #[rustfmt::skip]
const SER_H2_HP: [f64; 12] = [1298.2570871573448, 1172.6193045292146, 1298.2570871573448, 1256.3778262813014, 1298.2570871573448, 1256.3778262813014, 1298.2570871573448, 1298.2570871573448, 1256.3778262813014, 1298.2570871573448, 1256.3778262813014, 1298.2570871573448];
    #[rustfmt::skip]
const SER_H2_BOILER: [f64; 12] = [1446.657974509565, 1306.6588156860587, 1446.657974509565, 1399.991588235063, 1446.657974509565, 1399.991588235063, 1446.657974509565, 1446.657974509565, 1399.991588235063, 1446.657974509565, 1399.991588235063, 1446.657974509565];
    #[rustfmt::skip]
const SER_H2_ELEC: [f64; 12] = [927.3264908266749, 837.5852175208676, 927.3264908266749, 897.4127330580725, 927.3264908266749, 897.4127330580725, 927.3264908266749, 927.3264908266749, 897.4127330580725, 927.3264908266749, 897.4127330580725, 927.3264908266749];
    #[rustfmt::skip]
const SER_H2_GAS: [f64; 12] = [1607.3977494550722, 1451.8431285400652, 1607.3977494550722, 1555.5462091500701, 1607.3977494550722, 1555.5462091500701, 1607.3977494550722, 1607.3977494550722, 1555.5462091500701, 1607.3977494550722, 1555.5462091500701, 1607.3977494550722];
    #[rustfmt::skip]
const SER_H2_AUX: [f64; 12] = [1.5913237719605218, 1.4373246972546647, 1.5913237719605218, 1.5399907470585694, 1.5913237719605218, 1.5399907470585694, 1.5913237719605218, 1.5913237719605218, 1.5399907470585694, 1.5913237719605218, 1.5399907470585694, 1.5913237719605218];
    #[rustfmt::skip]
const SER_H2_SHARE: [f64; 12] = [0.47296803652968034, 0.47296803652968034, 0.47296803652968034, 0.47296803652968034, 0.47296803652968034, 0.47296803652968034, 0.47296803652968034, 0.47296803652968034, 0.47296803652968034, 0.47296803652968034, 0.47296803652968034, 0.47296803652968034];

    fn hotfill_case() -> HotWaterAssessment {
        let mut input = dwelling_100(HotWaterGenerator::GasAppliance {
            appliance: GasAppliance::CombiGaskeurHrCw,
            measured_class: None,
            kitchen_only: false,
            declared: None,
            annex_t: None,
            annex_t_conditions: None,
        });
        input.nominal_power_kw = Some(24.0);
        input.storage = vec![vessel(
            30.0,
            StorageLoss::Label {
                label: StorageLabel::B,
            },
            2,
        )];
        input.additional_generators = vec![AdditionalHotWaterGenerator {
            generator: HotWaterGenerator::ElectricBoiler,
            nominal_power_kw: Some(2.0),
            exhaust_air: None,
            declared_share: None,
            equipment_reference: "r77 synthetic".into(),
        }];
        input.series = Some(SeriesArrangement::HotfillElectricBoiler);
        run(&input, context(true, 100.0))
    }

    /// Hotfill: an individual combi appliance with an electric boiler of 30 l (label B, f_sto;dis;ls 2) in series
    /// (13.141a, p. 611): the combi supplies 0,8·Q_W;dis;nren, the electric boiler the rest at 1,0 (13.8.4.5.3,
    /// p. 636); the vessel loss counts in 13.7 and as recoverable loss (13.59, 13.63).
    #[test]
    fn hotfill_electric_boiler_after_a_combi() {
        let result = hotfill_case();
        let m = &result.months;
        months("Q_W;sto;ls 13.59", |i| m[i].storage_loss_kwh, &SER_H1_STO);
        months(
            "Q_W;dis;nren 13.7",
            |i| m[i].generator_output_kwh,
            &SER_H1_DIS,
        );
        let by_index = |index: usize| {
            result
                .generators
                .iter()
                .find(|item| item.index == index)
                .expect("generator")
        };
        months(
            "Q_W;gen;out combi 13.141a",
            |i| by_index(0).monthly_output_kwh[i],
            &SER_H1_FIRST,
        );
        months(
            "Q_W;gen;out electric boiler",
            |i| by_index(1).monthly_output_kwh[i],
            &SER_H1_SECOND,
        );
        months(
            "E_W electricity 13.3",
            |i| m[i].electricity_kwh,
            &SER_H1_ELEC,
        );
        months("W_W;aux", |i| m[i].auxiliary_electricity_kwh, &[0.0; 12]);
        months(
            "Q_W;ls;rbl 13.63",
            |i| m[i].recoverable_loss_kwh,
            &SER_H1_RBL,
        );
    }

    /// Kernel deviation (src/domestic_hot_water.rs:4493-4502, book_generator gets the system total): for an
    /// appliance in series p. 644 takes c_W;gen at the appliance's own Q_W;gen;gi;out;an instead of
    /// Q_W;dis;nren;an. The combi of the hotfill case supplies 0,8 of the demand, so table 13.26 class 4 is read at
    /// 0,8·Q_W;dis;nren;an on 0,675 of table 13.25 (p. 641).
    #[test]
    fn hotfill_combi_takes_c_w_gen_at_its_own_output() {
        let result = hotfill_case();
        months(
            "E_W gas 13.3",
            |i| result.months[i].natural_gas_kwh,
            &SER_H1_GAS,
        );
    }

    /// Collective series of 1 200 m² (15 dwellings): the first appliance, an indirect heat pump that also heats,
    /// takes f_pref;serie of 13.141b from ϑ_H,max per month, the HR 107 boiler 13.141c, both by 13.141d
    /// (p. 611-612); 1,4 and 0,90 (p. 645-646), a 1 000 l label C vessel and 13.181 for the boiler (p. 647).
    #[test]
    fn collective_series_with_f_pref() {
        let mut input = system(
            HotWaterNeed::Residential {
                dwelling_count: 15,
                source_reference: "r77 synthetic".into(),
            },
            residential_emission(),
            HotWaterGenerator::IndirectHeatPump {
                also_space_heating: true,
            },
        );
        input.collective = collective(1200.0);
        input.storage = vec![vessel(
            1000.0,
            StorageLoss::Label {
                label: StorageLabel::C,
            },
            3,
        )];
        input.additional_generators = vec![AdditionalHotWaterGenerator {
            generator: indirect_hr107(),
            nominal_power_kw: Some(100.0),
            exhaust_air: None,
            declared_share: None,
            equipment_reference: "r77 synthetic".into(),
        }];
        input.series = Some(SeriesArrangement::CollectiveFirstAlsoHeating {
            maximum_supply_c: vec![
                45.0, 45.0, 43.0, 40.0, 35.0, 30.0, 30.0, 30.0, 35.0, 40.0, 43.0, 45.0,
            ],
        });
        let result = run(&input, context(true, 1200.0));
        let m = &result.months;
        months(
            "Q_W;dis;nren 13.7",
            |i| m[i].generator_output_kwh,
            &SER_H2_DIS,
        );
        let by_index = |index: usize| {
            result
                .generators
                .iter()
                .find(|item| item.index == index)
                .expect("generator")
        };
        months(
            "f_pref;serie 13.141b",
            |i| by_index(0).monthly_share[i],
            &SER_H2_SHARE,
        );
        months(
            "Q_W;gen;out heat pump 13.141d",
            |i| by_index(0).monthly_output_kwh[i],
            &SER_H2_HP,
        );
        months(
            "Q_W;gen;out boiler 13.141c/d",
            |i| by_index(1).monthly_output_kwh[i],
            &SER_H2_BOILER,
        );
        months(
            "E_W electricity 13.3",
            |i| m[i].electricity_kwh,
            &SER_H2_ELEC,
        );
        months("E_W gas 13.3", |i| m[i].natural_gas_kwh, &SER_H2_GAS);
        months(
            "W_W;aux 13.181",
            |i| m[i].auxiliary_electricity_kwh,
            &SER_H2_AUX,
        );
    }
    #[rustfmt::skip]
const SET_U_EM: [f64; 12] = [47.56164383561644, 42.95890410958904, 47.56164383561644, 46.02739726027397, 47.56164383561644, 46.02739726027397, 47.56164383561644, 47.56164383561644, 46.02739726027397, 47.56164383561644, 46.02739726027397, 47.56164383561644];
    #[rustfmt::skip]
const SET_U_CONV: [f64; 12] = [11.89041095890411, 10.73972602739726, 11.89041095890411, 11.506849315068493, 11.89041095890411, 11.506849315068493, 11.89041095890411, 11.89041095890411, 11.506849315068493, 11.89041095890411, 11.506849315068493, 11.89041095890411];
    #[rustfmt::skip]
const SET_U_DIS: [f64; 12] = [59.45205479452055, 53.6986301369863, 59.45205479452055, 57.534246575342465, 59.45205479452055, 57.534246575342465, 59.45205479452055, 59.45205479452055, 57.534246575342465, 59.45205479452055, 57.534246575342465, 59.45205479452055];
    #[rustfmt::skip]
const SET_U_AUX: [f64; 12] = [14.88, 13.44, 14.88, 14.4, 14.88, 14.4, 14.88, 14.88, 14.4, 14.88, 14.4, 14.88];
    #[rustfmt::skip]
const SET_U_RBL: [f64; 12] = [11.89041095890411, 10.73972602739726, 11.89041095890411, 11.506849315068493, 11.89041095890411, 11.506849315068493, 11.89041095890411, 11.89041095890411, 11.506849315068493, 11.89041095890411, 11.506849315068493, 11.89041095890411];
    #[rustfmt::skip]
const SET_U_HEAT: [f64; 12] = [59.45205479452055, 53.6986301369863, 59.45205479452055, 57.534246575342465, 59.45205479452055, 57.534246575342465, 59.45205479452055, 59.45205479452055, 57.534246575342465, 59.45205479452055, 57.534246575342465, 59.45205479452055];
    #[rustfmt::skip]
const SET_R_EM: [f64; 12] = [492.1416126379081, 444.51500496327185, 492.1416126379081, 476.26607674636267, 492.1416126379081, 476.26607674636267, 492.1416126379081, 492.1416126379081, 476.26607674636267, 492.1416126379081, 476.26607674636267, 492.1416126379081];
    #[rustfmt::skip]
const SET_R_CONV: [f64; 12] = [66.96, 60.48, 66.96, 64.8, 66.96, 64.8, 66.96, 66.96, 64.8, 66.96, 64.8, 66.96];
    #[rustfmt::skip]
const SET_R_DIS: [f64; 12] = [559.1016126379081, 504.99500496327187, 559.1016126379081, 541.0660767463627, 559.1016126379081, 541.0660767463627, 559.1016126379081, 559.1016126379081, 541.0660767463627, 559.1016126379081, 541.0660767463627, 559.1016126379081];
    #[rustfmt::skip]
const SET_R_AUX: [f64; 12] = [22.32, 20.16, 22.32, 21.6, 22.32, 21.6, 22.32, 22.32, 21.6, 22.32, 21.6, 22.32];
    #[rustfmt::skip]
const SET_R_HEAT: [f64; 12] = [559.1016126379081, 504.99500496327187, 559.1016126379081, 541.0660767463627, 559.1016126379081, 541.0660767463627, 559.1016126379081, 559.1016126379081, 541.0660767463627, 559.1016126379081, 541.0660767463627, 559.1016126379081];

    fn external_heat_with_sets(
        need: HotWaterNeed,
        emission: HotWaterEmission,
        count: u32,
    ) -> HotWaterSystem {
        let mut input = system(need, emission, HotWaterGenerator::ExternalHeat);
        input.delivery_sets = Some(DeliverySets {
            count,
            source_reference: "r77 synthetic".into(),
            shared_with_heating: false,
        });
        input
    }

    /// Utility building (office 400 m², table 13.1) on external heat with two individual delivery sets: the
    /// conversion loss 13.24a with η_W;dis;conv 0,75 (p. 546), 13.7 (p. 531), the electronics 13.46 per set
    /// (p. 560), the recoverable loss 13.47 with f_W;conv;rbl 1 (p. 560) and η 1,0 for external heat
    /// (13.8.4.9.2, p. 652).
    #[test]
    fn utility_delivery_sets_13_24a() {
        let input = external_heat_with_sets(
            HotWaterNeed::Utility {
                areas: vec![UtilityArea {
                    function: LabelFunction::Office,
                    area_m2: 400.0,
                }],
                source_reference: "r77 synthetic".into(),
            },
            HotWaterEmission::Utility {
                mean_length_m: 2.5,
                source_reference: "r77 synthetic".into(),
            },
            2,
        );
        let result = run(&input, context(false, 400.0));
        let m = &result.months;
        months("Q_W;em", |i| m[i].emission_input_kwh, &SET_U_EM);
        months(
            "Q_W;dis;conv;ls 13.24a",
            |i| m[i].conversion_loss_kwh,
            &SET_U_CONV,
        );
        months("Q_W;dis 13.7", |i| m[i].generator_output_kwh, &SET_U_DIS);
        months(
            "W_W;conv;aux 13.46",
            |i| m[i].auxiliary_electricity_kwh,
            &SET_U_AUX,
        );
        months(
            "Q_W;dis;rbl 13.47",
            |i| m[i].recoverable_loss_kwh,
            &SET_U_RBL,
        );
        months("E_W external heat", |i| m[i].district_heat_kwh, &SET_U_HEAT);
    }

    /// Three dwellings with an individual delivery set each: 13.24 with P_conv;stb 30 W summed over the sets
    /// (p. 546) and 13.46 with 10 W per set (p. 560).
    #[test]
    fn residential_delivery_sets_13_24() {
        let input = external_heat_with_sets(
            HotWaterNeed::Residential {
                dwelling_count: 3,
                source_reference: "r77 synthetic".into(),
            },
            residential_emission(),
            3,
        );
        let result = run(&input, context(true, 240.0));
        let m = &result.months;
        months("Q_W;em", |i| m[i].emission_input_kwh, &SET_R_EM);
        months(
            "Q_W;dis;conv;ls 13.24",
            |i| m[i].conversion_loss_kwh,
            &SET_R_CONV,
        );
        months("Q_W;dis 13.7", |i| m[i].generator_output_kwh, &SET_R_DIS);
        months(
            "W_W;conv;aux 13.46",
            |i| m[i].auxiliary_electricity_kwh,
            &SET_R_AUX,
        );
        months("E_W external heat", |i| m[i].district_heat_kwh, &SET_R_HEAT);
    }

    /// Annex T: T.4 with η_el;ow = 1/f_P;del;el (1,45, table 5.2) and T.5 with table T.7 (p. 1102-1105); the
    /// forfait combi conversion T.15-T.17 with K_f 0,5 and T.6 (p. 1107-1108); the measured combi T.12-T.14
    /// (p. 1106-1107). Values before the rounding of table 13.25 (p. 642).
    #[test]
    fn annex_t_efficiencies() {
        let water_heater = AnnexTTest::WaterHeater {
            useful_mj: 30.0,
            fuel_input_mj: 60.0,
            electricity_kwh: 0.1,
            fuel: TestFuel::NaturalGas,
            source_reference: "r77 synthetic".into(),
        };
        close(
            "T.4/T.5",
            water_heater.efficiency().expect("valid"),
            0.44711014176663033,
        );
        let forfait = AnnexTTest::CombiForfait {
            useful_mj: 30.0,
            fuel_input_mj: 55.0,
            electricity_kwh: 0.05,
            full_load_efficiency: 0.95,
            fuel: TestFuel::Propane,
            source_reference: "r77 synthetic".into(),
        };
        close(
            "T.15-T.17/T.6",
            forfait.efficiency().expect("valid"),
            0.5940925229090493,
        );
        let measured = AnnexTTest::CombiMeasured {
            summer_useful_mj_per_day: 25.0,
            summer_fuel_mj_per_day: 45.0,
            winter_useful_mj_per_day: 25.0,
            winter_fuel_mj_per_day: 60.0,
            heating_fuel_mj_per_day: 20.0,
            full_load_efficiency: 0.9,
            summer_electricity_kwh_per_day: 0.1,
            electronics_kwh_per_day: 0.03,
            fuel: TestFuel::Butane,
            source_reference: "r77 synthetic".into(),
        };
        close(
            "T.12-T.14/T.6",
            measured.efficiency().expect("valid"),
            0.5610803341621999,
        );
    }

    /// Annex U: U.5/U.6 (p. 1113), U.3/U.4 over the samples and U.2 for one run, and the mean of three runs
    /// rounded down to 0,025 (p. 1112-1113); table U.1 conditions (p. 1110).
    #[test]
    fn annex_u_efficiency() {
        close("U.5 10 °C", water_density(10.0), 999.6318319999999);
        close("U.5 40 °C", water_density(40.0), 992.3117479999999);
        close("U.6 10 °C", water_enthalpy(10.0), 42.01289592940001);
        close("U.6 40 °C", water_enthalpy(40.0), 167.44925355040002);
        let sample = |cold_in_c: f64, cold_out_c: f64, shower_c: f64| ShowerSample {
            cold_flow_m3_per_s: 2.0e-4,
            cold_in_c,
            cold_out_c,
            shower_flow_m3_per_s: 2.0e-4,
            shower_c,
        };
        let run = ShowerRun::Samples {
            sample_time_s: 1.0,
            samples: vec![
                sample(10.0, 24.0, 40.0),
                sample(10.2, 25.1, 39.8),
                sample(9.9, 25.6, 40.1),
            ],
        };
        close(
            "U.2-U.4",
            run.efficiency().expect("run"),
            0.5003933358817253,
        );
        let test = AnnexUTest {
            class: ShowerTestClass::Class3,
            runs: vec![
                run,
                ShowerRun::Energies {
                    recovered_kj: 512.0,
                    shower_kj: 1000.0,
                },
                ShowerRun::Energies {
                    recovered_kj: 498.0,
                    shower_kj: 1000.0,
                },
            ],
            source_reference: "r77 synthetic".into(),
        };
        close("mean of three runs", test.efficiency().expect("test"), 0.5);
        let conditions = [
            (ShowerTestClass::Class2, (5.8, 47.0)),
            (ShowerTestClass::Class3, (9.2, 73.0)),
            (ShowerTestClass::Class4, (12.5, 100.0)),
        ];
        for (class, (flow, volume)) in conditions {
            let (kernel_flow, kernel_volume) = class.conditions();
            close(&format!("table U.1 {class:?} flow"), kernel_flow, flow);
            close(
                &format!("table U.1 {class:?} volume"),
                kernel_volume,
                volume,
            );
        }
    }

    /// Tables 13.22 (p. 637) and 13.28 (p. 645) row by row and the 0,50 of §13.8.4.5.4 (p. 636-637), all forfait
    /// values with f_prac 1 (13.152): the carrier input is Q_W;dis;nren/η in every month.
    #[test]
    fn tables_13_22_and_13_28_rows() {
        let check = |generator: HotWaterGenerator, eta: f64, carrier: fn(&HotWaterMonth) -> f64| {
            let label = format!("{generator:?}");
            let indirect = matches!(generator, HotWaterGenerator::IndirectBoiler { .. });
            let mut input = dwelling_100(generator);
            if indirect {
                input.storage = vec![vessel(
                    120.0,
                    StorageLoss::Label {
                        label: StorageLabel::B,
                    },
                    3,
                )];
            }
            let result = run(&input, context(true, 100.0));
            for month in &result.months {
                close(
                    &format!("{label} m{}", month.month),
                    carrier(month),
                    month.generator_output_kwh / eta,
                );
            }
        };
        use BiomassStoreInsulation::*;
        for (insulation, inside, eta) in [
            (AtLeast20Mm, true, 0.65),
            (AtLeast20Mm, false, 0.60),
            (AtLeast10Mm, true, 0.575),
            (AtLeast10Mm, false, 0.525),
            (None, true, 0.35),
            (None, false, 0.325),
        ] {
            check(
                HotWaterGenerator::BiomassCombi {
                    insulation,
                    inside_boundary: inside,
                },
                eta,
                |month| month.biomass_kwh,
            );
        }
        use IndirectBoiler::*;
        for (boiler, inside, eta) in [
            (ConventionalOrUnknown, true, 0.75),
            (Vr, true, 0.80),
            (Hr100Or104, true, 0.85),
            (Hr107, true, 0.90),
            (ConventionalOrUnknown, false, 0.70),
            (Vr, false, 0.75),
            (Hr100Or104, false, 0.80),
            (Hr107, false, 0.85),
        ] {
            check(
                HotWaterGenerator::IndirectBoiler {
                    boiler,
                    oil: false,
                    inside_boundary: inside,
                    also_space_heating: true,
                    declared: Option::None,
                    pilot_flame: false,
                },
                eta,
                |month| month.natural_gas_kwh,
            );
        }
        check(
            HotWaterGenerator::IndirectBoiler {
                boiler: ConventionalOrUnknown,
                oil: true,
                inside_boundary: true,
                also_space_heating: true,
                declared: Option::None,
                pilot_flame: false,
            },
            0.75,
            |month| month.oil_kwh,
        );
        check(
            HotWaterGenerator::LargeDirectStorage { gas_fired: true },
            0.50,
            |month| month.natural_gas_kwh,
        );
    }

    /// Table 13.16 (p. 604): f_PVT;th of a single-glazed PVT collector from A_sol;mod·N_col/V_sto;tot (below 0,015,
    /// 0,015-0,03 and above 0,03 m²/dm³), 0,9 unglazed and 1 when tested to NEN-EN-ISO 9806 (13.66, p. 574-575).
    #[test]
    fn pvt_reduction_table_13_16() {
        let heater = |pvt: Option<PvtCover>, area: f64, volume: f64| SolarWaterHeater {
            id: "pvt".into(),
            solar_use: SolarUse::WaterHeating,
            count: 1,
            method: SolarMethod::Calculated {
                solar_type: SolarType::Preheater,
                collectors: CollectorField {
                    module_area_m2: area,
                    module_count: 2,
                    orientation: Orientation::South,
                    tilt_deg: 45.0,
                    obstruction: CollectorObstruction::Minimal,
                    efficiency: CollectorEfficiency::Forfait {
                        collector: CollectorType::Glazed,
                    },
                    heat_exchanger_w_per_k: None,
                    loop_pipes: LoopPipes::Forfait,
                    pump_power_w: None,
                },
                storage: SolarStorage {
                    total_volume_l: volume,
                    backup_volume_l: None,
                    loss: StorageLoss::Label {
                        label: StorageLabel::B,
                    },
                    backup_loss_in_generator_efficiency: false,
                },
            },
            pvt,
            source_reference: "r77 synthetic".into(),
        };
        let glazed = Some(PvtCover::SingleGlazed);
        // 2 modules: 2·0,7/200 = 0,007; 2·2/200 = 0,02; 2·3/150 = 0,04; the bounds 0,015 and 0,03 inclusive.
        for (pvt, area, volume, expected) in [
            (glazed, 0.7, 200.0, 0.76),
            (glazed, 2.0, 200.0, 0.83),
            (glazed, 1.5, 200.0, 0.83),
            (glazed, 3.0, 200.0, 0.83),
            (glazed, 3.0, 150.0, 0.89),
            (Some(PvtCover::Unglazed), 2.0, 200.0, 0.9),
            (Some(PvtCover::TestedIso9806), 2.0, 200.0, 1.0),
            (None, 2.0, 200.0, 1.0),
        ] {
            close(
                &format!("f_PVT;th {pvt:?} {area} {volume}"),
                heater(pvt, area, volume).pvt_factor().expect("factor"),
                expected,
            );
        }
    }
}

mod part_c {
    // Review 77 part C: coverage of NTA 8800:2025+C1:2026 chapters 14 (lighting), 16 (electricity
    // production), 17 (climate data) and annexes V, X and Y beyond what earlier reviews pinned. The
    // expected values come from a separate implementation written from the norm pages (Python, review
    // 77); only numbers and page references are kept here.

    use nta8800_core::annex_v::{
        calculate_regeneration, validate_regeneration, RegenerationContext, RegenerationInput,
        SolarRegeneration,
    };
    use nta8800_core::climate::{
        irradiance_at, ARGII_TEMPERATURE_C, COLD_RECOVERY_SUPPLY_TEMPERATURE_C, MONTH_HOURS,
        OUTDOOR_TEMPERATURE_C, WIND_SPEED_M_PER_S,
    };
    use nta8800_core::label_class::LabelFunction;
    use nta8800_core::lighting::{
        absence_factors, assess_zone_lighting, burning_hours, occupancy_factor, rooflight_supply,
        rooflight_utilance, round_up_annex_x, specific_power, validate_lighting,
        vertical_daylight_supply, Daylight, DaylightControl, DaylightSector, InstalledPower,
        LampTechnology, LightingContext, LightingZone, LuminaireGroup, LuminairePower, Occupancy,
        ParasiticPower, SwitchControl, UseArea, ZoneLighting,
    };
    use nta8800_core::pv::{
        monthly_yield_kwh, shading_correction, validate_pv, CollectivePv, PeakPower, PvModuleType,
        PvMounting, PvSystem, PvtCover,
    };
    use nta8800_core::significant_figures::{round_down, round_up};

    fn close(label: &str, actual: f64, expected: f64) {
        let tolerance = 1e-6 * expected.abs().max(1e-9);
        assert!(
            (actual - expected).abs() <= tolerance,
            "{label}: kernel {actual}, expected {expected}"
        );
    }

    fn occupancy(control: SwitchControl, central: bool, large_group: bool) -> Occupancy {
        Occupancy {
            control,
            central_on_control: central,
            large_office_group: large_group,
        }
    }

    fn lamps(
        count: u32,
        lamp_power_w: f64,
        lamp_count: u32,
        technology: LampTechnology,
    ) -> LuminaireGroup {
        LuminaireGroup {
            count,
            power: LuminairePower::Lamps {
                lamp_power_w,
                lamp_count,
                technology,
            },
        }
    }

    fn installed(luminaires: Vec<LuminaireGroup>, dynamic_factor: Option<f64>) -> InstalledPower {
        InstalledPower::Installed {
            luminaires,
            dynamic_factor,
            source_reference: "r77 luminaire schedule".into(),
        }
    }

    fn parasitic(emergency_charging_w: f64, control_standby_w: f64) -> ParasiticPower {
        ParasiticPower::Installed {
            emergency_charging_w,
            control_standby_w,
            source_reference: "r77 schedule".into(),
        }
    }

    fn lighting_zone(
        id: &str,
        area_m2: f64,
        power: InstalledPower,
        parasitic: ParasiticPower,
        occupancy: Occupancy,
        daylight: Daylight,
    ) -> LightingZone {
        LightingZone {
            id: id.into(),
            area_m2,
            power,
            parasitic,
            occupancy,
            daylight,
            extracted_luminaires: false,
            constant_illuminance: None,
        }
    }

    fn zone(functions: &[(LabelFunction, f64)], lighting_zones: Vec<LightingZone>) -> ZoneLighting {
        ZoneLighting {
            zone_id: "z1".into(),
            functions: functions
                .iter()
                .map(|(function, area_m2)| UseArea {
                    function: *function,
                    area_m2: *area_m2,
                })
                .collect(),
            lighting_zones,
            source_reference: "r77 lighting plan".into(),
            burning_hours_factor: None,
        }
    }

    /// Installed power and daylight in a two-function zone (office 150 m², retail 50 m²), p. 655-675:
    /// t_D/t_N and F_A weighted by area (tables 14.1 and 14.4), 14.8 with f_dyn 0,9 and 14.9 with every
    /// table 14.2 row, rounded up per annex X; parasitic 14.10-14.12 rounded up per annex X;
    /// occupancy 14.18 and 14.22 (table 14.5 rows 0,90 and 0,95), 14.16/14.17 for a central on-control
    /// and 14.16 for a large office group; daylight 14.25-14.30 (alternative 14.30, heavily shaded,
    /// table 14.7 interpolated), 14.31-14.35 and 14.41/14.42 (table 14.9 interpolated, table 14.8
    /// 0,68), and the forfait method 14.43/14.44 with and without a daylight control.
    #[test]
    fn lighting_installed_power_daylight_and_occupancy_14_7_to_14_44() {
        let zone_a = lighting_zone(
            "lz-a",
            120.0,
            installed(
                vec![
                    lamps(30, 18.0, 2, LampTechnology::Led),
                    lamps(10, 58.0, 1, LampTechnology::FluorescentT8Conventional),
                    lamps(6, 26.0, 2, LampTechnology::CompactFluorescentNotIntegrated),
                    LuminaireGroup {
                        count: 4,
                        power: LuminairePower::System { power_w: 45.0 },
                    },
                ],
                Some(0.9),
            ),
            parasitic(41.0, 12.3),
            occupancy(SwitchControl::ManualOnDimmed, false, false),
            Daylight::Sectors {
                sectors: vec![
                    DaylightSector::VerticalWindows {
                        sector_width_m: 8.0,
                        lintel_height_m: 2.55,
                        actual_depth_m: 5.0,
                        use_actual_depth_alternative: true,
                        opening_area_m2: 9.0,
                        heavily_shaded: true,
                        control: DaylightControl::AutomaticSwitchingOrUnknown,
                    },
                    DaylightSector::Rooflights {
                        rooflight_depth_m: 3.0,
                        rooflight_width_m: 1.5,
                        clear_height_m: 4.25,
                        wall_distances_m: [2.0, 8.0, 1.0, 6.0],
                        opening_area_m2: 4.5,
                        room_length_m: 12.0,
                        room_width_m: 9.0,
                        luminaire_height_m: 2.2,
                        control: DaylightControl::AutomaticDimming,
                    },
                ],
                source_reference: "r77 facade and roof drawing".into(),
            },
        );
        let zone_b = lighting_zone(
            "lz-b",
            50.0,
            installed(
                vec![
                    lamps(20, 36.0, 2, LampTechnology::FluorescentT12),
                    lamps(5, 58.0, 1, LampTechnology::FluorescentT8Electronic),
                    lamps(3, 60.0, 1, LampTechnology::NoSeparateBallast),
                    lamps(2, 40.0, 1, LampTechnology::UnknownOrOther),
                ],
                None,
            ),
            parasitic(8.2, 0.0),
            occupancy(SwitchControl::AutoOnAutoOff, true, false),
            Daylight::Forfait {
                daylight_control: true,
            },
        );
        let zone_c = lighting_zone(
            "lz-c",
            30.0,
            installed(
                vec![LuminaireGroup {
                    count: 10,
                    power: LuminairePower::System { power_w: 25.0 },
                }],
                None,
            ),
            parasitic(3.0, 0.95),
            occupancy(SwitchControl::ManualWithSweep, false, true),
            Daylight::Forfait {
                daylight_control: false,
            },
        );
        let input = zone(
            &[
                (LabelFunction::Office, 150.0),
                (LabelFunction::Retail, 50.0),
            ],
            vec![zone_a, zone_b, zone_c],
        );
        assert!(validate_lighting(&input, 200.0, "lighting").is_empty());
        // 14.44: sum of A_g of the plot 1 000 m², sum of A_w 150 m².
        let context = LightingContext {
            total_usable_floor_area_m2: 1000.0,
            total_window_area_m2: 150.0,
        };
        let result = assess_zone_lighting(&input, 200.0, context).expect("valid lighting");
        let [a, b, c] = &result.lighting_zones[..] else {
            panic!("three lighting zones expected");
        };
        // Zone A: 2 401,2 W x 0,9 = 2 161,08 W -> 2 200 W; parasitic 44 + 13 W.
        close("14.8 P_n zone A", a.installed_power_w, 2200.0);
        close("14.10 W_P zone A", a.parasitic_kwh, 499.32);
        close("14.18 F_o;D zone A", a.occupancy_day, 0.925);
        close("14.22 F_o;N zone A", a.occupancy_night, 0.6);
        close("14.25 F_D zone A", a.daylight_factor, 0.7024462205405406);
        close("14.7 W_L zone A", a.lighting_kwh, 3752.5364867100006);
        // Zone B: 2 294 W -> 2 400 W; 8,2 W -> 8,5 W; f_dayl;F = 0,27.
        close("14.8 P_n zone B", b.installed_power_w, 2400.0);
        close("14.10 W_P zone B", b.parasitic_kwh, 74.46);
        close("14.16 F_o;D zone B", b.occupancy_day, 1.0);
        close("14.17 F_o;N zone B", b.occupancy_night, 1.0);
        close("14.43 F_D zone B", b.daylight_factor, 0.8299);
        close("14.7 W_L zone B", b.lighting_kwh, 5410.842);
        // Zone C: 250 W -> 260 W; 3,0 W -> 3,2 W and 0,95 W -> 1,0 W.
        close("14.8 P_n zone C", c.installed_power_w, 260.0);
        close("14.10 W_P zone C", c.parasitic_kwh, 36.792);
        close("14.16 F_o;D zone C", c.occupancy_day, 1.0);
        close("14.22 F_o;N zone C", c.occupancy_night, 0.65);
        close("14.43 F_D zone C", c.daylight_factor, 1.0);
        close("14.7 W_L zone C", c.lighting_kwh, 659.425);
        close("14.5 W_t", result.annual_kwh, 10433.375486709998);
        close("W_t February", result.monthly_kwh[1], 800.3685304873424);
    }

    /// Forfait power 14.13 with table 14.3 weighted over five functions (footnote a: no LED value for
    /// cell, healthcare with beds, lodging and retail), tables 14.1/14.4 weighted, 14.18 and 14.22 with
    /// F_oc 0,95, 14.24 (F_D = 1 with forfait power, also with a daylight control) and 14.14 (p. 658-667).
    #[test]
    fn lighting_forfait_power_weighted_tables_14_1_14_3_14_4() {
        let functions = [
            (LabelFunction::Cell, 40.0),
            (LabelFunction::HealthcareWithBeds, 60.0),
            (LabelFunction::Lodging, 20.0),
            (LabelFunction::Retail, 30.0),
            (LabelFunction::AssemblyWithDayCare, 50.0),
        ];
        let context = LightingContext {
            total_usable_floor_area_m2: 1000.0,
            total_window_area_m2: 150.0,
        };
        for (led, power, lighting) in [
            (true, 3540.0, 13241.325749999998),
            (false, 3740.0, 13989.423249999998),
        ] {
            let item = lighting_zone(
                "lz-forfait",
                200.0,
                InstalledPower::Forfait { led_from_2017: led },
                ParasiticPower::Forfait,
                occupancy(SwitchControl::AutoOnDimmed, false, false),
                Daylight::Forfait {
                    daylight_control: true,
                },
            );
            let input = zone(&functions, vec![item]);
            assert!(validate_lighting(&input, 200.0, "lighting").is_empty());
            let result = assess_zone_lighting(&input, 200.0, context).expect("valid lighting");
            let item = &result.lighting_zones[0];
            close(
                &format!("14.13 P_n led {led}"),
                item.installed_power_w,
                power,
            );
            close("14.18 F_o;D", item.occupancy_day, 0.9724999999999999);
            close("14.22 F_o;N", item.occupancy_night, 0.65);
            close("14.24 F_D", item.daylight_factor, 1.0);
            close(&format!("14.7 W_L led {led}"), item.lighting_kwh, lighting);
            close("14.14 W_P", item.parasitic_kwh, 500.0);
            close("W_t", result.annual_kwh, lighting + 500.0);
        }
    }

    /// Every row of tables 14.1 (p. 658), 14.2 (p. 660), 14.3 (p. 662), 14.4-14.5 (p. 666) and 14.6
    /// (p. 672); tables 14.7 and 14.9 at and between the columns, table 14.8 at its class bounds
    /// (p. 673-674); 14.20/14.23 for F_A of 0,9 and above (p. 665).
    #[test]
    fn lighting_table_rows_and_occupancy_branches() {
        use LabelFunction::*;
        let rows = [
            (AssemblyWithDayCare, (2200.0, 300.0), 16.0, 12.0, 0.2),
            (AssemblyWithoutDayCare, (2200.0, 300.0), 16.0, 12.0, 0.2),
            (Cell, (4000.0, 1000.0), 17.0, 17.0, 0.2),
            (HealthcareWithBeds, (4000.0, 1000.0), 17.0, 17.0, 0.0),
            (HealthcareWithoutBeds, (2200.0, 300.0), 16.0, 12.0, 0.2),
            (Office, (2200.0, 300.0), 16.0, 12.0, 0.2),
            (Lodging, (4000.0, 1000.0), 17.0, 17.0, 0.2),
            (Education, (1600.0, 300.0), 16.0, 12.0, 0.2),
            (Sport, (2200.0, 800.0), 16.0, 12.0, 0.2),
            (Retail, (2700.0, 400.0), 30.0, 30.0, 0.0),
        ];
        for (function, hours, other, led, absence_day) in rows {
            assert_eq!(burning_hours(function), hours, "table 14.1 {function:?}");
            close("table 14.3 other", specific_power(function, false), other);
            close("table 14.3 led", specific_power(function, true), led);
            assert_eq!(
                absence_factors(function),
                (absence_day, 0.5),
                "table 14.4 {function:?}"
            );
        }
        for (technology, percent) in [
            (LampTechnology::NoSeparateBallast, 0.0),
            (LampTechnology::Led, 8.0),
            (LampTechnology::FluorescentT5, 10.0),
            (LampTechnology::FluorescentT8Electronic, 0.0),
            (LampTechnology::FluorescentT8Conventional, 20.0),
            (LampTechnology::FluorescentT12, 20.0),
            (LampTechnology::CompactFluorescentNotIntegrated, 15.0),
            (LampTechnology::UnknownOrOther, 20.0),
        ] {
            close("table 14.2", technology.ballast_percent(), percent);
        }
        for (control, factor) in [
            (SwitchControl::ManualOrUnknown, 1.0),
            (SwitchControl::ManualWithSweep, 0.95),
            (SwitchControl::AutoOnDimmed, 0.95),
            (SwitchControl::AutoOnAutoOff, 0.9),
            (SwitchControl::ManualOnDimmed, 0.9),
            (SwitchControl::ManualOnAutoOff, 0.8),
        ] {
            close("table 14.5", control.factor(), factor);
        }
        assert_eq!(DaylightControl::NoneOrManual.factor(), 0.0);
        close(
            "table 14.6",
            DaylightControl::AutomaticSwitchingOrUnknown.factor(),
            0.63,
        );
        close(
            "table 14.6",
            DaylightControl::AutomaticDimming.factor(),
            0.73,
        );
        for (daylight, supply) in [
            (0.13, 0.12),
            (0.3, 0.23027027027027025),
            (0.75, 0.43),
            (1.2, 0.556),
            (1.75, 0.65),
            (2.5, 0.7050000000000001),
            (4.0, 0.78),
            (6.5, 0.845),
            (10.0, 0.895),
            (15.0, 0.91),
            (18.0, 0.91),
        ] {
            close(
                &format!("table 14.7 D {daylight}"),
                vertical_daylight_supply(daylight),
                supply,
            );
        }
        for (index, utilance) in [
            (0.6, 0.4),
            (0.7, 0.47),
            (0.9, 0.5700000000000001),
            (1.1, 0.636),
            (1.4, 0.726),
            (1.75, 0.79),
            (2.25, 0.855),
            (2.75, 0.9),
            (3.5, 0.9450000000000001),
            (4.5, 0.985),
            (5.0, 1.0),
        ] {
            close(
                &format!("table 14.9 k {index}"),
                rooflight_utilance(index),
                utilance,
            );
        }
        for (daylight, supply) in [
            (0.0, 0.0),
            (1.999, 0.0),
            (2.0, 0.68),
            (3.999, 0.68),
            (4.0, 0.85),
            (6.999, 0.85),
            (7.0, 0.92),
            (20.0, 0.92),
        ] {
            assert_eq!(
                rooflight_supply(daylight),
                supply,
                "table 14.8 D {daylight}"
            );
        }
        for (absence, control, factor) in [
            (0.9, 0.8, 0.09999999999999998),
            (0.95, 0.8, 0.050000000000000044),
            (0.95, 0.95, 0.1250000000000001),
            (0.89999, 0.9, 0.20001000000000013),
        ] {
            close(
                &format!("14.19/14.20 F_A {absence}"),
                occupancy_factor(absence, control),
                factor,
            );
        }
        assert!(occupancy_factor(1.0, 0.8).abs() < 1e-12, "14.20 F_A 1,0");
    }

    /// Table X.1 (p. 1129): both kernel routines for rounding up (lighting::round_up_annex_x and
    /// significant_figures::round_up) agree with the next higher class over five decades, and rounding
    /// down keeps a class value and takes the next lower class otherwise; worked examples of p. 1129-1130
    /// not pinned elsewhere in the lighting routine.
    #[test]
    fn annex_x_table_x1_classes_over_five_decades() {
        const CLASSES: [f64; 33] = [
            10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 22.0, 24.0, 26.0,
            28.0, 30.0, 32.0, 34.0, 36.0, 38.0, 40.0, 44.0, 48.0, 52.0, 56.0, 60.0, 65.0, 70.0,
            75.0, 80.0, 85.0, 90.0, 95.0,
        ];
        for decade in [0.001, 0.01, 0.1, 1.0, 10.0, 100.0] {
            for (index, class) in CLASSES.iter().enumerate() {
                let next = CLASSES.get(index + 1).copied().unwrap_or(100.0);
                let value = class * decade;
                let label = format!("X.1 class {class} x {decade}");
                close(&format!("{label} up"), round_up(value), next * decade);
                close(
                    &format!("{label} up lighting"),
                    round_up_annex_x(value),
                    next * decade,
                );
                close(&format!("{label} down"), round_down(value), value);
                // Halfway to the next class: up gives the next class, down this class.
                let between = (class + next) / 2.0 * decade;
                close(
                    &format!("{label} between up"),
                    round_up(between),
                    next * decade,
                );
                close(
                    &format!("{label} between up lighting"),
                    round_up_annex_x(between),
                    next * decade,
                );
                close(&format!("{label} between down"), round_down(between), value);
            }
        }
        for (value, expected) in [(0.82, 0.85), (8.2, 8.5), (82.0, 85.0), (822.0, 850.0)] {
            close(
                &format!("X.2 up {value}"),
                round_up_annex_x(value),
                expected,
            );
        }
        assert_eq!(round_up_annex_x(0.0), 0.0);
        assert_eq!(round_down(0.0), 0.0);
    }

    fn pvt_system(cover: PvtCover) -> PvSystem {
        PvSystem {
            id: "pvt-1".into(),
            peak_power: PeakPower::Table16_1 {
                module_type: PvModuleType::MonocrystallineFrom2018,
                panel_area_m2: 12.3,
            },
            azimuth_deg: 202.5,
            tilt_deg: 30.0,
            mounting: PvMounting::StronglyVentilated,
            obstruction_factors: vec![0.87],
            obstruction: None,
            collective: Some(CollectivePv {
                building_usable_floor_area_m2: 400.0,
                source_reference: "r77 collective".into(),
            }),
            pvt: Some(cover),
            source_reference: "r77 datasheet".into(),
        }
    }

    /// PVT electricity 16.10 (p. 684) = 16.2/16.3 with 16.4a and table 16.1 (175 W/m², 12,3 m²), table
    /// 16.2 (0,82), table 16.3 interpolated at F_sh;obst 0,87 (0,848), f_prac 0,95, the collective
    /// split of p. 678 (100/400 m²) and table 16.4 at its class bounds 0,015 and 0,03 m²/dm³ (p. 685).
    /// The azimuth 202,5 lies midway between south and south-west: table 17.2 takes the higher of the
    /// two neighbouring values per month (16.2.2.1 note 2 and 17.2, p. 679 and 694); in August that is
    /// the south-west column.
    #[test]
    fn pvt_electricity_table_16_4_bounds_and_orientation_midway() {
        close("table 16.3 at 0,87", shading_correction(0.87), 0.848);
        let one = [
            9.76066328409883,
            12.063194712280048,
            23.676856481229837,
            35.44514261903663,
            40.80150533214382,
            39.50403230153318,
            37.90229841607486,
            38.327515430431646,
            28.805023553201277,
            19.79191921369743,
            10.250099290359934,
            7.402641659029409,
        ];
        for (area, factor) in [(3.0, 0.84), (6.0, 0.84), (6.02, 0.80), (2.98, 0.88)] {
            let cover = PvtCover::Glazed {
                collector_area_m2: area,
                storage_volume_l: 200.0,
            };
            assert_eq!(cover.factor(), Some(factor), "table 16.4 {area}/200");
            let system = pvt_system(cover);
            assert!(validate_pv(&system, "pv").is_empty());
            let months = monthly_yield_kwh(&system, 100.0);
            for (month, value) in months.iter().enumerate() {
                close(
                    &format!("16.10 {area}/200 m{}", month + 1),
                    *value,
                    one[month] / 0.84 * factor,
                );
            }
        }
        let unglazed = monthly_yield_kwh(&pvt_system(PvtCover::Unglazed), 100.0);
        close("16.10 unglazed m8", unglazed[7], 38.327515430431646 / 0.84);
        for (module, kpk) in [
            (PvModuleType::AmorphousSingleJunction, 65.0),
            (PvModuleType::AmorphousMultiJunction, 55.0),
            (PvModuleType::Cigs, 105.0),
            (PvModuleType::CdTe, 95.0),
        ] {
            close("table 16.1 thin film", module.peak_power_w_per_m2(), kpk);
        }
        for (mounting, factor) in [
            (PvMounting::NotVentilated, 0.76),
            (PvMounting::ModeratelyVentilated, 0.80),
            (PvMounting::StronglyVentilated, 0.82),
            (PvMounting::Unknown, 0.76),
        ] {
            close("table 16.2", mounting.performance_factor(), factor);
        }
    }

    /// Table 17.1 (p. 690): t_mi, θ_e;avg, θ_e;argII (none in January and December), u_site and
    /// θ_ODA;preh;WTWC; table 17.2 (p. 691-694) at the midway azimuths 157,5, 202,5, 67,5 and 337,5
    /// (higher neighbouring value) and between the 30° and 45° columns.
    #[test]
    fn climate_table_17_1_columns_and_table_17_2_orientation_rule() {
        let hours = [
            744.0, 672.0, 744.0, 720.0, 744.0, 720.0, 744.0, 744.0, 720.0, 744.0, 720.0, 744.0,
        ];
        let outdoor = [
            2.61, 4.82, 5.91, 9.32, 14.73, 16.12, 18.05, 18.48, 15.63, 10.40, 7.99, 4.00,
        ];
        let argii = [
            None,
            Some(13.97),
            Some(13.00),
            Some(13.70),
            Some(16.42),
            Some(16.76),
            Some(17.51),
            Some(18.24),
            Some(16.74),
            Some(15.04),
            Some(13.43),
            None,
        ];
        let wind = [
            3.04, 4.15, 2.99, 3.06, 2.97, 2.78, 2.63, 2.51, 2.71, 2.78, 2.83, 2.83,
        ];
        let cold_recovery = [
            0.0, 0.0, 0.0, 0.0, 25.63, 27.49, 26.34, 27.29, 25.30, 0.0, 0.0, 0.0,
        ];
        assert_eq!(MONTH_HOURS, hours);
        assert_eq!(OUTDOOR_TEMPERATURE_C, outdoor);
        assert_eq!(ARGII_TEMPERATURE_C, argii);
        assert_eq!(WIND_SPEED_M_PER_S, wind);
        assert_eq!(COLD_RECOVERY_SUPPLY_TEMPERATURE_C, cold_recovery);
        for (azimuth, tilt, month, expected) in [
            (202.5, 30.0, 1, 50.5),
            (202.5, 30.0, 8, 198.3),
            (202.5, 45.0, 6, 199.3),
            (157.5, 30.0, 1, 50.5),
            (157.5, 30.0, 6, 211.2),
            (67.5, 30.0, 3, 97.6),
            (337.5, 30.0, 5, 155.8),
            (200.0, 30.0, 8, 197.9),
            (180.0, 37.5, 5, 206.89999999999998),
            (180.0, 37.5, 9, 155.8),
        ] {
            close(
                &format!("table 17.2 az {azimuth} tilt {tilt} m{month}"),
                irradiance_at(azimuth, tilt, month).expect("in range"),
                expected,
            );
        }
    }

    /// Annex V (p. 1114-1116): V.1 with free cooling V.2 and without V.3, solar regeneration V.4/V.5
    /// (May-September, five F_sh;obst values, S 37,5° between the table 17.2 columns), V.6 with table V.2
    /// interpolated, a declared efficiency 0,47 rounded down to 0,44 (annex X), and table V.1.
    #[test]
    fn annex_v_regeneration_degree_and_correction() {
        let context = RegenerationContext {
            heating_kwh: 9000.0,
            heating_efficiency: 4.2,
            hot_water_kwh: 2500.0,
            hot_water_efficiency: 2.6,
            annual_cooling_need_kwh: 3500.0,
            electricity_efficiency: 1.0 / 1.45,
        };
        let field = |declared: Option<f64>| SolarRegeneration {
            collector_area_m2: 6.0,
            azimuth_deg: 180.0,
            tilt_deg: 37.5,
            obstruction_factors: vec![0.9, 0.95, 1.0, 0.85, 0.8],
            declared_efficiency: declared,
            source_reference: "r77 collector".into(),
        };
        for (free_cooling, declared, solar, degree, correction) in [
            (true, None, 1523.3176337158013, 0.536734426599834, 1.02),
            (false, None, 1523.3176337158013, 0.16276434744522947, 1.0),
            (true, Some(0.47), 1672.4873088, 0.5526729966566067, 1.02),
        ] {
            let input = RegenerationInput {
                free_cooling_from_source: free_cooling,
                solar: vec![field(declared)],
                source_reference: "r77 design".into(),
            };
            assert!(validate_regeneration(&input, "regeneration").is_empty());
            let result = calculate_regeneration(&input, context);
            let label = format!("free cooling {free_cooling}, declared {declared:?}");
            close(
                &format!("V.1 denominator {label}"),
                result.source_extraction_kwh,
                9359.037514209927,
            );
            close(&format!("V.4 {label}"), result.solar_kwh, solar);
            close(&format!("V.1 R {label}"), result.degree, degree);
            close(&format!("table V.1 {label}"), result.correction, correction);
        }
    }
}
