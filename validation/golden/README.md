# Données de validation figées — HeliosPro moteur scientifique

**Statut** : figé au tag `v1.0.0-scientific-core`. Contenu de ce dossier
**immuable** à partir de ce tag — voir « Règle de gel » ci-dessous.

## Contenu

- `datasets/pvlib_reference_dataset.json` — copie figée du jeu de 91 cas
  de référence pvlib-python (source vivante :
  `engine/validation-data/pvlib_reference_dataset.json`, régénérée par
  `engine/validation-data/generate_pvlib_reference_dataset.py`).
- `datasets/global_campaign_dataset.csv` — copie figée de la campagne
  mondiale de 2918 cas (source vivante :
  `engine/validation-data/global_campaign_dataset.csv`, régénérée par
  `engine/validation-data/generate_global_campaign.py`).
- `datasets/SHA256SUMS.txt` — empreintes SHA-256 des deux fichiers
  ci-dessus, au moment du gel.
- `reports/baseline_*_v1.0.0-scientific-core.{md,txt}` — copies figées des
  quatre rapports auto-générés (validation 91 cas, campagne mondiale,
  benchmarks, couverture) tels que produits par la suite de tests au
  moment du tag. Ce sont les chiffres de référence officiels de la Phase 1
  — voir `docs/design/20-rapport-cloture-phase1.md` pour leur synthèse.
- `reports/SHA256SUMS.txt` — empreintes SHA-256 des quatre rapports.

## Pourquoi une copie figée, séparée des fichiers « vivants »

`engine/validation-data/*.{json,csv}` et
`engine/validation-data/latest_*.{md,txt}` restent les fichiers **actifs**
du projet : les jeux de données peuvent être régénérés (mêmes scripts,
même graine aléatoire fixe pour la campagne mondiale — reproductible mais
pas nécessairement byte-identique si pvlib ou son environnement
d'exécution changent de version), et les rapports sont ré-écrits à chaque
`cargo test`. Ce dossier `validation/golden/` capture un instantané exact
et non regénérable de l'état exact validé à la clôture de la Phase 1, pour
servir de **référence immuable** aux versions futures du moteur : toute
version ultérieure doit être capable de reproduire (ou d'améliorer, jamais
de dégrader silencieusement) les résultats figés ici.

## Règle de gel

**Ces fichiers ne doivent plus jamais être modifiés en place.** Si les
scripts source de `engine/validation-data/` produisent, dans une version
future, un jeu de données différent (nouvelle campagne, nouveau tirage
aléatoire, mise à jour de pvlib), le nouveau jeu de données est ajouté ici
sous un **nouveau tag** (ex. `validation/golden-v1.1.0/`), jamais en
écrasant `validation/golden/` — de la même façon qu'un tag Git existant
n'est jamais déplacé. Toute exception (erreur découverte dans les données
figées elles-mêmes, ex. bug de génération) doit être documentée
explicitement dans un commit dédié et référencée depuis
`docs/design/16-phase1-validation-report.md`, jamais silencieuse.

## Vérification d'intégrité

```bash
cd validation/golden/datasets && sha256sum -c SHA256SUMS.txt
cd ../reports && sha256sum -c SHA256SUMS.txt
```

## Provenance et méthodologie complète

Voir `engine/validation-data/README.md` (génération des jeux de données,
choix de pvlib-python comme oracle indépendant, méthodologie complète) et
`engine/crates/validation/src/global_campaign/mod.rs` (module doc :
catégories de la campagne mondiale, effets de bord documentés et bornés).
