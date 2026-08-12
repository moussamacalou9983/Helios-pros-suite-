# Principes scientifiques HeliosPro

**Statut** : constitution scientifique du projet, adoptée à la clôture de
la Phase 1 (tag `v1.0.0-scientific-core`). Applicable à **tout** module
présent ou futur du moteur `engine/`, sans exception. Toute dérogation à
l'un de ces principes doit être justifiée par écrit dans un ADR
(`docs/adr/`, voir `docs/governance/adr-policy.md`) et enregistrée dans le
registre de dette scientifique (`docs/scientific-debt-register.md`) si
elle introduit une approximation.

Ce document ne réénonce pas les décisions déjà prises (voir
`docs/design/16-phase1-validation-report.md` pour l'historique complet) ;
il en extrait les principes directeurs sous une forme stable et
applicable à toute décision future, pour que chaque contributeur — humain
ou agent — puisse vérifier une proposition de modèle contre une liste
close plutôt que contre une intuition.

## 1. Priorité à la physique

Aucun modèle numérique n'est ajouté sans modèle physique sous-jacent
identifié. Une corrélation empirique (ex. Erbs, Perez, Sæmundsson) est
acceptable **seulement** si elle est elle-même issue d'une publication
évaluée par les pairs et appliquée dans son domaine de validité documenté
— jamais une approximation ad hoc introduite pour « faire fonctionner »
le code. Quand un modèle physique de premier principe et un modèle
empirique coexistent pour le même phénomène (ex. série héliocentrique
VSOP87 complète de SPA vs. ellipse képlérienne à un terme de Meeus), les
deux sont conservés et leurs domaines de validité respectifs sont
documentés plutôt que de n'en garder qu'un par simplification.

## 2. Reproductibilité

Toute valeur numérique publiée dans une documentation, un rapport ou une
revendication de précision doit être **régénérable** par une commande
documentée (`cargo test`, un script de `validation-data/`, une commande
`cargo llvm-cov`/`cargo bench`) — jamais recopiée à la main d'une
exécution passée sans moyen de la reproduire. C'est le principe qui a
gouverné la construction de `validation-data/latest_*.md` (rapports
régénérés à chaque `cargo test`) plutôt que des chiffres figés dans la
documentation.

## 3. Transparence

Chaque écart, chaque limite, chaque compromis est documenté explicitement
au moment où il est fait — jamais découvert plus tard par un tiers. Les
sections « limites connues » de chaque document de clôture de phase
(`docs/templates/phase-closure-report-template.md`) et le registre de
dette scientifique (`docs/scientific-debt-register.md`) sont les
mécanismes formels de ce principe. Un test qui échoue n'est jamais
« assoupli » sans que la raison physique de l'échec soit d'abord comprise
et documentée (voir l'exemple de la singularité d'azimut au zénith,
`engine/crates/validation/src/global_campaign/mod.rs`, où l'écart a été
expliqué et borné plutôt que masqué par un seuil élargi arbitraire).

## 4. Validation indépendante

Aucun modèle n'est considéré validé sur la seule base de ses propres
tests unitaires internes. Chaque modèle scientifique doit être comparé à
au moins une source externe indépendante : une implémentation
tierce reconnue (ex. pvlib-python), un jeu de données publié, ou une
norme de test. Voir `docs/governance/scientific-review-process.md` pour
la procédure formelle qui applique ce principe à chaque nouveau modèle.

## 5. Traçabilité

Toute exigence (qu'elle vienne du cahier des charges produit ou d'un
brief d'ingénierie) doit être reliée explicitement au module qui la
couvre, aux tests qui la vérifient et à la documentation qui la décrit —
voir `docs/design/21-matrice-tracabilite.md` pour la matrice en vigueur.
Une exigence sans couverture identifiée n'est jamais silencieusement
abandonnée ; elle est marquée explicitement « hors périmètre » avec un
renvoi vers la phase qui la traitera.

## 6. Déterminisme

Le moteur scientifique ne contient aucune source de non-déterminisme :
pas de nombre aléatoire dans le chemin de calcul de production (les
graines aléatoires n'apparaissent que dans les *générateurs de jeux de
test*, jamais dans le moteur lui-même, et sont alors fixes et
documentées — voir `engine/validation-data/generate_global_campaign.py`),
pas d'accès disque/réseau, pas de dépendance à l'ordre d'itération d'une
structure non ordonnée, pas d'état global mutable partagé entre appels.
Deux appels avec les mêmes entrées produisent bit-à-bit la même sortie —
propriété exploitée directement par les tests de régression « golden
dataset » (`engine/crates/tests/tests/regression.rs`).

## 7. Unités SI

Toute grandeur physique interne au moteur est portée par un type fort du
système international (`core_units` : `Angle`, `Length`, `Pressure`,
`Ratio`, `Temperature`, `Velocity`, `Irradiance`), jamais par un `f64` nu
dont l'unité seule le nom de la variable indiquerait. Les conversions
d'unité (degré ↔ radian, hPa ↔ Pa, °C ↔ K) sont encapsulées dans ces
types, jamais dupliquées de façon ad hoc dans la logique métier. Une
grandeur non-SI ne franchit une frontière de module que sous une forme
explicitement typée (jamais un `f64` implicite dont l'unité serait
seulement documentée en commentaire).

## 8. Aucune constante sans référence

Toute constante physique ou astronomique porte, à sa définition, un
commentaire `**Source**:` citant la publication ou la norme exacte dont
elle provient (voir `core_constants/src/lib.rs` pour l'ensemble des 13
constantes du moteur, chacune ainsi référencée). Un littéral numérique
« magique » dans le code métier — même correct — sans référence à une
constante nommée et sourcée est une violation de ce principe : voir
l'exemple corrigé pendant la Phase 1 (`solar_position::spa` utilisait un
littéral `8.794` dupliquant une constante déjà nommée et référencée,
corrigé pour utiliser `core_constants::SOLAR_PARALLAX_ARCSEC_AT_1AU`,
documenté dans `docs/design/17-analyse-incertitude-numerique.md` §17.4c).

## 9. Aucune formule sans publication

Toute équation implémentée porte, à son point d'implémentation, un
commentaire `**Source**:` citant l'article, l'ouvrage ou la norme et,
quand elle existe, le numéro d'équation exact de la publication source
(voir la convention systématique dans tous les modules de `solar_geometry`,
`irradiance`, `transposition`, `thermal_model`). Aucune formule n'est
« devinée », « approchée à vue » ou « simplifiée pour aller plus vite »
sans que cette simplification soit elle-même une simplification
**publiée** et référencée comme telle (ex. le moteur Meeus réduit n'est
pas une approximation inventée du SPA complet, c'est l'algorithme publié
de Meeus (1998), avec sa propre précision documentée).

## Application de ces principes

Ces neuf principes ne sont pas déclaratifs : ils sont vérifiés
mécaniquement ou procéduralement à chaque contribution —

| Principe | Mécanisme de vérification |
|---|---|
| Priorité à la physique, aucune formule sans publication | Procédure de revue scientifique (`docs/governance/scientific-review-process.md`) |
| Reproductibilité | Rapports auto-générés (`cargo test`), scripts de génération de données committés |
| Transparence | Registre de dette scientifique, sections « limites » obligatoires |
| Validation indépendante | Campagnes `validation`/`validation::global_campaign` contre pvlib |
| Traçabilité | Matrice de traçabilité (`docs/design/21-matrice-tracabilite.md`) |
| Déterminisme | Tests de régression golden dataset, revue de code (pas de `rand`/E-S dans le moteur) |
| Unités SI | `core_units`, lint `unsafe_code`/`missing_docs`, revue de code |
| Aucune constante sans référence | `core_constants`, audit de documentation (`RUSTFLAGS="-D missing_docs"`) |

et pour toute décision qui touche à l'un de ces principes (nouvel
algorithme, nouveau modèle, nouvelle dépendance, nouvelle architecture,
modification d'une API publique), un ADR est obligatoire — voir
`docs/governance/adr-policy.md`.
