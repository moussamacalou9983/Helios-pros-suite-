# Architecture Decision Records — HeliosPro

Voir `docs/governance/adr-policy.md` pour la politique complète (quand un
ADR est obligatoire, format, processus). Ce fichier est l'index de tous
les ADR ; il est mis à jour à chaque nouvel ADR créé.

Gabarit pour tout nouvel ADR : `docs/adr/0000-template.md`.

| ADR | Titre | Statut | Catégorie |
|---|---|---|---|
| [0001](0001-spa-primary-engine.md) | SPA (Reda & Andreas 2004) comme moteur de position solaire primaire | Accepté | Nouvel algorithme |
| [0002](0002-zero-external-dependencies.md) | Zéro dépendance externe hors bibliothèque standard pour le moteur | Accepté | Nouvelle dépendance |
| [0003](0003-utc-only-time-architecture.md) | Le moteur travaille exclusivement en temps UTC | Accepté | Nouvelle architecture |
| [0004](0004-pv-module-architecture.md) | Architecture du crate `pv_module` | **Proposé** | Nouvelle architecture + Modification d'API publique |
| [0005](0005-pv-module-data-model.md) | Modèle de données et incertitudes de `pv_module` | **Proposé** — D-3 remplacée par ADR-0006 | Nouvelle architecture (modèle de données) |
| [0006](0006-measurement-provenance-conflicts.md) | Porteur de valeur, provenance et gestion formelle des conflits | **Proposé** | Nouvelle architecture (modèle de données) |
| [0007](0007-invariant-inventory.md) | Inventaire des invariants : décompte de référence et ajout d'`INV-STR-004` | **Proposé** | Règles de validation |
| [0008](0008-electrical-model-architecture.md) | Architecture d'`electrical_model` : multi-modèles, et séparation points / courbe | **Proposé** — 7 décisions fermées par ADR-0009 | Nouvelle architecture |
| [0009](0009-electrical-model-decisions.md) | Fermeture des sept décisions ouvertes d'`electrical_model` | **Proposé** — D-4 et D-7 corrigées par ADR-0010 | Nouvelle architecture + méthode numérique |
| [0010](0010-electrical-model-proven-guarantees.md) | Garanties mathématiques : deux retraits, une démonstration | **Proposé** | Méthode numérique — correction de garanties |
| [0011](0011-electrical-model-formalisation.md) | Formalisation : 35 propriétés prouvées, 35 invariants logiciels | **Proposé** | Méthode numérique + spécification formelle |

Les ADR 0001-0003 sont **rétroactifs** : ils documentent des décisions
déjà prises et mises en œuvre pendant la Phase 1, rédigés au moment de la
clôture de la gouvernance scientifique (Phase 1.5) pour que la politique
ADR démarre avec un historique complet plutôt qu'à vide. Tout ADR
numéroté à partir de 0004 documente une décision prise **après**
l'adoption de cette politique.
