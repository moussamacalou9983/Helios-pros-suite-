# Matrice des tests de propriétés — `electrical_model`

Générée par `cargo test -p electrical_model --test properties`.
**Ne pas éditer à la main** : ce fichier est une mesure, pas une
déclaration.

- Graine de campagne : `0x48454c494f535052`
- Cas par propriété : **256** (plancher `200`)
- Générateur : SplitMix64 (Steele, Lea & Flood, OOPSLA 2014)

## Propriétés vérifiées

| Propriété | Énoncé | Traduit | Cas | Échecs |
|---|---|---|---:|---:|
| `PB-01` | two independent methods agree within their error bounds | P-08, P-09 | 256 | 0 |
| `PB-02` | I(V) decreases, strictly wherever the drop exceeds the error | P-11 | 256 | 0 |
| `PB-03` | I(V) in [0, I_sc], and I(V_oc) = 0 within the residual bound | P-10 | 256 | 0 |
| `PB-04` | \|I(V+d) - I(V)\| bounded by the analytic slope times d | P-11 | 256 | 0 |
| `PB-05` | no NaN and no infinity crosses the API boundary | absolute invariant | 256 | 0 |
| `PB-10` | R_s = 0 matches equation (18) and takes no iteration | P-23, P-24 | 256 | 0 |
| `PB-11` | \|I - I_leakfree\| <= V/R_sh, the exact residual perturbation | P-11 | 256 | 0 |
| `PB-12` | I_ph -> 0 gives V_oc -> 0 and I_sc -> 0 without NaN | continuity | 256 | 0 |
| `PB-14` | \|f(I_returned)\| <= tau_f, hence \|I - I*\| <= tau_f | P-15 | 256 | 0 |
| `PB-15` | iterations <= 82 in every circumstance | P-19 | 256 | 0 |
| `PB-16` | df/dI <= -1 at every evaluated point, not only at the root | P-04 | 256 | 0 |
| `PB-17` | I_k in [I*, I_ph] for every k >= 1 | P-13 | 256 | 0 |
| `PB-18` | outside H: a refusal, never a result | H-1..H-7 | 256 | 0 |
| `PB-19` | determinism, to the bit | absolute invariant | 256 | 0 |

**14 propriétés, 3584 cas générés, 0 échec(s).**

## Propriétés différées

Listées pour que le nombre de propriétés vérifiées ne puisse pas être
confondu avec le nombre de propriétés spécifiées.

| Propriété | Énoncé | Motif du report |
|---|---|---|
| `PB-06` | P = V·I to 2e-12 relative | power is not computed in this sub-phase |
| `PB-07` | the MPP exists | no MPP search — forbidden until this kernel is validated |
| `PB-08` | the MPP is unique: two starts converge to one point | no MPP search — forbidden until this kernel is validated |
| `PB-09` | P(V_mp) >= P(V) for every drawn V | no MPP search — forbidden until this kernel is validated |
| `PB-13` | G = 0 exactly: null output, solver not called | irradiance belongs to the translation model, not the kernel |
| `PB-20` | psi strictly increasing on the domain | psi is defined from dP/dV_j, which does not exist yet |
| `PB-21` | dP/dV_j > 0 wherever psi < 0 | power derivatives belong to the MPP sub-phase |
| `PB-22` | d2P/dV_j2 < 0 wherever psi >= 0 | power derivatives belong to the MPP sub-phase |

**8 différées sur 22 spécifiées au §7.2.**
