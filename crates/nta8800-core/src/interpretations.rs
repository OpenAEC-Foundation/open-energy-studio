//! The kernel's recorded interpretation choices in one list, for the
//! report appendix and the attest dossier. Each module keeps its own
//! `INTERPRETATIONS` (or omitted-term) list next to the formulas; this
//! module only collects them.

use serde::Serialize;

/// Interpretations of one part of the norm.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterpretationGroup {
    /// Norm part, e.g. "hoofdstuk 9" or "bijlage Q".
    pub part: &'static str,
    /// Kernel module that applies them.
    pub module: &'static str,
    pub items: Vec<&'static str>,
}

/// All interpretation lists of the kernel, in norm order.
pub fn kernel_interpretations() -> Vec<InterpretationGroup> {
    let group = |part, module, items: &[&'static str]| InterpretationGroup {
        part,
        module,
        items: items.to_vec(),
    };
    vec![
        group(
            "hoofdstuk 7",
            "monthly_demand",
            crate::monthly_demand::OMITTED_CORRECTIONS,
        ),
        group(
            "hoofdstuk 8 en bijlage C",
            "constructions",
            crate::constructions::INTERPRETATIONS,
        ),
        group(
            "hoofdstuk 8",
            "envelope_elements",
            crate::envelope_elements::INTERPRETATIONS,
        ),
        group(
            "hoofdstuk 9",
            "space_heating_chain",
            crate::space_heating_chain::OMITTED_TERMS,
        ),
        group(
            "hoofdstuk 9, micro-wkk",
            "micro_chp",
            crate::micro_chp::INTERPRETATIONS,
        ),
        group(
            "hoofdstuk 10",
            "space_cooling",
            crate::space_cooling::COOLING_INTERPRETATIONS,
        ),
        group(
            "hoofdstuk 11",
            "ventilation",
            crate::ventilation::INTERPRETATIONS,
        ),
        group(
            "hoofdstuk 13, zonneboilers",
            "solar_thermal",
            crate::solar_thermal::INTERPRETATIONS,
        ),
        group("hoofdstuk 14", "lighting", crate::lighting::INTERPRETATIONS),
        group("hoofdstuk 16", "pv", crate::pv::INTERPRETATIONS),
        group("bijlage N", "annex_n", crate::annex_n::INTERPRETATIONS),
        group("bijlage Q", "annex_q", crate::annex_q::INTERPRETATIONS),
        group(
            "maatwerkadvies",
            "maatwerkadvies",
            crate::maatwerkadvies::INTERPRETATIONS,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_group_has_items() {
        let groups = kernel_interpretations();
        assert!(groups.len() >= 13);
        for group in &groups {
            assert!(!group.items.is_empty(), "{}", group.module);
        }
    }
}
