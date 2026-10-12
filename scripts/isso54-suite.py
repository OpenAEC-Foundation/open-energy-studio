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
        24,
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
#   Open interpretation (review 19, R2), to confirm against ISSO 54 v5.0: the
#   "gekleurd rolluik" could also be movable shading. That alternative moves
#   BENG 2 by about +12 % and TOjuli from 8,91 to 2,28. To try it, add:
#     setp(f"{NTA}/windowSolar/movableShading", {"device": {"kind": "external_roller_shutter", "colour": "other"}, "control": "manual_residential", "sourceReference": "ISSO 54 v2.0 EP-W003c p. 7: gekleurd rolluik"})
CASES += [
    ("EPW003a", 7, windows("uValue", 2.7)),
    ("EPW003c", 7, windows("uValue", 1.6)),
]


# --- EP-W015a/b: vertical pipes through the thermal envelope (p. 18; 7.3.3) ---
# EP-W015a: no penetrations, no pipe. EP-W015b: one insulated pipe running
# through both storeys (table 7.1 footnote a).
CASES += [
    ("EPW015a", 19, [setp(f"{NTA}/verticalPipes", [])]),
    (
        "EPW015b",
        19,
        [setp(f"{NTA}/verticalPipes", [{"id": "leiding-1", "storeys": 2, "insulated": True, "sourceReference": "ISSO 54 v2.0 EP-W015b p. 19: 1 geisoleerde verticale leiding per bouwlaag"}])],
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
    ("EPW016a", 19, [door(3.4, 0.0)]),
    ("EPW016b", 19, [door(3.1, round(0.7 * 0.5 / 0.75, 4))]),
    ("EPW016c", 19, [door(2.9, round(0.7 * 0.8 / 0.75, 4))]),
]


# --- EP-W012: movable shading with the building turned (p. 15) ---
# Table 7.5/7.6 devices, operated from inside: manual residential control.
def residential_screen(device):
    return setp(
        f"{NTA}/windowSolar/movableShading",
        {"device": device, "control": "manual_residential", "sourceReference": "zonwering volgens de deeltest, van binnenuit bediend"},
    )


CASES += [
    ("EPW012a", 16, [residential_screen({"kind": "external_screen", "colour": "dark"})]),
    ("EPW012b", 16, rotate("SW") + [residential_screen({"kind": "external_venetian_blind", "colour": "white"})]),
    ("EPW012c", 16, rotate("W") + [residential_screen({"kind": "external_screen", "colour": "other"})]),
    ("EPW012d", 16, rotate("SE") + [residential_screen({"kind": "internal_metallised_fabric"})]),
    ("EPW012e", 16, rotate("E") + [residential_screen({"kind": "drop_arm_awning"})]),
    ("EPW012f", 16, [residential_screen({"kind": "folding_arm_awning"})]),
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


def declared_sector(area, factor, test_id, note):
    """A daylight sector with F_D;dayl from NEN-EN 15193-1 annex F, the
    alternative below table 14.7 (2022 p. 648): the kernel rounds it down to
    two significant digits."""
    return {
        "kind": "declared",
        "areaM2": area,
        "daylightDependencyFactor": factor,
        "sourceReference": f"ISSO 54 v2.0 {test_id}: {note}",
    }


def daylight_zone(zone_id, area, watts, sector, test_id):
    """EP-U602g/h: lamp power with unknown ballast and unknown constant-
    illuminance compensation (none), auto on/auto off as EP-U001, and one
    daylight sector."""
    return {
        "id": zone_id,
        "areaM2": area,
        "power": {
            "method": "installed",
            "luminaires": [{"count": 1, "power": {"method": "lamps", "lampPowerW": watts, "lampCount": 1, "technology": "unknown_or_other"}}],
            "sourceReference": f"ISSO 54 v2.0 {test_id}: lampvermogen {watts} W, voorschakeling onbekend, als een groep",
        },
        "parasitic": {"method": "forfait"},
        "occupancy": {"control": "auto_on_auto_off", "centralOnControl": False, "largeOfficeGroup": False},
        "daylight": {"method": "sectors", "sectors": [sector], "sourceReference": f"ISSO 54 v2.0 {test_id} p. 50"},
        "extractedLuminaires": False,
    }


# EP-U602g (p. 50): one zone of 96 m², a daylight sector of 48 m² with
# F_D;S 0,3 and automatic dimming: F_D;dayl = 1 - 0,73 x 0,3 = 0,781 (the
# test's F_D 0,8905 = (48 x 0,781 + 48)/96). The norm has no input for a
# given F_D;S, and 0,3 is out of reach of 14.37 for vertical windows (at
# least 0,65 x 0,12 + 0,25 = 0,328), so the test's F_D;dayl enters through
# the 15193-1 alternative, which rounds it down to 0,78 (F_D 0,89).
U602G = setp(
    f"{NTA}/lighting/0/lightingZones",
    [daylight_zone("kantoor", 96.0, 768.0, declared_sector(48.0, 0.781, "EP-U602g", "F_D;S 0,3, automatisch dimmen, F_D;dayl 0,781"), "EP-U602g")],
)
# EP-U602h (p. 50-51): two equal lighting zones of 48 m² (half the power
# each). The lower one as 602g (F_D 0,8905, so a sector of 24 m²); the upper
# one with rooflights, F_D;S 0,68: F_D;dayl 0,5036, F_D 0,7518, so a sector
# of 24 m² as well. F_D;S 0,68 is the table 14.9 value (2022 p. 650; 14.8 in
# 2025+C1) for 2 % <= D < 4 %, so
# the upper sector goes through 14.31-14.36 and 14.41/14.42 (no rounding):
# a 2 x 4 m rooflight in a corner of an 8 x 6 m room, clear height 2,75 m
# (extra depth 2 m, cut by the walls at 0 m), 3 m² opening, luminaires 2 m
# above the work plane: A_D 4 x 6 = 24 m², room index 1,71, D about 3,2 %.
# Only the lower sector (F_D;S 0,3 is below the 0,328 that 14.37 reaches
# for vertical windows) needs the 15193-1 route.
def u602h_rooflight():
    return {
        "kind": "rooflights",
        "rooflightDepthM": 2.0,
        "rooflightWidthM": 4.0,
        "clearHeightM": 2.75,
        "wallDistancesM": [0.0, 6.0, 0.0, 2.0],
        "openingAreaM2": 3.0,
        "roomLengthM": 8.0,
        "roomWidthM": 6.0,
        "luminaireHeightM": 2.0,
        "control": "automatic_dimming",
    }


U602H = setp(
    f"{NTA}/lighting/0/lightingZones",
    [
        daylight_zone("onder", 48.0, 384.0, declared_sector(24.0, 0.781, "EP-U602h", "onderste zone als 602g, F_D;dayl 0,781"), "EP-U602h"),
        daylight_zone("boven", 48.0, 384.0, u602h_rooflight(), "EP-U602h"),
    ],
)


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
    *[(f"EPU002{letter}", 45, function(letter)) for letter in "abcdefghi"],
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
    # EP-U602a-i: lighting control (p. 50-51; table 14.5).
    ("EPU602a", 50, [switch("manual_or_unknown", central=True)]),
    ("EPU602b", 50, [switch("manual_or_unknown")]),
    ("EPU602c", 50, [switch("manual_with_sweep")]),
    ("EPU602d", 50, function("d") + [switch("auto_on_dimmed")]),
    ("EPU602e", 50, [switch("manual_on_dimmed")]),
    ("EPU602f", 50, [switch("manual_on_auto_off")]),
    # EP-U602g/h: given daylight factors (p. 50-51), see U602G/U602H.
    ("EPU602g", 50, [U602G]),
    ("EPU602h", 50, [U602H]),
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
        47,
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
        47,
        [
            setp(f"{NTA}/distributionSystem/actualPipeLengthM", 80.0),
            setp(f"{NTA}/distributionSystem/unheatedPipeLengthM", 0.0),
        ],
    ),
    # EP-U302b: construction year 1950, heating and cooling pipes insulated in
    # 2000 (p. 46).
    (
        "EPU302b",
        47,
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
            "sourceReference": "ISSO 54 v2.0 EP-U502a p. 48: circulatie, forfaitaire lengte, 35/32 mm, 25 mm isolatie, kleppen en beugels geisoleerd, pomp onbekend zonder regeling",
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
        48,
        [setp(f"{HW}/emission", {"method": "utility", "meanLengthM": 5.0, "sourceReference": "ISSO 54 v2.0 EP-U501a p. 48: sommige tappunten verder dan 3 m (tabel 13.3, rij > 3 m)"})],
    ),
    ("EPU502a", 48, U502A),
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
        24,
        [
            radiators("EP-W201b", "radiators_or_convectors", "two_pipe30_k", "outer_wall", "network_with_override_and_adaptive", why="ISSO 54 v2.0 EP-W201b p. 24: radiatoren tegen buitenwand, 55/47, netwerk met handmatig overrulen en adaptief, geen waterzijdige inregeling"),
            temperature_class("55_47"),
        ]
        + boiler_temperature(55, 47),
    ),
    (
        "EPW201c",
        24,
        [
            radiators("EP-W201c", "fan_assisted_radiators_or_convectors", "fan_assisted", "unknown", "individual_per_room", fans=6, why="ISSO 54 v2.0 EP-W201c p. 24: convectoren met boosterventilatoren, 90/70, individueel per ruimte, geen waterzijdige inregeling"),
            temperature_class("90_70"),
        ]
        + boiler_temperature(90, 70),
    ),
    (
        "EPW201d",
        24,
        [
            radiators("EP-W201d", "fan_assisted_radiators_or_convectors", "fan_assisted", "unknown", "individual_per_room", certified=True, fans=6, why="ISSO 54 v2.0 EP-W201d p. 24: convectoren met boosterventilatoren, 50/42, individueel per ruimte, regeling volgens NEN-EN 215 en NEN-EN 15500, geen waterzijdige inregeling"),
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
        20,
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
        20,
        [
            unit(
                "d4a",
                "luka_a_b_c",
                recovery(
                    declared_recovery(0.85, "en13141_7", "ISSO 54 v2.0 EP-W101r p. 20: WTW-verklaring 85 %, dissipatie verdisconteerd"),
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
    # 11.124). EP-W102b (review 19, R1): the test's q_v;inst 60 dm3/s is the
    # installed capacity of 11.62, which the kernel also uses for the design
    # flow (11.55/11.58), not only for the preheated share of 11.128; the
    # preheated flow itself requires an installed natural capacity. So
    # EP-W102b also differs from EP-W102a through the capacity: against C1
    # about +6,4 kWh/m2 BENG 2 from the capacity and +1,3 from the preheating.
    # The kernel has no route to give 60 dm3/s to the preheating share only;
    # this follows the test text literally and is an open interpretation.
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
        21,
        [
            unit("c1", "luka_a_b_c"),
            NO_PASSIVE,
            installed(60.0, "ISSO 54 v2.0 EP-W102b p. 21: q_v;inst 60 dm3/s"),
            setp(f"{VENT}/installedCapacity/naturalSupplyDm3PerS", 60.0),
            setp(
                f"{VENT}/grillePreheating",
                {
                    "control": {"method": "specified", "maxPowerWPerDm3PerS": 10.0, "maxTemperatureRiseK": 5.0, "switchOnBelowC": 18.0, "maxSupplyTemperatureC": 15.0},
                    "preheatedDesignFlowM3PerH": 108.0,
                    "sourceReference": "ISSO 54 v2.0 EP-W102b p. 21: 50 % voorverwarmd (30 dm3/s), sprong 5 K, 10 W/(dm3/s), aan onder 18 C, inblaas max 15 C",
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
        39,
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
                        "equipmentReference": "ISSO 54 v2.0 EP-W406n p. 39: gesloten gastoestel met Gaskeur, CW4",
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
        40,
        EXHAUST_C1
        + exhaust_hot_water(True)
        + [heat_pump("EP-W406v", "exhaust_air", 45.0), renewable("EP-W406v", True, exhaust=True)],
    ),
]


# --- Apartment variant (EP-W202f, EP-W302f, EP-W402; p. 24, 32, 33-34) ---
# The reference dwelling as a corner apartment on the top floor of a
# building of 4 dwellings and 4 storeys, 10,8 m high: the ground floor is
# absent (a heated apartment below), the rest is unchanged. Infiltration:
# table 11.14 "storey, end, top"; dwelling type apartment building (7.13).
# The surface list is replaced, so the window indices shift: combine only
# with patches that do not address surfaces by index.
BUILDING_AREA = 4 * 96.0
REFERENCE = json.loads((ROOT / "training-data" / "isso54" / "EPW001.json").read_text())
APARTMENT = [
    # The patch runner sets values only: the surface list without the floor.
    setp("/zones/0/surfaces", [s for s in REFERENCE["zones"][0]["surfaces"] if s["id"] != "bg-vloer"]),
    remove(f"{NTA}/groundFloors"),
    setp(f"{NTA}/dwellingType", "apartment_building"),
    setp(f"{VENT}/apartmentBuilding", True),
    setp(f"{VENT}/buildingHeightM", 10.8),
    setp(f"{VENT}/infiltration/buildingType", "storey_end_top"),
]
COLLECTIVE_PUMP = setp(
    f"{NTA}/distributionSystem/pump",
    {"method": "calculated", "heatMeterPresent": False, "sourceReference": "pompvermogen en EEI onbekend, geen warmtemeters"},
)

CASES += [
    # EP-W202f: a collective HR107 of 50 kW outside the envelope; pipes through
    # unheated spaces of unknown length, uninsulated, valves uninsulated
    # (p. 24; 9.91 with P_H;gen 50 kW, f_gebouw with 4 x 96 m2).
    (
        "EPW202f",
        25,
        APARTMENT
        + [
            setp(f"{BOILER}/role", "collective"),
            setp(f"{BOILER}/location", "outside_thermal_boundary"),
            setp(f"{GEN}/auxiliary", {"electricallyConnectedDevices": 1, "nominalPowerKw": 50.0, "sourceReference": "ISSO 54 v2.0 EP-W202f p. 25: collectieve HR107, 50 kW"}),
            setp(f"{NTA}/collectiveConnection", {"connectedUsableAreaM2": BUILDING_AREA, "sourceReference": "ISSO 54 v2.0 EP-W202f p. 25: 4 woningen van 96 m2"}),
            setp(f"{NTA}/distributionSystem/installation", "collective"),
            setp(f"{NTA}/distributionSystem/connectedStoreys", 4),
            remove(f"{NTA}/distributionSystem/unheatedPipeLengthM"),
            setp(f"{NTA}/distributionSystem/pipeTransmittance/insulation", {"state": "uninsulated"}),
            setp(f"{NTA}/distributionSystem/valvesInsulated", False),
            COLLECTIVE_PUMP,
        ],
    ),
    # EP-W302f: a collective electric compression chiller; cooling pipes
    # through unconditioned spaces of unknown length, uninsulated, fittings
    # uninsulated (p. 32).
    (
        "EPW302f",
        32,
        APARTMENT
        + W301A
        + [
            setp(f"{COOL}/collective", {"buildingUsableFloorAreaM2": BUILDING_AREA, "sourceReference": "ISSO 54 v2.0 EP-W302f p. 32: 4 woningen van 96 m2"}),
            remove(f"{COOL}/distribution/unconditionedPipeLengthM"),
            setp(f"{COOL}/distribution/pipe", {"kind": "uninsulated"}),
            setp(f"{COOL}/distribution/fittingsInsulated", False),
            setp(f"{COOL}/distribution/pump/individualDwellingInstallation", False),
        ],
    ),
]

# EP-W402: a collective indirectly fired 1000 l vessel on an HR100 outside the
# envelope, T-pieces insulated (f_sto;dis;ls 2), circulation for hot water
# only 28/25 with 20 mm insulation, none in unheated spaces, fittings
# insulated, pump power unknown and uncontrolled (p. 33-34). The delivery set
# of the test belongs to external heat (§13.4.2) and is not entered.
W402A = APARTMENT + [
    setp(f"{HW}/collective", {"buildingUsableFloorAreaM2": BUILDING_AREA, "sourceReference": "ISSO 54 v2.0 EP-W402 p. 33: 4 woningen van 96 m2"}),
    # The test gives no vessel label (p. 33): label unknown, produced from 2018
    # (the building of EP-W001), which 13.59 takes as label C; review 19.
    setp(f"{HW}/storage", [vessel(1000.0, {"method": "unknown_label", "producedFrom2018": True}, 2, False)]),
    hw_generator({"kind": "indirect_boiler", "boiler": "hr100_or104", "oil": False, "insideBoundary": False, "alsoSpaceHeating": False}),
    setp(
        f"{HW}/circulation",
        {
            "outerDiameterMm": 28.0,
            "insulation": "mm20",
            "fittingsInsulated": True,
            "unheatedLengthM": 0.0,
            "floorCount": 4,
            "connectedDwellings": 4,
            "pump": {"control": "uncontrolled_or_unknown"},
            "sourceReference": "ISSO 54 v2.0 EP-W402a p. 34: circulatie alleen tapwater, 28/25, 20 mm isolatie, kleppen en beugels geisoleerd, pomp onbekend zonder regeling",
        },
    ),
]
CASES += [
    ("EPW402a", 34, W402A),
    ("EPW402b", 34, W402A + [remove(f"{HW}/circulation/unheatedLengthM")]),
    ("EPW402c", 34, W402A + [setp(f"{HW}/circulation/unheatedLengthM", 20.0), setp(f"{HW}/circulation/fittingsInsulated", False)]),
    (
        "EPW402d",
        34,
        W402A
        + [
            remove(f"{HW}/circulation/unheatedLengthM"),
            setp(f"{HW}/circulation/insulation", "none"),
            setp(f"{HW}/circulation/fittingsInsulated", False),
            setp(f"{HW}/circulation/pump/control", "constant_pressure"),
        ],
    ),
    (
        "EPW402e",
        34,
        W402A
        + [
            remove(f"{HW}/circulation/unheatedLengthM"),
            setp(f"{HW}/circulation/pump", {"control": "uncontrolled_or_unknown", "labelPowerKw": 0.04, "energyEfficiencyIndex": 0.3}),
        ],
    ),
    # EP-W402g: unheated length unknown, uninsulated, diameter unknown, fittings
    # uninsulated (p. 34).
    (
        "EPW402g",
        34,
        W402A
        + [
            remove(f"{HW}/circulation/unheatedLengthM"),
            remove(f"{HW}/circulation/outerDiameterMm"),
            setp(f"{HW}/circulation/insulation", "none"),
            setp(f"{HW}/circulation/fittingsInsulated", False),
        ],
    ),
]


# --- EP-W010: an obstruction on the plot parallel to the facade (p. 13-14) ---
# Figure 6: windows 3,0 x 2,0 m with the sill at 0,5 m, storey height 2,7 m,
# so the window centres are at 1,5 m and 4,2 m. h_b;perp of 17.3.2b (2022
# p. 671): (h - centre) / x; an obstruction below the centre is no
# obstruction (minimal, 17.3.2a). raam-zuid-1/2 are taken as the ground-floor
# windows, raam-zuid-3/4 as the upper floor.
WINDOW_CENTRES = {"raam-zuid-1": 1.5, "raam-zuid-2": 1.5, "raam-zuid-3": 4.2, "raam-zuid-4": 4.2}


def plot_obstruction(test_id, distance, height):
    entries = []
    for window_id, centre in WINDOW_CENTRES.items():
        relative = (height - centre) / distance
        obstruction = {"method": "parallel_obstruction", "relativeHeight": round(relative, 4)} if relative > 0 else {"method": "minimal"}
        entries.append(
            {
                "windowId": window_id,
                "obstruction": obstruction,
                "sourceReference": f"ISSO 54 v2.0 {test_id} p. 14: x {distance} m, h {height} m, raammidden {centre} m: h_b = {round(relative, 4)}",
            }
        )
    return setp(f"{NTA}/windowObstructions", entries)


CASES += [
    ("EPW010a", 14, [plot_obstruction("EP-W010a", 0.6, 2.7)]),
    ("EPW010b", 14, [plot_obstruction("EP-W010b", 0.6, 5.4)]),
    ("EPW010c", 14, [plot_obstruction("EP-W010c", 1.2, 2.7)]),
    ("EPW010d", 14, [plot_obstruction("EP-W010d", 1.2, 5.4)]),
    # EP-W010e: full obstruction for all windows (17.3.2e); the cooling
    # conditions of table 17.14 are not stated, so table 17.5 applies.
    ("EPW010e", 14, [setp(f"{NTA}/windowSolar/obstruction", {"method": "full"})]),
]


# --- EP-U303f/g: gas-engine heat pump and CHP for the office (p. 47) ---
UTILITY_CASES += [
    (
        "EPU303f",
        47,
        [
            setp(
                GEN,
                {
                    "kind": "gas_heat_pump",
                    "table": "utility_collective_or_above25_kw",
                    "source": "surface_water",
                    "designSupplyTemperatureC": 35.0,
                    "auxiliary": {"electricallyConnectedDevices": 1, "nominalPowerKw": 50.0, "sourceReference": "ISSO 54 v2.0 EP-U303f p. 47: nominaal thermisch vermogen 50 kW"},
                    "equipmentReference": "ISSO 54 v2.0 EP-U303f p. 47: gasmotorwarmtepomp op oppervlaktewater (tabel 9.29)",
                },
            ),
            temperature_class("35_30"),
            renewable("EP-U303f", True),
            CALCULATED_PUMP,
        ],
    ),
    # EP-U303g: CHP of 20-200 kW electric (table 9.31 row, 100 kW taken), built
    # after 2006, 50 kW thermal, 45/40 (low temperature) (p. 47).
    (
        "EPU303g",
        47,
        [
            setp(
                GEN,
                {
                    "kind": "chp",
                    "chp": {"powerKw": 100.0, "builtAfter2006": True, "lowTemperature": True},
                    "auxiliary": {"electricallyConnectedDevices": 1, "nominalPowerKw": 50.0, "sourceReference": "ISSO 54 v2.0 EP-U303g p. 47: nominaal thermisch vermogen 50 kW"},
                    "equipmentReference": "ISSO 54 v2.0 EP-U303g p. 47: WKK 20-200 kW elektrisch (tabel 9.31; 100 kW als klassewaarde), na 2006, 45/40",
                },
            ),
            temperature_class("45_40"),
            CALCULATED_PUMP,
        ],
    ),
]


# --- EP-W205a/c: collective installations of several boilers (p. 28-29) ---
# Existing building, calculated per apartment (the apartment variant). The
# HR107 is taken as the preferred generator (9.6.1, table 9.1), the VR boilers
# after it; the forfait auxiliary energy is 9.91 per boiler.
def collective_boiler(gen_id, kind, temperature, pilot, why):
    return {
        "kind": "gas_boiler",
        "boiler": {
            "generatorId": gen_id,
            "role": "collective",
            "location": "outside_thermal_boundary",
            "kind": kind,
            "fuel": "natural_gas",
            "averageDesignEmissionTemperatureC": temperature,
            "emissionCircuit": "direct",
            "equipmentReference": why,
            "locationReference": "buiten de thermische schil",
            "temperatureAndCircuitReference": f"gemiddelde ontwerptemperatuur {temperature} C",
            "pilotFlamePresent": pilot,
            "installationYear": 2021,
            "installationYearReference": "ISSO 54 v2.0: installatiejaar als EP-W001",
        },
        "auxiliary": {"electricallyConnectedDevices": 1, "nominalPowerKw": 10.0, "sourceReference": f"{why}, 10 kW"},
    }


def boiler_set(test_id, boilers, temperature, dwellings, cls):
    return APARTMENT + [
        setp(
            GEN,
            {
                "kind": "multiple",
                "generators": [{"preference": k + 1, "nominalPowerKw": 10.0, "generator": collective_boiler(f"ketel-{k + 1}", kind, temperature, pilot, f"ISSO 54 v2.0 {test_id}: {kind}")} for k, (kind, pilot) in enumerate(boilers)],
                "sourceReference": f"ISSO 54 v2.0 {test_id}: collectieve installatie, HR107 voorkeur",
            },
        ),
        temperature_class(cls),
        setp(f"{NTA}/collectiveConnection", {"connectedUsableAreaM2": 96.0 * dwellings, "sourceReference": f"ISSO 54 v2.0 {test_id}: {dwellings} woningen van 96 m2"}),
        setp(f"{NTA}/distributionSystem/installation", "collective"),
        setp(f"{NTA}/distributionSystem/connectedStoreys", 4),
        COLLECTIVE_PUMP,
    ]


CASES += [
    ("EPW205a", 28, boiler_set("EP-W205a", [("hr107", False), ("vr", False)], 70.0, 4, "80_60")),
    ("EPW205c", 29, boiler_set("EP-W205c", [("hr107", False), ("vr", True), ("vr", True)], 46.0, 10, "50_42")),
]


# --- EP-W008/EP-W009: overhangs and side obstructions per window (p. 10-13) ---
# Window positions of figures 2-5: 3,0 x 2,0 m, sills at 0,5 m and 3,2 m,
# horizontally 0,5-3,5 m and 4,5-7,5 m of the 8 m facade. raam-zuid-1/3 are
# taken as the left windows (lower, upper), raam-zuid-2/4 as the right ones.
# h_o (2022 p. 672): height from the window centre to the underside of the
# overhang over its depth. b_b (p. 673): distance parallel to the window from
# the centre to the far point P over the distance perpendicular to it.
# Left/right is seen from inside, looking out of the south facade (west is
# right, east is left).
LEFT = ["raam-zuid-1", "raam-zuid-3"]
RIGHT = ["raam-zuid-2", "raam-zuid-4"]


def per_window(test_id, page, entries):
    return setp(
        f"{NTA}/windowObstructions",
        [{"windowId": window_id, "obstruction": body, "sourceReference": f"ISSO 54 v2.0 {test_id} p. {page}: {why}"} for window_id, body, why in entries],
    )


def overhang_all(test_id, depth):
    relative = round(1.0 / depth, 4)
    return per_window(
        test_id,
        10,
        [(w, {"method": "overhang", "relativeHeight": relative}, f"overstek direct boven elk raam, diepte {depth} m: h_o = 1,0/{depth} = {relative}") for w in LEFT + RIGHT],
    )


def roof_overhang(test_id, depth):
    """Fig. 3: the roof (5,4 m) continues over the left 4 m: only the left
    windows are covered; centres 1,5 m and 4,2 m."""
    entries = []
    for window_id, centre in (("raam-zuid-1", 1.5), ("raam-zuid-3", 4.2)):
        relative = round((5.4 - centre) / depth, 4)
        entries.append((window_id, {"method": "overhang", "relativeHeight": relative}, f"dakoverstek diepte {depth} m over 4 m vanaf links, raammidden {centre} m: h_o = {relative}"))
    return per_window(test_id, 11, entries)


def side_all(test_id, depth):
    relative = round(1.5 / depth, 4)
    return per_window(
        test_id,
        12,
        [(w, {"method": "side_obstruction", "side": "right", "relativeWidth": relative}, f"zijbelemmering aan de westzijde tegen het raam, diepte {depth} m: b_b = 1,5/{depth} = {relative}") for w in LEFT + RIGHT],
    )


# 17.3.2 situation a (2022 p. 674): an obstruction with a relative width b_b
# of 3,73 or more counts as minimal obstruction; only b_b < 3,73 is a side
# obstruction (situation d). Review 19 (D3): the far windows of EP-W009c/d
# (b_b 10,0 and 5,0) were encoded as side obstruction.
MIN_SIDE_WIDTH = 3.73


def east_obstruction(test_id, depth):
    """Fig. 5: one obstruction at the east edge, 0,5 m from the right
    windows; the far point is 2,0 m from the right and 6,0 m from the left
    window centres."""
    entries = []
    for windows, distance in ((RIGHT, 2.0), (LEFT, 6.0)):
        relative = round(distance / depth, 4)
        for window_id in windows:
            if relative >= MIN_SIDE_WIDTH:
                entries.append((window_id, {"method": "minimal"}, f"zijbelemmering oostzijde, diepte {depth} m: b_b = {distance}/{depth} = {relative} >= 3,73, minimale belemmering (17.3.2 situatie a, 2022 p. 674)"))
            else:
                entries.append((window_id, {"method": "side_obstruction", "side": "left", "relativeWidth": relative}, f"zijbelemmering oostzijde, diepte {depth} m: b_b = {distance}/{depth} = {relative}"))
    return per_window(test_id, 13, entries)


CASES += [
    ("EPW008a", 10, [overhang_all("EP-W008a", 5.0)]),
    ("EPW008b", 10, [overhang_all("EP-W008b", 1.8)]),
    ("EPW008c", 10, [overhang_all("EP-W008c", 2.2)]),
    ("EPW008d", 11, [roof_overhang("EP-W008d", 2.2)]),
    ("EPW008e", 11, [roof_overhang("EP-W008e", 5.0)]),
    ("EPW009a", 12, [side_all("EP-W009a", 0.6)]),
    ("EPW009b", 12, [side_all("EP-W009b", 2.0)]),
    ("EPW009c", 13, [east_obstruction("EP-W009c", 0.6)]),
    ("EPW009d", 13, [east_obstruction("EP-W009d", 1.2)]),
    ("EPW009e", 13, [east_obstruction("EP-W009e", 2.4)]),
    ("EPW009f", 13, [east_obstruction("EP-W009f", 6.0)]),
]



# --- Geometry and envelope variants as a leaf-level diff of the reference ---
# A deeltest that rebuilds the building (EP-W002, W006, W013, W014, W103c,
# EP-U202, U502c) is written as a changed copy of the reference project and
# turned into "set"/"remove" patches by `diff`, so the suite still holds only
# patches. Lists that change length are replaced as a whole.
import copy

REFERENCE_U = json.loads((ROOT / "training-data" / "isso54" / "EPU001.json").read_text())


def diff(base, new, pointer=""):
    if isinstance(base, dict) and isinstance(new, dict):
        ops = []
        for key, value in new.items():
            if key not in base:
                ops.append(setp(f"{pointer}/{key}", value))
            else:
                ops += diff(base[key], value, f"{pointer}/{key}")
        ops += [remove(f"{pointer}/{key}") for key in base if key not in new]
        return ops
    if isinstance(base, list) and isinstance(new, list) and len(base) == len(new):
        ops = []
        for index, (a, b) in enumerate(zip(base, new)):
            ops += diff(a, b, f"{pointer}/{index}")
        return ops
    return [] if base == new else [setp(pointer, new)]


def variant(change, reference=None):
    reference = REFERENCE if reference is None else reference
    project = copy.deepcopy(reference)
    change(project)
    return diff(reference, project)


def surface(project, surface_id):
    return next(s for s in project["zones"][0]["surfaces"] if s["id"] == surface_id)


def window(number, area=6.0, orientation="S", surface_id="gevel-zuid", u_value=1.8, g_value=0.7):
    return {
        "id": f"raam-zuid-{number}",
        "name": f"Raam zuid {number}",
        "area": area,
        "uValue": u_value,
        "gValue": g_value,
        "orientation": orientation,
        "surfaceId": surface_id,
    }


def u_from_rc(rc, transfer):
    """U of an opaque construction from R_c and R_si + R_se, rounded as the
    reference construction (0,162 for R_c 6,0 with 0,17)."""
    return round(1.0 / (rc + transfer), 3)


# EP-W002a/b: glass, frame, R_c of the facade, roof and floor and the
# construction year, which ISSO 54 applies to infiltration, the fans and the
# insulation period of the heating pipes, not to the boiler (p. 6-7). Walls
# take R_si + R_se 0,17 and the roof 0,14 (table 1, p. 5); the floor on
# ground takes R_c + 0,17 as the reference (EPW001.json, 6,17 for R_c 6,0).
def insulation(test_id, year_value, pipes, wall_rc, roof_rc, floor_rc, window_u, window_g):
    def change(project):
        project["constructions"] = [
            {"id": "c-gevel", "name": f"Gevel Rc {wall_rc}", "layers": [], "rcValue": wall_rc, "uValue": u_from_rc(wall_rc, 0.17)},
            {"id": "c-dak", "name": f"Dak Rc {roof_rc}", "layers": [], "rcValue": roof_rc, "uValue": u_from_rc(roof_rc, 0.14)},
            {"id": "c-vloer", "name": f"Vloer Rc {floor_rc}", "layers": [], "rcValue": floor_rc, "uValue": u_from_rc(floor_rc, 0.17)},
        ]
        # The floor's U comes from groundFloors; its construction id names the
        # floor's own Rc (review 19: it carried the roof's).
        by_type = {"wall": "c-gevel", "roof": "c-dak", "floor": "c-vloer"}
        for item in project["zones"][0]["surfaces"]:
            item["constructionId"] = by_type[item["type"]]
        for item in surface(project, "gevel-zuid")["windows"]:
            item["uValue"] = window_u
            item["gValue"] = window_g
        nta = project["ntaCalculation"]
        nta["groundFloors"][0]["constructionResistanceM2kPerW"] = round(floor_rc + 0.17, 2)
        nta["groundFloors"][0]["sourceReference"] = f"ISSO 54 v2.0 {test_id}: vloer op grond Rc {floor_rc}"
        nta["constructionYear"] = year_value
        nta["ventilation"]["constructionYear"] = year_value
        nta["ventilation"]["fans"]["manufactureYear"] = year_value
        nta["distributionSystem"]["pipeTransmittance"]["insulation"] = pipes

    return variant(change)


# EP-W002c: the detailed thermal-bridge route with the four given psi-values
# (p. 7); the vertical corners are 4 x 5,4 = 21,6 m (the test prints 20,8 and
# 21,6). The floor-facade joint is the floor edge of 8.36.
def detailed_bridges(project):
    bridges = [("hoeken", 0.1, 21.6), ("dak-gevel", 0.04, 28.0), ("kozijn-gevel", 0.05, 40.0)]
    project["zones"][0]["thermalBridges"] = [
        {"id": f"tb-{name}", "name": f"{name} psi {psi}", "psiValue": psi, "length": length, "zoneId": "epw001-zone", "thermalBoundary": "outdoor"}
        for name, psi, length in bridges
    ]
    project["ntaCalculation"]["groundFloors"][0]["edgeThermalBridges"] = {
        "method": "detailed",
        "bridges": [{"lengthM": 28.0, "psiWPerMk": -0.18, "sourceReference": "ISSO 54 v2.0 EP-W002c p. 7: aansluiting vloer-gevel psi -0,18 W/mK, 28 m"}],
    }


# EP-W006a-e: the ground floor above a crawlspace (p. 8; 8.3.4.2, 8.33-8.48).
# R_bw is the facade R_c 6,0 with its U 0,162 (8.47 allows the facade U); h
# of (8.47), required in 2022, is 0 for a floor at ground level and 0,25 m
# where the test says so. ISSO 54 notes the boundary also acts on
# infiltration (table 11.1 crawlspace, only before 1992).
def crawlspace(test_id, rbf, height=0.0, opening=0.0012):
    def change(project):
        floor = project["ntaCalculation"]["groundFloors"][0]
        floor["constructionResistanceM2kPerW"] = 0.32
        floor["below"] = {
            "kind": "crawlspace",
            "floorResistanceM2kPerW": rbf,
            "depthClass": "other",
            "wallResistanceM2kPerW": 6.0,
            "wallUValueWPerM2k": 0.162,
            "ventilationOpeningM2PerM": opening,
            "wallHeightAboveGroundM": height,
        }
        floor["sourceReference"] = f"ISSO 54 v2.0 {test_id} p. 8: vloer Rc 0,15 boven kruipruimte, Rbf {rbf}, ventilatieopeningen {opening} m2/m, h {height} m"
        project["ntaCalculation"]["ventilation"]["floorAboveCrawlspace"] = True

    return variant(change)


# EP-W006f: the floor above the crawlspace of EP-W006a (R_c 0,15, R_bf 6,
# openings 0,0012 m2/m) with the ground level at 0,5 m (p. 9). The heated
# floor lies 0,5 m below ground level: 8.39 with U_fl of 8.43 and U_x;T = 0
# (2022 p. 229, 230, 236), so h of (8.47) is not given. The lower 0,5 m of
# every facade borders the ground and is the wall term z·P·U_bw of 8.39
# (z = 0,5, R_c of the facade 6,0); the outdoor facade areas (gross) lose
# 0,5 m x their length. The windows of the lower storey start at 0,5 m and
# stay outdoor.
def sunken_crawlspace(project):
    floor = project["ntaCalculation"]["groundFloors"][0]
    floor["constructionResistanceM2kPerW"] = 0.32
    floor["below"] = {
        "kind": "crawlspace",
        "floorResistanceM2kPerW": 6.0,
        "depthClass": "other",
        "wallResistanceM2kPerW": 6.0,
        "wallUValueWPerM2k": 0.162,
        "ventilationOpeningM2PerM": 0.0012,
    }
    floor["heatedBasement"] = {"depthM": 0.5, "wallResistanceM2kPerW": 6.0}
    floor["sourceReference"] = "ISSO 54 v2.0 EP-W006f p. 9: vloer Rc 0,15 boven kruipruimte Rbf 6, ventilatieopeningen 0,0012 m2/m, maaiveld op 0,5 m (8.39 met 8.43, U_x;T = 0)"
    project["ntaCalculation"]["ventilation"]["floorAboveCrawlspace"] = True
    for surface_id, length in [("gevel-zuid", 8.0), ("gevel-noord", 8.0), ("gevel-oost", 6.0), ("gevel-west", 6.0)]:
        item = surface(project, surface_id)
        item["area"] = round(item["area"] - 0.5 * length, 2)


# EP-W006g: a dike dwelling, the floor and the whole north facade against the
# ground (p. 8). Modelled as a heated room below ground level (8.3.3.2,
# 8.42/D.12): the north wall part (8 m) 5,4 m deep, the other parts at
# ground level; the north facade is no longer an outdoor surface.
def dike(project):
    floor = project["ntaCalculation"]["groundFloors"][0]
    floor["heatedBasement"] = {
        "wallDepths": [{"lengthM": 8.0, "depthM": 5.4}, {"lengthM": 6.0, "depthM": 0.0}, {"lengthM": 8.0, "depthM": 0.0}, {"lengthM": 6.0, "depthM": 0.0}],
        "wallResistanceM2kPerW": 6.0,
    }
    floor["sourceReference"] = "ISSO 54 v2.0 EP-W006g p. 9: dijkwoning, vloer en noordgevel volledig tegen grond (8.42/D.12)"
    # The wall below ground is part of the basement route (8.42), not a
    # surface of its own.
    zone = project["zones"][0]
    zone["surfaces"] = [s for s in zone["surfaces"] if s["id"] != "gevel-noord"]


# EP-W013: the usable area (p. 15-17). The reference module is 8 x 6 m, 2
# storeys of 2,7 m, 4 windows of 6 m2 in the south facade; the facade areas
# below are gross (net + glazing of tabel 3).
def scale(project, *, area, volume, height, roof, floor, south, north, east_west, windows_, perimeter, storeys, building_height):
    zone = project["zones"][0]
    zone["floorArea"] = area
    zone["volume"] = volume
    zone["height"] = height
    surface(project, "dak")["area"] = roof
    surface(project, "bg-vloer")["area"] = floor
    surface(project, "gevel-zuid")["area"] = south
    surface(project, "gevel-noord")["area"] = north
    surface(project, "gevel-oost")["area"] = east_west
    surface(project, "gevel-west")["area"] = east_west
    surface(project, "gevel-zuid")["windows"] = [window(k + 1) for k in range(windows_)]
    nta = project["ntaCalculation"]
    nta["groundFloors"][0]["exposedPerimeterM"] = perimeter
    nta["distributionSystem"]["connectedStoreys"] = storeys
    nta["verticalPipes"][0]["storeys"] = storeys
    vent = nta["ventilation"]
    vent["usableFloorAreaM2"] = area
    vent["functions"][0]["areaM2"] = area
    vent["buildingHeightM"] = building_height
    nta["areaSourceReference"] = f"ISSO 54 v2.0 EP-W013: Ag {area} m2"


def dwellings(project, count):
    nta = project["ntaCalculation"]
    nta["internalGains"]["dwellingCount"] = count
    nta["internalGains"]["sourceReference"] = f"{count} woningen"
    nta["ventilation"]["dwellingCount"] = count
    nta["hotWater"]["need"]["dwellingCount"] = count
    nta["hotWater"]["need"]["sourceReference"] = f"{count} woningen"


def double(project):
    scale(project, area=192.0, volume=518.4, height=5.4, roof=96.0, floor=96.0, south=86.4, north=86.4, east_west=32.4, windows_=8, perimeter=44.0, storeys=2, building_height=5.4)


# Every dwelling of EP-W013b/d has its own installation (p. 17-18): 4 and 10
# identical systems (§9.1), as in EP-W206c, so the 9.85 standby term and the
# 500 m2 limit of the recoverable losses apply per appliance (review 19, D4).
def four_dwellings(project):
    double(project)
    dwellings(project, 4)
    project["ntaCalculation"]["identicalSystems"] = 4


def small(project):
    scale(project, area=24.0, volume=64.8, height=2.7, roof=24.0, floor=24.0, south=10.8, north=10.8, east_west=16.2, windows_=1, perimeter=20.0, storeys=1, building_height=2.7)


# EP-W013d: 10 modules of 2 storeys stacked, 54 m, 960 m2, 10 dwellings with
# an individual installation each (connected storeys per dwelling: 2).
def tower(project):
    scale(project, area=960.0, volume=2592.0, height=54.0, roof=48.0, floor=48.0, south=432.0, north=432.0, east_west=324.0, windows_=40, perimeter=28.0, storeys=2, building_height=54.0)
    dwellings(project, 10)
    nta = project["ntaCalculation"]
    nta["identicalSystems"] = 10
    nta["dwellingType"] = "apartment_building"
    nta["ventilation"]["apartmentBuilding"] = True
    nta["ventilation"]["infiltration"]["buildingType"] = "multi_storey_whole_building"


# EP-W014a: a mono-pitched roof of 45 degrees facing south; the north facade
# rises to 11,4 m (8 x 11,4 = 91,2 m2), the side facades become trapezia
# (6 x (5,4 + 11,4) / 2 = 50,4 m2), the roof 8 x 6 / cos 45 = 67,9 m2; the
# usable area stays 96 m2 and the volume becomes 8 x 50,4 = 403,2 m3
# (p. 18). "Vrijstaand met puntdak": table 11.14 pitched roof, detached.
def pitched(project):
    roof = surface(project, "dak")
    roof["area"] = 67.9
    roof["orientation"] = "S"
    roof["name"] = "Lessenaarsdak 45 graden op het zuiden"
    surface(project, "gevel-noord")["area"] = 91.2
    surface(project, "gevel-oost")["area"] = 50.4
    surface(project, "gevel-west")["area"] = 50.4
    project["zones"][0]["volume"] = 403.2
    nta = project["ntaCalculation"]
    nta["surfaceTilts"] = [{"surfaceId": "dak", "tiltDeg": 45.0, "sourceReference": "ISSO 54 v2.0 EP-W014a p. 19: dakhelling 45 graden"}]
    nta["ventilation"]["buildingHeightM"] = 11.4
    nta["ventilation"]["infiltration"]["buildingType"] = "pitched_roof_detached"


# EP-W103c: as EP-W103a with cross ventilation: one window of each storey
# moves to the east facade (p. 22); the south facade keeps 2 windows, the
# east facade gets 2 (net 31,2 and 20,4 m2 as the test states).
def cross_ventilation(project):
    south = surface(project, "gevel-zuid")
    moved = [w for w in south["windows"] if w["id"] in ("raam-zuid-2", "raam-zuid-4")]
    south["windows"] = [w for w in south["windows"] if w["id"] not in ("raam-zuid-2", "raam-zuid-4")]
    for item in moved:
        item["orientation"] = "E"
        item["surfaceId"] = "gevel-oost"
    surface(project, "gevel-oost")["windows"] = moved
    openings = [
        {
            "id": f"raam-{k + 1}",
            "area": {"method": "opening_angle", "maxNetAreaM2": 2.5, "maxAngleDeg": 90.0},
            "centreHeightM": 1.2 if k in (0, 1) else 3.9,
            "openingHeightM": 2.0,
            "azimuthDeg": 180.0 if k in (0, 2) else 90.0,
            "tiltDeg": 90.0,
        }
        for k in range(4)
    ]
    project["ntaCalculation"]["ventilation"]["ventilativeCooling"] = {
        "openings": openings,
        "operation": "manual",
        "conditionsEvidence": "ISSO 54 v2.0 EP-W103c p. 22: zomernachtventilatie dubbelzijdig (zuid en oost), handbediend; voorwaarden 11.2.3.3 als vervuld aangenomen",
    }


CASES += [
    ("EPW002a", 7, insulation("EP-W002a", 1964, {"state": "insulated", "period": "before1980_or_unknown"}, 0.85, 0.22, 0.15, 5.4, 0.85)),
    ("EPW002b", 7, insulation("EP-W002b", 1995, {"state": "insulated", "period": "from1995"}, 2.5, 2.5, 2.5, 1.8, 0.7)),
    ("EPW002c", 7, variant(detailed_bridges)),
    ("EPW006a", 8, crawlspace("EP-W006a", 6.0)),
    ("EPW006b", 8, crawlspace("EP-W006b", 3.5)),
    ("EPW006c", 8, crawlspace("EP-W006c", 0.0)),
    ("EPW006d", 8, crawlspace("EP-W006d", 6.0, height=0.25)),
    ("EPW006e", 8, crawlspace("EP-W006e", 6.0, height=0.25, opening=0.0006)),
    ("EPW006f", 9, variant(sunken_crawlspace)),
    ("EPW006g", 9, variant(dike)),
    ("EPW013a", 17, variant(double)),
    ("EPW013b", 17, variant(four_dwellings)),
    ("EPW013c", 17, variant(small)),
    ("EPW013d", 18, variant(tower)),
    # EP-W013e: only the top apartment (2 storeys) of the EP-W013d tower, no
    # floor (p. 17): the apartment variant in a 54 m building.
    ("EPW013e", 18, APARTMENT + [setp(f"{VENT}/buildingHeightM", 54.0)]),
    ("EPW014a", 19, variant(pitched)),
    ("EPW103c", 22, variant(cross_ventilation)),
]


# --- EP-W303c/d: free cooling with storage next to a chiller (p. 32-33) ---
# Free cooling (table 10.34) of 1 or 2 kW plus a compression chiller of 9 or
# 8 kW: priority by table 10.15, capacities required with two priorities.
# The heat pump uses the storage as its source (10.84, "regeneratie via de
# warmtepomp"). The heating heat pump does not meet table 9.28 (base row of
# table 9.27); its supply temperature stays the reference 45/40. Aquifer is
# taken as groundwater below 15 degrees C (table 9.27). The test gives no
# degree of regeneration R, so c_source stays 1,00 (table V.1 lowest row; for
# EP-W303c also table V.3 recirculation).
def storage_cooling(test_id, page, source, free_kw, chiller_kw):
    return W301A + [
        cooling_generators(
            {
                "id": "vrije-koeling",
                "generator": {"kind": "free_cooling", "source": source, "heatPumpSource": True},
                "capacityKw": free_kw,
                "equipmentReference": f"ISSO 54 v2.0 {test_id} p. {page}: vrije koeling met opslag, {free_kw} kW, regeneratie via de warmtepomp",
            },
            chiller({"kind": "compression"}, capacity=chiller_kw, why=f"ISSO 54 v2.0 {test_id} p. {page}: compressiekoelmachine {chiller_kw} kW"),
        ),
    ]


def regenerated(test_id, why):
    return [
        setp(f"{GEN}/forfait/sourceCorrectionFactor", 1.0),
        setp(f"{GEN}/forfait/sourceCorrectionReference", f"ISSO 54 v2.0 {test_id}: {why}; geen regeneratiegraad R gegeven, tabel V.1 laagste rij c 1,00"),
    ]


CASES += [
    (
        "EPW303c",
        33,
        storage_cooling("EP-W303c", 33, "aquifer_from2013", 1.0, 9.0)
        + [heat_pump("EP-W303c", "groundwater_below15_c", 45.0), temperature_class("45_40"), renewable("EP-W303c", True)]
        + regenerated("EP-W303c", "bron aquifer, bronsysteem recirculatie (tabel V.3 c 1,00)")
        # Hot water: a heat pump with an integrated vessel on the same
        # aquifer, permit after 2013 (table 13.25 with c_source 1,00).
        + [hw_generator({"kind": "heat_pump", "exhaustAirSource": False, "sourceCorrection": 1.0})],
    ),
    (
        "EPW303d",
        33,
        storage_cooling("EP-W303d", 33, "closed_ground_loop", 2.0, 8.0)
        + [heat_pump("EP-W303d", "ground", 45.0), temperature_class("45_40"), renewable("EP-W303d", True)]
        + regenerated("EP-W303d", "bron bodem, regeneratie via de warmtepomp"),
    ),
]


# --- EP-U202, EP-U502c, EP-U701d: the office rebuilt (p. 45-51) ---
# Gross facade areas (net of tabel + glazing); every area field of the
# reference office follows the usable area.
def scale_u(project, *, area, volume, height, roof, floor, south, north, east_west, windows_, perimeter, storeys, building_height):
    zone = project["zones"][0]
    zone["floorArea"] = area
    zone["volume"] = volume
    zone["height"] = height
    surface(project, "dak")["area"] = roof
    surface(project, "bg-vloer")["area"] = floor
    surface(project, "gevel-zuid")["area"] = south
    surface(project, "gevel-noord")["area"] = north
    surface(project, "gevel-oost")["area"] = east_west
    surface(project, "gevel-west")["area"] = east_west
    surface(project, "gevel-zuid")["windows"] = [window(k + 1) for k in range(windows_)]
    nta = project["ntaCalculation"]
    nta["groundFloors"][0]["exposedPerimeterM"] = perimeter
    nta["distributionSystem"]["connectedStoreys"] = storeys
    nta["cooling"]["distribution"]["pump"]["floorCount"] = storeys
    vent = nta["ventilation"]
    vent["usableFloorAreaM2"] = area
    vent["functions"][0]["areaM2"] = area
    vent["buildingHeightM"] = building_height
    # 7.17a: the fictitious toilet-group pipe follows the building height
    # (one toilet group; stacked groups count as one, 2022 p. 155-156).
    nta["verticalPipes"][0]["buildingHeightM"] = building_height
    nta["hotWater"]["need"]["areas"][0]["areaM2"] = area
    nta["lighting"][0]["functions"][0]["areaM2"] = area
    nta["lighting"][0]["lightingZones"][0]["areaM2"] = area
    nta["areaSourceReference"] = f"ISSO 54 v2.0: Ag {area} m2"


# EP-U202: 8 modules stacked (p. 45): Ag 384 m2, 1036,8 m3, 21,6 m; south
# 76,8 + 96 glazing (16 windows), east/west 129,6, north 172,8.
def eight_storeys(project):
    scale_u(project, area=384.0, volume=1036.8, height=21.6, roof=48.0, floor=48.0, south=172.8, north=172.8, east_west=129.6, windows_=16, perimeter=28.0, storeys=8, building_height=21.6)


U202 = variant(eight_storeys, REFERENCE_U)


def ahu(inside, ducts="none", why=""):
    return setp(
        f"{UNIT}/airHandlingUnit",
        {"insideThermalZone": inside, "supplyDuctsOutside": ducts, "heatingCoil": True, "coolingCoil": True},
    )


# EP-U502c: 32 x 24 x 5,4 m, Ag 1536 m2 (2 storeys), the 4 windows of 24 m2
# remain; front 148,8 + 24, back 172,8, sides 129,6, roof and floor 768 m2,
# perimeter 112 m; sport with 500 m2 of sport halls (13.32a), the rest as
# EP-U502a (p. 49).
def sport_hall(project):
    scale_u(project, area=1536.0, volume=4147.2, height=5.4, roof=768.0, floor=768.0, south=172.8, north=172.8, east_west=129.6, windows_=4, perimeter=112.0, storeys=2, building_height=5.4)


UTILITY_CASES += [
    # EP-U202a: an AHU in the heated zone that heats and cools (p. 45; 11.3.2).
    ("EPU202a", 46, U202 + [ahu(True)]),
    # EP-U202b: design flow 1500 dm3/s, flow control down to 60 % (11.61 x 60,
    # design evidence), speed control, nominal fan power 100 W (p. 45).
    (
        "EPU202b",
        46,
        U202
        + [
            ahu(True),
            installed(1500.0, "ISSO 54 v2.0 EP-U202b p. 46: ontwerpdebiet 1500 dm3/s"),
            setp(f"{VENT}/flowReduction", {"collective": True, "flowControlPercent": 60, "evidenceReference": "ISSO 54 v2.0 EP-U202b p. 46: terugregeling tot 60 % van de maximale capaciteit"}),
            setp(
                f"{VENT}/fans",
                {
                    "method": "declared",
                    "fans": [{"id": "ventilatoren", "power": {"method": "nominal", "nominalPowerW": 100.0}}],
                    "control": {"method": "flow_control", "control": "speed_control"},
                    "sourceReference": "ISSO 54 v2.0 EP-U202b p. 46: toerenregeling, nominaal vermogen 100 W",
                },
            ),
        ],
    ),
    # EP-U202c: the AHU outside the heated zone, supply ducts 20-40 m outside
    # the thermal zone, insulated: table 11.19 situation 2 (2022 p. 489).
    ("EPU202c", 46, U202 + [ahu(False, "situation2")]),
    (
        "EPU502c",
        49,
        U502A
        + function("h")
        + variant(sport_hall, REFERENCE_U)
        + [setp(f"{HW}/circulation/sportHallAreaM2", 500.0)],
    ),
    # EP-U701d: on EP-U202a, a steam humidifier with a central gas-fired
    # generator, heat recovery by a heat wheel (table 11.18 rotary; chapter 12
    # with the wheel), other healthcare (p. 51).
    (
        "EPU701d",
        51,
        # function() sets the areas to 96 m2: it goes before the geometry.
        function("e")
        + U202
        + [ahu(True)]
        + [
            setp(f"{UNIT}/heatRecovery/efficiency", {"method": "table", "exchanger": "rotary"}),
            setp(
                f"{NTA}/humidifiers",
                [{"zoneId": "epw001-zone", "humidification": {"humidifier": {"kind": "steam", "carrier": "gas_or_oil"}, "rotaryWheel": True, "equipmentReference": "ISSO 54 v2.0 EP-U701d p. 51: stoombevochtiger met centrale gasgestookte opwekker, warmtewiel"}}],
            ),
        ],
    ),
]


# --- EP-W203l-n: two calculation zones with a hybrid heat pump (p. 26) ---
# The dwelling as two zones, the ground floor (epw001-zone) and the upper
# floor (epw001-boven), 48 m2 each: the floor and the lower half of the
# facades with windows 1-2 below, the roof and the upper half with windows
# 3-4 above; the intermediate floor lies inside the envelope. Each zone takes
# half the dwelling (6.2b dwellingShare 0,5) and its own ventilation input
# for its 48 m2, sized for the whole 96 m2 dwelling (wholeDwellingAreaM2 for
# f_tau of table 11.8 and the 11.65 minimum). Each zone is one storey, so its
# forfait vertical pipe counts 1 storey (7.3.3, N_bouwlaag of the zone): the
# two zones together give the H_p of EP-W001. Review 19 (D1): with 2 storeys
# per zone and a 48 m2 dwelling per zone the split alone added 8 kWh/m2 BENG 2. The hybrid: an outdoor-air heat pump of 10 kW not meeting
# table 9.28 (table 9.27 COP 2,3 at 45 degrees C in 2022) and an HR107 with
# CW4 of 35 kW (table 9.25 eta 0,95), inside the envelope; the priority
# efficiencies of table 9.1 are these table values. Hot water stays the
# HR107 combi (kitchen below, bathroom above).
UPPER = "epw001-boven"
HP_COP_2022 = 2.3
HR107_ETA_2022 = 0.95


def two_zones(project):
    zone = project["zones"][0]
    upper = copy.deepcopy(zone)
    for item, area in ((zone, 48.0), (upper, 48.0)):
        item["floorArea"] = area
        item["volume"] = 129.6
        item["height"] = 2.7
    upper["id"] = UPPER
    upper["name"] = "Bovenverdieping"
    halves = {"gevel-zuid": 21.6, "gevel-noord": 21.6, "gevel-oost": 16.2, "gevel-west": 16.2}
    lower_surfaces, upper_surfaces = [], []
    for item in zone["surfaces"]:
        if item["id"] == "dak":
            upper_surfaces.append(dict(item, zoneId=UPPER))
            continue
        if item["id"] == "bg-vloer":
            lower_surfaces.append(item)
            continue
        below = dict(item, area=halves[item["id"]])
        above = dict(item, id=f"{item['id']}-boven", area=halves[item["id"]], zoneId=UPPER)
        if item["id"] == "gevel-zuid":
            below["windows"] = [w for w in item["windows"] if w["id"] in ("raam-zuid-1", "raam-zuid-2")]
            above["windows"] = [dict(w, surfaceId="gevel-zuid-boven") for w in item["windows"] if w["id"] in ("raam-zuid-3", "raam-zuid-4")]
        lower_surfaces.append(below)
        upper_surfaces.append(above)
    zone["surfaces"] = lower_surfaces
    upper["surfaces"] = upper_surfaces
    project["zones"].append(upper)
    nta = project["ntaCalculation"]
    ventilation = nta.pop("ventilation")
    entries = []
    for zone_id in (zone["id"], UPPER):
        zone_ventilation = copy.deepcopy(ventilation)
        zone_ventilation["zoneId"] = zone_id
        zone_ventilation["usableFloorAreaM2"] = 48.0
        zone_ventilation["functions"] = [{"function": "residential", "areaM2": 48.0}]
        zone_ventilation["wholeDwellingAreaM2"] = 96.0
        gains = dict(nta["internalGains"], dwellingShare=0.5, sourceReference="een woning over twee rekenzones (6.2b, aandeel 0,5)")
        entries.append(
            {
                "zoneId": zone_id,
                "verticalPipes": [{"id": f"{zone_id}-leiding", "storeys": 1, "insulated": False, "sourceReference": "ISSO 54 v2.0 EP-W001 p. 5: leidingdoorvoeren forfaitair (7.3.3); 1 bouwlaag per rekenzone"}],
                "ventilationFlows": [],
                "ventilation": zone_ventilation,
                "internalGains": gains,
            }
        )
    nta["zoneData"] = entries
    del nta["verticalPipes"]


def hybrid(test_id, declaration=None, suffix=""):
    pump = {
        "generatorId": f"wp{suffix}",
        "classificationSourceReference": f"ISSO 54 v2.0 {test_id}: elektrische lucht/waterwarmtepomp op buitenlucht",
        "scope": "residential_at_most25_kw",
        "source": "outdoor_air",
        "sink": "hydronic",
        "designSupplyTemperatureC": 45.0,
        "thermalCapacityKw": 10.0,
        "capacitySourceReference": f"ISSO 54 v2.0 {test_id}: vermogen warmtepomp 10 kW",
        "collectiveBuildingInstallation": False,
    }
    cop = HP_COP_2022
    if declaration is not None:
        pump["qualityDeclaration"] = declaration
        cop = declaration["generationEfficiency"]
    boiler = copy.deepcopy(REFERENCE["ntaCalculation"]["generator"]["boiler"])
    boiler["generatorId"] = f"ketel{suffix}"
    boiler["role"] = "individual_supplementary"
    boiler["equipmentReference"] = f"ISSO 54 v2.0 {test_id}: HR107-ketel CW4, 35 kW, bijstook"
    return {
        "kind": "hybrid_heat_pump",
        "designContext": "new_build",
        "generators": [
            {"id": f"wp{suffix}", "class": "heat_pump", "classificationReference": f"ISSO 54 v2.0 {test_id}: warmtepomp", "nominalThermalPowerKw": 10.0, "powerReference": f"ISSO 54 v2.0 {test_id}: 10 kW", "priorityEfficiency": cop, "efficiencyReference": "tabel 9.27 (2022) of de kwaliteitsverklaring"},
            {"id": f"ketel{suffix}", "class": "other_boiler", "classificationReference": f"ISSO 54 v2.0 {test_id}: HR107-ketel", "nominalThermalPowerKw": 35.0, "powerReference": f"ISSO 54 v2.0 {test_id}: 35 kW", "priorityEfficiency": HR107_ETA_2022, "efficiencyReference": "tabel 9.25 (2022)"},
        ],
        "forfait": pump,
        "boiler": boiler,
        "sourceSystem": "individual",
        "sourceSystemReference": f"ISSO 54 v2.0 {test_id}: individuele installatie binnen de thermische schil",
    }


def hybrid_two_zones(test_id, declaration=None):
    def change(project):
        two_zones(project)
        nta = project["ntaCalculation"]
        nta["generator"] = hybrid(test_id, declaration)
        nta["heatPumpRenewable"] = {"sourceBelow20C": True, "exhaustAirSource": False, "sourceReference": f"ISSO 54 v2.0 {test_id}: buitenlucht"}

    return change


# EP-W203m: every zone has its own installation for heating, hot water and
# ventilation (p. 26): a second hybrid for the upper zone, and two combis,
# the kitchen below and the bathroom above (13.19a connectedTaps).
def own_installations(project):
    hybrid_two_zones("EP-W203m")(project)
    nta = project["ntaCalculation"]
    nta["additionalHeatingSystems"] = [
        {
            "zoneIds": [UPPER],
            "generator": hybrid("EP-W203m", suffix="-boven"),
            "distributionSystem": copy.deepcopy(nta["distributionSystem"]),
        }
    ]
    kitchen = copy.deepcopy(nta["hotWater"])
    kitchen["emission"] = {"method": "residential", "served": "kitchen_only", "kitchenLengthM": 8.5, "kitchenPipeDiameter": "other", "sourceReference": "ISSO 54 v2.0 EP-W203m p. 26: keuken in de onderste rekenzone, uittapleiding 8,5 m"}
    kitchen["connectedTaps"] = {"bathrooms": 0, "kitchens": 1}
    bathroom = copy.deepcopy(nta["hotWater"])
    bathroom["emission"] = {"method": "residential", "served": "bathroom_only", "bathroomLengthM": 5.0, "sourceReference": "ISSO 54 v2.0 EP-W203m p. 26: badkamer in de bovenste rekenzone, douche 5 m"}
    bathroom["connectedTaps"] = {"bathrooms": 1, "kitchens": 0}
    bathroom["equipmentReference"] = "ISSO 54 v2.0 EP-W203m p. 26: eigen HR107-combi voor de bovenste rekenzone"
    nta["hotWater"] = kitchen
    nta["additionalHotWaterSystems"] = [bathroom]


CASES += [
    ("EPW203l", 26, variant(hybrid_two_zones("EP-W203l"))),
    ("EPW203m", 26, variant(own_installations)),
    (
        "EPW203n",
        26,
        variant(
            hybrid_two_zones(
                "EP-W203n",
                {"declarationReference": "ISSO 54 v2.0 EP-W203n p. 26: kwaliteitsverklaring COP 4,3, fractie 0,72", "generationEfficiency": 4.3, "energyFraction": 0.72},
            )
        ),
    ),
]


# --- EP-W406i/k: a booster heat pump on a collective heating system ---
# The apartment variant (top-floor corner, 4 dwellings, 10,8 m) heated by a
# collective HR107 of 100 kW outside the envelope, 45/40, pipes through
# unheated spaces of unknown length, insulated, valves insulated, no heat
# meters, pump power and EEI unknown (p. 37-38). Hot water: a booster heat
# pump with no test data, so the forfait of 13.162/13.163 (2022 p. 607-609)
# on the 45 degrees C supply of that heating system (held at the 44 degrees C
# value); its heat loads the space-heating chain (9.4).
W406I = APARTMENT + [
    setp(f"{BOILER}/role", "collective"),
    setp(f"{BOILER}/location", "outside_thermal_boundary"),
    setp(f"{GEN}/auxiliary", {"electricallyConnectedDevices": 1, "nominalPowerKw": 100.0, "sourceReference": "ISSO 54 v2.0 EP-W406i p. 37: collectieve HR107, 100 kW"}),
    setp(f"{NTA}/collectiveConnection", {"connectedUsableAreaM2": BUILDING_AREA, "sourceReference": "ISSO 54 v2.0 EP-W406i p. 37: 4 woningen van 96 m2"}),
    setp(f"{NTA}/distributionSystem/installation", "collective"),
    setp(f"{NTA}/distributionSystem/connectedStoreys", 4),
    remove(f"{NTA}/distributionSystem/unheatedPipeLengthM"),
    COLLECTIVE_PUMP,
    hw_generator(
        {
            "kind": "booster_heat_pump_forfait",
            "designSupplyTemperatureC": 45.0,
            "heatSource": {"kind": "heating_system"},
            "sourceReference": "ISSO 54 v2.0 EP-W406i p. 38: boosterwarmtepomp zonder meetgegevens op de collectieve HR107, ontwerpaanvoer 45 C (13.162/13.163)",
        }
    ),
]

CASES += [
    ("EPW406i", 37, W406I),
    # EP-W406k: as EP-W406i with an individual compression chiller, floor
    # cooling, control in the main room (table 10.4 central with room
    # control), design 12/16 (17/21 for floor cooling only, table 10.8 note
    # 2), insulated pipes, none outside the cooled zone, fittings insulated,
    # a pump of unknown power (p. 38). The A/B/C/B_nom of the test are the
    # 9.85 constants of an individual appliance; the collective HR107 takes
    # 9.91, so they are not entered.
    (
        "EPW406k",
        38,
        W406I
        + W301A
        + [cooling_emission("floor_cooling", control="central_with_room_control", why="ISSO 54 v2.0 EP-W406k p. 38: vloerkoeling, regeling in het hoofdvertrek")],
    ),
]

# --- Routes added for ISSO 54 (EP-W011a, W203j, W204h, W202e, W302e, W402f) ---
# EP-W011a: closed horizontal louvres on every window; g_gl from 7.41 with
# g_gl;alt;wi 0,045 at 45 degrees solar altitude and g_gl;dif;wi 0,2 (p. 14;
# 2022 7.6.6.1.3). The rest of the window is as EP-W001.
W011A = [
    setp(
        f"{NTA}/windowGlazings",
        [
            {
                "windowId": f"raam-zuid-{k}",
                "glazing": {
                    "diffusing": {
                        "gAltitude45": 0.045,
                        "gDiffuse": 0.2,
                        "sourceReference": "ISSO 54 v2.0 EP-W011a p. 15: gesloten horizontale lamellen, g_gl;alt 0,045 en g_gl;dif 0,2 (7.41)",
                    }
                },
                "sourceReference": "ISSO 54 v2.0 EP-W011a p. 15: alle ramen",
            }
            for k in range(1, 5)
        ],
    )
]

# EP-W202e, W302e, W402f: insulated pipes embedded in the construction (p. 24,
# 32, 34). The test's "binnendiameter 0,02 m" is taken as d_i of 9.34/10.25/
# 13.28, which those formulas define as the pipe diameter excluding the
# insulation; d_a 0,04 m, depth 0,03 m, lambda insulation 0,04, lambda
# construction 2 W/(m.K).
EMBEDDED = {
    "method": "insulated_embedded",
    "pipeOuterDiameterM": 0.02,
    "insulatedDiameterM": 0.04,
    "insulationLambdaWPerMK": 0.04,
    "embeddingLambdaWPerMK": 2.0,
    "depthM": 0.03,
}

CASES += [
    ("EPW011a", 15, W011A),
    # EP-W203j: the HR107 with a quality declaration (p. 25): efficiency
    # 0,975 and fraction 1, equal to the 2022 table 9.25 value of an HR107
    # inside the envelope at 45/40 (so not entered); A 10 kWh, B 0,12 kW,
    # C 0,3 and B_nom 20 kW replace the 9.85 constants (§9.1). The test's
    # W;H;aux of 106,48/111,55 kWh is an expected result.
    (
        "EPW203j",
        26,
        [
            setp(
                f"{GEN}/declaredAuxiliaryConstants",
                {"aKwh": 10.0, "bKw": 0.12, "c": 0.3, "nominalLoadKw": 20.0, "declarationReference": "ISSO 54 v2.0 EP-W203j p. 26: kwaliteitsverklaring HR107, A 10 kWh, B 0,12 kW, C 0,3, B_nom 20 kW"},
            )
        ],
    ),
    # EP-W204h: as EP-W204g (micro-CHP below 2 kW, LT, 45/40) but with an HRe
    # label and a quality declaration eps_th 0,8 and eps_el 0,12 (p. 28;
    # method 2 with §9.1 instead of table 9.31).
    (
        "EPW204h",
        28,
        micro_chp("EP-W204h", True, True, "ISSO 54 v2.0 EP-W204h p. 28: micro-WKK < 2 kW met HRe-label, LT, kwaliteitsverklaring")
        + [
            setp(f"{GEN}/declaredEfficiencies", {"thermal": 0.8, "electric": 0.12, "declarationReference": "ISSO 54 v2.0 EP-W204h p. 28: kwaliteitsverklaring eps_th 0,8, eps_el 0,12"}),
            temperature_class("45_40"),
        ],
    ),
    (
        "EPW202e",
        24,
        [setp(f"{NTA}/distributionSystem/pipeTransmittance", EMBEDDED)],
    ),
    (
        "EPW302e",
        32,
        W301A + [setp(f"{COOL}/distribution/pipe", {"kind": "calculated", "geometry": EMBEDDED, "sourceReference": "ISSO 54 v2.0 EP-W302e p. 32: geisoleerde leidingen in de constructie (10.25)"})],
    ),
    (
        "EPW402f",
        34,
        W402A + [setp(f"{HW}/circulation/calculatedPsi", EMBEDDED)],
    ),
]


# EP-W011b: all glass replaced by glass with a quality declaration (figure 7,
# p. 15): g_gl;n 0,63 and U_gl 1,1, in a wooden frame U_f 2,4, psi_g 0,08.
# U_W with the simplified 8.15 as EP-W003a: max(0,7.1,1 + 0,3.2,4;
# 0,8.1,1 + 0,2.2,4) + 2,5.0,08 = 1,69 -> 1,7 (8.2.2.1). The declared
# g_gl;n is the window's own g-value (no 7.41 data).
CASES += [("EPW011b", 15, windows("gValue", 0.63) + windows("uValue", 1.7))]


# --- EP-W206: generation in an apartment building (p. 29-30) ---
# 48 apartments: 8 reference modules side by side and 6 above each other,
# 12 storeys, 32,4 m; roof and ground floor 384 m2, facades as the table
# (south 921,6 m2 net + 1152 m2 glazing = 2073,6 gross, north 2073,6,
# east = west 194,4), perimeter 140 m, A_g 4608 m2, volume 64 x 6 x 32,4.
# R_c as EP-W001. A new-build calculation for the whole building (6.2b),
# individual ventilation per apartment (the reference system, scaled), hot
# water an individual electric instantaneous heater per apartment with the
# draw-off lengths of EP-W001.
BUILDING48 = 48 * 96.0


def building48(project):
    scale(project, area=BUILDING48, volume=12441.6, height=32.4, roof=384.0, floor=384.0, south=2073.6, north=2073.6, east_west=194.4, windows_=192, perimeter=140.0, storeys=12, building_height=32.4)
    dwellings(project, 48)
    nta = project["ntaCalculation"]
    nta["dwellingType"] = "apartment_building"
    nta["areaSourceReference"] = "ISSO 54 v2.0 EP-W206 p. 29: 48 appartementen, Ag 4608 m2"
    vent = nta["ventilation"]
    vent["apartmentBuilding"] = True
    vent["infiltration"]["buildingType"] = "multi_storey_whole_building"
    nta["hotWater"]["generator"] = {"kind": "electric_instantaneous"}


def collective48(project, test_id, connected_area, storeys):
    """A collective HR107 of 10 kW; pipes through unheated spaces of unknown
    length, insulated, valves insulated; pump power unknown, no control, no
    heat meter (p. 29). The boiler location is not given: the reference
    location (inside the envelope) is kept; for a collective boiler the
    table 9.25 row does not depend on it."""
    nta = project["ntaCalculation"]
    nta["generator"]["boiler"]["role"] = "collective"
    nta["generator"]["boiler"]["equipmentReference"] = f"ISSO 54 v2.0 {test_id} p. 29: gemeenschappelijke HR107-ketel, 10 kW"
    nta["generator"]["auxiliary"] = {"electricallyConnectedDevices": 1, "nominalPowerKw": 10.0, "sourceReference": f"ISSO 54 v2.0 {test_id} p. 29: HR107, 10 kW"}
    nta["collectiveConnection"] = {"connectedUsableAreaM2": connected_area, "sourceReference": f"ISSO 54 v2.0 {test_id} p. 29: aangesloten gebruiksoppervlakte"}
    system = nta["distributionSystem"]
    system["installation"] = "collective"
    system["connectedStoreys"] = storeys
    del system["unheatedPipeLengthM"]
    system["pump"] = {"method": "calculated", "heatMeterPresent": False, "sourceReference": f"ISSO 54 v2.0 {test_id} p. 29: pompvermogen onbekend, geen regeling, geen warmtemeter"}
    system["sourceReference"] = f"ISSO 54 v2.0 {test_id} p. 29: leidingen door onverwarmde ruimte, lengte onbekend (9.36), geisoleerd, kleppen en beugels geisoleerd"


def w206a(project):
    building48(project)
    collective48(project, "EP-W206a", BUILDING48, 12)


# EP-W206c: one collective boiler per 4 adjacent apartments, 12 boilers of
# 10 kW, each serving 2 storeys (p. 29-30): 12 identical systems (§9.1) that
# together serve the whole building (the kernel's connected area covers the
# calculated zone; each appliance is one twelfth).
def w206c(project):
    building48(project)
    collective48(project, "EP-W206c", BUILDING48, 2)
    project["ntaCalculation"]["identicalSystems"] = 12


# EP-W206b: the 32 apartments of the lower four apartment layers (8 storeys)
# on the collective boiler, the upper 16 (4 storeys) with an individual HR107
# each, no pipes through unheated spaces (p. 29). Two calculation zones: the
# lower zone takes 2/3 of the facades (as the test states), the floor and 128
# of the 192 windows; the upper zone the roof, 1/3 of the facades and 64
# windows. The upper zone is served by 16 identical individual HR107 as an
# additional heating system (§9.2); each zone has its own ventilation and
# internal-gains input for its apartments.
UPPER48 = "w206-boven"


def w206b(project):
    w206a(project)
    zone = project["zones"][0]
    upper = copy.deepcopy(zone)
    zone.update(floorArea=32 * 96.0, volume=8294.4, height=21.6)
    upper.update(id=UPPER48, name="Bovenste vier bouwlagen", floorArea=16 * 96.0, volume=4147.2, height=10.8)
    lower_surfaces, upper_surfaces = [], []
    for item in zone["surfaces"]:
        if item["id"] == "dak":
            upper_surfaces.append(dict(item, zoneId=UPPER48))
            continue
        if item["id"] == "bg-vloer":
            lower_surfaces.append(item)
            continue
        below = dict(item, area=round(item["area"] * 2 / 3, 1))
        above = dict(item, id=f"{item['id']}-boven", area=round(item["area"] / 3, 1), zoneId=UPPER48)
        if item["id"] == "gevel-zuid":
            below["windows"] = item["windows"][:128]
            above["windows"] = [dict(w, surfaceId="gevel-zuid-boven") for w in item["windows"][128:]]
        lower_surfaces.append(below)
        upper_surfaces.append(above)
    zone["surfaces"] = lower_surfaces
    upper["surfaces"] = upper_surfaces
    project["zones"].append(upper)
    nta = project["ntaCalculation"]
    nta["collectiveConnection"]["connectedUsableAreaM2"] = 32 * 96.0
    nta["distributionSystem"]["connectedStoreys"] = 8
    ventilation = nta.pop("ventilation")
    vertical = nta.pop("verticalPipes")[0]
    entries = []
    for zone_id, count, storeys in ((zone["id"], 32, 8), (UPPER48, 16, 4)):
        zone_ventilation = copy.deepcopy(ventilation)
        zone_ventilation.update(zoneId=zone_id, usableFloorAreaM2=count * 96.0, dwellingCount=count, functions=[{"function": "residential", "areaM2": count * 96.0}])
        entries.append(
            {
                "zoneId": zone_id,
                "verticalPipes": [dict(vertical, id=f"{zone_id}-leiding", storeys=storeys)],
                "ventilationFlows": [],
                "ventilation": zone_ventilation,
                "internalGains": dict(nta["internalGains"], dwellingCount=count, sourceReference=f"{count} appartementen"),
            }
        )
    nta["zoneData"] = entries
    individual = copy.deepcopy(REFERENCE["ntaCalculation"])
    individual["generator"]["boiler"]["generatorId"] = "hr107-boven"
    individual["generator"]["boiler"]["equipmentReference"] = "ISSO 54 v2.0 EP-W206b p. 30: individuele HR107 per appartement in de bovenste vier bouwlagen"
    individual["distributionSystem"]["sourceReference"] = "ISSO 54 v2.0 EP-W206b p. 30: individuele ketel, geen leidingen door onverwarmde ruimten"
    nta["additionalHeatingSystems"] = [
        {
            "zoneIds": [UPPER48],
            "generator": individual["generator"],
            "distributionSystem": individual["distributionSystem"],
            "identicalSystems": 16,
        }
    ]


# EP-W203o: a local gas-fired air heater HR104 of 5 kW (table 9.25), no
# distribution system, air heating with 4 heaters: one with an axial and three
# with a radial recirculation fan (p. 25). Their fans and controls are the
# table 9.12 auxiliary energy of 9.23, which is part of the emission (2022
# p. 282), so the generator itself has no electrically connected device
# (9.92 would count it twice). Room control as the reference (per room).
W203O = [
    setp(
        GEN,
        {
            "kind": "forfait_heater",
            "heaterKind": "air_heater_hr104",
            "fuel": "natural_gas",
            "equipmentReference": "ISSO 54 v2.0 EP-W203o p. 26: lokale gasgestookte luchtverwarmer HR104, 5 kW",
            "auxiliary": {"electricallyConnectedDevices": 0, "nominalPowerKw": 5.0, "sourceReference": "ISSO 54 v2.0 EP-W203o: hulpenergie van ventilatoren en regeling in de afgifte (9.23, tabel 9.12)"},
        },
    ),
    setp(
        f"{NTA}/emission",
        {
            "system": "air_heating",
            "balancing": "not_applicable",
            "control": "individual_room_thermostats",
            "airHeaters": {
                "kind": {"kind": "direct", "radialFan": False},
                "count": 1,
                "further": [{"kind": {"kind": "direct", "radialFan": True}, "count": 3}],
                "sourceReference": "ISSO 54 v2.0 EP-W203o p. 26: 4 luchtverwarmers, 1 met axiale en 3 met radiale recirculatieluchtventilator; Q_h;b geschat (tabel 9.12)",
            },
            "sourceReference": "ISSO 54 v2.0 EP-W203o p. 26: luchtverwarming, regeling per ruimte als de referentie",
        },
    ),
    remove(f"{NTA}/distributionSystem"),
    setp(f"{NTA}/distribution", {"method": "declared", "monthlyLossKwh": [0.0] * 12, "sourceReference": "ISSO 54 v2.0 EP-W203o p. 26: geen distributiesysteem"}),
]

# EP-W006h: the east facade borders an unheated space (AOR) without further
# specification; every facade R_c 0,35, the rest as EP-W001, floor on ground
# (p. 9). The reference uses the forfait thermal bridges, so 8.4.1 note 5
# with C.10 applies (2022 p. 240, 748): the partition joins H_D;for as
# U_C.b_U. The AOR is not specified; b_U follows 8.53 with the normative
# H_ue = 5 x A_T;iu of formula I.8 (2025+C1 p. 841; I.8 in 2022 annex I),
# entered as one transmission element of U 5,0 over A_T;iu (the east facade
# of 32,4 m2) without ventilation. Assumption: annex I.2.4 is strictly for
# the forfait Rc/U values of I.2.1-I.2.3 (2022 p. 240) and, in 2025+C1, for
# the basisopname (p. 266).
def aor_east(project):
    project["constructions"].append({"id": "c-gevel-035", "name": "Gevel Rc 0,35", "layers": [], "rcValue": 0.35, "uValue": u_from_rc(0.35, 0.17)})
    for item in project["zones"][0]["surfaces"]:
        if item["type"] == "wall":
            item["constructionId"] = "c-gevel-035"
    east = surface(project, "gevel-oost")
    east["thermalBoundary"] = "unheated_space"
    east["unheatedSpaceId"] = "aor-oost"
    area = east["area"]
    project["unheatedSpaces"] = [
        {
            "id": "aor-oost",
            "name": "Aangrenzende onverwarmde ruimte oost",
            "outside": {
                "transmission": {"elements": [{"id": "aor-buiten", "areaM2": round(area, 3), "uValueWPerM2k": 5.0, "sourceReference": "ISSO 54 v2.0 EP-W006h p. 9: AOR zonder specificatie; formule I.8 (2025+C1 p. 841): H_ue = 5 x A_T;iu (aanname, zie docs/nta8800-isso54-voorbereiding.md)"}]},
                "ventilation": {"method": "flow", "airflowM3PerH": 0.0, "sourceReference": "formule I.8: H_ue = 5 x A_T;iu omvat transmissie en ventilatie"},
                "otherZonesConductanceWPerK": 0.0,
            },
        }
    ]


CASES += [
    ("EPW006h", 9, variant(aor_east)),
    ("EPW203o", 26, W203O),
    ("EPW206a", 30, variant(w206a)),
    ("EPW206b", 30, variant(w206b)),
    ("EPW206c", 30, variant(w206c)),
]


# --- EP-W104: further ventilation aspects (p. 22-23) ---
# EP-W104a: an open conventional (CR) boiler with a pilot flame inside the
# envelope, 35 kW, 90/70 (HT), radiators against the outer wall, control in
# the main room, no hydronic balancing; hot water a bath geyser with Gaskeur
# CW3 of 15 kW, also open with a flue; system A1 (p. 22-23). Both open
# appliances draw combustion air (table 11.11 gas type B, table 11.12 rows
# for a central-heating boiler and a bath geyser at their nominal input).
# The 35 kW acts only there: an individual boiler takes the 9.85 constants
# for its auxiliary energy, not a nominal power.
def open_appliance(appliance_id, kind, power, why):
    return {"id": appliance_id, "kind": kind, "class": "gas_type_b", "nominalInputKw": power, "sourceReference": why}


W104A = [
    unit("a1", "no_ducts"),
    NO_PASSIVE,
    setp(f"{BOILER}/kind", "conventional"),
    setp(f"{BOILER}/pilotFlamePresent", True),
    setp(f"{BOILER}/equipmentReference", "ISSO 54 v2.0 EP-W104a p. 22: individuele CR-ketel, open toestel met rookgasafvoer, waakvlam, 35 kW"),
    temperature_class("90_70"),
    radiators("EP-W104a", "radiators_or_convectors", "two_pipe60_k_or_unknown", "outer_wall", "unknown", why="ISSO 54 v2.0 EP-W104a p. 22-23: radiatoren tegen buitenwand, regeling in het hoofdvertrek, geen waterzijdige inregeling"),
    setp(f"{NTA}/emission/control", "main_room_thermostat"),
    hw_generator({"kind": "gas_appliance", "appliance": "water_heater_gaskeur_cw", "measuredClass": "class3"}),
    setp(f"{HW}/equipmentReference", "ISSO 54 v2.0 EP-W104a p. 23: badgeiser met Gaskeur CW3, 15 kW, open toestel met rookgasafvoer"),
    setp(
        f"{VENT}/combustionAppliances",
        [
            open_appliance("cr-ketel", "gas_central_heating_boiler", 35.0, "ISSO 54 v2.0 EP-W104a p. 22: CR-ketel, open, 35 kW"),
            open_appliance("badgeiser", "gas_bath_geyser", 15.0, "ISSO 54 v2.0 EP-W104a p. 23: badgeiser, open, 15 kW"),
        ],
    ),
] + boiler_temperature(90, 70)


# EP-W104g: a corner apartment on the lowest layer of a multi-family
# building: the reference without the roof and the east facade (heated
# neighbours), perimeter 22 m, building height 10,8 m, construction year
# 1945 (infiltration), heating pipes insulated "before 1995" (table 9.x row
# before 1980 or unknown: the test does not narrow it down); D4a with a
# declared 85 % (NEN-EN 13142, dissipation included), Luka C, insulated duct
# 1 m, central, bypass 100 % with cold recovery on the declaration; forfait
# DC fans of 2020, no flow control, no passive cooling, no AHU (p. 23).
def corner_bottom(project):
    zone = project["zones"][0]
    zone["surfaces"] = [s for s in zone["surfaces"] if s["id"] not in ("dak", "gevel-oost")]
    nta = project["ntaCalculation"]
    nta["groundFloors"][0]["exposedPerimeterM"] = 22.0
    nta["groundFloors"][0]["sourceReference"] = "ISSO 54 v2.0 EP-W104g p. 23: perimeter 22 m, oostzijde grenst aan een verwarmde woning"
    nta["dwellingType"] = "apartment_building"
    nta["constructionYear"] = 1945
    nta["distributionSystem"]["pipeTransmittance"]["insulation"] = {"state": "insulated", "period": "before1980_or_unknown"}
    vent = nta["ventilation"]
    vent["apartmentBuilding"] = True
    vent["buildingHeightM"] = 10.8
    vent["constructionYear"] = 1945
    vent["infiltration"]["buildingType"] = "storey_end_lower_or_intermediate"
    vent["fans"] = {"method": "forfait", "current": "dc", "manufactureYear": 2020}
    vent.pop("maximumCapacityForCooling", None)
    vent["system"]["unit"] = {
        "variant": "d4a",
        "ducts": "luka_a_b_c",
        "equipmentReference": "ISSO 54 v2.0 EP-W104g p. 23: D4a, Luka C, centraal",
        "heatRecovery": recovery(
            declared_recovery(0.85, "en13142", "ISSO 54 v2.0 EP-W104g p. 23: WTW-verklaring 85 % volgens EN 13142, dissipatie verdisconteerd"),
            {"kind": "full", "coldRecoveryEvidence": "ISSO 54 v2.0 EP-W104g p. 23: koudeterugwinning via de WTW op de kwaliteitsverklaring"},
            length=1.0,
        ),
    }


CASES += [
    ("EPW104a", 22, W104A),
    # EP-W104b-d, f: changes on the reference EP-W001 (p. 23).
    ("EPW104b", 23, [installed(150.0, "ISSO 54 v2.0 EP-W104b p. 23: ontwerpdebiet 150 dm3/s")]),
    ("EPW104c", 23, [declared_fans({"method": "nominal", "nominalPowerW": 100.0}, "ISSO 54 v2.0 EP-W104c p. 23: nominaal vermogen ventilatoren 100 W")]),
    # EP-W104d: 80 W motor, U.I.cos(phi) = 220 V x 0,6 A x 0,8 = 105,6 W for a
    # single-phase AC motor (11.136).
    ("EPW104d", 23, [declared_fans({"method": "motor", "motorPowerW": 80.0, "electricalInputW": 105.6}, "ISSO 54 v2.0 EP-W104d p. 23: 80 W, 220 V, 0,6 A, cos phi 0,8, eenfasewisselstroommotor")]),
    (
        "EPW104f",
        23,
        [unit("c4b", "luka_a_b_c"), NO_PASSIVE, declared_fans({"method": "nominal", "nominalPowerW": 100.0}, "ISSO 54 v2.0 EP-W104f p. 23: C4b, nominaal vermogen ventilatoren 100 W")],
    ),
    ("EPW104g", 23, variant(corner_bottom)),
]


# --- EP-W406o/p/t: further hot-water heat pumps (p. 39) ---
# EP-W406o/p: an exhaust-air heat-pump water heater with a quality
# declaration, for hot water only: COP 3 as the declared eta_W;gen (rounded
# down to 0,05; f_prac 0,95 of 13.152 for a non-forfait value, which is the
# f_prac the test gives), P_nom 5 and 0,3 kW (13.141), q_ve;hp;W 200 m3/h
# from the declaration (13.148a). The "aandeel hernieuwbare energie 1000
# kWh" of the test is not an input: the kernel derives the renewable share
# itself. EP-W406p needs back-up heating (the test says so): the heat pump's
# electric element heats the remainder, an electric instantaneous heater
# (table 13.25, 0,95) as the further generator, since no vessel is given;
# its power is not given either and 13.141 needs one: 2 kW is assumed
# (fictitious), enough to cover the remainder.
def declared_exhaust(test_id, power, backup=False):
    patch = EXHAUST_C1 + [
        hw_generator(
            {
                "kind": "heat_pump",
                "exhaustAirSource": True,
                "declared": {"value": 3.0, "sourceReference": f"ISSO 54 v2.0 {test_id} p. 39: kwaliteitsverklaring COP 3, fprac 0,95"},
            }
        ),
        setp(f"{HW}/nominalPowerKw", power),
        setp(f"{HW}/exhaustAir", {"ventilationSuitable": True, "declaredFlowM3PerH": 200.0}),
    ]
    if backup:
        patch.append(
            setp(
                f"{HW}/additionalGenerators",
                [
                    {
                        "generator": {"kind": "electric_instantaneous"},
                        "nominalPowerKw": 2.0,
                        "equipmentReference": f"ISSO 54 v2.0 {test_id} p. 39: bijverwarming nodig, elektrisch; vermogen niet gegeven, 2 kW aangenomen (fictief), geen vat gegeven",
                    }
                ],
            )
        )
    return patch


# EP-W406t: an outdoor-air heat pump (table 9.28) for heating and hot water
# with a separate 150 l vessel, label D, T-pieces insulated, inside the
# envelope, 0,3 kW; it supplies 64,3 % of the hot water (13.146). An HR107
# with CW4 (10 kW), hot water only, supplies the rest (p. 39).
W406T = [
    heat_pump("EP-W406t", "outdoor_air", 45.0, high=True),
    renewable("EP-W406t", True),
    hw_generator({"kind": "indirect_heat_pump", "alsoSpaceHeating": True}),
    setp(f"{HW}/nominalPowerKw", 0.3),
    setp(f"{HW}/declaredShare", {"points": [{"annualKwh": 1000.0, "share": 0.643}], "sourceReference": "ISSO 54 v2.0 EP-W406t p. 39: de warmtepomp levert 64,3 % van de tapwatervraag"}),
    setp(f"{HW}/storage", [vessel(150.0, LABEL("d"), 2, True)]),
    setp(
        f"{HW}/additionalGenerators",
        [
            {
                "generator": {"kind": "gas_appliance", "appliance": "combi_gaskeur_hr_cw", "measuredClass": "class4"},
                "nominalPowerKw": 10.0,
                "equipmentReference": "ISSO 54 v2.0 EP-W406t p. 39: HR107-ketel met CW4, alleen tapwater, 10 kW",
            }
        ],
    ),
]

CASES += [
    ("EPW406o", 39, declared_exhaust("EP-W406o", 5.0)),
    ("EPW406p", 39, declared_exhaust("EP-W406p", 0.3, backup=True)),
    ("EPW406t", 39, W406T),
]


# --- EP-W407: generation for hot water in an apartment building (p. 41-42) ---
# The building of EP-W206 (48 apartments, 12 storeys, A_g 4608 m2). a-d are
# the whole building, e-h the same installations for one middle apartment
# on an intermediate layer (only the front and back facades are external),
# i the whole building on a CHP.
def individual48(project):
    """EP-W407a: every apartment has the installations of EP-W001."""
    building48(project)
    project["ntaCalculation"]["hotWater"]["generator"] = copy.deepcopy(REFERENCE["ntaCalculation"]["hotWater"]["generator"])
    project["ntaCalculation"]["identicalSystems"] = 48


def middle_apartment(project):
    """One apartment between heated neighbours on all sides but front and back."""
    zone = project["zones"][0]
    zone["surfaces"] = [s for s in zone["surfaces"] if s["id"] in ("gevel-zuid", "gevel-noord")]
    nta = project["ntaCalculation"]
    del nta["groundFloors"]
    nta["dwellingType"] = "apartment_building"
    vent = nta["ventilation"]
    vent["apartmentBuilding"] = True
    vent["buildingHeightM"] = 32.4
    vent["infiltration"]["buildingType"] = "storey_middle_lower_or_intermediate"


def collective_hot_water(project, test_id, boiler="hr107", inside=False, storeys=12):
    """EP-W407b/d: a collective boiler of 100 kW for heating and hot water
    with an indirectly fired 2000 l vessel, T-pieces insulated (f_sto;dis;ls
    2; label not given, produced from 2018 as EP-W402), circulation 28/25
    with 20 mm insulation, none in unheated spaces, fittings insulated, pump
    unknown and uncontrolled, no heat meters, no delivery sets (p. 41)."""
    nta = project["ntaCalculation"]
    gen = nta["generator"]
    gen["boiler"]["role"] = "collective"
    gen["boiler"]["kind"] = boiler
    gen["boiler"]["location"] = "inside_thermal_boundary" if inside else "outside_thermal_boundary"
    gen["boiler"]["pilotFlamePresent"] = False
    gen["boiler"]["equipmentReference"] = f"ISSO 54 v2.0 {test_id}: gemeenschappelijke {boiler.upper()}-ketel, 100 kW, {'binnen' if inside else 'buiten'} de verwarmde zone"
    gen["auxiliary"] = {"electricallyConnectedDevices": 1, "nominalPowerKw": 100.0, "sourceReference": f"ISSO 54 v2.0 {test_id}: 100 kW"}
    nta["collectiveConnection"] = {"connectedUsableAreaM2": BUILDING48, "sourceReference": f"ISSO 54 v2.0 {test_id}: 48 appartementen, 4608 m2"}
    system = nta["distributionSystem"]
    system["installation"] = "collective"
    system["connectedStoreys"] = storeys
    system["pump"] = {"method": "calculated", "heatMeterPresent": False, "sourceReference": f"ISSO 54 v2.0 {test_id}: pompvermogen en EEI onbekend, geen regeling, geen warmtemeters"}
    hot_water = nta["hotWater"]
    hot_water["collective"] = {"buildingUsableFloorAreaM2": BUILDING48, "sourceReference": f"ISSO 54 v2.0 {test_id}: 48 appartementen, 4608 m2"}
    hot_water["generator"] = {"kind": "indirect_boiler", "boiler": "vr" if boiler == "vr" else "hr107", "oil": False, "insideBoundary": inside, "alsoSpaceHeating": True}
    hot_water["storage"] = [vessel(2000.0, {"method": "unknown_label", "producedFrom2018": True}, 2, inside)]
    hot_water["circulation"] = {
        "outerDiameterMm": 28.0,
        "insulation": "mm20",
        "fittingsInsulated": True,
        "unheatedLengthM": 0.0,
        "floorCount": 12,
        "connectedDwellings": 48,
        "pump": {"control": "uncontrolled_or_unknown"},
        "sourceReference": f"ISSO 54 v2.0 {test_id}: circulatie 28/25, 20 mm isolatie, geen leidingen door onverwarmde ruimten, kleppen en beugels geisoleerd, pomp onbekend zonder regeling",
    }
    hot_water["equipmentReference"] = f"ISSO 54 v2.0 {test_id}: gemeenschappelijke installatie met indirect gestookt vat van 2000 l"


def direct_storage(project, test_id):
    """EP-W407c: hot water from a collective directly fired gas storage
    heater above 150 kW (13.8.4.5.4, 0,50); heating as EP-W407b; the vessel
    data of EP-W407b do not apply (p. 41)."""
    collective_hot_water(project, test_id)
    hot_water = project["ntaCalculation"]["hotWater"]
    hot_water["generator"] = {"kind": "large_direct_storage", "gasFired": True}
    hot_water["storage"] = []
    hot_water["equipmentReference"] = f"ISSO 54 v2.0 {test_id}: gemeenschappelijk direct gestookt vat, vermogen groter dan 150 kW"


# EP-W407i: a CHP of 15 kW electric and 30 kW thermal for heating (LT) and,
# through a 2000 l vessel with insulated T-pieces, for hot water; measured
# eps_th 0,6 and eps_el 0,3 replace the table 9.31 row (p. 42). Built after
# 2006 is assumed (the test gives no year; the declared factors replace the
# row either way).
def chp48(project):
    building48(project)
    declared = {"thermal": 0.6, "electric": 0.3, "declarationReference": "ISSO 54 v2.0 EP-W407i p. 42: gemeten epsilon_th 0,6 en epsilon_el 0,3"}
    nta = project["ntaCalculation"]
    nta["generator"] = {
        "kind": "chp",
        "chp": {"powerKw": 15.0, "builtAfter2006": True, "lowTemperature": True},
        "declaredEfficiencies": declared,
        "auxiliary": {"electricallyConnectedDevices": 1, "nominalPowerKw": 30.0, "sourceReference": "ISSO 54 v2.0 EP-W407i p. 42: thermisch vermogen 30 kW"},
        "equipmentReference": "ISSO 54 v2.0 EP-W407i p. 42: WKK, 15 kW elektrisch, 30 kW thermisch, LT",
    }
    nta["collectiveConnection"] = {"connectedUsableAreaM2": BUILDING48, "sourceReference": "ISSO 54 v2.0 EP-W407i p. 42: 48 appartementen, 4608 m2"}
    system = nta["distributionSystem"]
    system["installation"] = "collective"
    system["connectedStoreys"] = 12
    system["pump"] = {"method": "calculated", "heatMeterPresent": False, "sourceReference": "distributiepomp, vermogen en EEI onbekend"}
    hot_water = nta["hotWater"]
    hot_water["collective"] = {"buildingUsableFloorAreaM2": BUILDING48, "sourceReference": "ISSO 54 v2.0 EP-W407i p. 42: 48 appartementen, 4608 m2"}
    hot_water["generator"] = {
        "kind": "chp",
        "chp": {"powerKw": 15.0, "builtAfter2006": True, "lowTemperature": True},
        "alsoSpaceHeating": True,
        "declaredEfficiencies": declared,
        "equipmentReference": "ISSO 54 v2.0 EP-W407i p. 42: de WKK verwarmt een vat van 2000 l",
    }
    # 13.8.4.7.4 (2022 p. 621): the losses of the indirectly heated vessel
    # follow 13.6, as for a boiler; label not given, produced from 2018.
    hot_water["storage"] = [vessel(2000.0, {"method": "unknown_label", "producedFrom2018": True}, 2, False)]
    hot_water["equipmentReference"] = "ISSO 54 v2.0 EP-W407i p. 42: WKK met voorraadvat van 2000 l, T-stukken geisoleerd"


def apartment_of(change):
    """EP-W407e-h: the installation of a-d for one middle apartment."""

    def apply(project):
        change(project)
        nta = project["ntaCalculation"]
        # One apartment: the area-scaled inputs of building48 go back to one
        # dwelling of 96 m2; the collective areas keep the whole building.
        scale(project, area=96.0, volume=259.2, height=5.4, roof=48.0, floor=48.0, south=43.2, north=43.2, east_west=32.4, windows_=4, perimeter=28.0, storeys=2, building_height=32.4)
        middle_apartment(project)
        dwellings(project, 1)
        nta.pop("identicalSystems", None)
        if nta["distributionSystem"].get("installation") == "collective":
            nta["distributionSystem"]["connectedStoreys"] = 12

    return apply


W407 = [
    ("a", individual48),
    ("b", lambda p: (building48(p), collective_hot_water(p, "EP-W407b"))),
    ("c", lambda p: (building48(p), direct_storage(p, "EP-W407c"))),
    ("d", lambda p: (building48(p), collective_hot_water(p, "EP-W407d", boiler="vr", inside=True))),
]
CASES += [(f"EPW407{letter}", 41, variant(change)) for letter, change in W407]
CASES += [(f"EPW407{chr(ord(letter) + 4)}", 42, variant(apartment_of(change))) for letter, change in W407]
CASES += [("EPW407i", 42, variant(chp48))]


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

# Control for EP-W203l-n (review 19, D1): the two-zone split alone, with the
# reference HR107 of EP-W001, must calculate as EP-W001 itself. The kernel
# test crates/nta8800-core/tests/isso54_controls.rs applies this patch.
CONTROL = ROOT / "training-data" / "isso54" / "EPW001-two-zones-control.json"
CONTROL.write_text(
    json.dumps(
        {
            "description": "EP-W001 als twee rekenzones (splitsing van EP-W203l-n zonder de hybride opwekker); moet rekenen als EP-W001.",
            "projectFile": "EPW001.json",
            "normVersion": "2022",
            "projectPatch": variant(two_zones),
        },
        ensure_ascii=False,
        indent=2,
    )
    + "\n"
)
print(f"{CONTROL.relative_to(ROOT)}: two-zone control")
