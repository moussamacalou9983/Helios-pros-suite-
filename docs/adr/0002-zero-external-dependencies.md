# ADR-0002 — Zéro dépendance externe hors bibliothèque standard pour le moteur

**Statut** : Accepté
**Date** : rétroactif, décision prise dès le premier brief d'ingénierie
de la Phase 1
**Catégorie** : Nouvelle dépendance (politique de non-adoption)

## Contexte

Le moteur scientifique `engine/` doit être fiable, auditable, et
utilisable dans des contextes contraints (embarqué, air-gapped, longue
durée de vie) typiques des bureaux d'études. Le choix classique en Rust
serait d'utiliser des crates de l'écosystème pour l'algèbre linéaire, les
statistiques, ou les dates (`nalgebra`, `chrono`, `statrs`, etc.), ce qui
accélérerait le développement initial.

## Options considérées

### Option A — Dépendances de l'écosystème pour les besoins courants

- Avantages : développement plus rapide, code potentiellement mieux
  optimisé (SIMD, etc.) sur certaines opérations.
- Inconvénients : surface d'audit non maîtrisée (chaque dépendance et ses
  propres dépendances transitives doivent être auditées pour un usage en
  ingénierie réglementée) ; risque de rupture de compatibilité lors des
  mises à jour ; risque de vulnérabilité de la chaîne
  d'approvisionnement ; empreinte de compilation plus lourde.

### Option B — Zéro dépendance hors bibliothèque standard

- Avantages : surface d'audit strictement limitée au code écrit par le
  projet ; aucun risque de rupture de compatibilité externe ; compilation
  reproductible sans accès réseau (propriété exploitée directement dans
  cet environnement de développement, où l'accès réseau était
  intermittent) ; force une compréhension complète de chaque algorithme
  implémenté (aucune boîte noire).
- Inconvénients : développement de types/fonctions utilitaires
  (`core_units`, `core_math`) qui existent déjà dans l'écosystème ; pas
  de bénéfice des optimisations bas niveau de crates spécialisées.

## Décision

**Option B.** Le critère décisif : pour un moteur de calcul scientifique
destiné à produire des résultats sur lesquels des décisions d'ingénierie
réelles seront prises, la maîtrise complète de la surface de code
auditable prime sur la vitesse de développement initiale. Cette politique
s'applique au moteur `engine/` lui-même ; les outils de développement
(`cargo-llvm-cov` pour la mesure de couverture, les scripts Python de
génération de jeux de données de validation) en sont explicitement
exemptés, n'étant jamais compilés dans le binaire du moteur — voir
`engine/README.md` et `engine/validation-data/README.md`.

## Conséquences

- `core_units`, `core_math`, `core_constants` ont dû être écrits de
  zéro (types SI forts, Jour Julien, statistiques d'erreur) plutôt que
  d'utiliser `chrono`/`statrs`/équivalents.
- `cargo tree --workspace` reste vérifiable en une commande comme preuve
  de conformité à cette politique (aucune dépendance `path` externe au
  workspace).
- Cette contrainte a directement motivé le choix méthodologique de la
  campagne de validation mondiale (`validation::global_campaign`) :
  plutôt qu'une dépendance `csv`, un analyseur CSV minimal a été écrit à
  la main (`engine/crates/validation/src/global_campaign/dataset.rs`).
- Exception documentée : l'allocateur compteur de
  `engine/crates/tests/tests/benchmarks.rs` nécessite `unsafe` (implémenter
  `GlobalAlloc`) ; cette crate relâche `unsafe_code` de `forbid` à `allow`
  dans son propre `Cargo.toml`, une dérogation locale et documentée plutôt
  qu'un assouplissement silencieux de la politique globale — voir
  `engine/crates/tests/Cargo.toml`.

## Références

- `engine/README.md` §« No network access, no external services, and no
  dependency outside the Rust standard library ».
- `engine/crates/tests/Cargo.toml` (dérogation documentée pour
  l'allocateur compteur).
