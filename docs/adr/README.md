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

Les ADR 0001-0003 sont **rétroactifs** : ils documentent des décisions
déjà prises et mises en œuvre pendant la Phase 1, rédigés au moment de la
clôture de la gouvernance scientifique (Phase 1.5) pour que la politique
ADR démarre avec un historique complet plutôt qu'à vide. Tout ADR
numéroté à partir de 0004 documente une décision prise **après**
l'adoption de cette politique.
