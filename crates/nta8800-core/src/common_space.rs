//! Allocation of common spaces (gemeenschappelijke ruimten) over the use
//! functions that rely on them.
//!
//! 6.2 (2025+C1 p. 142; 2023 p. 129): a common space goes pro rata to the
//! use functions it serves; a space serving one function belongs to that
//! function. 6.6.4 (2025+C1 p. 158; 2023 p. 145) applies the same rule twice:
//!
//! * for the use function in the building (Bbl limits, 5.3.1), by the
//!   initial areas of all functions served;
//! * for the use function within a calculation zone (6.5.3 values of the
//!   zone: setpoint, internal heat, ventilation capacity), only over the
//!   served functions that lie within that zone.
//!
//! Note 2 under 6.5.2 (2025+C1 p. 156) checks the 90 % rule after the
//! allocation. The kernel takes the initial (non-common) function areas per
//! zone and the common spaces with the functions they serve, and writes the
//! allocated areas into `functionAreas` of the zone, the zone's chapter 11
//! ventilation functions and `bblFunctions` (when not given).
//!
//! Hot water and lighting weight their table values per calculation zone by
//! "de gebruiksfuncties usi in de beschouwde rekenzone" (13.19, 2025+C1
//! p. 538) and "de gebruiksfuncties die in de rekenzone zijn gelegen"
//! (t_D and t_N p. 657, P_n;spec p. 661), which is the within-zone split of
//! 6.6.4. The same wording is in 2020+A1 (p. 515, 628–636), 2022 (p. 519,
//! 633–641), 2023 (p. 527, 643–651) and 2024. When those lists are empty
//! the kernel fills them from that split (`withinZones`, the zones'
//! `allocated`); entered lists take precedence.

use crate::bbl_requirements::{BblFunction, BblFunctionArea};
use crate::label_class::{LabelFunction, LabelFunctionArea};
use crate::monthly_demand::{UsageFunction, UsageFunctionArea};
use serde::{Deserialize, Serialize};

/// A common space lying within a calculation zone.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommonSpace {
    #[serde(default)]
    pub id: Option<String>,
    /// Usable floor area of the common space (NEN 2580 4.5.1), m².
    pub area_m2: f64,
    /// Use functions that rely on this space; empty: all functions of the
    /// building.
    #[serde(default)]
    pub served_functions: Vec<UsageFunction>,
    pub source_reference: String,
}

/// Initial areas and common spaces of one zone.
#[derive(Debug, Clone)]
pub struct ZoneCommonSpaces<'a> {
    pub zone_id: &'a str,
    /// Index of the zone entry in `ntaCalculation.zoneData`.
    pub data_index: usize,
    pub floor_area_m2: f64,
    /// The zone's own function areas; empty for a single-function zone.
    pub initial: &'a [UsageFunctionArea],
    /// Zone usage function for a single-function zone.
    pub usage_function: UsageFunction,
    pub common_spaces: &'a [CommonSpace],
}

/// One common space and how it was allocated.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonSpaceAllocation {
    pub zone_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub area_m2: f64,
    /// Share per function within the zone (6.6.4, last paragraph).
    pub within_zone: Vec<UsageFunctionArea>,
    /// Share per function for the building (6.6.4, first paragraph).
    pub building: Vec<UsageFunctionArea>,
}

/// Function areas of one zone after allocation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneFunctionAreas {
    pub zone_id: String,
    pub initial: Vec<UsageFunctionArea>,
    pub allocated: Vec<UsageFunctionArea>,
}

/// The traceable result of the allocation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonSpaceSplit {
    pub source: &'static str,
    pub spaces: Vec<CommonSpaceAllocation>,
    pub zones: Vec<ZoneFunctionAreas>,
    /// Use-function areas of the building (Bbl limits, label functions).
    pub building: Vec<UsageFunctionArea>,
    /// Use-function areas summed over the calculation zones, each zone with
    /// its within-zone split (hot water, 13.19).
    pub within_zones: Vec<UsageFunctionArea>,
    /// Input lists the kernel filled from the split, below `ntaCalculation`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub filled: Vec<String>,
}

/// A refusal with the path below `ntaCalculation`.
#[derive(Debug, Clone, PartialEq)]
pub struct CommonSpaceIssue {
    pub code: &'static str,
    pub path: String,
}

const SOURCE: &str = "6.2 and 6.6.4: a common space is allocated pro rata to the initial areas of the use functions it serves; within a calculation zone only over the served functions lying in that zone";

fn add(list: &mut Vec<UsageFunctionArea>, function: UsageFunction, area_m2: f64) {
    match list.iter_mut().find(|part| part.function == function) {
        Some(part) => part.area_m2 += area_m2,
        None => list.push(UsageFunctionArea { function, area_m2 }),
    }
}

/// Pro rata share of `area` over the `served` functions present in `base`.
fn share(
    area: f64,
    served: &[UsageFunction],
    base: &[UsageFunctionArea],
) -> Option<Vec<UsageFunctionArea>> {
    let parts: Vec<&UsageFunctionArea> = base
        .iter()
        .filter(|part| part.area_m2 > 0.0)
        .filter(|part| served.is_empty() || served.contains(&part.function))
        .collect();
    let total: f64 = parts.iter().map(|part| part.area_m2).sum();
    if parts.is_empty() || total <= 0.0 {
        return None;
    }
    Some(
        parts
            .iter()
            .map(|part| UsageFunctionArea {
                function: part.function,
                area_m2: area * part.area_m2 / total,
            })
            .collect(),
    )
}

/// Allocates the common spaces of all zones. `None` when no zone has one.
pub fn allocate(
    zones: &[ZoneCommonSpaces<'_>],
) -> Option<Result<CommonSpaceSplit, Vec<CommonSpaceIssue>>> {
    if zones.iter().all(|zone| zone.common_spaces.is_empty()) {
        return None;
    }
    let mut issues = Vec::new();
    // Initial (non-common) areas per zone.
    let mut initial_by_zone = Vec::with_capacity(zones.len());
    for zone in zones {
        let common: f64 = zone.common_spaces.iter().map(|space| space.area_m2).sum();
        let initial: Vec<UsageFunctionArea> = if zone.initial.is_empty() {
            vec![UsageFunctionArea {
                function: zone.usage_function,
                area_m2: zone.floor_area_m2 - common,
            }]
        } else {
            zone.initial.to_vec()
        };
        initial_by_zone.push(initial);
        for (index, space) in zone.common_spaces.iter().enumerate() {
            let path = format!("zoneData[{}].commonSpaces[{index}]", zone.data_index);
            if !(space.area_m2.is_finite() && space.area_m2 > 0.0) {
                issues.push(CommonSpaceIssue {
                    code: "common_space_invalid",
                    path: format!("{path}.areaM2"),
                });
            }
            if space.source_reference.trim().is_empty() {
                issues.push(CommonSpaceIssue {
                    code: "source_reference_required",
                    path: format!("{path}.sourceReference"),
                });
            }
            let mut seen = Vec::new();
            for function in &space.served_functions {
                if seen.contains(function) {
                    issues.push(CommonSpaceIssue {
                        code: "common_space_invalid",
                        path: format!("{path}.servedFunctions"),
                    });
                    break;
                }
                seen.push(*function);
            }
        }
    }
    let mut building_initial = Vec::new();
    for initial in &initial_by_zone {
        for part in initial {
            add(&mut building_initial, part.function, part.area_m2);
        }
    }
    let mut spaces = Vec::new();
    let mut building = building_initial.clone();
    let mut zone_results = Vec::new();
    for (zone, initial) in zones.iter().zip(&initial_by_zone) {
        // A zone without common spaces keeps its own validation of the
        // function areas (function_areas_invalid); only its positive parts
        // take part in the building split.
        if zone.common_spaces.is_empty() {
            continue;
        }
        if initial
            .iter()
            .any(|part| !(part.area_m2.is_finite() && part.area_m2 > 0.0))
        {
            issues.push(CommonSpaceIssue {
                code: "common_space_exceeds_zone",
                path: format!("zoneData[{}].commonSpaces", zone.data_index),
            });
            continue;
        }
        // 6.6.3: the zone area is its own (initial) function areas plus its
        // common spaces. Function areas that already hold the common space
        // would count it twice.
        if !zone.initial.is_empty() {
            let own: f64 = initial.iter().map(|part| part.area_m2).sum();
            let common: f64 = zone.common_spaces.iter().map(|space| space.area_m2).sum();
            if (own + common - zone.floor_area_m2).abs() > 0.01 * zone.floor_area_m2 {
                issues.push(CommonSpaceIssue {
                    code: "common_space_area_mismatch",
                    path: format!("zoneData[{}].commonSpaces", zone.data_index),
                });
                continue;
            }
        }
        let mut allocated = initial.clone();
        for (index, space) in zone.common_spaces.iter().enumerate() {
            let path = format!(
                "zoneData[{}].commonSpaces[{index}].servedFunctions",
                zone.data_index
            );
            let within = share(space.area_m2, &space.served_functions, initial);
            let whole = share(space.area_m2, &space.served_functions, &building_initial);
            match (within, whole) {
                (Some(within), Some(whole)) => {
                    for part in &within {
                        add(&mut allocated, part.function, part.area_m2);
                    }
                    for part in &whole {
                        add(&mut building, part.function, part.area_m2);
                    }
                    spaces.push(CommonSpaceAllocation {
                        zone_id: zone.zone_id.to_string(),
                        id: space.id.clone(),
                        area_m2: space.area_m2,
                        within_zone: within,
                        building: whole,
                    });
                }
                _ => issues.push(CommonSpaceIssue {
                    code: "common_space_function_outside_zone",
                    path,
                }),
            }
        }
        zone_results.push(ZoneFunctionAreas {
            zone_id: zone.zone_id.to_string(),
            initial: initial.clone(),
            allocated,
        });
    }
    if !issues.is_empty() {
        return Some(Err(issues));
    }
    let mut within_zones = Vec::new();
    for (zone, initial) in zones.iter().zip(&initial_by_zone) {
        let parts = zone_results
            .iter()
            .find(|result| result.zone_id == zone.zone_id)
            .map_or(initial.as_slice(), |result| result.allocated.as_slice());
        for part in parts.iter().filter(|part| part.area_m2 > 0.0) {
            add(&mut within_zones, part.function, part.area_m2);
        }
    }
    Some(Ok(CommonSpaceSplit {
        source: SOURCE,
        spaces,
        zones: zone_results,
        building,
        within_zones,
        filled: Vec::new(),
    }))
}

/// The use-function areas of every zone after the within-zone split: the
/// allocated areas of a zone with common spaces, otherwise its own function
/// areas (or its single function over the whole zone). `None` when a zone
/// has a non-positive part; that zone keeps its own validation.
pub fn zone_use_areas(
    zones: &[ZoneCommonSpaces<'_>],
    split: &CommonSpaceSplit,
) -> Option<Vec<(String, Vec<UsageFunctionArea>)>> {
    zones
        .iter()
        .map(|zone| {
            let parts = match split
                .zones
                .iter()
                .find(|result| result.zone_id == zone.zone_id)
            {
                Some(result) => result.allocated.clone(),
                None if zone.initial.is_empty() => vec![UsageFunctionArea {
                    function: zone.usage_function,
                    area_m2: zone.floor_area_m2,
                }],
                None => zone.initial.to_vec(),
            };
            if !parts
                .iter()
                .all(|part| part.area_m2.is_finite() && part.area_m2 > 0.0)
            {
                return None;
            }
            // The lists filled from these parts take A_g of the zone (14.4,
            // 13.19); own areas plus common spaces may differ from it by up
            // to 1 % (common_space_area_mismatch), so scale to the zone.
            let total: f64 = parts.iter().map(|part| part.area_m2).sum();
            let scale = if zone.floor_area_m2.is_finite() && zone.floor_area_m2 > 0.0 {
                zone.floor_area_m2 / total
            } else {
                1.0
            };
            Some((
                zone.zone_id.to_string(),
                parts
                    .iter()
                    .map(|part| UsageFunctionArea {
                        function: part.function,
                        area_m2: part.area_m2 * scale,
                    })
                    .collect(),
            ))
        })
        .collect()
}

/// Bbl function of a utility use function; `None` for the residential
/// function, whose Bbl category (1a–1e) the use function does not fix.
pub fn bbl_function(function: UsageFunction) -> Option<BblFunction> {
    Some(match function {
        UsageFunction::AssemblyChildCare => BblFunction::AssemblyChildCare,
        UsageFunction::OtherAssembly => BblFunction::OtherAssembly,
        UsageFunction::Cell => BblFunction::Cell,
        UsageFunction::HealthcareWithBeds => BblFunction::HealthcareWithBeds,
        UsageFunction::OtherHealthcare => BblFunction::OtherHealthcare,
        UsageFunction::Office => BblFunction::Office,
        UsageFunction::Lodging => BblFunction::LodgingInLodgingBuilding,
        UsageFunction::Education => BblFunction::Education,
        UsageFunction::Sport => BblFunction::Sport,
        UsageFunction::Retail => BblFunction::Retail,
        UsageFunction::Residential => return None,
    })
}

/// Label function (Omgevingsregeling bijlage X) of a utility use function;
/// `None` for the residential function.
pub fn label_function(function: UsageFunction) -> Option<LabelFunction> {
    Some(match function {
        UsageFunction::AssemblyChildCare => LabelFunction::AssemblyWithDayCare,
        UsageFunction::OtherAssembly => LabelFunction::AssemblyWithoutDayCare,
        UsageFunction::Cell => LabelFunction::Cell,
        UsageFunction::HealthcareWithBeds => LabelFunction::HealthcareWithBeds,
        UsageFunction::OtherHealthcare => LabelFunction::HealthcareWithoutBeds,
        UsageFunction::Office => LabelFunction::Office,
        UsageFunction::Lodging => LabelFunction::Lodging,
        UsageFunction::Education => LabelFunction::Education,
        UsageFunction::Sport => LabelFunction::Sport,
        UsageFunction::Retail => LabelFunction::Retail,
        UsageFunction::Residential => return None,
    })
}

/// The building split as label functions (the label is set per use
/// function of the building, like the Bbl limits); `None` when a residential
/// function takes part.
pub fn label_functions(split: &CommonSpaceSplit) -> Option<Vec<LabelFunctionArea>> {
    split
        .building
        .iter()
        .map(|part| {
            Some(LabelFunctionArea {
                function: label_function(part.function)?,
                area_m2: part.area_m2,
            })
        })
        .collect()
}

/// The building split as Bbl functions; `None` when a residential function
/// takes part.
pub fn bbl_functions(split: &CommonSpaceSplit) -> Option<Vec<BblFunctionArea>> {
    split
        .building
        .iter()
        .map(|part| {
            Some(BblFunctionArea {
                function: bbl_function(part.function)?,
                area_m2: part.area_m2,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(function: UsageFunction, area_m2: f64) -> UsageFunctionArea {
        UsageFunctionArea { function, area_m2 }
    }

    fn space(area_m2: f64, served: Vec<UsageFunction>) -> CommonSpace {
        CommonSpace {
            id: None,
            area_m2,
            served_functions: served,
            source_reference: "test".into(),
        }
    }

    /// Public case H (Uniec 3.2.9.2, NTA 8800:2023): 49,20 m² common space in
    /// RZ-1 serving all four functions; sport lies in RZ-2.
    #[test]
    fn case_h_split_within_zone_and_building() {
        let rz1 = [
            area(UsageFunction::Office, 14.70),
            area(UsageFunction::OtherAssembly, 26.30),
            area(UsageFunction::OtherHealthcare, 27.60),
        ];
        let spaces = [space(49.20, Vec::new())];
        let zones = [
            ZoneCommonSpaces {
                zone_id: "rz1",
                data_index: 0,
                floor_area_m2: 117.8,
                initial: &rz1,
                usage_function: UsageFunction::OtherHealthcare,
                common_spaces: &spaces,
            },
            ZoneCommonSpaces {
                zone_id: "rz2",
                data_index: 1,
                floor_area_m2: 626.0,
                initial: &[],
                usage_function: UsageFunction::Sport,
                common_spaces: &[],
            },
        ];
        let split = allocate(&zones).unwrap().unwrap();
        let rz1 = &split.zones[0].allocated;
        let get = |list: &[UsageFunctionArea], function| {
            list.iter()
                .find(|part| part.function == function)
                .map(|part| part.area_m2)
                .unwrap_or_default()
        };
        assert!((get(rz1, UsageFunction::Office) - 25.24).abs() < 0.01);
        assert!((get(rz1, UsageFunction::OtherAssembly) - 45.16).abs() < 0.01);
        assert!((get(rz1, UsageFunction::OtherHealthcare) - 47.40).abs() < 0.01);
        assert_eq!(get(rz1, UsageFunction::Sport), 0.0);
        let building = &split.building;
        assert!((get(building, UsageFunction::Office) - 15.74).abs() < 0.01);
        assert!((get(building, UsageFunction::OtherAssembly) - 28.16).abs() < 0.01);
        assert!((get(building, UsageFunction::OtherHealthcare) - 29.55).abs() < 0.01);
        assert!((get(building, UsageFunction::Sport) - 670.35).abs() < 0.01);
        let total: f64 = building.iter().map(|part| part.area_m2).sum();
        assert!((total - 743.80).abs() < 1e-9);
        assert_eq!(bbl_functions(&split).unwrap().len(), 4);
    }

    #[test]
    fn served_functions_restrict_the_split() {
        let initial = [
            area(UsageFunction::Office, 60.0),
            area(UsageFunction::Retail, 40.0),
        ];
        let spaces = [space(20.0, vec![UsageFunction::Office])];
        let zones = [ZoneCommonSpaces {
            zone_id: "z",
            data_index: 0,
            floor_area_m2: 120.0,
            initial: &initial,
            usage_function: UsageFunction::Office,
            common_spaces: &spaces,
        }];
        let split = allocate(&zones).unwrap().unwrap();
        assert_eq!(split.zones[0].allocated[0].area_m2, 80.0);
        assert_eq!(split.zones[0].allocated[1].area_m2, 40.0);
    }

    #[test]
    fn refusals() {
        let initial = [area(UsageFunction::Office, 60.0)];
        let spaces = [
            space(0.0, Vec::new()),
            space(10.0, vec![UsageFunction::Retail]),
        ];
        let zones = [ZoneCommonSpaces {
            zone_id: "z",
            data_index: 2,
            floor_area_m2: 70.0,
            initial: &initial,
            usage_function: UsageFunction::Office,
            common_spaces: &spaces,
        }];
        let issues = allocate(&zones).unwrap().unwrap_err();
        let codes: Vec<_> = issues.iter().map(|issue| issue.code).collect();
        assert!(codes.contains(&"common_space_invalid"), "{codes:?}");
        assert!(issues
            .iter()
            .any(|issue| issue.code == "common_space_function_outside_zone"
                && issue.path == "zoneData[2].commonSpaces[1].servedFunctions"));
        // A single-function zone whose common spaces fill the whole zone.
        let spaces = [space(80.0, Vec::new())];
        let zones = [ZoneCommonSpaces {
            zone_id: "z",
            data_index: 0,
            floor_area_m2: 70.0,
            initial: &[],
            usage_function: UsageFunction::Office,
            common_spaces: &spaces,
        }];
        let issues = allocate(&zones).unwrap().unwrap_err();
        assert_eq!(issues[0].code, "common_space_exceeds_zone");
        assert!(allocate(&[]).is_none());
    }

    /// Review 24: function areas that already hold the common space count it
    /// twice (6.6.3: the zone is its own areas plus its common spaces).
    #[test]
    fn function_areas_with_the_common_space_inside_are_refused() {
        let initial = [
            area(UsageFunction::Office, 70.0),
            area(UsageFunction::Retail, 50.0),
        ];
        let spaces = [space(20.0, Vec::new())];
        let zones = [ZoneCommonSpaces {
            zone_id: "z",
            data_index: 1,
            floor_area_m2: 120.0,
            initial: &initial,
            usage_function: UsageFunction::Office,
            common_spaces: &spaces,
        }];
        let issues = allocate(&zones).unwrap().unwrap_err();
        assert_eq!(
            issues,
            vec![CommonSpaceIssue {
                code: "common_space_area_mismatch",
                path: "zoneData[1].commonSpaces".into(),
            }]
        );
    }

    /// Review 24: a zone without common spaces is not refused for its own
    /// function areas here; it keeps function_areas_invalid of the zone.
    #[test]
    fn a_zone_without_common_spaces_keeps_its_own_validation() {
        let own = [area(UsageFunction::Office, 60.0)];
        let spaces = [space(10.0, Vec::new())];
        let blank = [
            area(UsageFunction::Retail, 0.0),
            area(UsageFunction::Sport, 40.0),
        ];
        let zones = [
            ZoneCommonSpaces {
                zone_id: "a",
                data_index: 0,
                floor_area_m2: 70.0,
                initial: &own,
                usage_function: UsageFunction::Office,
                common_spaces: &spaces,
            },
            ZoneCommonSpaces {
                zone_id: "b",
                data_index: 1,
                floor_area_m2: 40.0,
                initial: &blank,
                usage_function: UsageFunction::Sport,
                common_spaces: &[],
            },
        ];
        let split = allocate(&zones).unwrap().unwrap();
        assert_eq!(split.zones.len(), 1);
        let labels = label_functions(&split).unwrap();
        assert!(labels
            .iter()
            .any(|part| part.function == LabelFunction::Office));
    }

    #[test]
    fn label_functions_follow_the_building_split() {
        let rz1 = [
            area(UsageFunction::Office, 14.70),
            area(UsageFunction::OtherAssembly, 26.30),
            area(UsageFunction::OtherHealthcare, 27.60),
        ];
        let spaces = [space(49.20, Vec::new())];
        let zones = [
            ZoneCommonSpaces {
                zone_id: "rz1",
                data_index: 0,
                floor_area_m2: 117.8,
                initial: &rz1,
                usage_function: UsageFunction::OtherHealthcare,
                common_spaces: &spaces,
            },
            ZoneCommonSpaces {
                zone_id: "rz2",
                data_index: 1,
                floor_area_m2: 626.0,
                initial: &[],
                usage_function: UsageFunction::Sport,
                common_spaces: &[],
            },
        ];
        let split = allocate(&zones).unwrap().unwrap();
        let labels = label_functions(&split).unwrap();
        let bbl = bbl_functions(&split).unwrap();
        assert_eq!(labels.len(), bbl.len());
        for (label, bbl) in labels.iter().zip(&bbl) {
            assert_eq!(label.area_m2, bbl.area_m2);
        }
        let sport = labels
            .iter()
            .find(|part| part.function == LabelFunction::Sport)
            .unwrap();
        assert!((sport.area_m2 - 670.35).abs() < 0.01);
        assert_eq!(
            label_function(UsageFunction::OtherHealthcare),
            Some(LabelFunction::HealthcareWithoutBeds)
        );
        assert_eq!(label_function(UsageFunction::Residential), None);
    }
}
