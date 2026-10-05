//! Editions of NTA 8800 the kernel can calculate with.
//!
//! The default and only registrable edition is NTA 8800:2025+C1:2026. Older
//! editions are available for comparison and control calculations; their
//! results are marked "niet voor registratie" and refused by the registration
//! check. Only editions with an implemented [`NormProfile`] are accepted;
//! the others are listed but refused with `edition_not_implemented`.
//!
//! The edition is part of the input (`normVersion` on the building and project
//! inputs). During one synchronous kernel run the chosen edition is active
//! through [`with_version`]; switch points read it with [`profile`] or
//! [`current`]. The value lives in a thread-local cell that the guard restores
//! when the run ends, so concurrent runs on other threads never see it and a
//! nested run cannot leak its edition into the caller.
//!
//! An older edition is described by its differences to the next newer one, as
//! a [`NormProfile`] with values or route variants. Every value carries the
//! page of the edition it comes from; no norm text is reproduced.

use serde::{Deserialize, Serialize};
use std::cell::Cell;

mod v2024;
mod v2025;

/// NTA 8800 edition. Ordered, so a rule introduced in edition E reads
/// `if version < E`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deserialize, Serialize,
)]
pub enum NormVersion {
    #[serde(rename = "2020+A1")]
    V2020A1,
    #[serde(rename = "2022")]
    V2022,
    #[serde(rename = "2023")]
    V2023,
    #[serde(rename = "2024")]
    V2024,
    #[default]
    #[serde(rename = "2025+C1")]
    V2025C1,
}

impl NormVersion {
    pub const ALL: [NormVersion; 5] = [
        NormVersion::V2020A1,
        NormVersion::V2022,
        NormVersion::V2023,
        NormVersion::V2024,
        NormVersion::V2025C1,
    ];

    /// Edition label as stamped on results.
    pub const fn label(self) -> &'static str {
        match self {
            NormVersion::V2020A1 => "NTA 8800:2020+A1:2020",
            NormVersion::V2022 => "NTA 8800:2022",
            NormVersion::V2023 => "NTA 8800:2023",
            NormVersion::V2024 => "NTA 8800:2024 met INT-V1:2024",
            NormVersion::V2025C1 => crate::TARGET_NORM_VERSION,
        }
    }

    /// Identifier used in the input (`normVersion`).
    pub const fn id(self) -> &'static str {
        match self {
            NormVersion::V2020A1 => "2020+A1",
            NormVersion::V2022 => "2022",
            NormVersion::V2023 => "2023",
            NormVersion::V2024 => "2024",
            NormVersion::V2025C1 => "2025+C1",
        }
    }

    pub fn is_default(&self) -> bool {
        *self == NormVersion::V2025C1
    }

    /// Only the designated edition may be used for a label or a
    /// registration in EP-Online.
    pub const fn registration_eligible(self) -> bool {
        matches!(self, NormVersion::V2025C1)
    }

    /// Whether the kernel has a profile for this edition.
    pub const fn implemented(self) -> bool {
        matches!(self, NormVersion::V2024 | NormVersion::V2025C1)
    }

    /// Designation period, as dates (inclusive start, exclusive end), for
    /// documentation and the version listing.
    pub const fn designation_period(self) -> (&'static str, Option<&'static str>) {
        match self {
            NormVersion::V2020A1 => ("2021-01-01", Some("2022-06-01")),
            NormVersion::V2022 => ("2022-06-01", Some("2023-07-01")),
            NormVersion::V2023 => ("2023-07-01", Some("2024-07-01")),
            NormVersion::V2024 => ("2024-07-01", Some("2026-05-29")),
            NormVersion::V2025C1 => ("2026-05-29", None),
        }
    }

    pub fn profile(self) -> &'static NormProfile {
        match self {
            NormVersion::V2024 => &v2024::PROFILE,
            _ => &v2025::PROFILE,
        }
    }
}

/// Description of one edition listed by the service.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditionInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub implemented: bool,
    pub registration_eligible: bool,
    pub default: bool,
    pub designated_from: &'static str,
    pub designated_until: Option<&'static str>,
}

pub fn supported_editions() -> Vec<EditionInfo> {
    NormVersion::ALL
        .iter()
        .map(|version| {
            let (from, until) = version.designation_period();
            EditionInfo {
                id: version.id(),
                label: version.label(),
                implemented: version.implemented(),
                registration_eligible: version.registration_eligible(),
                default: version.is_default(),
                designated_from: from,
                designated_until: until,
            }
        })
        .collect()
}

/// Route of annex AA (TOjuli capacity proof) per edition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnexAaVariant {
    /// 2024 p. 1115–1127: July monthly irradiance, SWM step and the t_max
    /// table by orientation.
    Monthly2024,
    /// 2025+C1 p. 1136–1147: hourly peak irradiance of table AA.3.
    Hourly2025,
}

/// Collective heat-pump source route (9.6.3.1.3 / 9.6.8.1.1.2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeatPumpSourceRoute {
    /// 2024 p. 323, with INT-V1: every source of at least 15 °C counts as
    /// external heat; source auxiliary energy Q/EER with EER 23 (aquifer
    /// built 2013 or later) or 16.
    AnySourceFrom15C2024,
    /// 2025+C1 p. 362–363: collective sources only, forfait f_P;el/23 below
    /// 20 °C.
    CollectiveOnly2025,
}

/// Table 13.18 lookup between columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Table1318Lookup {
    /// 2024 p. 614: no interpolation rule; the column at or below the demand
    /// is used (interpretation, see docs/nta8800-normversies.md).
    LowerColumn,
    /// 2025+C1 p. 631: linear interpolation.
    Interpolate,
}

/// Version-dependent values and routes. Each field names the place where it
/// switches; the edition files give the value and the page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormProfile {
    pub version: NormVersion,
    /// 5.5.8/15.3: f_BACS 1,05 for utility buildings without adequate BACS.
    /// Absent before 2025+C1.
    pub bacs_factor_route: bool,
    /// 5.14a/5.14b storage credit. Absent before 2025+C1.
    pub storage_credit: bool,
    /// Table 5.3 K_CO2 (kg/kWh): electricity, gas, oil, biomass, district
    /// heat/hot water forfait.
    pub k_co2_electricity: f64,
    pub k_co2_gas: f64,
    pub k_co2_oil: f64,
    pub k_co2_biomass: f64,
    pub k_co2_district_heat_forfait: f64,
    /// Table 5.6 K_CO2 of waste incineration (AVI) heat.
    pub k_co2_waste_incineration: f64,
    /// Indicators new in 2025+C1 (5.3.3–5.3.5, 5.5.6.1–2, 5.5.7, 5.6.4, 5.9,
    /// annex AB). Older editions report them as absent.
    pub indicators_2025: bool,
    /// Table 9.16: Ψ of uninsulated or unknown collective heating plus
    /// hot-water pipes for a connected area up to 500 m², W/(m·K).
    pub psi_collective_combined_small: f64,
    /// (9.62) collective-source correction f_cor.bron.col applies.
    pub collective_source_correction: bool,
    pub heat_pump_source_route: HeatPumpSourceRoute,
    /// 17.3.2 f): roof-edge (parapet) obstruction for collectors and PV.
    pub roof_edge_obstruction: bool,
    /// 7.6.6.1.2–7.6.6.1.3.4: solar-control glass g 0,40, fixed and rotatable
    /// louvres, vertical lamellae.
    pub permanent_shading_routes: bool,
    /// (13.157)/(13.160): f_gebouw;si;W in the daily delivered heat and the
    /// auxiliary energy of collective hot-water systems.
    pub hot_water_building_share: bool,
    pub table_13_18: Table1318Lookup,
    /// 11.4.3.2.3.3 (11.136): a measured motor efficiency is a free choice;
    /// 2024 uses it only when higher than table 11.20.
    pub motor_efficiency_free_choice: bool,
    pub annex_aa: AnnexAaVariant,
    /// AA.3.2.3/AA.3.2.4: B_C;req;TO is at least 0 kW.
    pub annex_aa_requirement_floor: bool,
    /// Table E.5: ageing factor 1,00 for cellulose, wood fibre and other
    /// bio-based blown or sprayed products (2024: 1,30 under "overig").
    pub bio_based_ageing_one: bool,
    /// Table E.11/E.12: revised bio-based λ values and reed as insulation.
    pub bio_based_lambda_2025: bool,
    /// (I.3): forfait extra resistance of reed thatch, divisor in m·K/W.
    pub reed_thatch_divisor: f64,
    /// (W.9) lower bound and W.3 cap for booster heat pumps.
    pub booster_bounds: bool,
    /// P.3 item 6: surface water and other low-temperature systems count as
    /// collective sources.
    pub wide_collective_low_temperature_sources: bool,
    /// Biomass class threshold, kW (table 5.2/5.4).
    pub biomass_threshold_kw: f64,
}

thread_local! {
    static ACTIVE: Cell<NormVersion> = const { Cell::new(NormVersion::V2025C1) };
}

/// Restores the previous edition when dropped.
struct Restore(NormVersion);

impl Drop for Restore {
    fn drop(&mut self) {
        ACTIVE.with(|cell| cell.set(self.0));
    }
}

/// Run `body` with `version` as the active edition on this thread.
pub fn with_version<R>(version: NormVersion, body: impl FnOnce() -> R) -> R {
    let previous = ACTIVE.with(|cell| cell.replace(version));
    let _restore = Restore(previous);
    body()
}

/// The edition active for the current kernel run.
pub fn current() -> NormVersion {
    ACTIVE.with(Cell::get)
}

/// Label of the active edition, for result stamps.
pub fn current_label() -> &'static str {
    current().label()
}

/// Profile of the active edition.
pub fn profile() -> &'static NormProfile {
    current().profile()
}

#[cfg(test)]
mod tests;
