# 26. Validation empirique du modèle `pv_module` contre les bases CEC et Sandia

**Statut** : validation de conception sur données réelles. Aucun code
Rust, aucune logique métier.

## 0. Pourquoi ce document

La spécification `25-specification-pv-module.md` §6.3 décrit le contenu
des bases CEC et Sandia **de mémoire**, et son §1.1 renvoie la
vérification sur données réelles à l'implémentation. Or ces deux bases
sont embarquées dans pvlib-python, déjà installé dans cet environnement
pour la validation du moteur solaire (Phase 1).

La vérification était donc réalisable **immédiatement**, et le principe
de reproductibilité (`docs/scientific-principles.md` §2) demande de
mesurer plutôt que d'affirmer. Ce document remplace des hypothèses par
des constats.

**Sources inspectées** (fichiers embarqués dans pvlib 0.15.2) :

| Base | Fichier | Volume |
|---|---|---|
| CEC | `sam-library-cec-modules-2019-03-05.csv` | **21 535 modules × 25 champs** |
| Sandia | `sam-library-sandia-modules-2015-6-30.csv` | **523 modules × 42 champs** |

> Ces bases sont des instantanés (2019 et 2015). Les versions actuelles
> publiées par le CEC et Sandia comportent davantage de modules ; la
> **structure** des champs, seule objet de cette validation, est stable.

---

## 1. Constat majeur — la règle §3.4 doit être corrigée

### 1.1 Ce qui a été mesuré

La spécification §3.4 (et ADR-0005 Décision 2) tranchait : `gross_area`
est **dérivé** de `longueur × largeur`, la valeur affichée n'étant
conservée que pour recoupement.

Test sur les 19 954 modules CEC renseignant à la fois longueur, largeur
et surface affichée (`A_c`) :

| Écart relatif entre `A_c` et `L × l` | Modules | Part |
|---|---|---|
| > 0,1 % | 289 | 1,45 % |
| > 1 % | 82 | 0,41 % |
| > 5 % | 29 | 0,15 % |
| > 10 % | 29 | 0,15 % |
| **maximum observé** | **50,0 %** | |

### 1.2 Analyse d'un cas extrême

Enregistrement `Sharp_ND_72ELUF` :

| Champ | Valeur |
|---|---|
| `STC` (puissance) | 71,8 W |
| `A_c` (surface affichée) | 0,577 m² |
| `Length` × `Width` | 1,165 × 0,990 = **1,153 m²** |
| `N_s` (cellules en série) | 21 |
| `V_oc_ref` | 12,7 V |

Deux rendements incompatibles en découlent :

- avec `A_c` : 71,8 / (0,577 × 1000) = **12,4 %** ;
- avec `L × l` : 71,8 / (1,153 × 1000) = **6,2 %**.

**Trois recoupements indépendants désignent `A_c` comme la valeur
correcte** :

1. 12,4 % est plausible pour du multi-c-Si ancien ; 6,2 % ne l'est pour
   aucune technologie c-Si commercialisée.
2. `V_oc = 12,7 V` pour 21 cellules ⇒ ≈ 0,60 V/cellule, valeur normale
   d'une cellule silicium — les 21 cellules déclarées sont donc
   cohérentes, et le « 72 » du nom commercial ne désigne pas un nombre
   de cellules.
3. `A_c / N_s` = 0,0275 m²/cellule ⇒ cellule de 166 mm, format standard.
   `(L × l) / N_s` = 0,0549 m²/cellule ⇒ **le double** d'une cellule
   standard, format qui n'existe pas.

**Ce sont donc les dimensions qui sont erronées dans cet enregistrement,
pas la surface.**

### 1.3 Conséquence : la règle §3.4 était juste sur le principe, fausse sur la conclusion opérationnelle

La spécification aurait fait de `L × l` la valeur **autoritaire**. Sur ces
29 modules réels, cela aurait produit un rendement **faux d'un facteur
2** — silencieusement, sans aucun signal.

Le raisonnement initial (« ne pas avoir deux sources de vérité ») reste
valide. Ce qui était faux, c'est la conclusion implicite que la valeur
dérivée est nécessairement la bonne. **Les données réelles montrent que
la valeur dérivée peut être la mauvaise.**

### 1.4 Règle corrigée

> **`gross_area` reste dérivé de `L × l`**, mais le contrôle de
> cohérence avec `stated_gross_area` devient **bloquant** au-delà d'un
> seuil, et non plus consultatif.
>
> Au-delà du seuil, la construction du `PvModule` **échoue** avec une
> erreur nommant les deux valeurs en conflit. Elle n'arbitre pas
> elle-même : arbitrer silencieusement serait précisément l'erreur que
> ce constat révèle.
>
> Seuil proposé : **1 %**. Il laisse passer les arrondis de fiche
> technique (98,55 % des modules CEC s'accordent à mieux que 0,1 %) et
> bloque 0,41 % des enregistrements, dont les 29 cas manifestement
> défectueux.

### 1.5 Invariant d'appoint : la surface par cellule

Sur les 19 665 modules CEC cohérents (écart < 0,1 %), la surface par
cellule `A_c / N_s` est remarquablement groupée :

| Statistique | Valeur (m²/cellule) |
|---|---|
| Médiane | 0,0270 |
| 5ᵉ centile | 0,0177 |
| 95ᵉ centile | 0,0277 |

à comparer aux formats de cellule standard : 125 mm ⇒ 0,0156 ·
156 mm ⇒ 0,0243 · 166 mm ⇒ 0,0276 · 182 mm ⇒ 0,0331 · 210 mm ⇒ 0,0441.

Cet invariant peut **informer** le diagnostic d'un conflit de surface,
mais **ne doit pas l'automatiser** : les formats 182 mm et 210 mm,
absents de cet instantané de 2019, sortent de la plage observée et
seraient signalés à tort sur des modules récents. C'est un indice de
qualité de donnée, pas une preuve.

---

## 2. Cartographie vérifiée — base CEC (25 champs)

Schéma constaté par inspection directe, pas reconstitué.

| Champ CEC | Destination | Remarque |
|---|---|---|
| `Technology` | `technology` / `architecture` | Vocabulaire grossier, §2.1 |
| `Bifacial` | présence de `bifacial` | Drapeau 0/1 **sans** facteur φ, §2.2 |
| `STC` | `stc.p_max` | Puissance nominale |
| `PTC` | **absent du modèle** | §2.3 — constat |
| `A_c` | `geometry.stated_gross_area` | Recoupement bloquant, §1.4 |
| `Length`, `Width` | `geometry.length/width` | Autoritaires sous contrôle |
| `N_s` | `configuration.cells_in_series` | |
| `I_sc_ref`, `V_oc_ref`, `I_mp_ref`, `V_mp_ref` | `stc.*` | |
| `alpha_sc` | `temperature_coefficients.alpha_i_sc` | **Absolu (A/°C)**, §2.4 |
| `beta_oc` | `temperature_coefficients.beta_v_oc` | **Absolu (V/°C)**, §2.4 |
| `gamma_r` | `temperature_coefficients.gamma_p_max` | **Relatif (%/°C)**, §2.4 |
| `T_NOCT` | `noct.cell_temperature` | |
| `BIPV` | **absent du modèle** | §2.5 — constat |
| `Version`, `Date` | `identity.datasheet_revision`, provenance | |
| `a_ref`, `I_L_ref`, `I_o_ref`, `R_s`, `R_sh_ref`, `Adjust` | **`electrical_model::DeSotoParameters`** | §2.6 — confirme ADR-0004 Décision 4 |

### 2.1 Le vocabulaire `Technology` est grossier et mélange les niveaux

| Valeur | Modules |
|---|---|
| `Multi-c-Si` | 11 221 |
| `Mono-c-Si` | 9 725 |
| `Thin Film` | 561 |
| `CdTe` | 20 |
| `CIGS` | 8 |

Deux enseignements :

1. **`Thin Film` coexiste avec `CdTe` et `CIGS`**, qui en sont pourtant
   des sous-catégories : la base mélange deux niveaux de granularité.
   Une énumération fermée calquée sur ce vocabulaire serait donc
   incohérente. La variante `Other { name: String }` de `CellTechnology`
   (spécification §4.3) est ce qui permet d'absorber `Thin Film` sans
   inventer une classification que la source ne fournit pas.
2. **Aucune information d'architecture** (PERC, TOPCon, HJT, IBC) n'est
   présente. Un import CEC laissera donc `architecture` indéterminée —
   ce qui confirme la nécessité d'une variante par défaut explicite
   plutôt qu'un choix arbitraire.

### 2.2 `Bifacial` est un drapeau sans facteur de bifacialité

133 modules sur 21 535 (0,6 %) portent `Bifacial = 1`. Aucun champ ne
donne φ.

Conséquence de conception : un import CEC d'un module bifacial **ne peut
pas** renseigner `BifacialCharacteristics.bifaciality_factor`, qui est
pourtant obligatoire dans ce bloc. Deux issues cohérentes avec les
principes du projet :

- ne pas construire le bloc `bifacial` (l'information « c'est un module
  bifacial » est alors perdue) ;
- rendre `bifaciality_factor` optionnel **à l'intérieur** du bloc, le
  bloc signifiant alors « module bifacial, caractérisation
  éventuellement incomplète ».

**La seconde est retenue** : elle préserve l'information disponible sans
inventer la manquante. La spécification §4.8 est corrigée en
conséquence : `bifaciality_factor` devient `Option<Ratio>` à l'intérieur
d'un bloc dont la seule présence signifie « bifacial ».

### 2.3 `PTC` — une condition de mesure absente du modèle

`PTC` (*PVUSA Test Conditions*) est renseigné pour l'intégralité des
21 535 modules. C'est une **condition de mesure distincte des STC**
(1000 W/m², mais température **ambiante** de 20 °C et vent de 1 m/s, au
lieu d'une température de cellule imposée de 25 °C), utilisée comme
référence réglementaire sur le marché nord-américain.

Exemple : `Sharp_ND_72ELUF` ⇒ STC 71,8 W, PTC 63,2 W.

**Le modèle spécifié ne peut pas représenter cette valeur.** `noct` et
`nmot` sont typés par une énumération à deux variantes
(`NoctIec61215`, `NmotIec61853_2`) qui n'inclut pas PTC.

**Correction retenue** : ajouter une variante `PtcPvusa` à
`OperatingConditionPoint.kind`. Le champ est déjà structurellement
adapté (une condition de référence avec son point de fonctionnement) ;
seule l'énumération doit s'ouvrir. C'est exactement le cas d'usage de
`#[non_exhaustive]` défendu en ADR-0004 Décision 3 — sauf qu'ici
l'omission a été détectée **avant** l'implémentation, au coût de zéro.

### 2.4 Base à unités mixtes — confirmation mesurée

Test décisif sur `gamma_r` : corrélation avec la puissance STC = **0,19**
(faible), valeurs groupées entre −0,679 et −0,166, médiane −0,45. Un
coefficient exprimé en W/°C croîtrait proportionnellement à la puissance
du module et donnerait une corrélation proche de 1.

Sur `Sharp_ND_72ELUF` : `alpha_sc` = 0,002146 pour `I_sc` = 5,17 A ⇒
0,0415 %/°C en relatif ; `beta_oc` = −0,159 pour `V_oc` = 43,99 V ⇒
−0,362 %/°C. Ces valeurs relatives sont typiques, les valeurs brutes ne
le seraient pas.

**Conclusion : `alpha_sc` et `beta_oc` sont absolus (A/°C, V/°C),
`gamma_r` est relatif (%/°C), dans la même base.**

Cela **valide par la mesure** la décision de la spécification §4.7 : le
type `TemperatureCoefficient` doit accepter les deux représentations et
exposer la conversion explicite. Ce n'est plus une précaution
théorique — c'est une exigence dictée par la principale base publique du
domaine.

### 2.5 `BIPV` — drapeau absent du modèle

164 modules sur 21 535 portent `BIPV = 'Y'` (photovoltaïque intégré au
bâti). Le modèle spécifié ne porte pas cette information.

**Correction retenue** : ajouter `building_integrated: Option<bool>` à
`ModuleIdentity`. `Option` et non `bool` : l'absence d'information
(source qui ne le renseigne pas) se distingue de « non intégré au bâti ».

C'est une propriété de **destination d'usage**, pas une propriété
physique — cohérent avec son placement dans l'identité plutôt que dans
la géométrie ou l'électrique.

### 2.6 Les six paramètres De Soto — décision confirmée

`a_ref`, `I_L_ref`, `I_o_ref`, `R_s`, `R_sh_ref`, `Adjust` sont exactement
le jeu de paramètres du modèle à diode unique ajusté, tel que défini par
De Soto et al. (2006) et étendu par le CEC (`Adjust`).

Leur présence dans la même ligne CSV que les grandeurs mesurées confirme
concrètement la tension décrite en ADR-0004 Décision 4 — et la décision
retenue : ces six champs partent vers `electrical_model::DeSotoParameters`,
pas vers `pv_module`. Un import CEC produit bien un couple.

---

## 3. Cartographie vérifiée — base Sandia (42 champs)

Schéma constaté : `Vintage`, `Area`, `Material`, `Cells_in_Series`,
`Parallel_Strings`, `Isco`, `Voco`, `Impo`, `Vmpo`, `Aisc`, `Aimp`,
`C0`, `C1`, `Bvoco`, `Mbvoc`, `Bvmpo`, `Mbvmp`, `N`, `C2`, `C3`,
`A0`–`A4`, `B0`–`B5`, `DTC`, `FD`, `A`, `B`, `C4`–`C7`, `IXO`, `IXXO`,
`Notes`.

| Groupe | Champs | Destination |
|---|---|---|
| Identité / date | `Vintage`, `Notes` | `identity` |
| Géométrie | `Area` | `geometry.stated_gross_area` |
| Technologie | `Material` | `technology` |
| Configuration | `Cells_in_Series`, `Parallel_Strings` | `configuration` |
| Point STC | `Isco`, `Voco`, `Impo`, `Vmpo` | `stc.*` |
| Coefficients thermiques | `Aisc`, `Bvoco` | `temperature_coefficients` (formes SAPM) |
| **Coefficients propres au modèle SAPM** | `Aimp`, `Mbvoc`, `Bvmpo`, `Mbvmp`, `N`, `C0`–`C7`, `A0`–`A4`, `B0`–`B5`, `DTC`, `FD`, `A`, `B`, `IXO`, `IXXO` | **`electrical_model::SapmParameters`** |

**28 des 42 champs sont des paramètres propres au modèle SAPM** — sans
signification hors de ce modèle. `Mbvoc` et `Mbvmp`, par exemple,
décrivent la dépendance à l'irradiance du coefficient thermique de
tension : cette grandeur n'existe **dans aucune fiche technique** et
n'a pas d'équivalent dans le modèle De Soto.

C'est la démonstration la plus nette de la décision ADR-0004
Décision 4 : deux tiers d'un enregistrement Sandia n'est pas une
description de module, c'est la paramétrisation d'un modèle particulier.
Les stocker dans `pv_module` aurait rendu ce crate dépendant de la forme
du modèle SAPM.

---

## 4. Corrections apportées à la spécification

| # | Objet | Origine du constat | Nature |
|---|---|---|---|
| C-1 | Contrôle de cohérence surface : **bloquant** au-delà de 1 %, sans arbitrage automatique | §1 — 29 modules à écart > 10 %, jusqu'à ×2 sur le rendement | Correction de la règle §3.4 et d'ADR-0005 Décision 2 |
| C-2 | `bifaciality_factor` devient `Option<Ratio>` dans un bloc dont la présence signifie « bifacial » | §2.2 — CEC donne un drapeau sans φ | Assouplissement §4.8 |
| C-3 | Ajout d'une variante `PtcPvusa` à `OperatingConditionPoint.kind` | §2.3 — PTC renseigné sur 21 535 modules | Ajout §4.6 |
| C-4 | Ajout de `building_integrated: Option<bool>` à `ModuleIdentity` | §2.5 — drapeau BIPV sur 164 modules | Ajout §4.2 |
| C-5 | Double représentation des coefficients thermiques : **exigence confirmée**, non plus précaution | §2.4 — base à unités mixtes mesurée | Confirmation §4.7 |
| C-6 | Séparation paramètres ajustés / description physique : **exigence confirmée** | §2.6 et §3 — 6 champs De Soto, 28 champs SAPM | Confirmation ADR-0004 D-4 |

Aucune de ces corrections ne remet en cause l'architecture (dépendances,
découpage en crates, taxonomie). Elles portent toutes sur le contenu du
modèle de données — ce que cette étape de conception avait précisément
pour but de stabiliser avant écriture de code.

---

## 5. Ce que cette validation ne couvre pas

1. **Les fiches techniques fabricants** restent à vérifier : les bases
   CEC et Sandia sont des sources *dérivées*, pas les fiches
   elles-mêmes. Le §1.1 de la spécification (≥ 5 fiches réelles) demeure
   une exigence ouverte, non satisfaite ici.
2. **Les instantanés sont datés** (CEC 2019, Sandia 2015). Les modules
   grand format (182 mm, 210 mm), les TOPCon et les bifaciaux modernes y
   sont sous-représentés — d'où la prudence du §1.5 sur l'invariant de
   surface par cellule.
3. **IEC 61853 et PAN** ne sont pas couverts : aucune des deux bases
   inspectées ne les contient. Le jeu exact des points de la matrice
   61853-1 reste à lire dans le texte de la norme.
4. **Aucun test de conversion n'a été exécuté** : ce document valide la
   *structure* du modèle, pas une implémentation d'import, qui n'existe
   pas.

## 6. Reproductibilité

Les constats de ce document sont reproductibles avec pvlib installé :

```python
import pvlib
cec     = pvlib.pvsystem.retrieve_sam('CECMod')     # 25 champs × 21535 modules
sandia  = pvlib.pvsystem.retrieve_sam('SandiaMod')  # 42 champs ×   523 modules
```

Aucun script n'est committé : ces inspections sont des mesures
ponctuelles de conception, pas un artefact du moteur. Les chiffres
ci-dessus sont datés de l'instantané pvlib 0.15.2 et seront à recalculer
sur une base plus récente au moment de l'implémentation de l'import.
