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
