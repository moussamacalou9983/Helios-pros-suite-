# Politique de gestion des décisions d'architecture (ADR)

**Statut** : obligatoire à partir de la clôture de la Phase 1. Un ADR
(*Architecture Decision Record*) documente une décision majeure — le
contexte qui l'a motivée, les options considérées, la décision retenue,
et ses conséquences — pour qu'un futur contributeur (humain ou agent)
comprenne **pourquoi** le code est ainsi fait, pas seulement ce qu'il
fait. Applique le principe de traçabilité de
`docs/scientific-principles.md`.

## Quand un ADR est obligatoire

Un ADR **doit** être créé avant (ou, à défaut, immédiatement après, pour
les décisions déjà prises pendant la Phase 1 et documentées rétroactivement)
toute décision relevant d'une des cinq catégories suivantes :

1. **Nouvel algorithme** — ex. le choix de l'algorithme SPA complet
   plutôt qu'une série réduite (voir `docs/adr/0001-spa-primary-engine.md`).
2. **Nouveau modèle physique** — ex. le choix du modèle de transposition
   Perez comme modèle par défaut recommandé parmi les 4 implémentés.
3. **Nouvelle dépendance** — toute dépendance ajoutée au workspace `engine/`
   au-delà de la bibliothèque standard (actuellement aucune — voir
   `docs/adr/0002-zero-external-dependencies.md`), y compris les outils de
   développement dont le choix a un impact durable (ex. `cargo-llvm-cov`).
4. **Nouvelle architecture** — changement de structure du workspace
   (ex. la restructuration en 14 crates), changement de patron
   d'abstraction (ex. le trait `SolarPositionAlgorithm`), ou changement
   de convention transversale (ex. le choix UTC-only documenté dans
   `core_math::julian`).
5. **Modification d'une API publique** — tout changement listé comme
   « cassant » par `docs/design/19-politique-semver-api.md` §19.3.3.

Une décision qui ne relève d'aucune de ces cinq catégories (un bug fix,
un ajout de test, une clarification de documentation) ne nécessite pas
d'ADR.

## Format

Chaque ADR est un fichier `docs/adr/NNNN-titre-court-en-kebab-case.md`,
numéroté séquentiellement (`0001`, `0002`, ...), jamais réutilisé même si
un ADR est plus tard superseded. Le gabarit exact est
`docs/adr/0000-template.md` ; les sections obligatoires sont :

- **Statut** : `Proposé` / `Accepté` / `Rejeté` / `Remplacé par ADR-NNNN`.
- **Contexte** : la situation et la question à trancher, sans présumer de
  la réponse.
- **Options considérées** : au moins deux options réelles, avec leurs
  avantages/inconvénients respectifs — jamais une option unique présentée
  comme évidente sans alternative sérieusement pesée.
- **Décision** : l'option retenue et pourquoi, en une formulation
  vérifiable (pas « on a choisi la meilleure option » mais le critère qui
  a fait pencher la balance).
- **Conséquences** : ce que cette décision rend plus facile, ce qu'elle
  rend plus difficile, et toute dette scientifique/technique qu'elle
  introduit (avec renvoi vers `docs/scientific-debt-register.md` si
  applicable).
- **Références** : publications/normes citées à l'appui de la décision,
  le cas échéant.

## Index

Voir `docs/adr/README.md` pour la liste de tous les ADR existants et leur
statut.

## Processus

1. Un ADR est rédigé **avant** l'implémentation quand c'est possible (la
   décision doit être défendable sur le papier avant d'être codée).
2. Il est revu au même titre que le code qu'il accompagne — une
   modification d'API publique sans ADR associé est un motif de blocage
   de revue, au même titre qu'un modèle physique sans revue scientifique
   (`docs/governance/scientific-review-process.md`).
3. Un ADR n'est jamais supprimé ni réécrit après acceptation ; s'il est
   remis en cause, un nouvel ADR le remplace explicitement (« Remplacé
   par ADR-NNNN ») et l'ancien reste consultable pour la traçabilité
   historique — même principe que les sections « historique » conservées
   dans `docs/design/16-phase1-validation-report.md`.
