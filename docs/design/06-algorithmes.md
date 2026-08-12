# 6. Algorithmes

## 6.1 Position solaire

- **Algorithme** : NREL Solar Position Algorithm (SPA, Reda & Andreas 2004),
  précision ±0.0003°. Implémentation propre en Rust (performance, appelé
  8760+ fois par simulation).
- **Alternative rapide** : algorithme de Michalsky (1988) en mode brouillon
  (preview UI temps réel), bascule vers SPA pour le calcul certifié.
- Entrées : latitude, longitude, altitude, timestamp UTC, pression, température.
- Sorties : azimut, élévation/zénith solaire, angle horaire, déclinaison.

## 6.2 Décomposition et transposition du rayonnement

1. **Décomposition GHI → DNI/DHI** (si seule la GHI est disponible) :
   - Modèle **Erbs et al. (1982)** pour estimation rapide.
   - Modèle **DIRINT/DISC (Perez/Maxwell, NREL)** pour meilleure précision, utilisé
     par défaut.
2. **Transposition plan incliné** : modèle **Perez anisotropique (1990)**
   (par défaut, référence industrie), avec option **Hay-Davies** et **isotrope**
   pour comparaison/sensibilité.
3. **Modificateur d'angle d'incidence (IAM)** : modèle **ASHRAE / Martin-Ruiz**
   pour les pertes de réflexion angulaire.

## 6.3 Ombrage

- **Horizon lointain** : ray-casting sur le MNT/MNS (grille altimétrique) autour
  du site, calcul du profil d'horizon par pas azimutal (1°), comparaison à
  l'élévation solaire horaire → facteur d'ombrage binaire ou partiel (bord de
  masque via anti-aliasing sur sous-pas).
- **Ombrage proche (objets 3D)** : algorithme de lancer de rayons (BVH — Bounding
  Volume Hierarchy) sur la scène 3D (bâtiments, cheminées, végétation, rangées
  voisines) pour chaque cellule/sous-chaîne représentative.
- **Ombrage mutuel entre rangées (near-shading, structures fixes ou trackers)** :
  calcul géométrique analytique (facteur d'ombrage linéaire par rangée, méthode
  View-Factor) complété par ray-casting pour configurations non régulières.
- **Backtracking trackers** : algorithme de calcul de l'angle de rotation limitant
  l'auto-ombrage inter-rangées (formule standard trackers axe unique), avec
  bascule vers angle solaire pur si l'espacement rend le backtracking inutile.
- **Effet électrique de l'ombrage partiel** : répartition par sous-chaîne/diode de
  bypass, calcul de la courbe I-V résultante par sommation des courbes de
  sous-modules (accroché aux minima de tension par diode) — voir §7.4.

## 6.4 Modèle électrique du générateur PV

- **Modèle single-diode à 5 paramètres** (De Soto et al. 2006), résolu par
  méthode itérative (Newton-Raphson / Lambert W) pour obtenir la courbe I-V à
  chaque pas de temps, à partir des conditions d'irradiance/température réelles.
- **Alternative rapide** : modèle simplifié PVWatts-like pour le mode
  pré-dimensionnement instantané (résultat en < 1 s), avec écart affiché vs.
  modèle complet.
- **Recherche du point de puissance maximale (MPP)** : recherche par dichotomie/
  Golden-section sur la courbe I-V simulée.

## 6.5 Pertes onduleur et MPPT

- **Modèle de rendement dynamique** (courbe rendement vs. charge, type
  Sandia Inverter Model / EN 50530), interpolation sur la courbe constructeur.
- **Clipping** : écrêtage de puissance si P_DC × η > P_AC,max, avec comptabilisation
  des pertes de clipping.
- **Pertes MPPT multi-string** : simulation indépendante par MPPT lorsque plusieurs
  orientations/inclinaisons sont connectées à un même onduleur.

## 6.6 Simulation batteries

- **Modèle de capacité** : Kinetic Battery Model (KiBaM, Manwell & McGowan 1993)
  pour la disponibilité d'énergie en fonction du régime de charge/décharge.
- **SOC** : intégration du bilan de courant avec rendement coulombique et
  rendement aller-retour (round-trip efficiency).
- **Vieillissement (SOH)** : comptage de cycles par la méthode **Rainflow**
  (ASTM E1049) appliqué au profil de SOC, combiné à un modèle de vieillissement
  calendaire (fonction de la température et du SOC moyen).
- **Limites opérationnelles** : bornage SOC min/max, limitation en puissance
  (C-rate), protection contre décharge profonde.

## 6.7 Dispatch micro-réseau

- **Mode règles heuristiques** (par défaut, rapide) : priorité PV → charge directe,
  surplus → batterie, surplus résiduel → réseau/export, déficit → batterie puis
  groupe électrogène puis réseau, selon stratégie sélectionnée (autoconsommation,
  effacement de pointe, secours).
- **Mode optimisé (MILP)** : formulation en programmation linéaire mixte (variables
  binaires pour marche/arrêt groupe, contraintes de puissance/énergie), résolu par
  solveur **HiGHS/CBC** (Pyomo/OR-Tools), minimisation du coût total ou du carburant
  sur l'horizon de simulation (glissant, type MPC simplifié en roadmap V2).
- **Indicateur de fiabilité** : LPSP (Loss of Power Supply Probability) calculé sur
  la série horaire simulée.

## 6.8 Pompage solaire

- **Couplage direct PV-pompe** : recherche du point de fonctionnement à
  l'intersection de la courbe P-V du générateur (à l'irradiance/température du
  pas de temps) et de la courbe de charge électrique de la pompe (via variateur
  MPPT dédié), par résolution itérative.
- **Courbe hydraulique** : calcul du débit à partir de la hauteur manométrique
  totale (HMT = hauteur géométrique + pertes de charge) et de la courbe
  caractéristique H-Q-η de la pompe (interpolation catalogue).
- **Stockage hydraulique** : bilan de niveau de réservoir horaire en fonction du
  débit pompé et du besoin, avec vérification de débordement/pénurie.

## 6.9 Optimisation multi-objectif du dimensionnement

- **Algorithme principal** : **NSGA-II** (Non-dominated Sorting Genetic Algorithm)
  pour l'exploration multi-objectif (ex. minimiser LCOE et maximiser taux de
  couverture, sous contraintes budgétaires et normatives).
- **Sous-problèmes linéaires** (ex. répartition optimale de capacité batterie sur
  un dispatch donné) : résolution **MILP** directe pour rapidité et optimalité
  garantie localement.
- **Variables coûteuses à évaluer** (ex. calepinage 3D complexe) : **optimisation
  bayésienne** (processus gaussien) pour limiter le nombre d'évaluations complètes.
- **Parallélisation** : évaluation de la population répartie sur un pool de
  workers (§4.8), avec cache des résultats de simulation identiques (hash de
  configuration).
- **Contraintes bloquantes** (RG-002/003/008) : filtrage des individus non
  réalisables avant évaluation complète (pénalité infinie / rejet).

## 6.10 Placement automatique de modules (calepinage)

- **Algorithme de bin-packing contraint** sur la surface disponible (toiture/sol),
  avec contraintes de distances aux bords, zones d'exclusion, orientation des
  rangées, pas de structure ; heuristique gloutonne (balayage en bandes) suivie
  d'un raffinement local (recherche taboue) pour maximiser la puissance installée
  ou un critère de production.

## 6.11 Calcul de câblage et protections électriques

- **Section de câble** : calcul par la méthode des courants admissibles
  (référentiel IEC 60364-5-52 / NF C 15-100) combiné à la vérification de chute
  de tension maximale admissible (seuil paramétrable, typiquement 1-3 % DC).
- **Protections** : calcul du courant de court-circuit maximal par chaîne (Isc ×
  1.25 selon IEC 62548), calibre fusible/disjoncteur, dimensionnement parafoudre
  selon la classe de risque foudre (IEC 62305), vérification de la tenue en
  tension (Voc à température minimale historique du site).

## 6.12 Prévision et IA (modèles d'apprentissage)

- **Prévision de production/consommation** : modèles de séries temporelles
  (Prophet pour tendance/saisonnalité, LSTM/Temporal Fusion Transformer pour
  prévision court-terme avec covariables météo).
- **Détection d'anomalies de simulation** : modèle d'isolement (Isolation Forest)
  entraîné sur la distribution des indicateurs de performance historiques du
  parc de projets, pour signaler un résultat statistiquement improbable.
- **Assistant de pré-dimensionnement** : système de recommandation basé sur des
  règles expertes pondérées + apprentissage supervisé sur l'historique de
  projets similaires (typologie site, climat, usage) pour proposer une
  configuration initiale raisonnable, toujours soumise à validation utilisateur.
