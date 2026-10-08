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
            # Only the emitter, the NEN-EN 215 declaration and the missing
            # hydronic balancing change; room control stays as in EP-W001.
            emission2023(
                {"type": "surface", "control": "room", "system": "wall", "insulation": "minimal_insulation"},
                room_automation="individual_per_room",
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


# --- EP-W003a/c: window U from 8.15 and 8.22/8.23 (p. 7) ---
# The project takes the window U as a value; it is computed here with the
# kernel's window_u formulas and rounded per 8.2.2.1 (one decimal above 1,0).
# EP-W003a: 8.15 simplified, max(0,7.2,0+0,3.3,4; 0,8.2,0+0,2.3,4) + 2,5.0,11
#   = 2,695 -> 2,7.
# EP-W003c: U_W+shut = 1/(1/1,8 + 0,2) = 1,324 (8.23); effective
#   0,5.1,8 + 0,5.1,324 = 1,562 -> 1,6 (8.22, f_shut;with 0,5). Only the extra
#   resistance is varied; the roller shutter is not entered as sun shading.
CASES += [
    ("EPW003a", 7, windows("uValue", 2.7)),
    ("EPW003c", 7, windows("uValue", 1.6)),
]


# --- EP-W015a/b: vertical pipes through the thermal envelope (p. 18; 7.3.3) ---
# EP-W015a: no penetrations, no pipe. EP-W015b: one insulated pipe running
# through both storeys (table 7.1 footnote a).
CASES += [
    ("EPW015a", 18, [setp(f"{NTA}/verticalPipes", [])]),
    (
        "EPW015b",
        18,
        [setp(f"{NTA}/verticalPipes", [{"id": "leiding-1", "storeys": 2, "insulated": True, "sourceReference": "ISSO 54 v2.0 EP-W015b p. 18: 1 geisoleerde verticale leiding per bouwlaag"}])],
    ),
]


# --- EP-W016: a door in the north facade, 2 m2 incl. frame (p. 18) ---
# A door is a window element of the north facade (gross 43,2 m2, net 41,2).
# Opaque: U 3,4 (8.20), g 0. With glass (U_gl 2,8, g_gl 0,7): U area-weighted
# over glass and opaque door, no edge psi given; the frame fraction is
# project-wide (0,25), so the glass share enters as an equivalent g:
# g_gl . share / (1 - 0,25).
def door(u_value, g_value):
    return setp(
        "/zones/0/surfaces/3/windows",
        [
            {
                "id": "deur-noord",
                "name": "Deur noord",
                "area": 2.0,
                "uValue": u_value,
                "gValue": g_value,
                "orientation": "N",
                "surfaceId": "gevel-noord",
            }
        ],
    )


CASES += [
    ("EPW016a", 18, [door(3.4, 0.0)]),
    ("EPW016b", 18, [door(3.1, round(0.7 * 0.5 / 0.75, 4))]),
    ("EPW016c", 18, [door(2.9, round(0.7 * 0.8 / 0.75, 4))]),
]


# --- EP-W012: movable shading with the building turned (p. 15) ---
# Table 7.5/7.6 devices, operated from inside: manual residential control.
def residential_screen(device):
    return setp(
        f"{NTA}/windowSolar/movableShading",
        {"device": device, "control": "manual_residential", "sourceReference": "zonwering volgens de deeltest, van binnenuit bediend"},
    )


CASES += [
    ("EPW012a", 15, [residential_screen({"kind": "external_screen", "colour": "dark"})]),
    ("EPW012b", 15, rotate("SW") + [residential_screen({"kind": "external_venetian_blind", "colour": "white"})]),
    ("EPW012c", 15, rotate("W") + [residential_screen({"kind": "external_screen", "colour": "other"})]),
    ("EPW012d", 15, rotate("SE") + [residential_screen({"kind": "internal_metallised_fabric"})]),
    ("EPW012e", 15, rotate("E") + [residential_screen({"kind": "drop_arm_awning"})]),
    ("EPW012f", 15, [residential_screen({"kind": "folding_arm_awning"})]),
]


# --- EP-W4xx: hot water (p. 33-40) ---
HW = f"{NTA}/hotWater"


def draw_off(kitchen, diameter, bathroom):
    return setp(
        f"{HW}/emission",
        {
            "method": "residential",
            "served": "kitchen_and_bathroom",
            "kitchenLengthM": kitchen,
            "kitchenPipeDiameter": diameter,
            "bathroomLengthM": bathroom,
            "sourceReference": "uittapleidingen volgens de deeltest",
        },
    )


def showers(units, connection):
    return setp(
        f"{HW}/showerHeatRecovery",
        {"showers": units, "connection": connection, "sourceReference": "douche-WTW volgens de deeltest"},
    )


def vessel(volume, loss, factor, heated, vessel_id="vat-1", **extra):
    body = {
        "id": vessel_id,
        "volumeL": volume,
        "loss": loss,
        "connectionFactor": factor,
        "inHeatedZone": heated,
        "sourceReference": "voorraadvat volgens de deeltest",
    }
    body.update(extra)
    return body


def hw_generator(body):
    return setp(f"{HW}/generator", body)


def indirect_hr107(inside, also_heating=True):
    return hw_generator(
        {"kind": "indirect_boiler", "boiler": "hr107", "oil": False, "insideBoundary": inside, "alsoSpaceHeating": also_heating}
    )


MINIMAL = {"method": "minimal"}
FULL = {"method": "full"}


def LABEL(label):
    return {"method": "label", "label": label}


def MEASURED(w_per_k):
    return {"method": "measured", "transmissionWPerK": w_per_k}


def storage(volume, loss, backup=None):
    body = {"totalVolumeL": volume, "loss": loss}
    if backup is not None:
        body["backupVolumeL"] = backup
    return body


def solar(area, collector, orientation, tilt, obstruction, vessel_body, solar_type="preheater", use="water_heating", pvt=None):
    heater = {
        "id": "zonneboiler-1",
        "solarUse": use,
        "method": {
            "method": "calculated",
            "solarType": solar_type,
            "collectors": {
                "moduleAreaM2": area,
                "moduleCount": 1,
                "orientation": orientation,
                "tiltDeg": tilt,
                "obstruction": obstruction,
                "efficiency": {"method": "forfait", "collector": collector},
                "loopPipes": {"method": "forfait"},
            },
            "storage": vessel_body,
        },
        "sourceReference": "zonneboiler volgens de deeltest, collector- en vatgegevens forfaitair waar niet gegeven",
    }
    if pvt is not None:
        heater["pvt"] = pvt
    return setp(f"{HW}/solar", [heater])


def pv(system_id, kpk, area, azimuth, tilt, mounting, obstruction=MINIMAL, **extra):
    body = {
        "id": system_id,
        "peakPower": {"method": "declared_specific", "peakPowerWPerM2": kpk, "panelAreaM2": area},
        "azimuthDeg": azimuth,
        "tiltDeg": tilt,
        "mounting": mounting,
        "obstruction": obstruction,
        "sourceReference": "PV-systeem volgens de deeltest",
    }
    body.update(extra)
    return body


CASES += [
    # EP-W401a/b: draw-off lengths and kitchen pipe diameter (p. 33; table 13.2).
    ("EPW401a", 33, [draw_off(7.0, "up_to_10_mm", 3.0)]),
    ("EPW401b", 33, [draw_off(3.0, "up_to_8_mm", 1.0)]),
    # EP-W403a-d: shower heat recovery (p. 34-35; table 13.8, 13.5.3).
    ("EPW403a", 34, [showers([{"unit": "vertical"}], "mixer_and_heater")]),
    ("EPW403b", 34, [showers([{"unit": "horizontal"}], "heater_only")]),
    (
        "EPW403c",
        34,
        [
            showers(
                [{"unit": "declared", "efficiency": 0.45, "sourceReference": "ISSO 54 v2.0 EP-W403c p. 34: rendement 45 % (verklaring aanwezig)"}],
                "mixer_only",
            )
        ],
    ),
    ("EPW403d", 35, [showers([{"unit": "horizontal"}, {"unit": "vertical"}], "mixer_only")]),
    # EP-W404a-d: storage vessels (p. 35; 13.6, f_sto;dis;ls 2022 p. 549-550).
    (
        "EPW404a",
        35,
        [setp(f"{HW}/storage", [vessel(100.0, {"method": "label", "label": "a_plus"}, 2, True)]), indirect_hr107(True)],
    ),
    (
        "EPW404b",
        35,
        [
            setp(
                f"{HW}/storage",
                [
                    vessel(200.0, {"method": "label", "label": "f"}, 5, False, "vat-1"),
                    vessel(200.0, {"method": "label", "label": "f"}, 5, False, "vat-2"),
                ],
            ),
            indirect_hr107(False),
            setp(f"{BOILER}/location", "outside_thermal_boundary"),
        ],
    ),
    (
        "EPW404c",
        35,
        [
            setp(
                f"{HW}/storage",
                [
                    vessel(
                        100.0,
                        {"method": "measured_standby", "standbyKwhPerDay": 2.0, "referenceStorageC": 60.0, "referenceAmbientC": 20.0},
                        1,
                        True,
                    )
                ],
            ),
            setp(f"{HW}/boilingWaterTap", True),
            hw_generator({"kind": "electric_boiler"}),
        ],
    ),
    (
        "EPW404d",
        35,
        [
            setp(
                f"{HW}/storage",
                [vessel(150.0, {"method": "label", "label": "c"}, 2, True, electricBoilerInsulatedPipe=True)],
            ),
            hw_generator({"kind": "electric_boiler"}),
        ],
    ),
    # EP-W406: hot-water generators (p. 37-40; tables 13.25-13.28).
    ("EPW406a", 37, [hw_generator({"kind": "gas_appliance", "appliance": "combi_gaskeur", "measuredClass": "class3"})]),
    (
        "EPW406b",
        37,
        [
            setp(f"{HW}/storage", [vessel(80.0, {"method": "label", "label": "c"}, 2, True, electricBoilerInsulatedPipe=True)]),
            hw_generator({"kind": "electric_boiler"}),
        ],
    ),
    ("EPW406c", 37, [hw_generator({"kind": "electric_instantaneous"})]),
    (
        "EPW406g",
        37,
        [hw_generator({"kind": "gas_storage_heater", "volumeL": 100.0, "before1985": False, "inHeatedZone": True})],
    ),
    # EP-W405a-f: solar water heaters, method 2 with table 13.14 forfaits
    # (p. 35-36; 13.7.2.2). The given area is taken as the reference area.
    ("EPW405a", 35, [solar(5.0, "glazed", "south", 45.0, MINIMAL, storage(100.0, LABEL("a")))]),
    ("EPW405b", 35, [solar(3.0, "unglazed_or_unknown", "west", 30.0, MINIMAL, storage(200.0, LABEL("c")))]),
    ("EPW405c", 36, [solar(3.0, "evacuated_tube", "south_east", 60.0, FULL, storage(150.0, MEASURED(0.5)))]),
    (
        "EPW405d",
        36,
        [
            solar(5.0, "glazed", "south", 30.0, FULL, storage(150.0, MEASURED(0.5), backup=150.0), solar_type="integrated_backup"),
            indirect_hr107(True, also_heating=False),
        ],
    ),
    (
        "EPW405e",
        36,
        [
            solar(5.0, "glazed", "south", 30.0, MINIMAL, storage(220.0, LABEL("b"), backup=100.0), solar_type="integrated_backup"),
            indirect_hr107(True),
        ],
    ),
    ("EPW405f", 36, [solar(5.0, "glazed", "south", 30.0, MINIMAL, storage(220.0, LABEL("b")), use="combi")]),
    (
        "EPW406q",
        39,
        [
            hw_generator(
                {
                    "kind": "gas_appliance",
                    "appliance": "combi_gaskeur_hr_cw",
                    "measuredClass": "class4",
                    "declared": {"value": 0.725, "sourceReference": "ISSO 54 v2.0 EP-W406q p. 39: kwaliteitsverklaring, gemeten tappatroon CW4, rendement 72,5 %"},
                }
            )
        ],
    ),
]


# --- EP-W203/W204: heating generators (p. 25-27) ---
GEN = f"{NTA}/generator"
GROUND_SOURCES = {"ground", "groundwater_below15_c"}
# Table 9.28 evidence: ISSO 54 states only "voldoet aan tabel 9.28"; the test
# points are set just above the table 9.28 minimums (fictitious values).
HIGH_POINTS = {
    "ground": [("b0_w45", 3.05), ("b0_w35", 3.55)],
    "groundwater_below15_c": [("w10_w45", 3.8), ("w10_w35", 4.45)],
    "outdoor_air": [("a7_wet6_w45", 2.8), ("a7_wet6_w35", 2.9), ("a_minus7_wet_minus8_w45", 1.95)],
}


def heat_pump(test_id, source, supply, high=False, declaration=None, sink="hydronic"):
    forfait = {
        "generatorId": "wp",
        "classificationSourceReference": f"ISSO 54 v2.0 {test_id}: elektrische warmtepomp, bron {source}",
        "scope": "residential_at_most25_kw",
        "source": source,
        "sink": sink,
        "designSupplyTemperatureC": supply if sink == "hydronic" else None,
    }
    if source in GROUND_SOURCES:
        forfait["sourceCorrectionFactor"] = 1.0
        forfait["sourceCorrectionReference"] = "geen regeneratie genoemd in de deeltest"
    if high:
        forfait["rowVariant"] = "table_9_28_high_efficiency"
        forfait["highEfficiencyEvidence"] = {
            "productReference": f"ISSO 54 v2.0 {test_id}: COP voldoet aan tabel 9.28",
            "testReportReference": "fictief: de deeltest geeft geen meetwaarden, hier net boven de minima van tabel 9.28",
            "testStandardEdition": "NEN-EN 14511-2:2007",
            "points": [{"condition": c, "measuredCop": v} for c, v in HIGH_POINTS[source]],
        }
    if declaration is not None:
        forfait["qualityDeclaration"] = declaration
    return setp(
        GEN,
        {
            "kind": "heat_pump_forfait",
            "forfait": forfait,
            "sourceSystem": "individual",
            "sourceSystemReference": f"ISSO 54 v2.0 {test_id}: individuele warmtepomp",
        },
    )


def temperature_class(value):
    return setp(f"{NTA}/distributionSystem/designTemperatureClass", value)


def biomass(test_id, appliance, location, compliant, automatic=False, power=None):
    body = {
        "kind": "biomass",
        "appliance": appliance,
        "location": location,
        "annexRCompliantAtMost500Kw": compliant,
        "annexRReference": f"ISSO 54 v2.0 {test_id}: biomassa {'voldoet' if compliant else 'voldoet niet'} aan bijlage R",
        "equipmentReference": f"ISSO 54 v2.0 {test_id}",
        "automaticFuelFeed": automatic,
    }
    if power is not None:
        body["auxiliary"] = {"electricallyConnectedDevices": 1, "nominalPowerKw": power, "sourceReference": f"ISSO 54 v2.0 {test_id}: vermogen {power} kW, aangesloten op het net"}
    else:
        body["auxiliary"] = {"electricallyConnectedDevices": 0, "sourceReference": f"ISSO 54 v2.0 {test_id}: geen aansluiting op het elektriciteitsnet"}
    if appliance != "central_boiler":
        # §9.6.5: the stove is the only heating in the rooms it serves (the
        # deeltest replaces the boiler by the stove).
        body["soleHeatingInServedRooms"] = True
    return setp(GEN, body)


def renewable(test_id, below_20, exhaust=False):
    """Annex P evidence for the renewable share of a heat pump."""
    return setp(
        f"{NTA}/heatPumpRenewable",
        {"sourceBelow20C": below_20, "exhaustAirSource": exhaust, "sourceReference": f"ISSO 54 v2.0 {test_id}: bron volgens de deeltest"},
    )


# Generators outside 9.85 (biomass, external heat) have a pump of their own
# (9.41-9.51, power and EEI unknown: forfait).
CALCULATED_PUMP = setp(
    f"{NTA}/distributionSystem/pump",
    {"method": "calculated", "heatMeterPresent": False, "sourceReference": "distributiepomp, vermogen en EEI onbekend"},
)
# Local stoves without a water-borne system: emission as local heater
# (table 9.2 "overige"), no distribution.
LOCAL = [
    setp(
        f"{NTA}/emission",
        {"system": "local_heater", "balancing": "not_applicable", "control": "individual_room_thermostats", "sourceReference": "lokale toestellen, regeling individueel per ruimte"},
    ),
    remove(f"{NTA}/distributionSystem"),
    setp(
        f"{NTA}/distribution",
        {"method": "declared", "monthlyLossKwh": [0.0] * 12, "sourceReference": "geen watergedragen distributiesysteem (lokale toestellen): geen distributieverlies"},
    ),
]


CASES += [
    (
        "EPW203a",
        25,
        [
            setp(
                GEN,
                {
                    "kind": "forfait_heater",
                    "heaterKind": "local_with_flue",
                    "fuel": "natural_gas",
                    "equipmentReference": "ISSO 54 v2.0 EP-W203a p. 25: lokale gasverwarming met afvoer, zonder elektriciteitsaansluiting",
                    "auxiliary": {"electricallyConnectedDevices": 0, "sourceReference": "ISSO 54 v2.0 EP-W203a: zonder elektriciteitsaansluiting"},
                },
            )
        ]
        + LOCAL,
    ),
    ("EPW203d", 25, [heat_pump("EP-W203d", "ground", 35.0), temperature_class("35_30"), renewable("EP-W203d", True)]),
    ("EPW203e", 25, [heat_pump("EP-W203e", "groundwater_below15_c", 45.0, high=True), temperature_class("45_40"), renewable("EP-W203e", True)]),
    ("EPW203f", 25, [heat_pump("EP-W203f", "outdoor_air", 55.0, high=True), temperature_class("55_47"), renewable("EP-W203f", True)]),
    ("EPW203h", 26, [heat_pump("EP-W203h", "ground", 50.0, high=True), temperature_class("50_42"), renewable("EP-W203h", True)]),
    (
        "EPW203i",
        26,
        [
            heat_pump(
                "EP-W203i",
                "outdoor_air",
                50.0,
                declaration={"declarationReference": "ISSO 54 v2.0 EP-W203i p. 26: kwaliteitsverklaring", "generationEfficiency": 3.8, "auxiliaryKwhPerYear": 100.0},
            ),
            temperature_class("50_42"),
            renewable("EP-W203i", True),
        ],
    ),
    (
        "EPW203p",
        27,
        [heat_pump("EP-W203p", "exhaust_air", 55.0), temperature_class("55_47"), renewable("EP-W203p", True, exhaust=True), unit("c1", "luka_a_b_c"), NO_PASSIVE],
    ),
    (
        "EPW204a",
        27,
        [
            setp(
                GEN,
                {
                    "kind": "external_heat",
                    "supplierReference": "ISSO 54 v2.0 EP-W204a p. 27: warmtelevering door derden",
                    "qualityDeclarationPresent": False,
                    "auxiliary": {"electricallyConnectedDevices": 1, "sourceReference": "ISSO 54 v2.0 EP-W204a: afleverset met hoofddistributiepomp"},
                },
            ),
            CALCULATED_PUMP,
        ],
    ),
    ("EPW204c", 27, [biomass("EP-W204c", "freestanding_wood_stove", "inside_thermal_boundary", True)] + LOCAL),
    # EP-W204d (pellet stove not meeting annex R): table 9.30 (2022 p. 337)
    # only covers appliances meeting annex R, so the forfait route has no
    # efficiency and the kernel refuses (biomass_class_unsupported). Not encoded.
    ("EPW204e", 28, [biomass("EP-W204e", "central_boiler", "outside_thermal_boundary", True, automatic=True, power=5.0), CALCULATED_PUMP]),
]


# --- EP-W501: PV panels (p. 42-43; 16.4a, tables 16.1/16.2, 17.15) ---
CASES += [
    ("EPW501a", 42, [setp(f"{NTA}/pvSystems", [pv("pv-1", 165.0, 16.0, 180.0, 30.0, "moderately_ventilated")])]),
    (
        "EPW501b",
        42,
        [
            setp(
                f"{NTA}/pvSystems",
                [
                    pv("pv-1", 170.0, 16.0, 225.0, 45.0, "not_ventilated"),
                    pv("pv-2", 140.0, 3.2, 135.0, 30.0, "strongly_ventilated", FULL),
                ],
            )
        ],
    ),
    (
        "EPW501c",
        42,
        [
            setp(
                f"{NTA}/pvSystems",
                [
                    {
                        "id": "pv-1",
                        "peakPower": {"method": "table16_1", "moduleType": "multicrystalline_before2001", "panelAreaM2": 6.4},
                        "azimuthDeg": 90.0,
                        "tiltDeg": 15.0,
                        "mounting": "moderately_ventilated",
                        "obstruction": MINIMAL,
                        "sourceReference": "ISSO 54 v2.0 EP-W501c p. 42: multikristallijn, geplaatst in 2000 (tabel 16.1)",
                    }
                ],
            )
        ],
    ),
    # EP-W501d: 10 m2 PVT covered with single glass on a solar preheater
    # (p. 42-43; table 16.4 and 13.16).
    (
        "EPW501d",
        42,
        [
            setp(
                f"{NTA}/pvSystems",
                [
                    pv(
                        "pvt-1",
                        150.0,
                        10.0,
                        270.0,
                        60.0,
                        "not_ventilated",
                        pvt={"kind": "glazed", "collectorAreaM2": 10.0, "storageVolumeL": 100.0},
                    )
                ],
            ),
            solar(10.0, "glazed", "west", 60.0, MINIMAL, storage(100.0, LABEL("a")), pvt="single_glazed"),
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


LZ = f"{NTA}/lighting/0/lightingZones/0"


def lamps(watts, technology):
    return setp(
        f"{LZ}/power",
        {
            "method": "installed",
            "luminaires": [{"count": 1, "power": {"method": "lamps", "lampPowerW": watts, "lampCount": 1, "technology": technology}}],
            "sourceReference": f"lampvermogen {watts} W volgens de deeltest, als een groep",
        },
    )


def switch(control, central=False):
    return setp(f"{LZ}/occupancy", {"control": control, "centralOnControl": central, "largeOfficeGroup": False})


PARASITIC_601A = setp(
    f"{LZ}/parasitic",
    {"method": "installed", "emergencyChargingW": 10.0, "controlStandbyW": 20.0, "sourceReference": "ISSO 54 v2.0 EP-U601a p. 50: noodverlichting 10 W, besturing 20 W"},
)
U601A = [lamps(768.0, "led"), setp(f"{LZ}/constantIlluminance", "led_l80"), PARASITIC_601A]


def utility_heat_pump(test_id, source, supply, page):
    """EP-U303: a utility heat pump, table 9.29 (scope utility)."""
    step = heat_pump(test_id, source, supply)
    forfait = step["value"]["forfait"]
    forfait["scope"] = "utility_collective_or_over25_kw"
    # c_source (annex V) is footnote a of table 9.27, dwellings only.
    forfait.pop("sourceCorrectionFactor", None)
    forfait.pop("sourceCorrectionReference", None)
    return step


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
    # EP-U303a-e: utility heat pumps (p. 47; table 9.29).
    ("EPU303a", 47, [utility_heat_pump("EP-U303a", "ground", 30.0, 47), temperature_class("30_27"), renewable("EP-U303a", True)]),
    ("EPU303b", 47, [utility_heat_pump("EP-U303b", "outdoor_air", 35.0, 47), temperature_class("35_30"), renewable("EP-U303b", True)]),
    (
        "EPU303c",
        47,
        [utility_heat_pump("EP-U303c", "exhaust_air", 40.0, 47), temperature_class("40_35"), renewable("EP-U303c", True, exhaust=True), unit("c1", "luka_a_b_c"), NO_PASSIVE],
    ),
    ("EPU303d", 47, [utility_heat_pump("EP-U303d", "groundwater_below15_c", 50.0, 47), temperature_class("50_42"), renewable("EP-U303d", True)]),
    ("EPU303e", 47, [utility_heat_pump("EP-U303e", "surface_water", 55.0, 47), temperature_class("55_47"), renewable("EP-U303e", True)]),
    # EP-U601a-c: lighting power (p. 50; 14.8/14.9, table 14.2, NTA 8800:2022
    # table 14.4 constant-illuminance compensation).
    ("EPU601a", 50, U601A),
    ("EPU601b", 50, [lamps(960.0, "unknown_or_other")]),
    ("EPU601c", 50, [lamps(1344.0, "fluorescent_t5"), setp(f"{LZ}/constantIlluminance", "linear_fluorescent")]),
    # EP-U602a-f/i: lighting control (p. 50-51; table 14.5).
    ("EPU602a", 50, [switch("manual_or_unknown", central=True)]),
    ("EPU602b", 50, [switch("manual_or_unknown")]),
    ("EPU602c", 50, [switch("manual_with_sweep")]),
    ("EPU602d", 50, function("d") + [switch("auto_on_dimmed")]),
    ("EPU602e", 50, [switch("manual_on_dimmed")]),
    ("EPU602f", 50, [switch("manual_on_auto_off")]),
    ("EPU602i", 51, U601A + [setp(f"{LZ}/extractedLuminaires", True)]),
]


# --- Cooling (EP-W3xx p. 31-32, EP-U4xx p. 48) ---
# NTA 8800:2022 has the cooling emission tables 10.2-10.5 of 2023 (p. 354-356;
# the kernel's `edition2023` block). The reference cooling of the dwelling is
# that of EP-U001: floor cooling, balancing and control unknown, insulated
# pipes, a pump, no heat meter, an individual electric compression chiller.
COOL = f"{NTA}/cooling"


def cooling_emission(emitter, room_automation="unknown", control="unknown_or_other", balancing="none_or_unknown", row2023="none_or_unknown", room_control="central", fans=0, why=""):
    body = {
        "emitter": emitter,
        "balancing": balancing,
        "control": control,
        "sourceReference": why,
        "edition2023": {"control": room_control, "certifiedControl": False, "balancing": row2023, "roomAutomation": room_automation},
    }
    if fans:
        body["fanCoilCount"] = fans
    return setp(f"{COOL}/emission", body)


def cooling_pump(dwelling, **extra):
    body = {"hydraulicallyBalanced": False, "floorCount": 2, "heatMeter": False, "individualDwellingInstallation": dwelling, "sourceReference": "pomp aanwezig, gegevens onbekend"}
    body.update(extra)
    return body


def cooling_distribution(design="t17_to21", pipe="insulated_from1995", fittings=True, unconditioned=0.0, dwelling=False, why="", **extra):
    body = {"designTemperature": design, "pipe": {"kind": pipe}, "fittingsInsulated": fittings, "pump": cooling_pump(dwelling), "sourceReference": why}
    if unconditioned is not None:
        body["unconditionedPipeLengthM"] = unconditioned
    body.update(extra)
    return setp(f"{COOL}/distribution", body)


def cooling_generators(*generators):
    return setp(f"{COOL}/generators", list(generators))


def chiller(kind_body, gen_id="koeling", capacity=None, why=""):
    body = {"id": gen_id, "generator": kind_body, "equipmentReference": why}
    if capacity is not None:
        body["capacityKw"] = capacity
    return body


def active(system, why):
    return setp(
        f"{NTA}/activeCooling",
        {"system": system, "capacity": {"method": "dynamic_cooling_load", "sourceReference": "ISSO 54 v2.0: koelinstallatie volgens de deeltest"}, "sourceReference": why},
    )


# EP-W301a: the cooled reference dwelling (p. 31). Design 12/16 per the test;
# table 10.8 note 2 (2022 p. 361) prescribes 17/21 for floor cooling only, as
# at EP-U001.
W301A = [
    active("compression_table10_29", "ISSO 54 v2.0 EP-W301a p. 31: individuele elektrische compressiekoelmachine"),
    setp(
        COOL,
        {
            "emission": {
                "emitter": "floor_cooling",
                "balancing": "none_or_unknown",
                "control": "unknown_or_other",
                "sourceReference": "ISSO 54 v2.0 EP-W301a p. 31: vloerkoeling, inregeling en regeling onbekend",
            },
            "distribution": {
                "designTemperature": "t17_to21",
                "pipe": {"kind": "insulated_from1995"},
                "fittingsInsulated": True,
                "pump": cooling_pump(True),
                "unconditionedPipeLengthM": 0.0,
                "sourceReference": "ISSO 54 v2.0 EP-W301a p. 31: ontwerp 12/16 volgens de test, tabel 10.8 opmerking 2 (2022 p. 361) schrijft voor uitsluitend vloerkoeling 17/21 voor; geen leidingen in ongeconditioneerde ruimte, leidingen, kleppen en beugels geisoleerd, pomp, geen warmtemeter",
            },
            "generators": [chiller({"kind": "compression"}, why="ISSO 54 v2.0 EP-W301a p. 31: individuele elektrische compressiekoelmachine")],
        },
    ),
]

CASES += [
    ("EPW301a", 31, W301A),
    # EP-W301b: fan coils on the ceiling, automatic control per room (p. 31).
    # Radiant 17/21 no longer applies: the test's 12/16 design is used. The
    # number of fan coils is not given (n_fan 0, 10.18).
    (
        "EPW301b",
        31,
        W301A
        + [
            cooling_emission("fan_coil_or_rac_on_ceiling", room_automation="standalone", control="standalone_per_room", room_control="room", why="ISSO 54 v2.0 EP-W301b p. 31: ventilatorconvector aan plafond, automatisch per ruimte"),
            setp(f"{COOL}/distribution/designTemperature", "t12_to16"),
        ],
    ),
    # EP-W301d: fan coils on the outer wall, 8 W in total, network control with
    # override and adaptive control, dynamic balancing with dynamic groups,
    # design 12/18 (p. 31). 10.18 uses 10 W per fan coil: one fan coil.
    (
        "EPW301d",
        31,
        W301A
        + [
            cooling_emission(
                "fan_coil_or_rac_on_outer_wall",
                room_automation="network_with_override_and_adaptive",
                control="standalone_per_room",
                balancing="dynamic",
                row2023="dynamic_or_direct_expansion",
                room_control="room",
                fans=1,
                why="ISSO 54 v2.0 EP-W301d p. 31: ventilatorconvector aan buitenmuur, 8 W totaal (10.18: 1 convector), automatisch per ruimte met handmatig overrulen en adaptief, dynamisch gebalanceerd met dynamische groepen",
            ),
            setp(f"{COOL}/distribution/designTemperature", "t12_to18"),
        ],
    ),
    # EP-W302a-d: cooling distribution (p. 32).
    ("EPW302a", 32, W301A + [remove(f"{COOL}/distribution/unconditionedPipeLengthM")]),
    ("EPW302b", 32, W301A + [setp(f"{COOL}/distribution/unconditionedPipeLengthM", 20.0), setp(f"{COOL}/distribution/fittingsInsulated", False)]),
    (
        "EPW302c",
        32,
        W301A
        + [
            remove(f"{COOL}/distribution/unconditionedPipeLengthM"),
            setp(f"{COOL}/distribution/pipe", {"kind": "uninsulated"}),
            setp(f"{COOL}/distribution/fittingsInsulated", False),
        ],
    ),
    (
        "EPW302d",
        32,
        W301A
        + [
            remove(f"{COOL}/distribution/unconditionedPipeLengthM"),
            setp(f"{COOL}/distribution/pump/labelPowerKw", 0.05),
            setp(f"{COOL}/distribution/pump/energyEfficiencyIndex", 0.3),
        ],
    ),
    # EP-W303a/b/e: cooling generators (p. 32-33; tables 10.29/10.30).
    (
        "EPW303a",
        32,
        W301A
        + [
            active("absorption_table10_30", "ISSO 54 v2.0 EP-W303a p. 32: gasgestookte absorptiekoelmachine"),
            cooling_generators(chiller({"kind": "gas_absorption"}, why="ISSO 54 v2.0 EP-W303a p. 32: met gas aangedreven absorptiekoelmachine")),
        ],
    ),
    (
        "EPW303b",
        33,
        W301A
        + [
            active("absorption_table10_30", "ISSO 54 v2.0 EP-W303b p. 33: absorptiekoelmachine op externe warmtelevering"),
            cooling_generators(chiller({"kind": "absorption_external_heat"}, why="ISSO 54 v2.0 EP-W303b p. 33: absorptie op externe warmtelevering")),
        ],
    ),
    # EP-W303e: EER 4,2 measured per NEN-EN 14825 without part-load points;
    # entered as a declared efficiency (§10.1) in place of table 10.29.
    (
        "EPW303e",
        33,
        W301A
        + [
            cooling_generators(
                chiller(
                    {"kind": "compression", "declared": {"value": 4.2, "sourceReference": "ISSO 54 v2.0 EP-W303e p. 33: EER 4,2 gemeten volgens NEN-EN 14825"}},
                    why="ISSO 54 v2.0 EP-W303e p. 33: compressiekoelmachine",
                )
            )
        ],
    ),
]

UCOOL = f"{NTA}/cooling"
UTILITY_CASES += [
    # EP-U401a: wall cooling (radiant, 17/21 per table 10.8), automatic per
    # room with manual override (p. 48).
    (
        "EPU401a",
        48,
        [
            cooling_emission(
                "wall_cooling",
                room_automation="standalone_with_manual_override",
                control="standalone_per_room",
                room_control="room",
                why="ISSO 54 v2.0 EP-U401a p. 48: wandkoeling, automatisch per ruimte met handmatig overrulen",
            )
        ],
    ),
    # EP-U402a: 80 m of cooling pipe, none in unconditioned spaces, L_max 40 m
    # (p. 48).
    (
        "EPU402a",
        48,
        [
            setp(f"{UCOOL}/distribution/pipeLengthM", 80.0),
            setp(f"{UCOOL}/distribution/unconditionedPipeLengthM", 0.0),
            setp(f"{UCOOL}/distribution/pump/maxPipeLengthM", 40.0),
        ],
    ),
    # EP-U403a: gas-engine compression, 30 kW mechanical (p. 48; table 10.29
    # with table 9.31, built after 2006: construction year 2021).
    (
        "EPU403a",
        48,
        [
            cooling_generators(
                chiller(
                    {"kind": "gas_engine_compression", "gasEngine": {"powerKw": 30.0, "builtAfter2006": True}},
                    why="ISSO 54 v2.0 EP-U403a p. 48: gasmotoraangedreven compressiekoelmachine, mechanisch vermogen 30 kW",
                )
            )
        ],
    ),
    # EP-U301a: a room height of 10 m (p. 46), floor heating with minimum
    # insulation within 10 cm: table 9.8/9.10 row (2022 9.3.3.7).
    (
        "EPU301a",
        46,
        [
            setp(
                f"{NTA}/emission/edition2023/kind",
                {"type": "high_room", "heightM": 10.0, "emitter": "floor_minimal_insulation_up_to10_cm", "control": "controlled"},
            )
        ],
    ),
    # EP-U302a: 80 m heating pipe, none in unheated spaces (p. 46). L_max 40 m
    # only matters for a calculated pump; the boiler pump is in 9.85.
    (
        "EPU302a",
        46,
        [
            setp(f"{NTA}/distributionSystem/actualPipeLengthM", 80.0),
            setp(f"{NTA}/distributionSystem/unheatedPipeLengthM", 0.0),
        ],
    ),
    # EP-U302b: construction year 1950, heating and cooling pipes insulated in
    # 2000 (p. 46).
    (
        "EPU302b",
        46,
        [
            setp(f"{NTA}/constructionYear", 1950),
            setp(f"{VENT}/constructionYear", 1950),
            setp(f"{VENT}/fans/manufactureYear", 1950),
            setp(f"{BOILER}/installationYear", 1950),
            setp(f"{NTA}/distributionSystem/pipeTransmittance/insulation", {"state": "insulated", "period": "from1995"}),
            setp(f"{UCOOL}/distribution/pipe", {"kind": "insulated_from1995"}),
        ],
    ),
]


# --- EP-U2xx: ventilation of the office (p. 45-46) ---
def installed(dm3_per_s, why):
    return setp(f"{VENT}/installedCapacity", {"totalDm3PerS": dm3_per_s, "sourceReference": why})


def flow_reduction(recirculation, why=None):
    body = {"collective": True, "recirculationPercent": recirculation}
    if why is not None:
        body["evidenceReference"] = why
    return setp(f"{VENT}/flowReduction", body)


def declared_fans(power, why):
    # Recirculation lowers the outdoor air, not the fan speed: table 11.22
    # "other" (f_regfan 1).
    return setp(
        f"{VENT}/fans",
        {"method": "declared", "fans": [{"id": "ventilatoren", "power": power}], "control": {"method": "flow_control", "control": "other"}, "sourceReference": why},
    )


# --- EP-U5xx: hot water of the office (p. 49) ---
# EP-U502a is the base of the further hot-water deeltests (p. 49): care with
# beds, a circulation loop (forfait length, none in unheated spaces, 25 mm
# insulation, 35/32 mm, fittings insulated, pump power unknown and
# uncontrolled), an indirectly fired 200 l vessel on the HR107 combi outside
# the thermal envelope, label C, straight parts of at most 4 connections
# insulated (f_sto;dis;ls 3, 2022 p. 550). The 2 delivery sets of the test
# belong to external heat (§13.4.2) and are not entered with a boiler.
U502A = function("d") + [
    setp(
        f"{HW}/circulation",
        {
            "outerDiameterMm": 35.0,
            "insulation": "mm25",
            "fittingsInsulated": True,
            "unheatedLengthM": 0.0,
            "floorCount": 2,
            "pump": {"control": "uncontrolled_or_unknown"},
            "sourceReference": "ISSO 54 v2.0 EP-U502a p. 49: circulatie, forfaitaire lengte, 35/32 mm, 25 mm isolatie, kleppen en beugels geisoleerd, pomp onbekend zonder regeling",
        },
    ),
    setp(f"{HW}/storage", [vessel(200.0, {"method": "label", "label": "c"}, 3, False)]),
    indirect_hr107(False),
    setp(f"{BOILER}/location", "outside_thermal_boundary"),
]

UTILITY_CASES += [
    ("EPU201a", 45, [installed(120.0, "ISSO 54 v2.0 EP-U201a p. 45: geinstalleerde ventilatiecapaciteit 120 dm3/s")]),
    (
        "EPU201c",
        45,
        [
            installed(200.0, "ISSO 54 v2.0 EP-U201c p. 45: ontwerpdebiet 200 dm3/s"),
            flow_reduction(30, "ISSO 54 v2.0 EP-U201c p. 45: recirculatie 30 % van de retourlucht"),
        ],
    ),
    (
        "EPU203a",
        46,
        [
            installed(200.0, "ISSO 54 v2.0 EP-U203a p. 46: ontwerpdebiet 200 dm3/s"),
            flow_reduction(20),
            declared_fans({"method": "nominal", "nominalPowerW": 100.0}, "ISSO 54 v2.0 EP-U203a p. 46: nominaal vermogen 100 W"),
        ],
    ),
    # EP-U203b: 80 W motor, measured U.I.e = 220 V x 0,6 A x 1 = 132 W (11.136).
    (
        "EPU203b",
        46,
        [
            installed(200.0, "ISSO 54 v2.0 EP-U203b p. 46: ontwerpdebiet 200 dm3/s"),
            flow_reduction(30, "ISSO 54 v2.0 EP-U203b p. 46: recirculatie 30 %"),
            declared_fans({"method": "motor", "motorPowerW": 80.0, "electricalInputW": 132.0}, "ISSO 54 v2.0 EP-U203b p. 46: 80 W, 220 V, 0,6 A, gelijkstroom e = 1"),
        ],
    ),
    (
        "EPU203c",
        46,
        [
            unit("c1", "luka_a_b_c"),
            NO_PASSIVE,
            declared_fans({"method": "nominal", "nominalPowerW": 100.0}, "ISSO 54 v2.0 EP-U203c p. 46: systeem C1, nominaal vermogen 100 W"),
        ],
    ),
    # EP-U501a: some draw-off points further than 3 m (p. 49; table 13.3).
    (
        "EPU501a",
        49,
        [setp(f"{HW}/emission", {"method": "utility", "meanLengthM": 5.0, "sourceReference": "ISSO 54 v2.0 EP-U501a p. 49: sommige tappunten verder dan 3 m (tabel 13.3, rij > 3 m)"})],
    ),
    ("EPU502a", 49, U502A),
    # EP-U502b: 80 m circulation, L_max 40 m (p. 49); L_max has no input for
    # the circulation pump (13.39).
    ("EPU502b", 49, U502A + [setp(f"{HW}/circulation/lengthM", 80.0)]),
    # EP-U503a: 10 showers, 2 vertical and 4 horizontal units, collective
    # arrangement (p. 49; table 13.8 "gedeelde units"; p. 564 assignment unknown).
    (
        "EPU503a",
        49,
        U502A
        + [
            setp(
                f"{HW}/showerHeatRecovery",
                {
                    "showers": [{"unit": "vertical"}] * 2 + [{"unit": "horizontal"}] * 4 + [{"unit": "none"}] * 4,
                    "assignmentUnknown": True,
                    "connection": "shared_units",
                    "sourceReference": "ISSO 54 v2.0 EP-U503a p. 49: 10 douches, 2 verticale en 4 horizontale douche-WTW, collectieve opstelling",
                },
            )
        ],
    ),
    # EP-U504a: HR100 indirect, inside the heated zone, also for heating, a
    # 2000 l vessel with insulated T-pieces (f_sto;dis;ls 2, p. 550) (p. 49).
    # The vessel keeps label C (the test does not change it).
    (
        "EPU504a",
        49,
        U502A
        + [
            setp(f"{HW}/storage", [vessel(2000.0, {"method": "label", "label": "c"}, 2, True)]),
            hw_generator({"kind": "indirect_boiler", "boiler": "hr100_or104", "oil": False, "insideBoundary": True, "alsoSpaceHeating": True}),
            setp(f"{BOILER}/location", "inside_thermal_boundary"),
        ],
    ),
    # EP-U701a-c: humidification (p. 51; chapter 12).
    (
        "EPU701a",
        51,
        [setp(f"{NTA}/humidifiers", [{"zoneId": "epw001-zone", "humidification": {"humidifier": {"kind": "atomising"}, "rotaryWheel": False, "equipmentReference": "ISSO 54 v2.0 EP-U701a p. 51: verneveling"}}])],
    ),
    (
        "EPU701b",
        51,
        function("d")
        + [
            setp(
                f"{NTA}/humidifiers",
                [{"zoneId": "epw001-zone", "humidification": {"humidifier": {"kind": "steam", "carrier": "electricity"}, "rotaryWheel": False, "equipmentReference": "ISSO 54 v2.0 EP-U701b p. 51: elektrische stoombevochtiger"}}],
            )
        ],
    ),
    (
        "EPU701c",
        51,
        function("g")
        + [
            setp(
                f"{NTA}/humidifiers",
                [{"zoneId": "epw001-zone", "humidification": {"humidifier": {"kind": "steam", "carrier": "gas_or_oil"}, "rotaryWheel": False, "equipmentReference": "ISSO 54 v2.0 EP-U701c p. 51: stoombevochtiger met centrale gasgestookte opwekker"}}],
            )
        ],
    ),
]


# --- EP-W201b-d: radiators and convectors (p. 23; 2022 table 9.3, p. 273) ---
# The over-temperature row follows the mean water temperature minus 20 °C:
# 55/47 is 31 K (row 30 K), 90/70 60 K, 50/42 26 K (row 30 K). Booster fans:
# table 9.11 gives 10 W per fan convector; the 40 W of the test needs the
# NEN-EN 16430 route, which NTA 8800:2022 does not have (switch point 56).
def radiators(test_id, system, over_temperature, position, room_automation, certified=False, fans=None, why=""):
    body = {
        "system": system,
        "balancing": "none_or_unknown",
        "control": "individual_room_thermostats",
        "sourceReference": why,
        "edition2023": {
            "kind": {"type": "radiators", "control": "room", "overTemperature": over_temperature, "position": position},
            "certifiedControl": certified,
            "roomAutomation": room_automation,
            "pipeSystem": "two_pipe",
            "balancing": "none_or_unknown",
        },
    }
    if fans is not None:
        body["fans"] = {"kind": "fan_convector", "count": fans, "sourceReference": f"ISSO 54 v2.0 {test_id}: {fans} ventilatoren (40 W in de test; tabel 9.11 forfaitair)"}
    return setp(f"{NTA}/emission", body)


def boiler_temperature(supply, ret):
    return [
        setp(f"{BOILER}/averageDesignEmissionTemperatureC", (supply + ret) / 2),
        setp(f"{BOILER}/temperatureAndCircuitReference", f"ontwerp {supply}/{ret} volgens de deeltest"),
    ]


CASES += [
    (
        "EPW201b",
        23,
        [
            radiators("EP-W201b", "radiators_or_convectors", "two_pipe30_k", "outer_wall", "network_with_override_and_adaptive", why="ISSO 54 v2.0 EP-W201b p. 23: radiatoren tegen buitenwand, 55/47, netwerk met handmatig overrulen en adaptief, geen waterzijdige inregeling"),
            temperature_class("55_47"),
        ]
        + boiler_temperature(55, 47),
    ),
    (
        "EPW201c",
        23,
        [
            radiators("EP-W201c", "fan_assisted_radiators_or_convectors", "fan_assisted", "unknown", "individual_per_room", fans=6, why="ISSO 54 v2.0 EP-W201c p. 23: convectoren met boosterventilatoren, 90/70, individueel per ruimte, geen waterzijdige inregeling"),
            temperature_class("90_70"),
        ]
        + boiler_temperature(90, 70),
    ),
    (
        "EPW201d",
        23,
        [
            radiators("EP-W201d", "fan_assisted_radiators_or_convectors", "fan_assisted", "unknown", "individual_per_room", certified=True, fans=6, why="ISSO 54 v2.0 EP-W201d p. 23: convectoren met boosterventilatoren, 50/42, individueel per ruimte, regeling volgens NEN-EN 215 en NEN-EN 15500, geen waterzijdige inregeling"),
            temperature_class("50_42"),
        ]
        + boiler_temperature(50, 42),
    ),
    # EP-W202a-d: heating distribution (p. 24; 9.36, table 9.16).
    ("EPW202a", 24, [remove(f"{NTA}/distributionSystem/unheatedPipeLengthM")]),
    ("EPW202b", 24, [setp(f"{NTA}/distributionSystem/unheatedPipeLengthM", 20.0), setp(f"{NTA}/distributionSystem/valvesInsulated", False)]),
    (
        "EPW202c",
        24,
        [
            remove(f"{NTA}/distributionSystem/unheatedPipeLengthM"),
            setp(f"{NTA}/distributionSystem/pipeTransmittance/insulation", {"state": "uninsulated"}),
            setp(f"{NTA}/distributionSystem/valvesInsulated", False),
        ],
    ),
    (
        "EPW202d",
        24,
        [
            remove(f"{NTA}/distributionSystem/unheatedPipeLengthM"),
            setp(
                f"{NTA}/distributionSystem/pump",
                {"method": "calculated", "heatMeterPresent": False, "electricPowerKw": 0.05, "energyEfficiencyIndex": 0.3, "sourceReference": "ISSO 54 v2.0 EP-W202d p. 24: aanvullende distributiepomp 50 W, EEI 0,3"},
            ),
        ],
    ),
]


# --- EP-W101c/r/s/w, EP-W102, EP-W103: further ventilation (p. 19-22) ---
D4_RECOVERY = {"kind": "insulated"}


def declared_recovery(value, standard, why):
    return {"method": "declared", "value": value, "standard": standard, "sourceReference": why}


# EP-W103a/b: four windows of the south facade, 2,5 m2 net each (8,33 m2 x
# 0,30 mesh), opening angle 90 degrees, centres 1,2 m and 3,9 m above ground,
# opening height 2 m (p. 22; 11.2.3.3, 11.71b).
def summer_night(operation, why):
    openings = [
        {
            "id": f"raam-{k + 1}",
            "area": {"method": "opening_angle", "maxNetAreaM2": 2.5, "maxAngleDeg": 90.0},
            "centreHeightM": 1.2 if k < 2 else 3.9,
            "openingHeightM": 2.0,
            "azimuthDeg": 180.0,
            "tiltDeg": 90.0,
        }
        for k in range(4)
    ]
    return setp(f"{VENT}/ventilativeCooling", {"openings": openings, "operation": operation, "conditionsEvidence": why})


CASES += [
    # EP-W101c: B1, Luka A, construction year 1985 acting on infiltration, the
    # fans and the insulation period of the heating pipes (not the boiler),
    # AC fans (p. 19).
    (
        "EPW101c",
        19,
        [
            unit("b1", "luka_a_b_c"),
            NO_PASSIVE,
            setp(f"{NTA}/constructionYear", 1985),
            setp(f"{VENT}/constructionYear", 1985),
            setp(f"{VENT}/fans", {"method": "forfait", "current": "ac", "manufactureYear": 1985}),
            setp(f"{NTA}/distributionSystem/pipeTransmittance/insulation", {"state": "insulated", "period": "from1980_to1995"}),
        ],
    ),
    # EP-W101r: D4a, declared 85 % with dissipation included (NEN-EN 13141-7),
    # insulated duct 1 m, central, bypass 70 % (p. 21).
    (
        "EPW101r",
        21,
        [
            unit(
                "d4a",
                "luka_a_b_c",
                recovery(
                    declared_recovery(0.85, "en13141_7", "ISSO 54 v2.0 EP-W101r p. 21: WTW-verklaring 85 %, dissipatie verdisconteerd"),
                    {"kind": "partial", "fraction": 0.7},
                    length=1.0,
                ),
            )
        ],
    ),
    # EP-W101s: D4b, declared 85 % per NEN-EN 13142 without dissipation,
    # uninsulated duct 1 m, central, bypass 100 % (p. 21).
    (
        "EPW101s",
        21,
        [
            unit(
                "d4b",
                "luka_a_b_c",
                recovery(
                    declared_recovery(0.85, "en13142", "ISSO 54 v2.0 EP-W101s p. 21: WTW-verklaring 85 % volgens EN 13142, dissipatie niet verdisconteerd"),
                    {"kind": "full"},
                    length=1.0,
                    insulation={"kind": "uninsulated"},
                ),
            )
        ],
    ),
    # EP-W101w: E1, decentral D5b on 40 m2 and C1 on 30 m2 of residence area
    # (p. 21; 11.29-11.45).
    (
        "EPW101w",
        21,
        [
            setp(
                f"{VENT}/system",
                {
                    "kind": "combined",
                    "decentralAreaM2": 40.0,
                    "totalResidenceAreaM2": 70.0,
                    "decentral": {
                        "variant": "d5b",
                        "ducts": "luka_a_b_c",
                        "equipmentReference": "ISSO 54 v2.0 EP-W101w p. 21: decentrale WTW (D5b)",
                        "heatRecovery": recovery(TABLE_PLASTIC, {"kind": "none"}, layout="decentral", length=1.0),
                    },
                    "other": {"variant": "c1", "ducts": "luka_a_b_c", "equipmentReference": "ISSO 54 v2.0 EP-W101w p. 21: mechanische afvoer (C1)"},
                },
            ),
            NO_PASSIVE,
        ],
    ),
    # EP-W102a/b: C1 with preheating of the natural supply (p. 21-22; 11.123,
    # 11.124).
    (
        "EPW102a",
        21,
        [
            unit("c1", "luka_a_b_c"),
            NO_PASSIVE,
            setp(f"{VENT}/grillePreheating", {"control": {"method": "fallback"}, "sourceReference": "ISSO 54 v2.0 EP-W102a p. 21: voorverwarming natuurlijke toevoer, geen nadere gegevens"}),
        ],
    ),
    (
        "EPW102b",
        22,
        [
            unit("c1", "luka_a_b_c"),
            NO_PASSIVE,
            installed(60.0, "ISSO 54 v2.0 EP-W102b p. 22: q_v;inst 60 dm3/s"),
            setp(f"{VENT}/installedCapacity/naturalSupplyDm3PerS", 60.0),
            setp(
                f"{VENT}/grillePreheating",
                {
                    "control": {"method": "specified", "maxPowerWPerDm3PerS": 10.0, "maxTemperatureRiseK": 5.0, "switchOnBelowC": 18.0, "maxSupplyTemperatureC": 15.0},
                    "preheatedDesignFlowM3PerH": 108.0,
                    "sourceReference": "ISSO 54 v2.0 EP-W102b p. 22: 50 % voorverwarmd (30 dm3/s), sprong 5 K, 10 W/(dm3/s), aan onder 18 C, inblaas max 15 C",
                },
            ),
        ],
    ),
    ("EPW103a", 22, [summer_night("manual", "ISSO 54 v2.0 EP-W103a p. 22: zomernachtventilatie, enkelzijdig, handbediend; voorwaarden 11.2.3.3 als vervuld aangenomen")]),
    ("EPW103b", 22, [summer_night("automatic", "ISSO 54 v2.0 EP-W103b p. 22: automatische bediening zonder temperatuurmeting; voorwaarden 11.2.3.3 als vervuld aangenomen")]),
]


# --- EP-W203g, EP-W204b/f/g/i: further heating generators (p. 25-28) ---
def external_heat(test_id, measured_only):
    """Heat supply with an annex P quality declaration (§5.8.0): f_P 0,5,
    f_Pren 0,3, K_CO2 0,1 kg/kWh. The distribution pump is outside 9.85."""
    basis = "uitsluitend op basis van metingen" if measured_only else "op basis van berekeningen"
    return [
        setp(
            GEN,
            {
                "kind": "external_heat",
                "supplierReference": f"ISSO 54 v2.0 {test_id}: warmtelevering met kwaliteitsverklaring",
                "qualityDeclarationPresent": True,
                "auxiliary": {"electricallyConnectedDevices": 1, "sourceReference": f"ISSO 54 v2.0 {test_id}: afleverset met distributiepomp"},
            },
        ),
        setp(
            f"{NTA}/externalSupply",
            {
                "heating": {
                    "method": "declared",
                    "primaryFactor": 0.5,
                    "renewableFactor": 0.3,
                    "co2KgPerKwh": 0.1,
                    "declarationReference": f"ISSO 54 v2.0 {test_id}: kwaliteitsverklaring, factoren {basis}",
                    "measuredOnly": measured_only,
                }
            },
        ),
        CALCULATED_PUMP,
    ]


def micro_chp(test_id, hre, low_temperature, why):
    """A micro-CHP below 2 kW electric (table 9.31; 2 kW taken as the class
    value, every power up to 2 kW is the same row), installed 2021."""
    return [
        setp(
            GEN,
            {
                "kind": "chp",
                "chp": {"powerKw": 1.0, "builtAfter2006": True, "hreDeclared": hre, "lowTemperature": low_temperature},
                "auxiliary": {"electricallyConnectedDevices": 1, "nominalPowerKw": 6.0, "sourceReference": f"ISSO 54 v2.0 {test_id}: micro-WKK; thermisch vermogen niet gegeven, 6 kW aangenomen (fictief) voor 9.91"},
                "equipmentReference": why,
            },
        ),
        CALCULATED_PUMP,
    ]


CASES += [
    ("EPW204b", 27, external_heat("EP-W204b", True)),
    ("EPW204i", 28, external_heat("EP-W204i", False)),
    (
        "EPW204f",
        28,
        micro_chp("EP-W204f", True, False, "ISSO 54 v2.0 EP-W204f p. 28: micro-WKK < 2 kW met HRe-label, HT")
        + [temperature_class("75_65")],
    ),
    (
        "EPW204g",
        28,
        micro_chp("EP-W204g", False, True, "ISSO 54 v2.0 EP-W204g p. 28: micro-WKK < 2 kW zonder HRe-label, LT")
        + [temperature_class("45_40")],
    ),
    # EP-W203g: air-to-air heat pump on outdoor air with air heating (p. 25):
    # no water-borne distribution.
    (
        "EPW203g",
        25,
        [
            heat_pump("EP-W203g", "outdoor_air", None, sink="indoor_air"),
            renewable("EP-W203g", True),
            setp(
                f"{NTA}/emission",
                {
                    "system": "air_heating",
                    "balancing": "not_applicable",
                    "control": "individual_room_thermostats",
                    "sourceReference": "ISSO 54 v2.0 EP-W203g p. 25: luchtverwarming, regeling als EP-W001",
                    "edition2023": {"kind": {"type": "dwelling_air", "control": "room"}, "roomAutomation": "individual_per_room", "pipeSystem": "not_hydronic", "balancing": "none_or_unknown"},
                },
            ),
            remove(f"{NTA}/distributionSystem"),
            setp(
                f"{NTA}/distribution",
                {"method": "declared", "monthlyLossKwh": [0.0] * 12, "sourceReference": "luchtverwarming zonder watergedragen distributie: geen distributieverlies"},
            ),
        ],
    ),
]


# --- EP-W406d/e/f/m/n/u/v: further hot-water generators (p. 37-40) ---
# Exhaust-air heat pumps: system C1, no maximum use of the ventilation
# capacity, Luka C, forfait DC fans of 2021 (the reference fans), 2 kW.
EXHAUST_C1 = [unit("c1", "luka_a_b_c"), NO_PASSIVE]
# 13.144a f_combi: 1 in October-March when the heat pump also heats.
HEATING_MONTHS = [1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]


def exhaust_hot_water(also_heating):
    body = {"kind": "heat_pump", "exhaustAirSource": True}
    patch = [hw_generator(body), setp(f"{HW}/nominalPowerKw", 2.0)]
    exhaust = {"ventilationSuitable": True}
    if also_heating:
        exhaust["heatingTimeFraction"] = HEATING_MONTHS
    patch.append(setp(f"{HW}/exhaustAir", exhaust))
    return patch


def bathroom_emission(length):
    return {"method": "residential", "served": "bathroom_only", "bathroomLengthM": length, "sourceReference": f"uittapleiding badkamer {length} m volgens de deeltest"}


CASES += [
    ("EPW406d", 37, EXHAUST_C1 + exhaust_hot_water(False)),
    # EP-W406e: solid-biomass combi appliance with a vessel meeting annex R, at
    # least 10 mm insulation, outside the heated zone; heating by an automatic
    # biomass boiler outside the zone, 5 kW (p. 37; table 13.22).
    (
        "EPW406e",
        37,
        [
            hw_generator({"kind": "biomass_combi", "insulation": "at_least10_mm", "insideBoundary": False}),
            biomass("EP-W406e", "central_boiler", "outside_thermal_boundary", True, automatic=True, power=5.0),
            CALCULATED_PUMP,
        ],
    ),
    # EP-W406f: external heat for hot water, one delivery set (p. 37).
    (
        "EPW406f",
        37,
        [
            hw_generator({"kind": "external_heat"}),
            setp(f"{HW}/deliverySets", {"count": 1, "sourceReference": "ISSO 54 v2.0 EP-W406f p. 37: 1 afleverset"}),
        ],
    ),
    # EP-W406m: kitchen electric boiler 20 l label D, uninsulated hot pipe
    # (f_sto;dis;ls 2, 2022 p. 550), 0-2 m (2 m), > 10 mm; bathroom HR combi
    # CW4 at 5 m (p. 38; 13.19a).
    (
        "EPW406m",
        38,
        [
            setp(f"{HW}/emission", bathroom_emission(5.0)),
            setp(f"{HW}/connectedTaps", {"bathrooms": 1, "kitchens": 0}),
            setp(
                f"{NTA}/additionalHotWaterSystems",
                [
                    {
                        "need": {"method": "residential", "dwellingCount": 1, "sourceReference": "een woning"},
                        "emission": {"method": "residential", "served": "kitchen_only", "kitchenLengthM": 2.0, "kitchenPipeDiameter": "other", "sourceReference": "ISSO 54 v2.0 EP-W406m p. 38: keuken 0-2 m, > 10 mm"},
                        "storage": [vessel(20.0, {"method": "label", "label": "d"}, 2, True)],
                        "generator": {"kind": "electric_boiler"},
                        "connectedTaps": {"bathrooms": 0, "kitchens": 1},
                        "equipmentReference": "ISSO 54 v2.0 EP-W406m p. 38: elektroboiler 20 l, label D, warmwaterleiding niet geisoleerd",
                    }
                ],
            ),
        ],
    ),
    # EP-W406n: a second installation for a second bathroom: a closed gas
    # water heater with Gaskeur and CW4 at 3 m (p. 38). Installation 1 is
    # the reference (kitchen and bathroom 1).
    (
        "EPW406n",
        38,
        [
            setp(f"{HW}/connectedTaps", {"bathrooms": 1, "kitchens": 1}),
            setp(
                f"{NTA}/additionalHotWaterSystems",
                [
                    {
                        "need": {"method": "residential", "dwellingCount": 1, "sourceReference": "een woning"},
                        "emission": bathroom_emission(3.0),
                        "generator": {"kind": "gas_appliance", "appliance": "water_heater_gaskeur_cw", "measuredClass": "class4"},
                        "connectedTaps": {"bathrooms": 1, "kitchens": 0},
                        "equipmentReference": "ISSO 54 v2.0 EP-W406n p. 38: gesloten gastoestel met Gaskeur, CW4",
                    }
                ],
            ),
        ],
    ),
    # EP-W406u: as EP-W406d with construction year 1900 (infiltration); the
    # heating pipes stay insulated after 1995, the fans stay of 2021 (p. 39).
    (
        "EPW406u",
        39,
        EXHAUST_C1
        + exhaust_hot_water(False)
        + [setp(f"{NTA}/constructionYear", 1900), setp(f"{VENT}/constructionYear", 1900)],
    ),
    # EP-W406v: exhaust-air heat pump for heating (not table 9.28) and hot
    # water, 2 kW (p. 39). The heating supply stays 45 C (45/40).
    (
        "EPW406v",
        39,
        EXHAUST_C1
        + exhaust_hot_water(True)
        + [heat_pump("EP-W406v", "exhaust_air", 45.0), renewable("EP-W406v", True, exhaust=True)],
    ),
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
