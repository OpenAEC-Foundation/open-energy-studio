/*
 * Sentences for the kernel's technical `detail` texts (see `kernelDetail.ts`).
 * `{{p1}}`, `{{p2}}`, … are the values the kernel reported, in capture order.
 */

/** Dutch sentences. */
export const kernelDetailLabelsNl: Record<string, string> = {
  'kernel.detail.technical': 'Technisch detail',
  'kernel.detail.aboveBound': '{{p1}} is groter dan {{p2}}',
  'kernel.detail.areaPerDwelling': '{{p1}} m² per woning',
  'kernel.detail.lossAreaRatio': 'A_ls/A_g = {{p1}}',
  'kernel.detail.constructionYearMismatch': 'Registratie {{p1}}, rekeninvoer {{p2}}',
  'kernel.detail.ventilationYearBefore': 'Ventilatie (tabel 11.13) {{p1}} ligt vóór het bouwjaar {{p2}}',
  'kernel.detail.hotWaterEfficiency': 'Q_W;nd ≈ {{p1}} kWh/jaar tegenover {{p2}} kWh/jaar opgegeven brandstof: η_W ≈ {{p3}}, groter dan 1',
  'kernel.detail.ventilationBelowRequired': 'H_ve {{p1}} W/K ligt onder ρc·q_V;ODA;req ≈ {{p2}} W/K ({{p3}} m³/h, 11.22 met de laagste f_ctrl·f_sys {{p4}} uit tabel 11.5) van het mechanische systeem zonder warmteterugwinning',
  'kernel.detail.floorResistanceBelowRsi': '{{p1}} m²K/W ligt onder R_si = 0,17; de invoer is R_si + R_c van de vloer',
  'kernel.detail.floorPerimeterImplausible': 'P = {{p1}} m bij A = {{p2}} m² betekent een gemiddelde vloerbreedte onder {{p3}} m (B′ = {{p4}} m)',
  'kernel.detail.sunroomDiffers': 'Opgegeven b_U {{p1}} en H_zi;ztu {{p2}} W/K; onverwarmde ruimte {{p3}} geeft b_U {{p4}} en H_zi;ztu {{p5}} W/K (8.4.1)',
  'kernel.detail.basementDeltaU': '8.38 neemt ΔU_for uit 8.2.1 (8.3): {{p1}} W/(m²K)',
  'kernel.detail.bacsWithoutEvidence': '§5.5.8: zonder GBS-gegevens (systeemvermogens ≤ 290 kW of GBS-bewijs) is f_BACS 1,05',
  'kernel.detail.utilityOpenCeiling': 'Tabel 7.10 a: utiliteitsgebouwen nemen de kolom gesloten of verlaagd plafond, tenzij ten minste 15 % van een vrijhangend plafond open is',
  'kernel.detail.detailedBridgesNone': 'Gedetailleerde koudebrugroute (8.2.1) zonder lineaire koudebruggen naar buitenlucht: H_D bevat geen ψ·ℓ; voer de koudebruggen in of gebruik de forfaitaire ΔU_for (8.3)',
  'kernel.detail.verticalPipesConflicting': '7.3.3: de zonelijst [] (geen) spreekt de leidingen op projectniveau tegen; verwijder een van beide',
  'kernel.detail.verticalPipesUnknown': '7.3.3: geef de leidingen op, [] voor geen; bij onbekend één fictieve ongeïsoleerde leiding per bouwlaag van de zone (woning buiten een woongebouw), één per woning (woongebouw) of één per toiletgroep met N = H/3, naar gebruiksoppervlakte verdeeld over de zones (utiliteitsgebouw)',
};

/** English sentences. */
export const kernelDetailLabelsEn: Record<string, string> = {
  'kernel.detail.technical': 'Technical detail',
  'kernel.detail.aboveBound': '{{p1}} is above {{p2}}',
  'kernel.detail.areaPerDwelling': '{{p1}} m² per dwelling',
  'kernel.detail.lossAreaRatio': 'A_ls/A_g = {{p1}}',
  'kernel.detail.constructionYearMismatch': 'Registration {{p1}}, calculation input {{p2}}',
  'kernel.detail.ventilationYearBefore': 'Ventilation (table 11.13) {{p1}} is before the construction year {{p2}}',
  'kernel.detail.hotWaterEfficiency': 'Q_W;nd ≈ {{p1}} kWh/yr against {{p2}} kWh/yr declared fuel: η_W ≈ {{p3}}, above 1',
  'kernel.detail.ventilationBelowRequired': 'H_ve {{p1}} W/K is below ρc·q_V;ODA;req ≈ {{p2}} W/K ({{p3}} m³/h, 11.22 with the lowest table 11.5 f_ctrl·f_sys {{p4}}) of the mechanical system without heat recovery',
  'kernel.detail.floorResistanceBelowRsi': '{{p1}} m²K/W is below R_si = 0.17; the input is R_si + R_c of the floor',
  'kernel.detail.floorPerimeterImplausible': 'P = {{p1}} m on A = {{p2}} m² implies a mean floor width below {{p3}} m (B′ = {{p4}} m)',
  'kernel.detail.sunroomDiffers': 'Declared b_U {{p1}} and H_zi;ztu {{p2}} W/K; unheated space {{p3}} gives b_U {{p4}} and H_zi;ztu {{p5}} W/K (8.4.1)',
  'kernel.detail.basementDeltaU': '8.38 takes ΔU_for of 8.2.1 (8.3): {{p1}} W/(m²K)',
  'kernel.detail.bacsWithoutEvidence': '§5.5.8: without the BACS block (system powers ≤ 290 kW or BACS evidence) f_BACS is 1.05',
  'kernel.detail.utilityOpenCeiling': 'Table 7.10 a: utility buildings take the closed or suspended ceiling column unless at least 15 % of a free-hanging ceiling is open',
  'kernel.detail.detailedBridgesNone': 'Detailed thermal-bridge route (8.2.1) without linear thermal bridges to outside air: H_D has no ψ·ℓ; enter the bridges or use the default ΔU_for (8.3)',
  'kernel.detail.verticalPipesConflicting': '7.3.3: the zone list [] (none) conflicts with the project-level pipes; remove one of the two',
  'kernel.detail.verticalPipesUnknown': '7.3.3: state the pipes, [] for none; when unknown enter one fictitious uninsulated pipe per storey of the zone (dwelling outside a residential building), one per dwelling (residential building) or one per toilet group with N = H/3, shared by usable area over the zones (utility building)',
};
