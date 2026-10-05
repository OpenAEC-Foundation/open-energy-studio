//! Simplified cooling need and minimum cooling capacity in dwellings,
//! NTA 8800:2025+C1:2026 annex AA (pp. 1135–1147).
//!
//! Per habitable room (verblijfsruimte) and for the calculation zone:
//! internal load AA.1–AA.3, outdoor air AA.4 with table AA.1, opaque
//! transmission AA.5 with table AA.2, solar gain through windows AA.6 with
//! table AA.3 (hours 9–18) and window transmission AA.7; the governing
//! need AA.8/AA.9 and the capacity criteria AA.10–AA.13 with the fixed
//! 35 W/m² deduction. Used as capacity evidence for active cooling in
//! §5.7.1 (TOjuli).

use serde::{Deserialize, Serialize};

use crate::climate::Orientation;
use crate::monthly_demand::{
    occupants_per_dwelling, InternalGains, MonthlyDemandAssessment, MonthlyDemandInput, Window,
};
use crate::solar_shading::{obstruction_factor, Balance};

/// AA.1: internal heat per occupant, W.
pub const HEAT_PER_OCCUPANT_W: f64 = 180.0;
/// AA.11/AA.13: fixed deduction, W/m².
pub const FIXED_DEDUCTION_W_PER_M2: f64 = 35.0;
/// AA.6b: mean glass-to-window area ratio.
pub const GLASS_RATIO: f64 = 0.75;
/// AA.4/AA.7: indoor temperature, °C.
pub const INDOOR_C: f64 = 24.0;
/// ρ_a·c_a of AA.4, J/(m³·K).
const AIR_HEAT_CAPACITY: f64 = 1.205 * 1005.0;

/// Table AA.1: outdoor temperature at hours 9–21, °C.
const OUTDOOR_AT_HOUR_C: [f64; 13] = [
    24.7, 26.9, 28.2, 28.9, 29.7, 29.9, 29.8, 30.4, 30.6, 30.1, 29.5, 25.9, 23.4,
];

/// First hour of table AA.3; rows run 9 h–18 h.
const FIRST_HOUR: usize = 9;
const HOURS: usize = 10;

/// Table AA.3, β = 0° (no orientation), W/m².
const HORIZONTAL: [f64; HOURS] = [
    671.0, 832.0, 877.0, 914.0, 901.0, 823.0, 820.0, 602.0, 446.0, 308.0,
];
/// Table AA.3, β = 180°, W/m².
const DOWN: [f64; HOURS] = [
    134.0, 166.0, 175.0, 183.0, 180.0, 165.0, 164.0, 120.0, 89.0, 62.0,
];

/// Table AA.3 for β = 30°, 45°, 60°, 90°, 135°; columns S, SW, W, NW, N,
/// NE, E, SE; rows 9 h–18 h.
type TiltTable = [[f64; 8]; HOURS];
const TILT_30: TiltTable = [
    [729.0, 481.0, 307.0, 309.0, 486.0, 734.0, 908.0, 906.0],
    [948.0, 717.0, 505.0, 436.0, 551.0, 782.0, 993.0, 1062.0],
    [1026.0, 862.0, 656.0, 528.0, 553.0, 717.0, 923.0, 1051.0],
    [1078.0, 993.0, 808.0, 631.0, 566.0, 652.0, 837.0, 1013.0],
    [1057.0, 1061.0, 919.0, 713.0, 564.0, 560.0, 703.0, 909.0],
    [1004.0, 1097.0, 982.0, 727.0, 481.0, 388.0, 503.0, 758.0],
    [905.0, 1098.0, 1081.0, 864.0, 573.0, 380.0, 396.0, 614.0],
    [609.0, 829.0, 882.0, 738.0, 480.0, 260.0, 207.0, 351.0],
    [403.0, 644.0, 746.0, 650.0, 412.0, 172.0, 80.0, 165.0],
    [203.0, 474.0, 633.0, 588.0, 364.0, 93.0, 61.0, 61.0],
];
const TILT_45: TiltTable = [
    [694.0, 343.0, 97.0, 100.0, 350.0, 701.0, 947.0, 944.0],
    [921.0, 595.0, 295.0, 198.0, 360.0, 686.0, 986.0, 1083.0],
    [1008.0, 776.0, 485.0, 304.0, 339.0, 571.0, 863.0, 1044.0],
    [1063.0, 942.0, 680.0, 431.0, 339.0, 460.0, 721.0, 971.0],
    [1040.0, 1046.0, 844.0, 553.0, 343.0, 337.0, 539.0, 830.0],
    [1005.0, 1136.0, 974.0, 613.0, 266.0, 134.0, 296.0, 657.0],
    [867.0, 1140.0, 1116.0, 809.0, 398.0, 124.0, 148.0, 456.0],
    [559.0, 870.0, 946.0, 742.0, 377.0, 91.0, 91.0, 195.0],
    [347.0, 687.0, 832.0, 696.0, 360.0, 86.0, 86.0, 86.0],
    [133.0, 516.0, 742.0, 677.0, 361.0, 65.0, 65.0, 65.0],
];
const TILT_60: TiltTable = [
    [618.0, 189.0, 103.0, 103.0, 198.0, 627.0, 928.0, 925.0],
    [839.0, 439.0, 107.0, 107.0, 152.0, 551.0, 918.0, 1037.0],
    [928.0, 645.0, 287.0, 99.0, 110.0, 393.0, 751.0, 972.0],
    [983.0, 835.0, 514.0, 208.0, 101.0, 244.0, 565.0, 871.0],
    [959.0, 966.0, 719.0, 363.0, 106.0, 101.0, 346.0, 702.0],
    [945.0, 1106.0, 907.0, 466.0, 111.0, 111.0, 111.0, 519.0],
    [777.0, 1113.0, 1083.0, 706.0, 203.0, 111.0, 111.0, 274.0],
    [478.0, 859.0, 952.0, 702.0, 255.0, 100.0, 100.0, 100.0],
    [273.0, 689.0, 867.0, 701.0, 289.0, 92.0, 92.0, 92.0],
    [69.0, 528.0, 804.0, 725.0, 337.0, 69.0, 69.0, 69.0],
];
const TILT_90: TiltTable = [
    [368.0, 125.0, 125.0, 125.0, 125.0, 378.0, 726.0, 722.0],
    [535.0, 138.0, 138.0, 138.0, 138.0, 203.0, 627.0, 764.0],
    [611.0, 283.0, 138.0, 138.0, 138.0, 138.0, 405.0, 661.0],
    [653.0, 483.0, 142.0, 142.0, 142.0, 142.0, 171.0, 524.0],
    [633.0, 642.0, 357.0, 141.0, 141.0, 141.0, 141.0, 337.0],
    [663.0, 849.0, 620.0, 141.0, 141.0, 141.0, 141.0, 171.0],
    [472.0, 859.0, 825.0, 390.0, 140.0, 140.0, 140.0, 140.0],
    [244.0, 685.0, 791.0, 502.0, 116.0, 116.0, 116.0, 116.0],
    [100.0, 572.0, 776.0, 585.0, 109.0, 100.0, 100.0, 100.0],
    [74.0, 455.0, 774.0, 683.0, 235.0, 74.0, 74.0, 74.0],
];
const TILT_135: TiltTable = [
    [143.0; 8],
    [170.0; 8],
    [177.0; 8],
    [184.0; 8],
    [182.0; 8],
    [169.0; 8],
    [168.0; 8],
    [128.0, 153.0, 228.0, 128.0, 128.0, 128.0, 128.0, 128.0],
    [101.0, 167.0, 312.0, 177.0, 101.0, 101.0, 101.0, 101.0],
    [72.0, 160.0, 386.0, 321.0, 72.0, 72.0, 72.0, 72.0],
];

fn column(orientation: Orientation) -> usize {
    match orientation {
        Orientation::South => 0,
        Orientation::SouthWest => 1,
        Orientation::West => 2,
        Orientation::NorthWest => 3,
        Orientation::North => 4,
        Orientation::NorthEast => 5,
        Orientation::East => 6,
        Orientation::SouthEast => 7,
    }
}

fn level(tilt_index: usize, orientation: Orientation, hour_index: usize) -> f64 {
    let c = column(orientation);
    match tilt_index {
        0 => HORIZONTAL[hour_index],
        1 => TILT_30[hour_index][c],
        2 => TILT_45[hour_index][c],
        3 => TILT_60[hour_index][c],
        4 => TILT_90[hour_index][c],
        5 => TILT_135[hour_index][c],
        _ => DOWN[hour_index],
    }
}

const TILTS: [f64; 7] = [0.0, 30.0, 45.0, 60.0, 90.0, 135.0, 180.0];

/// Table AA.3 with linear interpolation between tilts; `hour` 9–18.
pub fn irradiance_at_hour(orientation: Orientation, tilt_deg: f64, hour: usize) -> f64 {
    let h = hour - FIRST_HOUR;
    let tilt = tilt_deg.clamp(0.0, 180.0);
    let upper = TILTS.iter().position(|t| *t >= tilt).unwrap_or(6);
    if upper == 0 || (TILTS[upper] - tilt).abs() < 1e-12 {
        return level(upper, orientation, h);
    }
    let lower = upper - 1;
    let fraction = (tilt - TILTS[lower]) / (TILTS[upper] - TILTS[lower]);
    level(lower, orientation, h)
        + fraction * (level(upper, orientation, h) - level(lower, orientation, h))
}

/// Table AA.1.
pub fn outdoor_temperature_at_hour(hour: usize) -> f64 {
    OUTDOOR_AT_HOUR_C[hour - FIRST_HOUR]
}

/// Table AA.2 `f_iso`, W/m²; post-insulation of more than half of A_in
/// moves one class up.
pub fn insulation_factor(construction_year: i32, post_insulated: bool) -> f64 {
    let classes = [17.0, 10.0, 3.2, 2.2];
    let index = match construction_year {
        y if y <= 1975 => 0,
        y if y <= 1992 => 1,
        y if y <= 2015 => 2,
        _ => 3,
    };
    let index = if post_insulated {
        (index + 1).min(3)
    } else {
        index
    };
    classes[index]
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnnexAaWindow {
    /// Id of a window of the zone's monthly-demand input.
    pub window_id: String,
    /// `U_w+shut` (8.22) when shading per 7.6.6.1.4 is present; otherwise
    /// `U_w` of the window is used.
    #[serde(default)]
    pub u_with_shutter_w_per_m2k: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnnexAaRoom {
    pub id: String,
    /// Full floor area of the habitable room, m².
    pub area_m2: f64,
    /// Living room, kitchen and/or dining room (double internal load).
    pub living: bool,
    /// Inner area of the opaque outer wall and roof of the room, m².
    pub opaque_inner_area_m2: f64,
    #[serde(default)]
    pub windows: Vec<AnnexAaWindow>,
    /// B_C;inst;zi,j of the emitter (or single-room generator), kW.
    pub installed_capacity_kw: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnnexAaInput {
    pub construction_year: i32,
    /// More than 50 % of A_in demonstrably post-insulated (table AA.2).
    #[serde(default)]
    pub post_insulated: bool,
    /// B_C;inst;zi of a generator serving the zone, kW; `None` when every
    /// room has its own generator (AA.3.2.3, last paragraph).
    #[serde(default)]
    pub generator_capacity_kw: Option<f64>,
    pub rooms: Vec<AnnexAaRoom>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnexAaIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnexAaRoomResult {
    pub id: String,
    pub peak_hour: usize,
    pub internal_w: f64,
    pub outdoor_air_w: f64,
    pub opaque_w: f64,
    pub solar_w: f64,
    pub glazing_w: f64,
    /// q_C;vr;zi,j (AA.9), W/m².
    pub need_w_per_m2: f64,
    /// B_C;req;TO;zi,j (AA.13), kW.
    pub required_kw: f64,
    pub installed_kw: f64,
    pub sufficient: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnexAaResult {
    pub peak_hour: usize,
    pub internal_w: f64,
    pub outdoor_air_w: f64,
    pub opaque_w: f64,
    pub solar_w: f64,
    pub glazing_w: f64,
    /// q_C;zi (AA.8), W/m².
    pub need_w_per_m2: f64,
    /// B_C;req;TO;zi (AA.11), kW.
    pub required_kw: f64,
    pub generator_kw: Option<f64>,
    pub rooms: Vec<AnnexAaRoomResult>,
    /// AA.3.2.2: generator and every room satisfy their criterion.
    pub sufficient: bool,
}

fn issue(code: &'static str, path: String) -> AnnexAaIssue {
    AnnexAaIssue { code, path }
}

/// AA.6b for one window and hour.
fn window_solar_w(window: &Window, hour: usize) -> f64 {
    let obstruction = obstruction_factor(
        &window.obstruction,
        window.orientation,
        window.tilt_deg,
        7,
        Balance::Cooling,
    )
    .unwrap_or(1.0);
    // Note 3: compliant shading counts at full effect, without the
    // table 7.7 time fraction.
    let shading = window
        .movable_shading
        .as_ref()
        .map_or(1.0, |item| item.reduction_factor_for(window.orientation));
    GLASS_RATIO
        * window.area_m2
        // AA.6b: g_gl;C;juli, the July value of a dynamic window (annex A),
        // with fixed louvres (7.41a); rotatable louvres count closed (note 3).
        * window.g_gl(6, 1.0)
        * obstruction
        * shading
        * irradiance_at_hour(window.orientation, window.tilt_deg, hour)
}

fn peak<'a>(windows: impl Iterator<Item = &'a Window> + Clone) -> (usize, f64) {
    (FIRST_HOUR..FIRST_HOUR + HOURS)
        .map(|hour| {
            let total: f64 = windows
                .clone()
                .map(|window| window_solar_w(window, hour))
                .sum();
            (hour, total)
        })
        .fold((FIRST_HOUR, f64::NEG_INFINITY), |best, item| {
            if item.1 > best.1 + 1e-12 {
                item
            } else {
                best
            }
        })
}

/// Annex AA for one zone. `demand` gives the July flows of chapter 11 when
/// the zone uses that route; otherwise the cooling conductances of the
/// explicit ventilation flows are used.
pub fn assess_annex_aa(
    aa: &AnnexAaInput,
    input: &MonthlyDemandInput,
    demand: &MonthlyDemandAssessment,
    path: &str,
) -> Result<AnnexAaResult, Vec<AnnexAaIssue>> {
    let mut issues = Vec::new();
    let InternalGains::Residential { dwelling_count, .. } = &input.internal_gains else {
        return Err(vec![issue(
            "annex_aa_residential_only",
            format!("{path}.method"),
        )]);
    };
    if aa.rooms.is_empty() {
        issues.push(issue("annex_aa_room_required", format!("{path}.rooms")));
    }
    if let Some(capacity) = aa.generator_capacity_kw {
        if !capacity.is_finite() || capacity < 0.0 {
            issues.push(issue(
                "annex_aa_capacity_invalid",
                format!("{path}.generatorCapacityKw"),
            ));
        }
    }
    let mut assigned = std::collections::HashSet::new();
    let mut rooms: Vec<(&AnnexAaRoom, Vec<(&Window, f64)>)> = Vec::new();
    for (index, room) in aa.rooms.iter().enumerate() {
        let room_path = format!("{path}.rooms[{index}]");
        if !(room.area_m2.is_finite() && room.area_m2 > 0.0) {
            issues.push(issue(
                "annex_aa_area_invalid",
                format!("{room_path}.areaM2"),
            ));
        }
        if !(room.opaque_inner_area_m2.is_finite() && room.opaque_inner_area_m2 >= 0.0) {
            issues.push(issue(
                "annex_aa_area_invalid",
                format!("{room_path}.opaqueInnerAreaM2"),
            ));
        }
        if !(room.installed_capacity_kw.is_finite() && room.installed_capacity_kw >= 0.0) {
            issues.push(issue(
                "annex_aa_capacity_invalid",
                format!("{room_path}.installedCapacityKw"),
            ));
        }
        let mut windows = Vec::new();
        for (w, item) in room.windows.iter().enumerate() {
            let window_path = format!("{room_path}.windows[{w}]");
            // The derived zone input names project windows `window:<id>`;
            // both spellings resolve.
            let derived_id = format!("window:{}", item.window_id);
            match input
                .windows
                .iter()
                .find(|window| window.id == item.window_id || window.id == derived_id)
            {
                None => issues.push(issue(
                    "annex_aa_window_unknown",
                    format!("{window_path}.windowId"),
                )),
                Some(window) => {
                    if !assigned.insert(window.id.as_str()) {
                        issues.push(issue(
                            "annex_aa_window_assigned_twice",
                            format!("{window_path}.windowId"),
                        ));
                    }
                    let u = item
                        .u_with_shutter_w_per_m2k
                        .unwrap_or(window.u_for_month(6));
                    if !(u.is_finite() && u > 0.0) {
                        issues.push(issue(
                            "annex_aa_u_invalid",
                            format!("{window_path}.uWithShutterWPerM2k"),
                        ));
                    }
                    windows.push((window, u));
                }
            }
        }
        rooms.push((room, windows));
    }
    if !issues.is_empty() {
        return Err(issues);
    }

    // AA.1–AA.3.
    let dwellings = input
        .internal_gains
        .zone_dwellings()
        .unwrap_or(f64::from(*dwelling_count));
    let internal = HEAT_PER_OCCUPANT_W
        * dwellings
        * occupants_per_dwelling(input.usable_floor_area_m2 / dwellings);
    let living: f64 = aa
        .rooms
        .iter()
        .filter(|r| r.living)
        .map(|r| r.area_m2)
        .sum();
    let other: f64 = aa
        .rooms
        .iter()
        .filter(|r| !r.living)
        .map(|r| r.area_m2)
        .sum();
    let q_int = internal / (2.0 * living + other);
    let total_area = living + other;

    // Step 1: peak hour of the zone.
    let all_windows = rooms
        .iter()
        .flat_map(|(_, windows)| windows.iter().map(|(w, _)| *w));
    let (zone_hour, _) = peak(all_windows);
    let zone_outdoor = outdoor_temperature_at_hour(zone_hour);

    // AA.4: July cooling-balance inflow.
    let factor = AIR_HEAT_CAPACITY / 3600.0;
    let conductance = match &demand.ventilation {
        Some(ventilation) => {
            let july = &ventilation.months[6].cooling;
            factor
                * (july.infiltration_m3_per_h
                    + july.natural_supply_m3_per_h
                    + july.mechanical_supply_m3_per_h)
        }
        None => input
            .ventilation_flows
            .iter()
            .filter_map(|flow| flow.months.iter().find(|row| row.month == 7))
            .map(|row| {
                row.cooling_conductance_w_per_k
                    .unwrap_or(row.conductance_w_per_k)
            })
            .sum(),
    };
    let outdoor_air = conductance * (zone_outdoor - INDOOR_C);
    let f_iso = insulation_factor(aa.construction_year, aa.post_insulated);

    let mut room_results = Vec::new();
    let mut solar_total = 0.0;
    let mut opaque_total = 0.0;
    let mut glazing_zone = 0.0;
    for (room, windows) in &rooms {
        let (hour, solar) = if windows.is_empty() {
            (zone_hour, 0.0)
        } else {
            peak(windows.iter().map(|(w, _)| *w))
        };
        let outdoor = outdoor_temperature_at_hour(hour);
        let ua: f64 = windows.iter().map(|(w, u)| w.area_m2 * u).sum();
        let glazing = ua * (outdoor - INDOOR_C);
        glazing_zone += ua * (zone_outdoor - INDOOR_C);
        let internal_room = if room.living { 2.0 } else { 1.0 } * q_int * room.area_m2;
        let air_room = outdoor_air * room.area_m2 / total_area;
        let opaque = f_iso * room.opaque_inner_area_m2;
        solar_total += solar;
        opaque_total += opaque;
        let need = (internal_room + air_room + opaque + solar + glazing) / room.area_m2;
        let required = ((need - FIXED_DEDUCTION_W_PER_M2) / 1000.0 * room.area_m2).max(0.0);
        room_results.push(AnnexAaRoomResult {
            id: room.id.clone(),
            peak_hour: hour,
            internal_w: internal_room,
            outdoor_air_w: air_room,
            opaque_w: opaque,
            solar_w: solar,
            glazing_w: glazing,
            need_w_per_m2: need,
            required_kw: required,
            installed_kw: room.installed_capacity_kw,
            sufficient: room.installed_capacity_kw + 1e-12 >= required,
        });
    }
    // AA.8: the internal load of all rooms sums to P_int.
    let need = (internal + outdoor_air + opaque_total + solar_total + glazing_zone) / total_area;
    let required = ((need - FIXED_DEDUCTION_W_PER_M2) / 1000.0 * total_area).max(0.0);
    let generator_ok = aa
        .generator_capacity_kw
        .map_or(true, |capacity| capacity + 1e-12 >= required);
    let sufficient = generator_ok && room_results.iter().all(|room| room.sufficient);
    Ok(AnnexAaResult {
        peak_hour: zone_hour,
        internal_w: internal,
        outdoor_air_w: outdoor_air,
        opaque_w: opaque_total,
        solar_w: solar_total,
        glazing_w: glazing_zone,
        need_w_per_m2: need,
        required_kw: required,
        generator_kw: aa.generator_capacity_kw,
        rooms: room_results,
        sufficient,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_lookups_follow_aa_1_to_aa_3() {
        assert_eq!(outdoor_temperature_at_hour(17), 30.6);
        assert_eq!(irradiance_at_hour(Orientation::South, 90.0, 12), 653.0);
        assert_eq!(irradiance_at_hour(Orientation::West, 0.0, 9), 671.0);
        assert_eq!(irradiance_at_hour(Orientation::West, 180.0, 18), 62.0);
        // Linear between 60° (928) and 90° (611) for south at 11 h.
        let value = irradiance_at_hour(Orientation::South, 75.0, 11);
        assert!((value - (928.0 + 0.5 * (611.0 - 928.0))).abs() < 1e-9);
        // Between horizontal (671) and 30° east (908) at 9 h.
        let value = irradiance_at_hour(Orientation::East, 15.0, 9);
        assert!((value - (671.0 + 0.5 * (908.0 - 671.0))).abs() < 1e-9);
        assert_eq!(insulation_factor(1975, false), 17.0);
        assert_eq!(insulation_factor(1990, true), 3.2);
        assert_eq!(insulation_factor(2020, true), 2.2);
    }

    fn zone() -> (MonthlyDemandInput, MonthlyDemandAssessment) {
        let input: MonthlyDemandInput = serde_json::from_value(serde_json::json!({
            "zoneId": "w", "usableFloorAreaM2": 100.0, "areaSourceReference": "s",
            "usageFunction": "residential", "dwellingType": "other",
            "setpoints": {"heatingC": 20.0, "coolingC": 24.0, "sourceReference": "s"},
            "transmission": {"method": "explicit", "conductanceWPerK": 80.0, "sourceReference": "s",
                "ground": null, "groundInventoryConfirmed": true},
            "ventilationFlows": [{"id": "v", "sourceReference": "s",
                "months": (1..=12).map(|m| serde_json::json!({"month": m, "conductanceWPerK": 40.0})).collect::<Vec<_>>()}],
            "thermalMass": {"floor": "heavy", "wall": "heavy", "ceiling": "open_or_none", "sourceReference": "s"},
            "internalGains": {"method": "residential", "dwellingCount": 1, "sourceReference": "s"},
            "windowInventoryComplete": true,
            "windows": [{"id": "south", "areaM2": 10.0, "orientation": "south", "tiltDeg": 90.0,
                "gPerpendicular": 0.6, "frameFraction": 0.25, "uValueWPerM2k": 1.2,
                "obstruction": {"method": "declared", "heating": vec![1.0; 12], "cooling": vec![1.0; 12], "sourceReference": "s"},
                "sourceReference": "s"}],
            "opaqueInventoryComplete": true, "opaqueElements": []
        }))
        .unwrap();
        let demand = crate::monthly_demand::assess_monthly_demand(&input);
        assert_eq!(
            demand.status, "calculated_unverified",
            "{:?}",
            demand.issues
        );
        (input, demand)
    }

    fn rooms(living_kw: f64) -> AnnexAaInput {
        AnnexAaInput {
            construction_year: 2020,
            post_insulated: false,
            generator_capacity_kw: Some(1.5),
            rooms: vec![
                AnnexAaRoom {
                    id: "living".into(),
                    area_m2: 40.0,
                    living: true,
                    opaque_inner_area_m2: 15.0,
                    windows: vec![AnnexAaWindow {
                        window_id: "south".into(),
                        u_with_shutter_w_per_m2k: None,
                    }],
                    installed_capacity_kw: living_kw,
                },
                AnnexAaRoom {
                    id: "bedroom".into(),
                    area_m2: 20.0,
                    living: false,
                    opaque_inner_area_m2: 10.0,
                    windows: Vec::new(),
                    installed_capacity_kw: 0.5,
                },
            ],
        }
    }

    #[test]
    fn dynamic_window_uses_the_july_g() {
        let (mut input, demand) = zone();
        input.windows[0].dynamic = Some(crate::annex_a::DynamicTransparent::SingleState {
            state: crate::annex_a::DynamicState {
                id: "tinted".into(),
                g_perpendicular: Some(0.3),
                u_value_w_per_m2k: Some(1.0),
                tau_solar: None,
                tau_visual: None,
            },
            source_reference: "product sheet".into(),
            correction: None,
        });
        let result = assess_annex_aa(&rooms(2.0), &input, &demand, "aa").unwrap();
        // AA.6b with g_gl;C;juli = 0,3 instead of the nominal 0,6.
        assert!((result.solar_w - 0.75 * 10.0 * 0.9 * 0.3 * 663.0).abs() < 1e-9);
    }

    #[test]
    fn hand_example_follows_aa_1_to_aa_13() {
        let (input, demand) = zone();
        let result = assess_annex_aa(&rooms(2.0), &input, &demand, "aa").unwrap();
        // N_p = 2,28 at 100 m²; P_int = 180·2,28 = 410,4 W; q = 410,4/100.
        assert!((result.internal_w - 410.4).abs() < 1e-9);
        // South 90° peaks at 14 h (663 W/m²): 0,75·10·0,9·0,6·663.
        assert_eq!(result.peak_hour, 14);
        assert!((result.solar_w - 4.05 * 663.0).abs() < 1e-9);
        // P_V = 40 W/K · (29,9 − 24).
        assert!((result.outdoor_air_w - 40.0 * 5.9).abs() < 1e-9);
        assert!((result.opaque_w - 2.2 * 25.0).abs() < 1e-9);
        let zone_need = (410.4 + 236.0 + 55.0 + 4.05 * 663.0 + 12.0 * 5.9) / 60.0;
        assert!((result.need_w_per_m2 - zone_need).abs() < 1e-9);
        assert!((result.required_kw - (zone_need - 35.0) / 1000.0 * 60.0).abs() < 1e-12);
        let living = &result.rooms[0];
        let living_need =
            (2.0 * 4.104 * 40.0 + 236.0 * 40.0 / 60.0 + 33.0 + 4.05 * 663.0 + 70.8) / 40.0;
        assert!((living.need_w_per_m2 - living_need).abs() < 1e-9);
        assert!((living.required_kw - (living_need - 35.0) / 25.0).abs() < 1e-12);
        // The bedroom stays below 35 W/m²: no capacity requirement.
        assert_eq!(result.rooms[1].required_kw, 0.0);
        assert!(result.sufficient);
        let short = assess_annex_aa(&rooms(1.8), &input, &demand, "aa").unwrap();
        assert!(!short.rooms[0].sufficient && !short.sufficient);
    }

    #[test]
    fn invalid_assignments_are_reported() {
        let (input, demand) = zone();
        let mut aa = rooms(2.0);
        aa.rooms[1].windows.push(AnnexAaWindow {
            window_id: "south".into(),
            u_with_shutter_w_per_m2k: None,
        });
        aa.rooms[0].windows.push(AnnexAaWindow {
            window_id: "missing".into(),
            u_with_shutter_w_per_m2k: None,
        });
        let codes: Vec<_> = assess_annex_aa(&aa, &input, &demand, "aa")
            .unwrap_err()
            .into_iter()
            .map(|item| item.code)
            .collect();
        assert!(codes.contains(&"annex_aa_window_unknown"));
        assert!(codes.contains(&"annex_aa_window_assigned_twice"));
    }
}
