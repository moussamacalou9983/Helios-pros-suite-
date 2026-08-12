# 3. Analyse des besoins des bureaux d'études

## 3.1 Constats sur les outils existants (analyse fonctionnelle indépendante)

Cette analyse se fonde sur des constats fonctionnels génériques et publiquement
documentés sur la catégorie d'outils de dimensionnement PV (limitations connues du
marché), sans accès ni reproduction d'aucun code, interface ou donnée propriétaire.

| Limite constatée sur le marché | Impact pour le BE |
|---|---|
| Un seul type de technologie par outil (PV seul, ou stockage seul) | Multiplication des outils, ressaisie des mêmes données site/météo |
| Fichiers projet locaux non versionnés | Perte de traçabilité, conflits de version en équipe |
| Ombrage 3D limité ou nécessitant un import CAO externe | Temps de modélisation élevé pour les toitures complexes |
| Base d'équipements figée, mise à jour manuelle | Modules/onduleurs récents indisponibles, risque d'erreur |
| Optimisation absente ou limitée à un paramètre | Dimensionnement sous-optimal, itérations manuelles chronophages |
| Rapport figé, peu personnalisable | Reformattage manuel pour chaque client/AO |
| Pas d'API ouverte | Impossible d'automatiser les études en série (grands portefeuilles) |
| Licence par poste, pas de collaboration cloud | Frein au travail à distance et à la sous-traitance |

## 3.2 Besoins exprimés par persona (enquête simulée à dire d'expert métier)

### BE solaire toiture résidentielle/tertiaire
- Saisie rapide (< 15 min) d'un projet simple avec devis en sortie.
- Bibliothèque de modules/onduleurs à jour, filtrable par disponibilité fournisseur.
- Vérification automatique des règles NF C 15-100 / Consuel.
- Génération directe de la notice technique de raccordement.

### BE centrale au sol / agrivoltaïsme
- Import LiDAR/MNT haute résolution, calepinage automatique sous contraintes
  (distances, pentes, zones exclues, hauteur de structure agrivoltaïque).
- Simulation bifacial avec albédo variable selon culture/saison.
- Optimisation du ratio DC/AC et de l'inclinaison/orientation à grande échelle
  (>10 000 modules) en temps raisonnable.

### BE off-grid / électrification rurale
- Dimensionnement fiabilité (LPSP, autonomie en jours) plutôt que ROI seul.
- Couplage PV-batterie-groupe électrogène avec stratégies de dispatch comparables.
- Prise en compte de profils de charge non standards (activités productives,
  pompage, réfrigération solaire).

### BE microgrid industriel
- Gestion de pointe (peak shaving), ilotage, contraintes de raccordement DSO.
- Simulation multi-sources avec règles de dispatch paramétrables ou optimisées
  (MILP), horizon multi-année avec évolution de charge.

### Chef de projet / développeur
- Comparaison de scénarios financiers (autoconsommation vs. revente, PPA, tiers-
  investissement), sensibilité aux hypothèses (prix électricité, taux d'actualisation).
- Rapports orientés investisseurs, exportables et personnalisables à la charte
  graphique du BE ou du client.

## 3.3 Exigences dérivées (traçabilité vers §1)

| Besoin exprimé | Exigence associée |
|---|---|
| Multi-technologies unifiées | O1, EF-006 à EF-009 |
| Collaboration cloud versionnée | O6, EF-015 |
| SIG et ombrage natifs | O3, EF-003 |
| Base équipements à jour et mutualisée | EF-016 |
| Optimisation automatique | O4, EF-011 |
| Rapport personnalisable et conforme | O5, EF-012 |
| API pour automatisation en série | O8, EF-020 |
| Dimensionnement fiabilité off-grid (LPSP) | EF-006, RG-005 |
| Dispatch microgrid paramétrable/optimisé | EF-007 |

## 3.4 Benchmark comparatif fonctionnel (positionnement HeliosPro)

| Fonctionnalité | Outils PV mono-techno traditionnels (catégorie) | HeliosPro |
|---|---|---|
| Simulation horaire PV | Oui | Oui (+ pas de temps configurable jusqu'à 1 min) |
| Stockage batterie intégré | Rarement natif | Natif, couplé au même moteur horaire |
| Microgrid hybride | Outil séparé | Intégré |
| Pompage solaire | Outil séparé | Intégré |
| IRVE | Absent | Intégré |
| SIG natif + ombrage 3D | Import externe | Natif |
| Optimisation multi-objectif | Absente/limitée | Native (génétique + MILP) |
| IA d'aide au dimensionnement | Absente | Native |
| Collaboration cloud | Absente/limitée | Native (temps réel) |
| API ouverte | Rare | REST/GraphQL documentée |
| On-premise pour confidentialité | Desktop uniquement | SaaS et on-premise |

## 3.5 Priorisation issue de l'analyse besoins → Roadmap

Les besoins les plus fréquemment cités (saisie rapide, précision, conformité
normative, rapport) orientent le MVP (§15 Phase 1) vers : PV toiture/sol,
simulation horaire précise, vérifications électriques normatives, génération de
rapport. Les besoins de niche à forte valeur ajoutée différenciante (microgrid,
pompage, IRVE, IA) sont planifiés en phases 3-4, une fois le socle de précision
scientifique validé et adopté.
