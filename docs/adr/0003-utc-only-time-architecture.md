# ADR-0003 — Le moteur travaille exclusivement en temps UTC

**Statut** : Accepté
**Date** : rétroactif, décision prise dès la conception de `core_math::julian`
**Catégorie** : Nouvelle architecture

## Contexte

Un calcul de position solaire a besoin d'un instant précis. Les
utilisateurs finaux du produit raisonneront en heure locale civile (avec
fuseau horaire et, dans de nombreux pays, changement heure d'été/heure
d'hiver). La question : le moteur scientifique doit-il accepter des
horaires civils avec fuseau/DST directement, ou exiger du code appelant
qu'il convertisse en UTC au préalable ?

## Options considérées

### Option A — Le moteur accepte une heure civile + un identifiant de fuseau

- Avantages : plus pratique pour un appelant naïf, pas de conversion à
  faire en amont.
- Inconvénients : nécessite d'embarquer ou de dépendre d'une base de
  données de fuseaux horaires/DST (viole la politique zéro dépendance,
  ADR-0002, une base de données IANA tz étant volumineuse et mise à jour
  régulièrement) ; couple le moteur scientifique — censé être pur et
  déterministe — à une préoccupation de couche application ; ambiguïté
  inhérente aux transitions DST (une heure locale peut correspondre à
  zéro ou deux instants UTC lors d'un changement d'heure).

### Option B — Le moteur n'accepte que des champs de calendrier UTC

- Avantages : aucune dépendance à une base de données de fuseaux ;
  aucune ambiguïté (UTC n'a pas de transition DST) ; le moteur reste pur
  et déterministe ; la responsabilité de la conversion fuseau→UTC est
  clairement laissée à la couche application, qui a de toute façon besoin
  de cette conversion pour d'autres raisons (affichage, journalisation).
- Inconvénients : tout appelant doit effectuer la conversion en amont ;
  aucune commodité de fuseau horaire dans l'API du moteur.

## Décision

**Option B.** Le critère décisif : le principe de déterminisme
(`docs/scientific-principles.md` §6) et la politique zéro dépendance
(ADR-0002) excluent ensemble l'option A — une base de données de fuseaux
horaires est à la fois une dépendance externe volumineuse et une source
potentielle d'ambiguïté (transitions DST). `core_math::julian::CalendarDateTime`
ne porte donc aucune information de fuseau ; toutes les crates du moteur
travaillent exclusivement en UTC.

## Conséquences

- Simplifie radicalement le moteur : `to_julian_day()` est une fonction
  pure de champs de calendrier UTC, sans état ni base de données externe.
- Reporte la responsabilité de conversion fuseau/DST à la couche
  application (non encore développée à ce jour — voir
  `docs/design/21-matrice-tracabilite.md`, EF-001).
- A directement motivé la conception de la campagne de validation
  mondiale (`validation::global_campaign`) : plutôt que de simuler des
  fuseaux civils, la couverture « changements d'heure »/« fuseaux
  horaires » demandée est représentée par le balayage complet de
  longitude (tout décalage UTC possible) et la catégorie
  `near_midnight_utc` (limites de jour UTC) — voir la documentation de
  module de `engine/crates/validation/src/global_campaign/mod.rs` pour la
  justification complète de cette équivalence.
- Aucune dette scientifique introduite : ce n'est pas une approximation,
  c'est une séparation de responsabilités délibérée entre le moteur
  scientifique et la couche application.

## Références

- `engine/crates/core_math/src/julian.rs` §« Architecture decision:
  engine time is always UTC ».
- `engine/crates/validation/src/global_campaign/mod.rs` §« Scope note:
  "changements d'heure" and "fuseaux horaires" ».
