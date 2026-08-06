# Procédure de revue scientifique

**Statut** : obligatoire à partir de la clôture de la Phase 1, pour tout
nouveau modèle physique ajouté au moteur `engine/` (position solaire déjà
livrée, ou tout futur moteur de la roadmap scientifique — voir
`docs/design/22-roadmap-scientifique.md`). Applique le principe de
« validation indépendante » de `docs/scientific-principles.md`.

**Règle absolue** : **aucun modèle scientifique ne peut être fusionné sur
la branche principale sans avoir satisfait aux six critères ci-dessous.**
Un modèle qui échoue un critère reste en développement (branche non
fusionnée) jusqu'à correction — jamais fusionné avec une case cochée
« provisoirement » ou une exception non documentée.

## Portée

S'applique à tout ajout ou modification qui introduit ou change un
**modèle physique** : une nouvelle formule, un nouveau coefficient
empirique, un nouvel algorithme, ou une modification de domaine de
validité d'un modèle existant. Ne s'applique **pas** aux changements
purement mécaniques (refactoring sans changement de comportement
numérique, correction de formatage, ajout de tests sur un modèle déjà
revu) — ceux-ci suivent la revue de code habituelle uniquement.

## Les six critères

### 1. Publications originales

- La publication source (article évalué par les pairs, ouvrage de
  référence, ou rapport technique d'un organisme reconnu — NREL, Sandia,
  NASA, etc.) est identifiée précisément : auteur(s), année, titre,
  revue/éditeur, et si possible numéro d'équation exact.
- La publication est **directement consultée** par la personne (ou
  l'agent) qui implémente le modèle — jamais une citation copiée d'une
  seconde source sans vérification que la formule correspond réellement à
  la publication citée en première main.
- Quand la publication primaire n'est pas accessible (paywall, absence
  d'accès réseau), une source secondaire fiable et largement citée
  (ex. Duffie & Beckman comme compilateur de nombreuses corrélations
  solaires) peut servir de référence de travail, **à condition que cette
  substitution soit documentée explicitement** — voir l'exemple de la
  Phase 1 : tables SPA extraites via pvlib faute d'accès à nrel.gov,
  documenté et justifié dans `engine/validation-data/README.md`.

### 2. Normes

- Toute norme applicable (IEC, ISO, ASHRAE, CEC, IEEE) au domaine du
  modèle est identifiée et son rôle précisé : condition de référence
  (STC, NOCT), méthode de mesure, format de données, ou exigence de
  conformité.
- Voir `docs/references/bibliography.md` pour le registre déjà en place ;
  toute norme nouvellement citée y est ajoutée avec les modules qui
  l'utilisent.

### 3. Jeux de validation

- Le modèle est testé contre au moins un jeu de données de référence
  externe (mesures publiées, sortie d'un logiciel tiers reconnu comme
  pvlib, ou jeu de données de norme comme les courbes I-V IEC 61853).
- Le jeu de données est soit committé (`validation/golden/`), soit
  régénérable par un script documenté et reproductible (voir le principe
  de reproductibilité).
- L'écart mesuré (erreur absolue max, RMSE, biais) est publié dans un
  rapport auto-généré, jamais estimé.

### 4. Benchmarks

- Le modèle est mesuré en performance (temps moyen, écart-type au
  minimum ; mémoire/allocations si le modèle a un profil d'allocation non
  trivial) sur un scénario représentatif — suivant le patron déjà en
  place dans `engine/crates/tests/tests/benchmarks.rs`.
- Un budget de non-régression est fixé et testé automatiquement (pas
  seulement mesuré une fois puis oublié).

### 5. Revue documentaire

- Chaque fonction publique du module porte un commentaire de
  documentation citant sa source (principe « aucune formule sans
  publication »).
- Le module expose, dans sa documentation de premier niveau (`//!` en
  tête de fichier), ses hypothèses, ses limites et son domaine de
  validité — pas seulement ce qu'il calcule, mais dans quelles conditions
  le résultat est fiable.
- La couverture de documentation est vérifiée mécaniquement :
  `RUSTFLAGS="-D missing_docs" cargo doc --workspace --no-deps` doit
  rester à zéro avertissement après l'ajout du module.

### 6. Analyse des limites

- Toute approximation, troncature, ou domaine de validité restreint est
  identifié explicitement et **enregistré dans le registre de dette
  scientifique** (`docs/scientific-debt-register.md`) — jamais laissé
  implicite dans le code seul.
- Les cas limites pertinents pour le modèle (valeurs extrêmes, angles
  singuliers, domaines hors validité) sont couverts par des tests de
  stabilité numérique dédiés, suivant le patron de
  `engine/crates/tests/tests/numerical_stability.rs`.

## Processus concret

1. **Avant l'implémentation** : un ADR est ouvert
   (`docs/governance/adr-policy.md`) documentant le choix du modèle, les
   publications retenues, et le domaine de validité visé.
2. **Pendant l'implémentation** : chaque équation porte sa source au
   moment où elle est écrite, pas après coup.
3. **Avant la demande de fusion** : les six critères ci-dessus sont
   vérifiés un par un ; toute case non satisfaite bloque la fusion.
4. **À la fusion** : le rapport de clôture de la sous-phase correspondante
   (`docs/templates/phase-closure-report-template.md`) documente les six
   critères et leur statut, et le registre de dette scientifique est mis
   à jour si une approximation nouvelle a été introduite.
5. **Après la fusion** : le modèle entre dans le périmètre couvert par la
   CI (`.github/workflows/ci.yml`) — toute régression future sur ses
   tests de validation ou de stabilité bloque automatiquement les
   contributions suivantes.

## Non-négociable

Cette procédure ne peut pas être court-circuitée par un délai de
livraison. Un modèle physique qui n'a pas encore satisfait les six
critères reste au statut « squelette documenté » (comme le sont
aujourd'hui `pv_module`, `electrical_model`, `losses`) plutôt que d'être
fusionné avec une revue incomplète.
