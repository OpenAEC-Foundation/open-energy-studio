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

mod v2022;
mod v2023;
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
        matches!(
            self,
            NormVersion::V2022 | NormVersion::V2023 | NormVersion::V2024 | NormVersion::V2025C1
        )
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
            NormVersion::V2022 => &v2022::PROFILE,
            NormVersion::V2023 => &v2023::PROFILE,
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
    /// 2023 p. 326 and 343: only a source of at least 20 °C counts as
    /// external heat (Q_HD;hp;in;bron in 9.84); a 15–20 °C (ground)water
    /// source keeps the source auxiliary energy Q/EER (p. 349).
    From20C2023,
    /// 2022: tables 9.27/9.29 have no source-temperature rows (p. 314–317)
    /// and 5.20/9.84 no Q_HD;hp;in;bron; no source heat is booked.
    None2022,
}

impl HeatPumpSourceRoute {
    /// The 2023/2024 routes that book table sources of a heat pump (9.27/9.29)
    /// as external heat, whatever the collective set-up.
    pub fn books_table_sources(self) -> bool {
        matches!(self, Self::AnySourceFrom15C2024 | Self::From20C2023)
    }
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
    /// Table 13.2: kitchen rows for an inner diameter of at most 8 or 10 mm
    /// over two thirds of the length. Only the "overig" row from 2024.
    pub kitchen_diameter_rows: bool,
    /// Table 11.7 τ_argII;mi, ventilative cooling (summer-night ventilation).
    pub tau_ventilative_cooling: [f64; 12],
    /// 11.2.3.3.1 f_argII for manual and automatic operation.
    pub ventilative_cooling_operation: [f64; 2],
    /// (11.71a) opening area from discharge and entry-loss coefficients.
    pub discharge_opening_route: bool,
    /// Table 11.8 f_τ of dwellings: a fixed value, or `None` for
    /// min(0,38 + 0,006·A_g; 0,8).
    pub dwelling_occupancy_factor: Option<f64>,
    /// Table 17.1 θ_e;argII;mi, °C.
    pub argii_temperature_c: [Option<f64>; 12],
    /// 11.3.2.7 ΔT_fan, K: heating, cooling of dwellings, cooling of utility
    /// buildings.
    pub fan_temperature_rise_k: [f64; 3],
    /// Table I.1 detail 17 (dormer frame in a pitched roof), Ψ columns A and
    /// B, W/(m·K).
    pub psi_detail_17: [f64; 2],
    /// Table E.10 λ of wood fibre (WF) and loose cellulose (LFCI), W/(m·K).
    pub wood_fibre_cellulose_lambda: f64,
    /// (P.25) divisor of the reference power.
    pub reference_power_divisor: f64,
    /// P.6.5.4.8: geothermal efficiency is the forfait 20, without the
    /// temperature-difference correction.
    pub geothermal_efficiency_fixed: bool,
    /// Annex AA as proof of the capacity of an active cooling system (5.7.1).
    pub annex_aa_route: bool,
    /// Table 5.7 renovatiestandaard (§5.3.1.2).
    pub renovation_standard: bool,
    /// Table 14.3 column "ledverlichting geïnstalleerd vanaf 2017".
    pub led_2017_column: bool,
    /// (8.17)/(8.18) glazing-bar (roeden) term in U_w.
    pub glazing_bar_term: bool,
    /// 7.6.6.1.4: f_sh;with = 0 on the heating balance of dwellings with
    /// manual or ISO 52016-3 tuned automatic shading; 2023 uses table 7.7 on
    /// both balances.
    pub dwelling_shading_heating_off: bool,
    /// (5.47)/(5.55) f_Pren of residual heat: 1 − f_rw;aux;spec (2024+), or
    /// 1 − f_P;del;rw with f_P;del;rw = f_rw;aux;spec·f_P;del;el (2023).
    pub residual_heat_pren_primary: bool,
    /// Note 2 of 8.2.2.1 e): U_rc of rooflights per NEN-EN 1873 converted
    /// to U_C. Absent in 2023.
    pub rooflight_route: bool,
    /// I.2.2.4.2 tables I.13/I.14: forfait panel U by build-year class for
    /// buildings from 1965 (2023 only).
    pub panel_build_year_tables: bool,
    /// (11.71) factor on the NEN 1087 net opening without louvre or screen
    /// specification.
    pub unspecified_screen_factor: f64,
    /// (11.77a/b): roof openings (β < 60°) count in every orientation sector.
    pub cross_area_roof_all_sectors: bool,
    /// 9.4.1/9.4.2: L_zi = 0 for heating-only pipes in heated zones, full
    /// month t_H;op for combined pipes and ϑ_H;mean ≥ 65 °C with delivery
    /// sets (9.30). All three absent in 2023.
    pub distribution_2024_rules: bool,
    /// Table 9.16 rows for collective heating plus hot-water pipes.
    pub table_9_16_combined_rows: bool,
    /// 9.4.3: f_H;dis;rbl = 0,5 for uninsulated pipes in an uninsulated
    /// outer wall or floor of a heated zone (2023 only).
    pub distribution_half_recoverable_route: bool,
    /// Table 13.4 rows "klein"/"overig" for an unknown diameter (2023)
    /// instead of table 13.29 and the 35/80 mm rule.
    pub table_13_4_system_rows: bool,
    /// 13.141a–d series arrangements and 13.8.4.10 heat pumps in series.
    pub hot_water_series_routes: bool,
    /// 9.3.2–9.3.3 heating emission per tables 9.2–9.10 of NTA 8800:2023
    /// (Δθ_str, Δθ_ctr, Δθ_emb, Δθ_rad, Δθ_im, Δθ_hydr, Δθ_roomaut) instead
    /// of tables 9.2–9.4 of 2024.
    pub emission_tables_2023: bool,
    /// 10.3.3 cooling emission per tables 10.2–10.5 of NTA 8800:2023 instead
    /// of tables 10.35/10.4/10.5 of 2024.
    pub cooling_emission_tables_2023: bool,
    /// Tables 9.27/9.29 and P.5 of NTA 8800:2022: columns up to 55 °C,
    /// no source-temperature rows, table 9.27 for all dwellings (no 25 kW
    /// or collective split). Above 55 °C annex Q applies (2022 p. 313).
    pub heat_pump_tables_2022: bool,
    /// 5.8 and P.6.5.4.11: electric generators in the flex mode.
    pub flex_mode_route: bool,
    /// (9.58): the installed power of the preferences times f_gebouw;si;H.
    pub preference_beta_building_share: bool,
    /// Table 9.14 design temperature classes 60/50 and 70/60.
    pub design_classes_60_and_70: bool,
    /// 9.3.3 table 9.11 note: fan power of an assembly tested to
    /// NEN-EN 16430.
    pub tested_emission_fan_power: bool,
    /// Table 7.5 rows "onbekende kleur".
    pub unknown_shade_colour_rows: bool,
    /// Table 7.10 by the mass of the zone per m² usable area (2022) instead
    /// of tables 7.10–7.12 by floor and wall type.
    pub thermal_mass_by_kg_per_m2: bool,
    /// (8.47): h is the fixed value 0,125 m; 2022 takes the actual height.
    pub crawl_wall_height_fixed: bool,
    /// Table E.5 F_A;iso of mineral-wool flakes (MW).
    pub mineral_wool_flakes_ageing: f64,
    /// (I.2) λ_equi;ntr, W/(m·K).
    pub lambda_equi_ntr: f64,
    /// (I.2): a known higher λ including anchors, moisture and ageing
    /// replaces λ_equi;ntr.
    pub lambda_equi_known_route: bool,
    /// 11.2.2.x (11.57): rooms with a swimming pool take twice the sport
    /// function's q_usi;spec.
    pub swimming_pool_route: bool,
    /// Table 11.x: heat-recovery efficiency declared per NEN-EN 13053.
    pub en_13053_route: bool,
    /// 13.6.3: f_sto;dis;ls = 1,5 for an electric boiler with insulated
    /// hot-water pipes (2022 only).
    pub electric_boiler_insulated_pipe_factor: bool,
    /// (13.69a)/(13.137a): ϑ_sto;amb = ϑ_int;set;H;stc with an exhaust-air
    /// heat pump for hot water.
    pub storage_ambient_exhaust_air: bool,
    /// 13.153b: C_W;mixed air of combi heat pumps on mixed air.
    pub mixed_air_route: bool,
    /// (14.15) with table 14.4: F_C from the maintenance factor MF of
    /// systems with constant-illuminance control (2022 only).
    pub lighting_maintenance_factor: bool,
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
