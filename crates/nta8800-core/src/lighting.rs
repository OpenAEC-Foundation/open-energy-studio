//! Lighting, NTA 8800:2025+C1:2026 chapter 14 (pages 655–675), with the
//! significant-figure rounding of annex X (p. 1100).
//!
//! Dwellings: `W_L;spec = 0` for the indicators, `W_P = 0` (14.1–14.3).
//! Utility zones: per lighting zone `W_t,j = W_L,j + W_P,j` (14.6) with
//! `W_L = P_n·F_C·(t_D·F_o;D·F_D + t_N·F_o;N)/1000` (14.7), table 14.1 hours,
//! installed power 14.8/14.9 with table 14.2 (rounded up per annex X) or the
//! forfait 14.13 with table 14.3, parasitic energy 14.10–14.12 or 14.14,
//! `F_C = 1` (14.15), occupancy 14.16–14.23 with tables 14.4/14.5, and the
//! daylight factor 14.24–14.44 with tables 14.6–14.9. The norm gives no
//! monthly split; months follow `t_mi / t_an` as in 7.28. The internal gain of
//! 7.28 (`f_L·W_t·1000/t_an`) is reported for the chapter 7 input.

use crate::climate::MONTH_HOURS;
use crate::label_class::LabelFunction;
use serde::{Deserialize, Serialize};

/// 14.10: hours per year.
pub const YEAR_HOURS: f64 = 8760.0;
/// 14.28/14.32: height of the visual task, m.
pub const TASK_HEIGHT_M: f64 = 0.75;
/// 14.14: forfait parasitic energy, kWh/m² per year (emergency + standby).
pub const PARASITIC_FORFAIT_KWH_PER_M2: f64 = 1.0 + 1.5;

/// Table X.1 significant figures.
const SERIES: [f64; 33] = [
    10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 22.0, 24.0, 26.0, 28.0, 30.0,
    32.0, 34.0, 36.0, 38.0, 40.0, 44.0, 48.0, 52.0, 56.0, 60.0, 65.0, 70.0, 75.0, 80.0, 85.0, 90.0,
    95.0,
];

/// Annex X: round up to two significant figures, taking the next higher
/// value of table X.1 for the first two digits (3 400 becomes 3 600).
pub fn round_up_annex_x(value: f64) -> f64 {
    if value <= 0.0 || !value.is_finite() {
        return value.max(0.0);
    }
    let exponent = value.log10().floor() - 1.0;
    let mut scale = 10f64.powf(exponent);
    let mut digits = (value / scale + 1e-9).floor();
    if digits >= 100.0 {
        scale *= 10.0;
        digits = (value / scale + 1e-9).floor();
    }
    match SERIES.iter().find(|item| **item > digits + 1e-9) {
        Some(next) => next * scale,
        None => 10.0 * scale * 10.0,
    }
}

/// Table 14.1 burning hours (t_D, t_N).
pub fn burning_hours(function: LabelFunction) -> (f64, f64) {
    match function {
        LabelFunction::Residential => (0.0, 0.0),
        LabelFunction::AssemblyWithDayCare
        | LabelFunction::AssemblyWithoutDayCare
        | LabelFunction::HealthcareWithoutBeds
        | LabelFunction::Office => (2200.0, 300.0),
        LabelFunction::Cell | LabelFunction::HealthcareWithBeds | LabelFunction::Lodging => {
            (4000.0, 1000.0)
        }
        LabelFunction::Education => (1600.0, 300.0),
        LabelFunction::Sport => (2200.0, 800.0),
        LabelFunction::Retail => (2700.0, 400.0),
    }
}

/// Table 14.4 absence factors (F_A;D, F_A;N).
pub fn absence_factors(function: LabelFunction) -> (f64, f64) {
    match function {
        LabelFunction::HealthcareWithBeds | LabelFunction::Retail => (0.0, 0.5),
        _ => (0.2, 0.5),
    }
}

/// Table 14.3 `P_n;spec` (W/m²); LED from 2017 only where the table has it.
pub fn specific_power(function: LabelFunction, led_from_2017: bool) -> f64 {
    match function {
        LabelFunction::Retail => 30.0,
        LabelFunction::Cell | LabelFunction::HealthcareWithBeds | LabelFunction::Lodging => {
            // Footnote a: no LED value, use "other or unknown".
            17.0
        }
        LabelFunction::Residential => 0.0,
        _ => {
            if led_from_2017 {
                12.0
            } else {
                16.0
            }
        }
    }
}

/// Table 14.2 ballast share.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LampTechnology {
    NoSeparateBallast,
    Led,
    FluorescentT5,
    FluorescentT8Electronic,
    FluorescentT8Conventional,
    FluorescentT12,
    CompactFluorescentNotIntegrated,
    UnknownOrOther,
}

impl LampTechnology {
    pub fn ballast_percent(self) -> f64 {
        match self {
            Self::NoSeparateBallast | Self::FluorescentT8Electronic => 0.0,
            Self::Led => 8.0,
            Self::FluorescentT5 => 10.0,
            Self::CompactFluorescentNotIntegrated => 15.0,
            Self::FluorescentT8Conventional | Self::FluorescentT12 | Self::UnknownOrOther => 20.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum LuminairePower {
    /// Measured or declared system power incl. ballast, W per luminaire.
    System {
        #[serde(rename = "powerW")]
        power_w: f64,
    },
    /// 14.9 from lamp power and table 14.2.
    Lamps {
        #[serde(rename = "lampPowerW")]
        lamp_power_w: f64,
        #[serde(rename = "lampCount")]
        lamp_count: u32,
        technology: LampTechnology,
    },
}

impl LuminairePower {
    fn watts(&self) -> f64 {
        match self {
            Self::System { power_w } => *power_w,
            Self::Lamps {
                lamp_power_w,
                lamp_count,
                technology,
            } => {
                (1.0 + technology.ballast_percent() / 100.0) * lamp_power_w * f64::from(*lamp_count)
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LuminaireGroup {
    pub count: u32,
    pub power: LuminairePower,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum InstalledPower {
    /// 14.13 with table 14.3 (also forces `F_D = 1`, `f_L = 0,3`).
    Forfait {
        #[serde(rename = "ledFrom2017")]
        led_from_2017: bool,
    },
    /// 14.8/14.9.
    Installed {
        luminaires: Vec<LuminaireGroup>,
        /// `f_dyn` for dynamic lighting; omitted means 1.
        #[serde(default, rename = "dynamicFactor")]
        dynamic_factor: Option<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum ParasiticPower {
    /// 14.14.
    Forfait,
    /// 14.10–14.12: Σ P_ei and Σ P_ci in W (rounded up per annex X).
    Installed {
        #[serde(rename = "emergencyChargingW")]
        emergency_charging_w: f64,
        #[serde(rename = "controlStandbyW")]
        control_standby_w: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

/// Table 14.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SwitchControl {
    ManualOrUnknown,
    ManualWithSweep,
    AutoOnDimmed,
    AutoOnAutoOff,
    ManualOnDimmed,
    ManualOnAutoOff,
}

impl SwitchControl {
    pub fn factor(self) -> f64 {
        match self {
            Self::ManualOrUnknown => 1.0,
            Self::ManualWithSweep | Self::AutoOnDimmed => 0.95,
            Self::AutoOnAutoOff | Self::ManualOnDimmed => 0.90,
            Self::ManualOnAutoOff => 0.80,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Occupancy {
    pub control: SwitchControl,
    /// §14.5.1: central on-control (timer or switch for a whole floor etc.).
    pub central_on_control: bool,
    /// §14.5.1: office area above 30 m² on one switched group (not a
    /// meeting room): `F_o;D = 1`.
    #[serde(default)]
    pub large_office_group: bool,
}

/// 14.18–14.23.
pub fn occupancy_factor(absence: f64, control: f64) -> f64 {
    if absence < 0.2 {
        1.0 - (1.0 - control) * absence / 0.2
    } else if absence < 0.9 {
        control + 0.2 - absence
    } else {
        (7.0 - 10.0 * control) * (absence - 1.0)
    }
}

/// Table 14.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DaylightControl {
    NoneOrManual,
    AutomaticSwitchingOrUnknown,
    AutomaticDimming,
}

impl DaylightControl {
    pub fn factor(self) -> f64 {
        match self {
            Self::NoneOrManual => 0.0,
            Self::AutomaticSwitchingOrUnknown => 0.63,
            Self::AutomaticDimming => 0.73,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DaylightSector {
    /// 14.27–14.30 and 14.37–14.40.
    VerticalWindows {
        /// `b_D;i` per figure 14.1, m.
        #[serde(rename = "sectorWidthM")]
        sector_width_m: f64,
        /// `h_Li;i`, lintel height above the floor, m.
        #[serde(rename = "lintelHeightM")]
        lintel_height_m: f64,
        /// `b_i`, actual depth of the space behind the facade, m.
        #[serde(rename = "actualDepthM")]
        actual_depth_m: f64,
        /// 14.30: use the actual depth when below 1,25·a_D;max.
        #[serde(default, rename = "useActualDepthAlternative")]
        use_actual_depth_alternative: bool,
        /// `A_Ca;i`, facade opening above 0,75 m, m².
        #[serde(rename = "openingAreaM2")]
        opening_area_m2: f64,
        /// `I_Sh = 0,2` when heavily shaded per NEN-EN 15193-1, else 0,7.
        #[serde(rename = "heavilyShaded")]
        heavily_shaded: bool,
        control: DaylightControl,
    },
    /// 14.31–14.35 and 14.41/14.42 (unshaded rooflights).
    Rooflights {
        /// `a_D;R` and the rooflight width, m.
        #[serde(rename = "rooflightDepthM")]
        rooflight_depth_m: f64,
        #[serde(rename = "rooflightWidthM")]
        rooflight_width_m: f64,
        /// `h_R;i`, clear height, m.
        #[serde(rename = "clearHeightM")]
        clear_height_m: f64,
        /// Distances from the rooflight projection to the walls on the four
        /// sides (depth side 1, depth side 2, width side 1, width side 2), m.
        #[serde(rename = "wallDistancesM")]
        wall_distances_m: [f64; 4],
        #[serde(rename = "openingAreaM2")]
        opening_area_m2: f64,
        #[serde(rename = "roomLengthM")]
        room_length_m: f64,
        #[serde(rename = "roomWidthM")]
        room_width_m: f64,
        /// `h_m;i`, luminaire height above the work plane, m.
        #[serde(rename = "luminaireHeightM")]
        luminaire_height_m: f64,
        control: DaylightControl,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Daylight {
    /// `f_dayl = 0`.
    None,
    /// 14.25–14.42.
    Sectors {
        sectors: Vec<DaylightSector>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// 14.43/14.44 (building-wide glazing ratio).
    Forfait {
        #[serde(rename = "daylightControl")]
        daylight_control: bool,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LightingZone {
    pub id: String,
    /// `A_g;vz`, m².
    pub area_m2: f64,
    pub power: InstalledPower,
    pub parasitic: ParasiticPower,
    pub occupancy: Occupancy,
    pub daylight: Daylight,
    /// 7.28: at least 70 % of the power is in extracted luminaires.
    #[serde(default)]
    pub extracted_luminaires: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UseArea {
    pub function: LabelFunction,
    pub area_m2: f64,
}

/// Lighting of one calculation zone.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ZoneLighting {
    pub zone_id: String,
    /// Use functions in the zone (weights for tables 14.1, 14.3, 14.4).
    pub functions: Vec<UseArea>,
    pub lighting_zones: Vec<LightingZone>,
    pub source_reference: String,
}

/// Building data for the forfait daylight method (14.44).
#[derive(Debug, Clone, Copy)]
pub struct LightingContext {
    pub total_usable_floor_area_m2: f64,
    pub total_window_area_m2: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LightingIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightingZoneResult {
    pub id: String,
    pub installed_power_w: f64,
    pub occupancy_day: f64,
    pub occupancy_night: f64,
    pub daylight_factor: f64,
    pub lighting_kwh: f64,
    pub parasitic_kwh: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneLightingResult {
    pub zone_id: String,
    pub annual_kwh: f64,
    pub monthly_kwh: [f64; 12],
    /// 7.28: `f_L·W_t·1000/t_an`, W.
    pub internal_gain_w: f64,
    pub lighting_zones: Vec<LightingZoneResult>,
}

fn weighted<F: Fn(LabelFunction) -> f64>(functions: &[UseArea], value: F) -> f64 {
    let area: f64 = functions.iter().map(|item| item.area_m2).sum();
    if area <= 0.0 {
        return 0.0;
    }
    functions
        .iter()
        .map(|item| value(item.function) * item.area_m2)
        .sum::<f64>()
        / area
}

fn positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

pub fn validate_lighting(zone: &ZoneLighting, zone_area_m2: f64, path: &str) -> Vec<LightingIssue> {
    let mut issues = Vec::new();
    let mut push = |code, field: String| {
        issues.push(LightingIssue {
            code,
            path: format!("{path}.{field}"),
        })
    };
    let tolerance = 1e-6 * zone_area_m2.max(1.0);
    let function_area: f64 = zone.functions.iter().map(|item| item.area_m2).sum();
    if zone.functions.is_empty()
        || zone
            .functions
            .iter()
            .any(|item| !positive(item.area_m2) || item.function == LabelFunction::Residential)
        || (function_area - zone_area_m2).abs() > tolerance
    {
        push("lighting_function_areas_invalid", "functions".into());
    }
    // 14.4: the lighting zones cover the calculation zone.
    let lit: f64 = zone.lighting_zones.iter().map(|item| item.area_m2).sum();
    if zone.lighting_zones.is_empty() || (lit - zone_area_m2).abs() > tolerance {
        push("lighting_zone_areas_invalid", "lightingZones".into());
    }
    let forfait_power = zone
        .lighting_zones
        .iter()
        .filter(|item| matches!(item.power, InstalledPower::Forfait { .. }))
        .count();
    if forfait_power != 0 && forfait_power != zone.lighting_zones.len() {
        // §14.3.4: the forfait applies to all lighting zones of the zone.
        push("lighting_forfait_mixed", "lightingZones".into());
    }
    let forfait_parasitic = zone
        .lighting_zones
        .iter()
        .filter(|item| matches!(item.parasitic, ParasiticPower::Forfait))
        .count();
    if forfait_parasitic != 0 && forfait_parasitic != zone.lighting_zones.len() {
        push("lighting_forfait_mixed", "lightingZones".into());
    }
    let mut ids = std::collections::HashSet::new();
    for (index, item) in zone.lighting_zones.iter().enumerate() {
        let base = format!("lightingZones[{index}]");
        if item.id.trim().is_empty() || !ids.insert(item.id.as_str()) {
            push("id_invalid", format!("{base}.id"));
        }
        if !positive(item.area_m2) {
            push("lighting_zone_area_invalid", format!("{base}.areaM2"));
        }
        if let InstalledPower::Installed {
            luminaires,
            dynamic_factor,
            source_reference,
        } = &item.power
        {
            if luminaires.is_empty()
                || luminaires.iter().any(|group| {
                    group.count == 0
                        || match &group.power {
                            LuminairePower::System { power_w } => !positive(*power_w),
                            LuminairePower::Lamps {
                                lamp_power_w,
                                lamp_count,
                                ..
                            } => !positive(*lamp_power_w) || *lamp_count == 0,
                        }
                })
            {
                push(
                    "lighting_luminaires_invalid",
                    format!("{base}.power.luminaires"),
                );
            }
            if dynamic_factor.is_some_and(|value| !positive(value) || value > 1.0) {
                push(
                    "lighting_dynamic_factor_invalid",
                    format!("{base}.power.dynamicFactor"),
                );
            }
            if source_reference.trim().is_empty() {
                push(
                    "source_reference_required",
                    format!("{base}.power.sourceReference"),
                );
            }
        }
        if let ParasiticPower::Installed {
            emergency_charging_w,
            control_standby_w,
            source_reference,
        } = &item.parasitic
        {
            if !emergency_charging_w.is_finite()
                || *emergency_charging_w < 0.0
                || !control_standby_w.is_finite()
                || *control_standby_w < 0.0
            {
                push("lighting_parasitic_invalid", format!("{base}.parasitic"));
            }
            if source_reference.trim().is_empty() {
                push(
                    "source_reference_required",
                    format!("{base}.parasitic.sourceReference"),
                );
            }
        }
        if let Daylight::Sectors {
            sectors,
            source_reference,
        } = &item.daylight
        {
            if source_reference.trim().is_empty() {
                push(
                    "source_reference_required",
                    format!("{base}.daylight.sourceReference"),
                );
            }
            let mut sector_area = 0.0;
            for (sector_index, sector) in sectors.iter().enumerate() {
                let sector_path = format!("{base}.daylight.sectors[{sector_index}]");
                match sector_geometry(sector) {
                    Some((area, _)) => sector_area += area,
                    None => push("lighting_daylight_sector_invalid", sector_path),
                }
            }
            if sector_area > item.area_m2 * (1.0 + 1e-9) {
                push(
                    "lighting_daylight_area_exceeds_zone",
                    format!("{base}.daylight.sectors"),
                );
            }
        }
    }
    if zone.source_reference.trim().is_empty() {
        push("source_reference_required", "sourceReference".into());
    }
    issues
}

/// Table 14.7 with linear interpolation (clamped at the ends).
pub fn vertical_daylight_supply(daylight_factor_percent: f64) -> f64 {
    const POINTS: [(f64, f64); 10] = [
        (0.13, 0.12),
        (0.5, 0.36),
        (1.0, 0.50),
        (1.5, 0.64),
        (2.0, 0.66),
        (3.0, 0.75),
        (5.0, 0.81),
        (8.0, 0.88),
        (12.0, 0.91),
        (18.0, 0.91),
    ];
    if daylight_factor_percent <= POINTS[0].0 {
        return POINTS[0].1;
    }
    for pair in POINTS.windows(2) {
        let (x0, y0) = pair[0];
        let (x1, y1) = pair[1];
        if daylight_factor_percent <= x1 {
            return y0 + (daylight_factor_percent - x0) / (x1 - x0) * (y1 - y0);
        }
    }
    POINTS[9].1
}

/// Table 14.9 with linear interpolation (clamped at the ends).
pub fn rooflight_utilance(room_index: f64) -> f64 {
    const POINTS: [(f64, f64); 10] = [
        (0.6, 0.4),
        (0.8, 0.54),
        (1.0, 0.6),
        (1.25, 0.69),
        (1.5, 0.75),
        (2.0, 0.83),
        (2.5, 0.88),
        (3.0, 0.92),
        (4.0, 0.97),
        (5.0, 1.0),
    ];
    if room_index <= POINTS[0].0 {
        return POINTS[0].1;
    }
    for pair in POINTS.windows(2) {
        let (x0, y0) = pair[0];
        let (x1, y1) = pair[1];
        if room_index <= x1 {
            return y0 + (room_index - x0) / (x1 - x0) * (y1 - y0);
        }
    }
    1.0
}

/// Table 14.8.
pub fn rooflight_supply(daylight_factor_percent: f64) -> f64 {
    if daylight_factor_percent < 2.0 {
        0.0
    } else if daylight_factor_percent < 4.0 {
        0.68
    } else if daylight_factor_percent < 7.0 {
        0.85
    } else {
        0.92
    }
}

/// Sector area `A_D;i` and `F_D;dayl;i`; `None` for invalid geometry.
fn sector_geometry(sector: &DaylightSector) -> Option<(f64, f64)> {
    match sector {
        DaylightSector::VerticalWindows {
            sector_width_m,
            lintel_height_m,
            actual_depth_m,
            use_actual_depth_alternative,
            opening_area_m2,
            heavily_shaded,
            control,
        } => {
            let effective = lintel_height_m - TASK_HEIGHT_M;
            if !positive(*sector_width_m)
                || !positive(effective)
                || !positive(*actual_depth_m)
                || !opening_area_m2.is_finite()
                || *opening_area_m2 < 0.0
            {
                return None;
            }
            // 14.28–14.30
            let maximum = 2.5 * effective;
            let depth = if *use_actual_depth_alternative && *actual_depth_m < 1.25 * maximum {
                *actual_depth_m
            } else {
                actual_depth_m.min(maximum)
            };
            let area = depth * sector_width_m;
            // 14.38–14.40
            let transmission = opening_area_m2 / area;
            let room_depth = depth / effective;
            let shading = if *heavily_shaded { 0.2 } else { 0.7 };
            let daylight = 0.34 * (4.13 + 20.0 * transmission - 1.36 * room_depth) * shading;
            // 14.37 and 14.36
            let supply = 0.65 * vertical_daylight_supply(daylight) + 0.25;
            Some((area, 1.0 - control.factor() * supply))
        }
        DaylightSector::Rooflights {
            rooflight_depth_m,
            rooflight_width_m,
            clear_height_m,
            wall_distances_m,
            opening_area_m2,
            room_length_m,
            room_width_m,
            luminaire_height_m,
            control,
        } => {
            let extra = clear_height_m - TASK_HEIGHT_M;
            if !positive(*rooflight_depth_m)
                || !positive(*rooflight_width_m)
                || !positive(extra)
                || wall_distances_m
                    .iter()
                    .any(|value| !value.is_finite() || *value < 0.0)
                || !positive(*opening_area_m2)
                || !positive(*room_length_m)
                || !positive(*room_width_m)
                || !positive(*luminaire_height_m)
            {
                return None;
            }
            // 14.31–14.35 (and the same for the width).
            let depth =
                rooflight_depth_m + extra.min(wall_distances_m[0]) + extra.min(wall_distances_m[1]);
            let width =
                rooflight_width_m + extra.min(wall_distances_m[2]) + extra.min(wall_distances_m[3]);
            let area = depth * width;
            // 14.41/14.42 with τ_D65 = 0,6. The printed 0,54 yields a fraction;
            // table 14.8 is in %, so the value is taken ×100 (see the
            // verification file).
            let room_index = room_length_m * room_width_m
                / (luminaire_height_m * (room_length_m + room_width_m));
            let daylight =
                0.54 * 0.6 * opening_area_m2 / area * rooflight_utilance(room_index) * 100.0;
            Some((area, 1.0 - control.factor() * rooflight_supply(daylight)))
        }
    }
}

/// Annual lighting per calculation zone; call after [`validate_lighting`].
pub fn assess_zone_lighting(zone: &ZoneLighting, context: LightingContext) -> ZoneLightingResult {
    let t_day = weighted(&zone.functions, |function| burning_hours(function).0);
    let t_night = weighted(&zone.functions, |function| burning_hours(function).1);
    let absence_day = weighted(&zone.functions, |function| absence_factors(function).0);
    let absence_night = weighted(&zone.functions, |function| absence_factors(function).1);
    let mut lighting_zones = Vec::with_capacity(zone.lighting_zones.len());
    let mut total = 0.0;
    let mut gain = 0.0;
    for item in &zone.lighting_zones {
        let (power, forfait) = match &item.power {
            InstalledPower::Forfait { led_from_2017 } => (
                weighted(&zone.functions, |function| {
                    specific_power(function, *led_from_2017)
                }) * item.area_m2,
                true,
            ),
            InstalledPower::Installed {
                luminaires,
                dynamic_factor,
                ..
            } => {
                let sum: f64 = luminaires
                    .iter()
                    .map(|group| f64::from(group.count) * group.power.watts())
                    .sum();
                (round_up_annex_x(sum * dynamic_factor.unwrap_or(1.0)), false)
            }
        };
        let control = item.occupancy.control.factor();
        let occupancy_day =
            if item.occupancy.central_on_control || item.occupancy.large_office_group {
                1.0
            } else {
                occupancy_factor(absence_day, control)
            };
        let occupancy_night = if item.occupancy.central_on_control {
            1.0
        } else {
            occupancy_factor(absence_night, control)
        };
        // 14.24–14.26 and 14.43/14.44.
        let daylight_factor = if forfait {
            1.0
        } else {
            match &item.daylight {
                Daylight::None => 1.0,
                Daylight::Sectors { sectors, .. } => {
                    let mut share = 0.0;
                    let mut sum = 0.0;
                    for sector in sectors {
                        if let Some((area, factor)) = sector_geometry(sector) {
                            let fraction = area / item.area_m2;
                            share += fraction;
                            sum += factor * fraction;
                        }
                    }
                    sum + (1.0 - share)
                }
                Daylight::Forfait { daylight_control } => {
                    let fraction = if context.total_usable_floor_area_m2 > 0.0 {
                        (1.8 * context.total_window_area_m2 / context.total_usable_floor_area_m2)
                            .min(1.0)
                    } else {
                        0.0
                    };
                    let sector = if *daylight_control { 0.37 } else { 1.0 };
                    sector * fraction + (1.0 - fraction)
                }
            }
        };
        // 14.7 with F_C = 1 (14.15).
        let lighting =
            power * (t_day * occupancy_day * daylight_factor + t_night * occupancy_night) / 1000.0;
        let parasitic = match &item.parasitic {
            ParasiticPower::Forfait => PARASITIC_FORFAIT_KWH_PER_M2 * item.area_m2,
            ParasiticPower::Installed {
                emergency_charging_w,
                control_standby_w,
                ..
            } => {
                (round_up_annex_x(*control_standby_w) + round_up_annex_x(*emergency_charging_w))
                    * YEAR_HOURS
                    / 1000.0
            }
        };
        let total_zone = lighting + parasitic;
        total += total_zone;
        // 7.28
        let f_l = if forfait {
            0.3
        } else if item.extracted_luminaires {
            0.5
        } else {
            1.0
        };
        gain += f_l * total_zone * 1000.0 / YEAR_HOURS;
        lighting_zones.push(LightingZoneResult {
            id: item.id.clone(),
            installed_power_w: power,
            occupancy_day,
            occupancy_night,
            daylight_factor,
            lighting_kwh: lighting,
            parasitic_kwh: parasitic,
        });
    }
    let year: f64 = MONTH_HOURS.iter().sum();
    ZoneLightingResult {
        zone_id: zone.zone_id.clone(),
        annual_kwh: total,
        monthly_kwh: std::array::from_fn(|index| total * MONTH_HOURS[index] / year),
        internal_gain_w: gain,
        lighting_zones,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> LightingContext {
        LightingContext {
            total_usable_floor_area_m2: 1000.0,
            total_window_area_m2: 150.0,
        }
    }

    fn office(lighting_zones: Vec<LightingZone>) -> ZoneLighting {
        ZoneLighting {
            zone_id: "z1".into(),
            functions: vec![UseArea {
                function: LabelFunction::Office,
                area_m2: 200.0,
            }],
            lighting_zones,
            source_reference: "lighting plan".into(),
        }
    }

    fn forfait_zone(area: f64) -> LightingZone {
        LightingZone {
            id: format!("lz{area}"),
            area_m2: area,
            power: InstalledPower::Forfait {
                led_from_2017: true,
            },
            parasitic: ParasiticPower::Forfait,
            occupancy: Occupancy {
                control: SwitchControl::ManualOrUnknown,
                central_on_control: false,
                large_office_group: false,
            },
            daylight: Daylight::Forfait {
                daylight_control: true,
            },
            extracted_luminaires: false,
        }
    }

    #[test]
    fn annex_x_rounding_examples() {
        for (input, expected) in [
            (0.09, 0.095),
            (0.14, 0.15),
            (0.33, 0.34),
            (3.3, 3.4),
            (33.0, 34.0),
            (332.0, 340.0),
            (3327.0, 3400.0),
            (3400.0, 3600.0),
            (3450.0, 3600.0),
            (8227.0, 8500.0),
            (0.0, 0.0),
        ] {
            let result = round_up_annex_x(input);
            assert!(
                (result - expected).abs() < 1e-9 * expected.max(1.0),
                "{input} → {result}"
            );
        }
        assert!((round_up_annex_x(96.0) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn forfait_office_follows_14_7_14_13_and_14_14() {
        let zone = office(vec![forfait_zone(200.0)]);
        assert!(validate_lighting(&zone, 200.0, "lighting").is_empty());
        let result = assess_zone_lighting(&zone, context());
        // P_n = 12·200; F_o;D = 1 + 0,2 − 0,2 ... F_A;D = 0,2 → F_oc + 0 = 1,0;
        // F_A;N = 0,5 → 1 + 0,2 − 0,5 = 0,7; F_D = 1 (forfait power).
        let lighting = 2400.0 * (2200.0 * 1.0 * 1.0 + 300.0 * 0.7) / 1000.0;
        let parasitic = 2.5 * 200.0;
        assert!((result.annual_kwh - (lighting + parasitic)).abs() < 1e-9);
        assert!(
            (result.internal_gain_w - 0.3 * (lighting + parasitic) * 1000.0 / 8760.0).abs() < 1e-9
        );
        let months: f64 = result.monthly_kwh.iter().sum();
        assert!((months - result.annual_kwh).abs() < 1e-9);
        assert!((result.monthly_kwh[1] - result.annual_kwh * 672.0 / 8760.0).abs() < 1e-9);
    }

    #[test]
    fn installed_power_with_occupancy_and_daylight_sectors() {
        let mut zone = forfait_zone(200.0);
        zone.power = InstalledPower::Installed {
            luminaires: vec![LuminaireGroup {
                count: 40,
                power: LuminairePower::Lamps {
                    lamp_power_w: 28.0,
                    lamp_count: 2,
                    technology: LampTechnology::FluorescentT5,
                },
            }],
            dynamic_factor: None,
            source_reference: "luminaire schedule".into(),
        };
        zone.parasitic = ParasiticPower::Installed {
            emergency_charging_w: 33.0,
            control_standby_w: 9.0,
            source_reference: "schedule".into(),
        };
        zone.occupancy.control = SwitchControl::ManualOnAutoOff;
        zone.daylight = Daylight::Sectors {
            sectors: vec![DaylightSector::VerticalWindows {
                sector_width_m: 10.0,
                lintel_height_m: 2.75,
                actual_depth_m: 6.0,
                use_actual_depth_alternative: false,
                opening_area_m2: 12.0,
                heavily_shaded: false,
                control: DaylightControl::AutomaticDimming,
            }],
            source_reference: "facade drawing".into(),
        };
        let input = office(vec![zone]);
        assert!(validate_lighting(&input, 200.0, "lighting").is_empty());
        let result = assess_zone_lighting(&input, context());
        let item = &result.lighting_zones[0];
        // 40·1,10·56 = 2 464 W → annex X 2 600 W.
        assert!((item.installed_power_w - 2600.0).abs() < 1e-9);
        // F_oc 0,8: F_o;D = 0,8 + 0,2 − 0,2 = 0,8; F_o;N = 0,5.
        assert!((item.occupancy_day - 0.8).abs() < 1e-12);
        assert!((item.occupancy_night - 0.5).abs() < 1e-12);
        // a_D;max = 5 m < 6 m: A_D = 50 m²; I_Tr = 0,24; I_RD = 2,5.
        let daylight = 0.34 * (4.13 + 20.0 * 0.24 - 1.36 * 2.5) * 0.7;
        let supply = 0.65 * vertical_daylight_supply(daylight) + 0.25;
        let sector = 1.0 - 0.73 * supply;
        let expected = sector * 0.25 + 0.75;
        assert!((item.daylight_factor - expected).abs() < 1e-12);
        let lighting = 2600.0 * (2200.0 * 0.8 * expected + 300.0 * 0.5) / 1000.0;
        assert!((item.lighting_kwh - lighting).abs() < 1e-9);
        // Parasitic: 34 W and 9,5 W rounded up per annex X.
        assert!((item.parasitic_kwh - (34.0 + 9.5) * 8760.0 / 1000.0).abs() < 1e-9);
        assert!(
            (result.internal_gain_w - (lighting + item.parasitic_kwh) * 1000.0 / 8760.0).abs()
                < 1e-9
        );
    }

    #[test]
    fn rooflights_and_table_lookups() {
        let sector = DaylightSector::Rooflights {
            rooflight_depth_m: 2.0,
            rooflight_width_m: 2.0,
            clear_height_m: 3.75,
            wall_distances_m: [1.0, 10.0, 5.0, 5.0],
            opening_area_m2: 4.0,
            room_length_m: 10.0,
            room_width_m: 10.0,
            luminaire_height_m: 2.0,
            control: DaylightControl::AutomaticSwitchingOrUnknown,
        };
        let (area, factor) = sector_geometry(&sector).unwrap();
        // Depth 2 + 1 + 3 = 6, width 2 + 3 + 3 = 8.
        assert!((area - 48.0).abs() < 1e-12);
        // k = 100/(2·20) = 2,5 → η_R 0,88; D = 0,54·0,6·4/48·0,88 = 2,4 %
        // → table 14.8 0,68; F_D;dayl = 1 − 0,63·0,68.
        assert_eq!(rooflight_utilance(2.5), 0.88);
        assert!((factor - (1.0 - 0.63 * 0.68)).abs() < 1e-12);
        assert_eq!(rooflight_supply(4.0), 0.85);
        assert!((vertical_daylight_supply(4.0) - 0.78).abs() < 1e-12);
        assert_eq!(occupancy_factor(0.0, 0.8), 1.0);
        assert!((occupancy_factor(0.1, 0.8) - 0.9).abs() < 1e-12);
        assert_eq!(burning_hours(LabelFunction::Education), (1600.0, 300.0));
        assert_eq!(specific_power(LabelFunction::Retail, true), 30.0);
    }

    #[test]
    fn validation_requires_complete_zones_and_one_method() {
        let mut second = forfait_zone(50.0);
        second.power = InstalledPower::Installed {
            luminaires: Vec::new(),
            dynamic_factor: Some(1.5),
            source_reference: String::new(),
        };
        let zone = office(vec![forfait_zone(100.0), second]);
        let codes: Vec<_> = validate_lighting(&zone, 200.0, "lighting")
            .iter()
            .map(|item| item.code)
            .collect();
        for code in [
            "lighting_zone_areas_invalid",
            "lighting_forfait_mixed",
            "lighting_luminaires_invalid",
            "lighting_dynamic_factor_invalid",
            "source_reference_required",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
    }
}
