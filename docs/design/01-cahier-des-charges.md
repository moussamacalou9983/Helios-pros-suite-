# 1. Cahier des charges — HeliosPro Engineering Suite

## 1.1 Contexte

Les bureaux d'études (BE) photovoltaïques utilisent aujourd'hui des outils cloisonnés
par technologie (PVsyst, PV*SOL, Archelios Calc pour le PV ; HOMER ou StorageVET pour
le stockage/microgrid ; RETScreen pour l'économie ; QGIS pour le SIG). Cette
fragmentation impose des ressaisies, des ruptures de traçabilité et un temps d'étude
élevé. HeliosPro répond à ce besoin par une plateforme unifiée, web-first, collaborative
et extensible.

## 1.2 Objectifs du produit

- **O1** — Fournir un moteur de simulation énergétique horaire (et sub-horaire, pas
  de temps configurable jusqu'à 1 min) couvrant PV, stockage, microgrid, pompage
  solaire et IRVE dans un modèle énergétique unique et cohérent.
- **O2** — Garantir une précision scientifique validée (écart < 3 % sur la production
  annuelle par rapport aux mesures de sites de référence et aux outils établis, sur
  un panel de cas de validation publiés).
- **O3** — Intégrer nativement un SIG (cadastre, MNT/MNS, occupation du sol, calcul
  d'ombrage lointain/proche) sans dépendance à un logiciel tiers.
- **O4** — Automatiser l'optimisation multi-objectif du dimensionnement (coût, LCOE,
  taux d'autoconsommation, empreinte carbone, contraintes de raccordement).
- **O5** — Générer une documentation d'étude conforme aux exigences normatives et
  administratives (dossier de raccordement, notice technique, note de calcul).
- **O6** — Permettre le travail collaboratif multi-utilisateur en temps quasi réel
  sur un même projet, avec historique et droits d'accès.
- **O7** — Offrir une IA d'aide à la décision (pré-dimensionnement automatique,
  détection d'anomalies de simulation, prévision de production/consommation,
  recommandations d'optimisation).
- **O8** — Assurer l'interopérabilité par API ouverte et formats d'échange
  standards (aucun format propriétaire fermé imposé à l'utilisateur).
- **O9** — Être déployable en mode SaaS multi-tenant, ou on-premise pour les BE
  ayant des contraintes de confidentialité (marchés publics, défense, données
  sensibles).

## 1.3 Utilisateurs cibles (personas)

| Persona | Besoin dominant |
|---|---|
| Ingénieur BE solaire toiture résidentielle/tertiaire | Rapidité de saisie, génération de devis et notice technique |
| Ingénieur BE centrale au sol / agrivoltaïsme | Précision d'ombrage à grande échelle, optimisation de calepinage, LCOE |
| Ingénieur BE off-grid / électrification rurale | Couplage PV-batterie-diesel, dimensionnement fiabilité (LPSP), pompage solaire |
| Bureau d'études microgrid industriel | Dispatch énergétique, gestion de pointe, ilotage, ANM (agrégation de multiples sources) |
| Chef de projet / développeur | Analyse financière, scénarios de sensibilité, rapports investisseurs |
| Responsable qualité / vérificateur normatif | Traçabilité des calculs, conformité IEC/NF, contrôle des résultats |
| Administrateur SI du BE | Gestion des utilisateurs, licences, sécurité, sauvegarde |

## 1.4 Exigences fonctionnelles (extrait — détail en §2 et §5)

Identifiants `EF-xxx`, priorité MoSCoW (Must/Should/Could/Won't pour la V1).

| ID | Exigence | Priorité |
|---|---|---|
| EF-001 | Créer un projet géolocalisé à partir d'une adresse, de coordonnées ou d'un tracé cadastral | Must |
| EF-002 | Récupérer automatiquement des données météo horaires typiques (TMY) depuis plusieurs sources (PVGIS, NASA POWER, Meteonorm, stations locales importées) | Must |
| EF-003 | Modéliser la géométrie 3D du site (toiture, obstacles, relief) et calculer les masques proches et lointains | Must |
| EF-004 | Concevoir un générateur PV (modules, chaînes, onduleurs, câblage) avec vérification électrique automatique | Must |
| EF-005 | Simuler la production horaire sur une année type et sur N années avec dégradation | Must |
| EF-006 | Dimensionner un système de stockage batterie (capacité, puissance, stratégie de pilotage) | Must |
| EF-007 | Simuler un micro-réseau hybride (PV + stockage + groupe électrogène + réseau) avec stratégies de dispatch | Should |
| EF-008 | Dimensionner un système de pompage solaire (au fil du soleil ou avec stockage hydraulique/batterie) | Should |
| EF-009 | Dimensionner une infrastructure de recharge VE couplée au productible PV | Should |
| EF-010 | Réaliser une analyse économique (LCOE, TRI, VAN, temps de retour, cashflows) avec scénarios | Must |
| EF-011 | Optimiser automatiquement le dimensionnement selon objectifs et contraintes définis par l'utilisateur | Should |
| EF-012 | Générer un rapport d'étude personnalisable (PDF/DOCX) incluant les justificatifs normatifs | Must |
| EF-013 | Importer/exporter des données (météo, calepinage, résultats) dans des formats standards | Must |
| EF-014 | Fournir une assistance IA (recommandations de dimensionnement, détection d'anomalies) | Could (V1), Should (V2) |
| EF-015 | Permettre la collaboration multi-utilisateur en temps réel sur un projet | Should |
| EF-016 | Gérer une base de données d'équipements (modules, onduleurs, batteries, pompes, câbles) mutualisée et versionnée | Must |
| EF-017 | Vérifier automatiquement la conformité électrique (IEC 60364/NF C 15-100, IEC 62548) | Must |
| EF-018 | Prendre en charge le bifacial et l'agrivoltaïsme (albédo, hauteur de structure, transmission) | Should |
| EF-019 | Prendre en charge les trackers (axe unique/double) avec algorithme de backtracking | Should |
| EF-020 | Exposer une API publique documentée (REST + Webhooks) pour intégration tierce | Should |

## 1.5 Exigences non fonctionnelles

| ID | Catégorie | Exigence |
|---|---|---|
| ENF-001 | Performance | Simulation horaire annuelle (8760 pas) d'un système PV+batterie < 5 s sur infrastructure de référence |
| ENF-002 | Performance | Optimisation multi-objectif (population 200, 100 générations) < 5 min pour un système simple |
| ENF-003 | Précision | Écart production annuelle vs. mesures de référence < 3 %, vs. corrélation inter-outils < 5 % |
| ENF-004 | Disponibilité | SLA SaaS ≥ 99.5 % (hors maintenance planifiée) |
| ENF-005 | Sécurité | Chiffrement des données au repos (AES-256) et en transit (TLS 1.3), authentification OIDC/MFA |
| ENF-006 | Portabilité | Web (Chrome/Firefox/Edge/Safari récents) + Desktop (Windows/macOS/Linux via Tauri) |
| ENF-007 | Localisation | i18n complète (FR, EN, ES, PT, AR minimum en V1), unités métriques/impériales |
| ENF-008 | Accessibilité | Conformité WCAG 2.1 AA sur les parcours critiques |
| ENF-009 | Maintenabilité | Couverture de tests ≥ 85 % sur le cœur métier, ≥ 95 % sur les modèles physiques |
| ENF-010 | Scalabilité | Support de projets jusqu'à 500 MWc / >100 000 modules sans dégradation d'UX |
| ENF-011 | Auditabilité | Journal d'audit immuable de toute action modifiant un projet ou un résultat certifié |
| ENF-012 | Interopérabilité | Aucune donnée « prisonnière » : export intégral possible à tout moment (JSON/CSV/PDF) |
| ENF-013 | Confidentialité | Isolation stricte multi-tenant (schema-per-tenant ou row-level security), option on-premise |

## 1.6 Contraintes réglementaires et normatives

Voir §8 pour la liste complète. Contraintes structurantes :
- Conformité IEC 60364-7-712 / NF C 15-100 (installations électriques BT PV) pour les
  vérifications de câblage, protections et chutes de tension.
- Conformité IEC 62548 (exigences de conception des générateurs PV).
- Formats météo compatibles ISO/normes internationales (TMY3, EPW) pour garantir la
  traçabilité des sources climatiques.
- RGPD (UE) pour les données utilisateurs et projets hébergés en mode SaaS.
- ISO/IEC 25010 comme référentiel qualité logiciel interne (voir §14).

## 1.7 Périmètre V1 (in/out)

**Dans le périmètre V1 :** PV toiture/sol/ombrières (mono/bifacial), stockage
batterie résidentiel/C&I, dimensionnement électrique de base, simulation horaire,
analyse économique, rapport PDF, SIG 2.5D (MNT + emprises bâtiment), import météo
TMY3/EPW/PVGIS, base équipements, API REST V1.

**Hors périmètre V1 (roadmap V2+) :** micro-réseaux multi-sources complexes avec
MPC (Model Predictive Control), pompage solaire, IRVE avancé (V2G), IA générative
de rapport, marketplace d'équipements tiers, module BIM/IFC bidirectionnel complet.

## 1.8 Glossaire (extrait)

| Terme | Définition |
|---|---|
| TMY | Typical Meteorological Year — année météorologique type |
| LCOE | Levelized Cost of Energy — coût actualisé de l'énergie |
| LPSP | Loss of Power Supply Probability — probabilité de perte de charge (off-grid) |
| SOC / SOH | State of Charge / State of Health (batteries) |
| MPPT | Maximum Power Point Tracking |
| IAM | Incidence Angle Modifier |
| MNT / MNS | Modèle Numérique de Terrain / de Surface |
| GHI/DNI/DHI | Global/Direct Normal/Diffuse Horizontal Irradiance |
