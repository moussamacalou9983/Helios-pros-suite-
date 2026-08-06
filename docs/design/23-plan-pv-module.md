# 23. Plan détaillé — `pv_module`

**Statut** : plan de développement uniquement. **Aucune logique métier
n'est implémentée à ce stade** — conformément à l'instruction reçue, ce
document doit être validé avant tout début d'implémentation. Suit
`docs/governance/scientific-review-process.md` et
`docs/governance/adr-policy.md` : les décisions de modèle ci-dessous
devront chacune faire l'objet d'un ADR au moment de leur implémentation
effective, pas seulement de ce plan.

## 1. Objectif et périmètre

`pv_module` porte le **modèle de données de fiche technique** d'un module
photovoltaïque : l'ensemble des paramètres électriques et physiques
publiés par un fabricant, dans les conditions de mesure normalisées, dont
`electrical_model` (modèle à diode unique) et `thermal_model` (déjà
livré) ont besoin. Ce crate ne contient **aucun calcul de performance** —
c'est un modèle de données typé et validé structurellement, pas un moteur
de simulation. Le calcul (extraction des 5 paramètres du modèle à diode
unique, évaluation de la courbe I-V à des conditions arbitraires) relève
d'`electrical_model`, qui consommera `pv_module` comme entrée.

Cette séparation reproduit la même discipline architecturale que le reste
du moteur (`core_units` ne calcule rien, `core_constants` non plus) : un
crate de données typées et sourcées, distinct des crates de calcul qui
les consomment.

## 2. Modèle de données envisagé

Champs regroupés par norme source, chacun portant sa référence exacte au
moment de l'implémentation (principe « aucune constante sans
référence », `docs/scientific-principles.md` §8) :

### Identité et certification

- Fabricant, référence commerciale, technologie de cellule
  (mono-Si PERC, poly-Si, HJT, CdTe, etc.).
- Certifications applicables (IEC 61215-1/2 pour la qualification de
  conception, IEC 61730-1/2 pour la sécurité) — champs booléens/texte de
  traçabilité, pas de logique de vérification dans ce crate.

### Paramètres électriques STC (IEC 61215, conditions 1000 W/m², 25°C, AM1.5)

- `voc_stc` (tension circuit ouvert), `isc_stc` (courant court-circuit),
  `v_mpp_stc`, `i_mpp_stc` (tension/courant au point de puissance
  maximale), `p_mpp_stc` (puissance nominale).
- Coefficients de température : `alpha_isc` (%/°C ou A/°C），`beta_voc`
  (%/°C ou V/°C), `gamma_pmp` (%/°C) — nécessaires à De Soto et al.
  (2006) pour extrapoler les paramètres STC à une température de cellule
  arbitraire (déjà calculée par `thermal_model`, livré).
- Nombre de cellules en série (`n_cells_series`) — paramètre requis par
  le modèle à diode unique (tension thermique multipliée par ce nombre).

### Paramètres énergie étendue (IEC 61853-1..4, optionnels)

- Matrice de performance à conditions multiples (irradiance ×
  température), quand disponible — permet une extraction de paramètres
  plus robuste que le seul point STC, et sert de jeu de validation direct
  pour `electrical_model` (comparaison à des points mesurés réels, pas
  seulement au point STC).

### Paramètres physiques

- Dimensions (longueur, largeur, épaisseur), masse, coefficient
  d'absorption/transmission optique si disponible (pour un futur usage
  bifacial, `docs/design/22-roadmap-scientifique.md` §Bifacial).

## 3. Architecture prévue

```
pv_module (données typées, aucun calcul)
    │
    ├─► consommé par electrical_model (modèle à diode unique, De Soto 2006)
    └─► consommé par thermal_model::noct (déjà livré — NOCT est un champ
        de fiche technique ; pv_module deviendra la source canonique de ce
        champ, thermal_model continuera de l'accepter en paramètre plutôt
        que de dépendre directement de pv_module, pour ne pas introduire
        de dépendance inverse — voir §4)
```

**Dépendances prévues** : `core_units` (types SI pour chaque champ —
aucun `f64` nu), `core_constants` (si des constantes de normalisation
STC sont nécessaires, ex. irradiance de référence 1000 W/m² déjà présente
si applicable). **Aucune dépendance à `electrical_model`, `thermal_model`,
`solar_position` ou tout autre crate de calcul** — `pv_module` reste une
feuille de l'arbre de dépendances, cohérent avec le principe d'architecture
« aucun module implémenté so far ne dépend d'un module en aval »
(`engine/README.md` §« Extending the engine »).

**Validation structurelle** (pas de calcul physique à valider ici, mais
la structure de données elle-même) : les invariants vérifiables sans
calcul de performance — `Voc > Vmpp > 0`, `Isc > Impp > 0`,
`n_cells_series` entier positif cohérent avec la technologie déclarée,
cohérence dimensionnelle des types SI — seront testés unitairement dès
la première implémentation.

## 4. Ce que ce plan NE couvre PAS

- L'extraction des 5 paramètres du modèle à diode unique à partir des
  champs de fiche technique — relève d'`electrical_model` (De Soto et
  al. 2006).
- Le calcul de courbe I-V à des conditions arbitraires — relève
  d'`electrical_model`.
- La base de données d'équipements mutualisée/versionnée (EF-016 du
  cahier des charges) — couche application/persistance, hors périmètre
  du moteur scientifique (voir `docs/design/21-matrice-tracabilite.md`).
- Le format d'import de fiches techniques (CSV/PAN/PVsyst) — décision
  à prendre séparément, potentiellement dans un ADR dédié si un format
  externe standard est adopté (ex. le format `.PAN` largement utilisé par
  l'industrie n'est pas un format ouvert documenté publiquement — à
  investiguer avant adoption, pas supposé disponible).

## 5. Critères de validation prévus (avant fusion, per la revue scientifique)

Suivant les six critères de `docs/governance/scientific-review-process.md` :

1. **Publications originales** : IEC 61215-1/2, IEC 61730-1/2,
   IEC 61853-1..4 (consultation directe des normes ou de résumés
   techniques fiables si l'accès complet à la norme payante n'est pas
   disponible — dérogation à documenter explicitement si nécessaire,
   comme cela a été fait pour les tables SPA).
2. **Normes** : les quatre normes IEC ci-dessus, déjà référencées dans
   `docs/references/bibliography.md`.
3. **Jeux de validation** : fiches techniques réelles de modules
   commerciaux (au moins 2-3, technologies différentes) comme cas de
   test structurel ; matrice IEC 61853 d'au moins un module comme jeu de
   validation pour la future extraction de paramètres d'`electrical_model`.
4. **Benchmarks** : construction d'une structure `PvModuleDatasheet`
   attendue quasi instantanée (pas de calcul) — un budget de
   non-régression sera fixé par cohérence avec le reste du moteur, sans
   attente de coût significatif.
5. **Revue documentaire** : chaque champ documenté avec sa norme source
   exacte, suivant la convention déjà en place dans tout le moteur.
6. **Analyse des limites** : champs optionnels (IEC 61853) clairement
   distingués des champs obligatoires (STC) ; toute fiche technique
   incomplète doit être un état représentable explicitement (pas une
   valeur par défaut arbitraire) — probablement via des champs `Option<T>`
   plutôt que des zéros silencieux.

## 6. Ordre d'implémentation interne recommandé

1. Types de données STC (champs obligatoires IEC 61215) + tests
   structurels.
2. Champs de coefficients de température + tests de cohérence de signe
   (ex. `beta_voc` doit être négatif pour la quasi-totalité des
   technologies silicium — invariant physique documentable et testable).
3. Champs IEC 61853 optionnels (matrice de performance étendue).
4. Documentation complète + audit `RUSTFLAGS="-D missing_docs"`.
5. ADR documentant les choix de modèle de données retenus (`docs/adr/`).
6. Rapport de clôture de sous-phase suivant
   `docs/templates/phase-closure-report-template.md`.

## 7. Prochaine étape

Ce plan doit être **validé explicitement** avant que l'étape 1 de la
section 6 ne commence. Aucun code n'est écrit dans `engine/crates/pv_module/`
au-delà du squelette déjà existant tant que cette validation n'a pas été
donnée.
