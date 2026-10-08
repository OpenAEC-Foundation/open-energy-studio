#!/usr/bin/env python3
"""Writes training-data/reference-suites/isso54-v2.json.

Each case is a deeltest of ISSO 54 versie 2.0 (CCvD InstallQ 12-05-2022),
encoded as a patch on the reference test EP-W001
(training-data/isso54/EPW001.json). The encoding is our own reading of the
test description, cited by test id and page; no ISSO text is copied.

The expected values are in annex 2 of ISSO 54, a separate results
document that is not in hand, so every case is `pending`: the gate
calculates it and records the indicators without a verdict. A case that
does not calculate fails the gate. When the results arrive, the pending
metrics move to `expected` with their values (docs/nta8800-isso54-voorbereiding.md).

Run from the repository root: python3 scripts/isso54-suite.py
"""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "training-data" / "reference-suites" / "isso54-v2.json"
DOC = "ISSO 54 versie 2.0, Energie Prestatie Rekenprogramma's, testen voor het deelgebied EDR attest energieprestatie (CCvD InstallQ, 12-05-2022)"
BAND = "ISSO 54 v2.0 bijlage 2 (p. 67): afkeuring bij meer dan 1,0 % afwijking naar boven of beneden"

# The indicators of ISSO 54 §5 (p. 53) that the reference harness can read.
RESIDENTIAL_METRICS = [
    ("beng1", "kWh/m2.year", "EP1 / BENG 1, energiebehoefte (5.3)"),
    ("beng2", "kWh/m2.year", "EP2 / BENG 2, primair fossiel energiegebruik (5.2)"),
    ("beng3", "%", "EP3 / BENG 3, aandeel hernieuwbare energie (5.4)"),
    ("tojuliMax", "K", "TOjuli;max (5.7)"),
]

UTILITY_METRICS = RESIDENTIAL_METRICS[:3]

SURFACE = {"dak": 0, "vloer": 1, "zuid": 2, "noord": 3, "oost": 4, "west": 5}
NTA = "/ntaCalculation"
VENT = f"{NTA}/ventilation"
UNIT = f"{VENT}/system/unit"
BOILER = f"{NTA}/generator/boiler"


def setp(pointer, value):
    return {"op": "set", "pointer": pointer, "value": value}


def remove(pointer):
    return {"op": "remove", "pointer": pointer}


def windows(field, value):
    return [setp(f"/zones/0/surfaces/2/windows/{k}/{field}", value) for k in range(4)]


# --- EP-W004: the whole building rotates with the main (south) facade ---
COMPASS = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"]


def rotate(target):
    """Rotates the building so the window facade faces `target`."""
    turn = COMPASS.index(target) - COMPASS.index("S")
    new = lambda side: COMPASS[(COMPASS.index(side) + turn) % 8]
    steps = [
        setp(f"/zones/0/surfaces/{SURFACE[name]}/orientation", new(side))
        for name, side in [("zuid", "S"), ("noord", "N"), ("oost", "E"), ("west", "W")]
    ]
    return steps + windows("orientation", new("S"))


def year(value, pipes):
    """A construction year that ISSO 54 lets act on infiltration, the fans,
    the boiler and the insulation period of the heating pipes."""
    return [
        setp(f"{NTA}/constructionYear", value),
        setp(f"{VENT}/constructionYear", value),
        setp(f"{VENT}/fans/manufactureYear", value),
        setp(f"{BOILER}/installationYear", value),
        setp(f"{NTA}/distributionSystem/pipeTransmittance/insulation", pipes),
    ]


def unit(variant, ducts, recovery=None):
    body = {"variant": variant, "ducts": ducts, "equipmentReference": f"systeem {variant.upper()}"}
    if recovery is not None:
        body["heatRecovery"] = recovery
    return setp(UNIT, body)


def recovery(efficiency, bypass, layout="central", length=None, insulation=None, **extra):
    body = {
        "efficiency": efficiency,
        "bypass": bypass,
        "layout": layout,
        "supplyDuctInsulation": insulation or {"kind": "insulated"},
        "equipmentReference": "WTW volgens de deeltest",
    }
    if length is not None:
        body["supplyDuctLengthM"] = length
    body.update(extra)
    return body


TABLE_PLASTIC = {"method": "table", "exchanger": "counter_flow_plastic"}
NO_PASSIVE = remove(f"{VENT}/maximumCapacityForCooling")


def emission2023(kind, room_automation="unknown", balancing="none_or_unknown", pipes="two_pipe", certified=False):
    return setp(
        f"{NTA}/emission/edition2023",
        {
            "kind": kind,
            "certifiedControl": certified,
            "roomAutomation": room_automation,
            "pipeSystem": pipes,
            "balancing": balancing,
        },
    )


# (id, page, patch)
CASES = [
    ("EPW001", 4, []),
    # EP-W003b: window width halved, the facade grows accordingly (p. 7).
    ("EPW003b", 7, windows("area", 3.0)),
    # EP-W004a-g: orientation of the main facade (p. 8).
    ("EPW004a", 8, rotate("SE")),
    ("EPW004b", 8, rotate("E")),
    ("EPW004c", 8, rotate("NE")),
    ("EPW004d", 8, rotate("N")),
    ("EPW004e", 8, rotate("NW")),
    ("EPW004f", 8, rotate("W")),
    ("EPW004g", 8, rotate("SW")),
    # EP-W005a-c: D_m 80, 180, 360 kJ/m2K, open ceiling (p. 8; table 7.10).
    ("EPW005a", 8, [setp(f"{NTA}/thermalMass/floor", "light"), setp(f"{NTA}/thermalMass/wall", "light")]),
    ("EPW005b", 8, [setp(f"{NTA}/thermalMass/floor", "light"), setp(f"{NTA}/thermalMass/wall", "heavy")]),
    ("EPW005c", 8, [setp(f"{NTA}/thermalMass/floor", "heavy"), setp(f"{NTA}/thermalMass/wall", "heavy")]),
    # EP-W007a-c: infiltration (p. 9).
    ("EPW007a", 9, year(1950, {"state": "insulated", "period": "before1980_or_unknown"})),
    (
        "EPW007b",
        9,
        year(1950, {"state": "insulated", "period": "before1980_or_unknown"})
        + [setp(f"{VENT}/infiltration/renovationYear", 1990)],
    ),
    (
        "EPW007c",
        9,
        [
            setp(
                f"{VENT}/infiltration",
                {
                    "method": "measured",
                    "qv10DmPerSM2": 0.2,
                    "sourceReference": "ISSO 54 v2.0 EP-W007c p. 9: blowerdoortest q_v;10 0,2 dm3/(s.m2)",
                },
            )
        ],
    ),
    # EP-W101: ventilation systems (p. 19-21). Natural and extract systems
    # have no passive cooling via the system (11.2.2.3.2 needs mechanical
    # supply), so the reference's capacity statement is dropped there.
    ("EPW101a", 19, [unit("a1", "no_ducts"), NO_PASSIVE]),
    ("EPW101b", 19, [unit("a2a", "no_ducts"), NO_PASSIVE]),
    ("EPW101d", 20, [unit("b2", "luka_a_b_c"), NO_PASSIVE]),
    ("EPW101e", 20, [unit("b3", "luka_a_b_c"), NO_PASSIVE]),
    ("EPW101f", 20, [unit("c1", "luka_a_b_c"), NO_PASSIVE]),
    ("EPW101g", 20, [unit("c2a", "luka_a_b_c"), NO_PASSIVE]),
    ("EPW101h", 20, [unit("c3a", "luka_a_b_c"), NO_PASSIVE]),
    ("EPW101i", 20, [unit("c3b", "luka_d"), NO_PASSIVE]),
    ("EPW101j", 20, [unit("c3c", "unknown"), NO_PASSIVE]),
    ("EPW101k", 20, [unit("c4a", "luka_a_b_c"), NO_PASSIVE]),
    ("EPW101l", 20, [unit("c4b", "luka_a_b_c"), NO_PASSIVE]),
    ("EPW101m", 20, [unit("c4c", "luka_a_b_c"), NO_PASSIVE]),
    ("EPW101n", 20, [unit("c5a", "luka_a_b_c"), NO_PASSIVE]),
    ("EPW101o", 20, [unit("c5b", "luka_a_b_c"), NO_PASSIVE]),
    ("EPW101p", 20, [unit("d1", "luka_a_b_c")]),
    (
        "EPW101q",
        20,
        [
            unit(
                "d3",
                "luka_a_b_c",
                recovery({"method": "table", "exchanger": "counter_flow_aluminium"}, {"kind": "none"}, length=2.0),
            )
        ],
    ),
    (
        "EPW101t",
        21,
        [
            unit(
                "d5a",
                "luka_a_b_c",
                recovery(TABLE_PLASTIC, {"kind": "full"}, insulation={"kind": "unknown"}),
            )
        ],
    ),
    (
        "EPW101u",
        21,
        [
            unit(
                "d5b",
                "luka_a_b_c",
                recovery(
                    TABLE_PLASTIC,
                    {"kind": "full"},
                    layout="decentral",
                    length=1.0,
                    insulation={"kind": "specified", "thicknessM": 0.05, "conductivityWPerMK": 0.04},
                ),
            )
        ],
    ),
    (
        "EPW101v",
        21,
        [
            unit(
                "d5c",
                "luka_a_b_c",
                recovery(TABLE_PLASTIC, {"kind": "full"}, length=1.0, constantVolumeControl=True),
            )
        ],
    ),
    # EP-W201: emission (p. 23); NTA 8800:2022 tables 9.2-9.4 (p. 275-277).
    (
        "EPW201a",
        23,
        [
            emission2023(
                {"type": "surface", "control": "room", "system": "floor_dry", "insulation": "without_insulation"},
                room_automation="individual_with_manual_override",
            ),
            setp(f"{NTA}/emission/balancing", "none_or_unknown"),
        ],
    ),
    (
        "EPW201e",
        23,
        [
            setp(f"{NTA}/emission/system", "other_or_unknown"),
            emission2023(
                {"type": "surface", "control": "room", "system": "wall", "insulation": "minimal_insulation"},
                certified=True,
            ),
        ],
    ),
    # EP-W203: generators (p. 25-27).
    (
        "EPW203b",
        25,
        [
            setp(f"{BOILER}/kind", "conventional"),
            setp(f"{BOILER}/averageDesignEmissionTemperatureC", 80.0),
            setp(f"{BOILER}/pilotFlamePresent", True),
            setp(f"{NTA}/distributionSystem/designTemperatureClass", "90_70"),
        ],
    ),
    (
        "EPW203c",
        25,
        [
            setp(f"{BOILER}/kind", "hr100"),
            setp(f"{BOILER}/location", "outside_thermal_boundary"),
            setp(f"{BOILER}/averageDesignEmissionTemperatureC", 70.0),
            setp(f"{NTA}/distributionSystem/designTemperatureClass", "80_60"),
        ],
    ),
]


# --- EP-U: the EP-U001 office (p. 44) ---
# One use function in four enums: demand (table 7.13), ventilation (table
# 11.8), label/hot water/lighting (tables 13.1, 14.x) and Bbl.
FUNCTIONS = {
    "a": ("assembly_child_care", "assembly_child_care", "assembly_with_day_care", "assembly_child_care"),
    "b": ("other_assembly", "other_assembly", "assembly_without_day_care", "other_assembly"),
    "c": ("cell", "cell", "cell", "cell"),
    "d": ("healthcare_with_beds", "healthcare_bed_area", "healthcare_with_beds", "healthcare_with_beds"),
    "e": ("other_healthcare", "other_healthcare", "healthcare_without_beds", "other_healthcare"),
    "f": ("lodging", "lodging_building", "lodging", "lodging_in_lodging_building"),
    "g": ("education", "education", "education", "education"),
    "h": ("sport", "sport", "sport", "sport"),
    "i": ("retail", "retail", "retail", "retail"),
}
# Table 7.13: heating setpoint per function; cooling is 24 for all.
HEATING_SETPOINT = {"healthcare_with_beds": 22.0, "sport": 16.0}


def function(letter):
    usage, ventilation, label, bbl = FUNCTIONS[letter]
    heating = HEATING_SETPOINT.get(usage, 21.0)
    return [
        setp(f"{NTA}/usageFunction", usage),
        setp(f"{NTA}/bblFunction", bbl),
        setp(f"{NTA}/setpoints/heatingC", heating),
        setp(f"{VENT}/heatingSetpointC", heating),
        setp(f"{VENT}/functions", [{"function": ventilation, "areaM2": 96.0}]),
        setp(f"{NTA}/hotWater/need/areas", [{"function": label, "areaM2": 96.0}]),
        setp(f"{NTA}/lighting/0/functions", [{"function": label, "areaM2": 96.0}]),
    ]


def screen(device, control):
    return setp(
        f"{NTA}/windowSolar/movableShading",
        {"device": device, "control": control, "sourceReference": "zonwering volgens de deeltest, van binnenuit bediend"},
    )


UTILITY_CASES = [
    ("EPU001", 44, []),
    *[(f"EPU002{letter}", 44, function(letter)) for letter in "abcdefghi"],
    # EP-U002j: 30 % office, 70 % other assembly in one zone (p. 45).
    (
        "EPU002j",
        45,
        [
            setp(f"{NTA}/usageFunction", "other_assembly"),
            setp(f"{NTA}/bblFunction", "other_assembly"),
            setp(
                f"{VENT}/functions",
                [{"function": "office", "areaM2": 28.8}, {"function": "other_assembly", "areaM2": 67.2}],
            ),
            setp(
                f"{NTA}/hotWater/need/areas",
                [{"function": "office", "areaM2": 28.8}, {"function": "assembly_without_day_care", "areaM2": 67.2}],
            ),
            setp(
                f"{NTA}/lighting/0/functions",
                [{"function": "office", "areaM2": 28.8}, {"function": "assembly_without_day_care", "areaM2": 67.2}],
            ),
        ],
    ),
    # EP-U102a-c: movable shading, operated from inside (p. 45; tables 7.5/7.6).
    ("EPU102a", 45, [screen({"kind": "external_screen", "colour": "dark"}, "manual_utility_with_glare_protection")]),
    (
        "EPU102b",
        45,
        rotate("W") + [screen({"kind": "external_screen", "colour": "white"}, "manual_utility_without_glare_protection")],
    ),
    ("EPU102c", 45, rotate("E") + [screen({"kind": "drop_arm_awning"}, "automatic")]),
]


def case(test_id, page, patch):
    utility = test_id.startswith("EPU")
    metrics = UTILITY_METRICS if utility else RESIDENTIAL_METRICS
    return {
        "caseId": f"isso54-v2-{test_id}",
        "normVersion": "2022",
        "projectFile": "../isso54/EPU001.json" if utility else "../isso54/EPW001.json",
        "projectPatch": patch,
        "source": {
            "publisher": "ISSO / InstallQ (CCvD)",
            "documentId": f"{DOC}, test {test_id[:3]}-{test_id[3:]}, p. {page}",
            "edition": "versie 2.0, 12-05-2022 (NTA 8800 januari 2022)",
            "usePermission": "openbaar document; alleen de invoer is in eigen woorden gecodeerd, geen tekst overgenomen",
            "independentReviewer": "geen: verwachte uitkomsten (bijlage 2) niet in bezit",
        },
        "pending": {
            "reason": "De verwachte uitkomsten staan in bijlage 2 van ISSO 54 (apart resultatendocument), niet in bezit; de deeltest wordt doorgerekend zonder oordeel.",
            "metrics": [
                {
                    "path": path,
                    "unit": unit_,
                    "normReference": f"{reference}; band: {BAND}",
                    "relativeTolerance": 0.01,
                }
                for path, unit_, reference in metrics
            ],
        },
    }


suite = {
    "suiteId": "isso54-v2",
    "description": (
        "Deeltesten van ISSO 54 versie 2.0 (EDR, 12-05-2022) als patch op referentietest EP-W001, "
        "in NTA 8800:2022. Zonder verwachte uitkomsten (bijlage 2 niet in bezit): elke deeltest moet "
        "rekenen; de indicatoren worden vastgelegd zonder oordeel. Niet gecodeerde deeltesten en hun "
        "ontbrekende route: docs/nta8800-isso54-voorbereiding.md."
    ),
    "cases": [case(*item) for item in CASES + UTILITY_CASES],
}

OUT.write_text(json.dumps(suite, ensure_ascii=False, indent=2) + "\n")
print(f"{OUT.relative_to(ROOT)}: {len(CASES) + len(UTILITY_CASES)} cases")
