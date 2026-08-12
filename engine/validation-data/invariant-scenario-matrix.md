# Matrice de scénarios par invariant — `pv_module`

Généré par `pv_module::tests::invariants::the_scenario_census_is_written_to_disk`.
**Ne pas modifier à la main.**

Chaque ligne compte les scénarios *réellement exécutés*, pas les fonctions de test.
Un scénario déclare son résultat attendu avant exécution ; il est compté PASS
lorsque l'invariant a produit exactement ce résultat, explication et champs
nommés compris.

- Invariants recensés : **52**
- Scénarios exécutés : **207**
- PASS : **207** · FAIL : **0**
- Seuil requis : **3** scénarios par invariant — invariants en dessous : **0**

| INVARIANT | NOMBRE DE SCÉNARIOS | PASS | FAIL |
|---|---:|---:|---:|
| `INV-BIF-001` | 4 | 4 | 0 |
| `INV-BIF-002` | 4 | 4 | 0 |
| `INV-BIF-003` | 3 | 3 | 0 |
| `INV-BIF-004` | 3 | 3 | 0 |
| `INV-CND-001` | 6 | 6 | 0 |
| `INV-CND-002` | 4 | 4 | 0 |
| `INV-CND-003` | 6 | 6 | 0 |
| `INV-CND-004` | 4 | 4 | 0 |
| `INV-CND-005` | 4 | 4 | 0 |
| `INV-CND-006` | 5 | 5 | 0 |
| `INV-DER-001` | 4 | 4 | 0 |
| `INV-DER-002` | 3 | 3 | 0 |
| `INV-DER-003` | 6 | 6 | 0 |
| `INV-ELE-001` | 3 | 3 | 0 |
| `INV-ELE-002` | 3 | 3 | 0 |
| `INV-ELE-003` | 5 | 5 | 0 |
| `INV-ELE-004` | 3 | 3 | 0 |
| `INV-ELE-005` | 5 | 5 | 0 |
| `INV-ELE-006` | 3 | 3 | 0 |
| `INV-ELE-010` | 4 | 4 | 0 |
| `INV-ELE-011` | 3 | 3 | 0 |
| `INV-ELE-012` | 3 | 3 | 0 |
| `INV-GEO-001` | 3 | 3 | 0 |
| `INV-GEO-002` | 4 | 4 | 0 |
| `INV-GEO-003` | 4 | 4 | 0 |
| `INV-GEO-004` | 5 | 5 | 0 |
| `INV-GEO-005` | 3 | 3 | 0 |
| `INV-POS-001` | 5 | 5 | 0 |
| `INV-POS-002` | 4 | 4 | 0 |
| `INV-POS-003` | 4 | 4 | 0 |
| `INV-POS-004` | 4 | 4 | 0 |
| `INV-POS-005` | 4 | 4 | 0 |
| `INV-POS-006` | 5 | 5 | 0 |
| `INV-POS-007` | 5 | 5 | 0 |
| `INV-POS-008` | 5 | 5 | 0 |
| `INV-POS-009` | 3 | 3 | 0 |
| `INV-POS-010` | 3 | 3 | 0 |
| `INV-POS-011` | 3 | 3 | 0 |
| `INV-POS-012` | 6 | 6 | 0 |
| `INV-PRV-001` | 3 | 3 | 0 |
| `INV-PRV-002` | 6 | 6 | 0 |
| `INV-PRV-003` | 4 | 4 | 0 |
| `INV-STR-001` | 4 | 4 | 0 |
| `INV-STR-002` | 3 | 3 | 0 |
| `INV-STR-003` | 4 | 4 | 0 |
| `INV-STR-004` | 4 | 4 | 0 |
| `INV-TMP-001` | 3 | 3 | 0 |
| `INV-TMP-002` | 3 | 3 | 0 |
| `INV-TMP-003` | 3 | 3 | 0 |
| `INV-TMP-004` | 4 | 4 | 0 |
| `INV-TMP-005` | 4 | 4 | 0 |
| `INV-TMP-006` | 4 | 4 | 0 |
| **Total** | **207** | **207** | **0** |

## Détail des scénarios

| INVARIANT | SCÉNARIO | ATTENDU |
|---|---|---|
| `INV-BIF-001` | a CEC-style flag with nothing else | WARNING |
| `INV-BIF-001` | a stated bifaciality factor of 0.70 | silent |
| `INV-BIF-001` | rear-side power measured | silent |
| `INV-BIF-001` | an explicitly empty rear-side block | WARNING |
| `INV-BIF-002` | a factor of 0.70 | silent |
| `INV-BIF-002` | unity: a fully symmetric module | silent |
| `INV-BIF-002` | a factor of zero | INVALID |
| `INV-BIF-002` | a negative source value, clamped to zero | INVALID |
| `INV-BIF-003` | no rear-side power stated | silent |
| `INV-BIF-003` | 280 W rear against 400 W front | silent |
| `INV-BIF-003` | 500 W rear against 400 W front | WARNING |
| `INV-BIF-004` | a PERC architecture | silent |
| `INV-BIF-004` | a heterojunction architecture | silent |
| `INV-BIF-004` | an interdigitated back contact, whose rear carries the contacts | WARNING |
| `INV-CND-001` | standard test conditions at 25 degC | silent |
| `INV-CND-001` | 500 degC | INVALID |
| `INV-CND-001` | -200 degC, below absolute zero | INVALID |
| `INV-CND-001` | a temperature that is NaN | INVALID |
| `INV-CND-001` | exactly at the lower bound | silent |
| `INV-CND-001` | exactly at the upper bound | silent |
| `INV-CND-002` | no range declared | silent |
| `INV-CND-002` | -40 degC to 85 degC, correctly ordered | silent |
| `INV-CND-002` | the bounds are inverted | INVALID |
| `INV-CND-002` | a degenerate range, minimum equal to maximum | INVALID |
| `INV-CND-003` | 44 degC at the nominal condition | silent |
| `INV-CND-003` | 80 degC: above the usual settled range | WARNING |
| `INV-CND-003` | 10 degC: below the usual settled range | WARNING |
| `INV-CND-003` | NMOT lets the cell settle in the same way | WARNING |
| `INV-CND-003` | STC imposes the cell temperature, so the range does not apply | silent |
| `INV-CND-003` | no settled temperature recorded | silent |
| `INV-CND-004` | 1000 W/m2 at standard test conditions | silent |
| `INV-CND-004` | zero irradiance | INVALID |
| `INV-CND-004` | negative irradiance | INVALID |
| `INV-CND-004` | irradiance is NaN | INVALID |
| `INV-CND-005` | one nominal condition | silent |
| `INV-CND-005` | two different conditions | silent |
| `INV-CND-005` | the same condition declared twice | INVALID |
| `INV-CND-005` | five matrix points, many by construction | silent |
| `INV-CND-006` | 295 W at NOCT against 400 W at STC | silent |
| `INV-CND-006` | 450 W at NOCT, above the STC value | WARNING |
| `INV-CND-006` | equal to the STC value at 800 W/m2 | WARNING |
| `INV-CND-006` | no nominal power stated to compare | silent |
| `INV-CND-006` | no standard-condition power to compare against | silent |
| `INV-DER-001` | 20 % stated against 20 % derived | silent |
| `INV-DER-001` | 1 % apart: within the threshold | silent |
| `INV-DER-001` | 5 % apart: a different area was used and not stated | CONFLICT |
| `INV-DER-001` | no stated efficiency to contradict | silent |
| `INV-DER-002` | 20 % derived efficiency | silent |
| `INV-DER-002` | exactly at the bound, which is not a breach | silent |
| `INV-DER-002` | 60 %: power or area must be wrong | INVALID |
| `INV-DER-003` | above the bound, INV-DER-002 takes over | silent |
| `INV-DER-003` | 20 %, an ordinary commercial module | silent |
| `INV-DER-003` | 2 %: below anything sold | WARNING |
| `INV-DER-003` | 35 %: above any commercial module, below the bound | WARNING |
| `INV-DER-003` | the aperture is preferred and the finding says so | WARNING |
| `INV-DER-003` | a zero area yields no efficiency at all | silent |
| `INV-ELE-001` | 49.5 V open circuit against 41.5 V at the knee | silent |
| `INV-ELE-001` | equal voltages (no current flows at v_oc) | INVALID |
| `INV-ELE-001` | maximum-power voltage above open circuit | INVALID |
| `INV-ELE-002` | 10.4 A short circuit against 9.64 A at the knee | silent |
| `INV-ELE-002` | equal currents | INVALID |
| `INV-ELE-002` | maximum-power current above short circuit | INVALID |
| `INV-ELE-003` | ratio 0.838, the ordinary case | silent |
| `INV-ELE-003` | ratio 0.606, below the usual range | WARNING |
| `INV-ELE-003` | ratio 0.960, above the usual range | WARNING |
| `INV-ELE-003` | ratio just inside the lower bound | silent |
| `INV-ELE-003` | ratio just inside the upper bound | silent |
| `INV-ELE-004` | 400 W stated against 400.06 W implied (rounding) | silent |
| `INV-ELE-004` | 2 % apart: beyond rounding, both plausible | WARNING |
| `INV-ELE-004` | 10 % apart: a contradiction the engine will not arbitrate | CONFLICT |
| `INV-ELE-005` | fill factor 0.777 | silent |
| `INV-ELE-005` | fill factor 0.40, below the commercial range | WARNING |
| `INV-ELE-005` | fill factor 0.95, above the commercial range | WARNING |
| `INV-ELE-005` | a degenerate module yields NaN, not a finding | silent |
| `INV-ELE-005` | above 1, INV-ELE-006 takes over exclusively | silent |
| `INV-ELE-006` | fill factor 0.777 | silent |
| `INV-ELE-006` | fill factor exactly 1: the limit, not a breach | silent |
| `INV-ELE-006` | fill factor 1.17: physically impossible | INVALID |
| `INV-ELE-010` | 0.688 V over 72 junctions | silent |
| `INV-ELE-010` | 0.278 V per junction: count or voltage is wrong | WARNING |
| `INV-ELE-010` | the 144-half-cell trap halves the volts | WARNING |
| `INV-ELE-010` | no junction to divide by | silent |
| `INV-ELE-011` | 1500 V system against a 49.5 V module | silent |
| `INV-ELE-011` | equal: a string of exactly one module | silent |
| `INV-ELE-011` | 40 V system below the module's own 49.5 V | INVALID |
| `INV-ELE-012` | no fuse rating stated | silent |
| `INV-ELE-012` | 20 A fuse against 10.4 A short circuit | silent |
| `INV-ELE-012` | 5 A fuse below the short-circuit current | WARNING |
| `INV-GEO-001` | no area stated | silent |
| `INV-GEO-001` | 1.8 m2 aperture inside a 2 m2 overall area | silent |
| `INV-GEO-001` | 2.5 m2 aperture on a 2 m2 module | INVALID |
| `INV-GEO-002` | stated and derived areas agree exactly | silent |
| `INV-GEO-002` | 0.05 % apart: within datasheet rounding | silent |
| `INV-GEO-002` | 0.5 % apart: beyond rounding | WARNING |
| `INV-GEO-002` | the Sharp_ND_72ELUF shape: a factor of two apart | CONFLICT |
| `INV-GEO-003` | no nested area stated | silent |
| `INV-GEO-003` | cell < active < aperture | silent |
| `INV-GEO-003` | cell area above active area | INVALID |
| `INV-GEO-003` | active area above aperture area | INVALID |
| `INV-GEO-004` | no cell area stated | silent |
| `INV-GEO-004` | 72 cells of 166 mm: 0.0276 m2 each | silent |
| `INV-GEO-004` | 0.0069 m2 per junction: below any format | WARNING |
| `INV-GEO-004` | 0.069 m2 per junction: above any format | WARNING |
| `INV-GEO-004` | no junction to divide by | silent |
| `INV-GEO-005` | no depth stated | silent |
| `INV-GEO-005` | 30 mm thick on a 1 m wide module | silent |
| `INV-GEO-005` | 1.5 m thick on a 1 m wide module | WARNING |
| `INV-POS-001` | a coherent 400 W module | silent |
| `INV-POS-001` | zero power | INVALID |
| `INV-POS-001` | negative power | INVALID |
| `INV-POS-001` | power is NaN | INVALID |
| `INV-POS-001` | power is infinite | INVALID |
| `INV-POS-002` | a coherent 49.5 V module | silent |
| `INV-POS-002` | zero open-circuit voltage | INVALID |
| `INV-POS-002` | negative open-circuit voltage | INVALID |
| `INV-POS-002` | open-circuit voltage is NaN | INVALID |
| `INV-POS-003` | a coherent 10.4 A module | silent |
| `INV-POS-003` | zero short-circuit current | INVALID |
| `INV-POS-003` | negative short-circuit current | INVALID |
| `INV-POS-003` | short-circuit current is NaN | INVALID |
| `INV-POS-004` | a coherent 41.5 V maximum-power point | silent |
| `INV-POS-004` | zero maximum-power voltage | INVALID |
| `INV-POS-004` | negative maximum-power voltage | INVALID |
| `INV-POS-004` | maximum-power voltage is NaN | INVALID |
| `INV-POS-005` | a coherent 9.64 A maximum-power point | silent |
| `INV-POS-005` | zero maximum-power current | INVALID |
| `INV-POS-005` | negative maximum-power current | INVALID |
| `INV-POS-005` | maximum-power current is NaN | INVALID |
| `INV-POS-006` | a 2 m by 1 m module | silent |
| `INV-POS-006` | zero length | INVALID |
| `INV-POS-006` | zero width | INVALID |
| `INV-POS-006` | negative length | INVALID |
| `INV-POS-006` | length is NaN | INVALID |
| `INV-POS-007` | no depth stated (absence is not a defect) | silent |
| `INV-POS-007` | a stated 30 mm depth | silent |
| `INV-POS-007` | zero depth stated | INVALID |
| `INV-POS-007` | negative depth stated | INVALID |
| `INV-POS-007` | depth is NaN | INVALID |
| `INV-POS-008` | no mass stated | silent |
| `INV-POS-008` | a stated 21.5 kg mass | silent |
| `INV-POS-008` | zero mass stated | INVALID |
| `INV-POS-008` | negative mass stated | INVALID |
| `INV-POS-008` | mass is NaN | INVALID |
| `INV-POS-009` | 72 cells in series | silent |
| `INV-POS-009` | no cell in series | INVALID |
| `INV-POS-009` | a monolithic module (INV-POS-011 governs it) | silent |
| `INV-POS-010` | one parallel path | silent |
| `INV-POS-010` | two parallel paths (half-cut) | silent |
| `INV-POS-010` | no parallel path | INVALID |
| `INV-POS-011` | 154 scribed segments | silent |
| `INV-POS-011` | no series segment | INVALID |
| `INV-POS-011` | a discrete module (INV-POS-009 governs it) | silent |
| `INV-POS-012` | no area stated | silent |
| `INV-POS-012` | a coherent 2 m2 overall area | silent |
| `INV-POS-012` | zero overall area | INVALID |
| `INV-POS-012` | negative active area | INVALID |
| `INV-POS-012` | aperture area is NaN | INVALID |
| `INV-POS-012` | zero cell area | INVALID |
| `INV-PRV-001` | a datasheet figure with no provenance | WARNING |
| `INV-PRV-001` | the same figure, attributed | silent |
| `INV-PRV-001` | a measurement is as unattributed as any other | WARNING |
| `INV-PRV-002` | an estimate carrying its reason | silent |
| `INV-PRV-002` | an estimate with no reason recorded | INVALID |
| `INV-PRV-002` | an empty string is not a rationale | INVALID |
| `INV-PRV-002` | manufacturer data needs no rationale | silent |
| `INV-PRV-002` | a measurement needs no rationale | silent |
| `INV-PRV-002` | a derived value needs no rationale | silent |
| `INV-PRV-003` | a manufacturer figure in a physical field | silent |
| `INV-PRV-003` | a De Soto coefficient in a physical field | INVALID |
| `INV-PRV-003` | a model parameter is not a fitted one | silent |
| `INV-PRV-003` | a measurement is not a fitted parameter | silent |
| `INV-STR-001` | half cells with two parallel paths | silent |
| `INV-STR-001` | half cells with three parallel paths | WARNING |
| `INV-STR-001` | whole cells impose no pairing | silent |
| `INV-STR-001` | a monolithic module has no cell to cut | silent |
| `INV-STR-002` | neither diodes nor sub-strings stated | silent |
| `INV-STR-002` | three diodes for three sub-strings | silent |
| `INV-STR-002` | four diodes for three sub-strings | WARNING |
| `INV-STR-003` | no additional output on silicon | silent |
| `INV-STR-003` | two additional outputs on a tandem | silent |
| `INV-STR-003` | one additional output on silicon | WARNING |
| `INV-STR-003` | one additional output on cadmium telluride | WARNING |
| `INV-STR-004` | silicon with discrete cells: the ordinary case | silent |
| `INV-STR-004` | cadmium telluride monolithically scribed | silent |
| `INV-STR-004` | cadmium telluride with countable discrete cells | WARNING |
| `INV-STR-004` | silicon monolithic: the invariant is one-directional | silent |
| `INV-TMP-001` | alpha positive, as expected | silent |
| `INV-TMP-001` | alpha negative | WARNING |
| `INV-TMP-001` | alpha exactly zero | WARNING |
| `INV-TMP-002` | beta negative, as expected | silent |
| `INV-TMP-002` | beta positive | WARNING |
| `INV-TMP-002` | beta exactly zero | WARNING |
| `INV-TMP-003` | gamma negative, as expected | silent |
| `INV-TMP-003` | gamma positive | WARNING |
| `INV-TMP-003` | gamma exactly zero | WARNING |
| `INV-TMP-004` | 0.34 %/K, an ordinary silicon module | silent |
| `INV-TMP-004` | 0.05 %/K: below any known technology | WARNING |
| `INV-TMP-004` | 1.5 %/K: above any known technology | WARNING |
| `INV-TMP-004` | an absolute coefficient: the range does not apply | silent |
| `INV-TMP-005` | a source stating no unit for its coefficient | CONFLICT |
| `INV-TMP-005` | coherent coefficients are never ambiguous | silent |
| `INV-TMP-005` | an out-of-range magnitude is not an ambiguity | silent |
| `INV-TMP-005` | wrong signs are not an ambiguity either | silent |
| `INV-TMP-006` | |gamma| 0.34 above |beta| 0.27 | silent |
| `INV-TMP-006` | equal magnitudes: not an inversion | silent |
| `INV-TMP-006` | |gamma| 0.20 below |beta| 0.27: informational | INFO |
| `INV-TMP-006` | an absolute basis is not compared | silent |
