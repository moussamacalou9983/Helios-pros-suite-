# 13. Formats d'import/export

## 13.1 Principe

Aucun format propriétaire fermé n'est imposé à l'utilisateur : toute donnée
entrée ou produite par HeliosPro est exportable dans un format ouvert et
documenté. Les formats propriétaires tiers (ex. fiches `.PAN`/`.OND`) sont
supportés **en import uniquement**, à titre de compatibilité pour la migration
de bibliothèques d'équipements existantes des utilisateurs — sans reproduction
d'aucune logique ou interface du logiciel d'origine, uniquement un parseur de
fichier de données.

## 13.2 Import

| Domaine | Formats supportés |
|---|---|
| Météo | TMY3 (NREL), EPW (DOE), CSV générique (mapping de colonnes assisté), API PVGIS, API NASA POWER, Meteonorm (export CSV) |
| SIG / géométrie | Shapefile (.shp), GeoJSON, KML/KMZ, GeoTIFF (MNT/MNS), nuages de points LAS/LAZ (LiDAR) |
| BIM / CAO | IFC (bâtiment), DXF/DWG (implantation 2D) |
| Équipements | CSV/Excel (template HeliosPro), fiches `.PAN`/`.OND` (parseur de compatibilité), PDF datasheet (extraction assistée par IA avec validation humaine obligatoire) |
| Profils de charge | CSV horaire/sub-horaire, format Load Profile Generator, courbes de charge Enedis (France, format Enedis) |
| Projets tiers | Import de résultats de simulation externes en CSV pour comparaison (aucun parsing de projets binaires propriétaires) |

## 13.3 Export

| Domaine | Formats |
|---|---|
| Rapports | PDF, DOCX (via templates éditables), HTML |
| Données tabulaires | Excel (.xlsx), CSV |
| Séries temporelles | CSV, Parquet (analyse data science) |
| Géodonnées | GeoJSON, KML, Shapefile, GeoTIFF |
| Implantation | DXF (calepinage 2D), IFC (maquette BIM enrichie) |
| Dossier administratif | PDF structuré (dossier de raccordement, notice technique Consuel-compatible pour la France) |
| API / intégration | JSON (REST), GraphQL, Webhooks (événements projet), OpenAPI 3.1 spec publique |
| Échange réseau (roadmap V2) | XML, formats compatibles OpenADR / IEC 61850 pour les échanges DSO |

## 13.4 API publique

- **REST** (ressources : `projects`, `scenarios`, `simulations`, `optimizations`,
  `equipment`, `reports`) documentée en **OpenAPI 3.1**, versionnée (`/v1/`).
- **GraphQL** pour les requêtes composites côté frontend (évite le
  sur/sous-fetching sur les objets projet complexes).
- **Webhooks** sortants pour intégration SI tiers (ex. CRM, ERP du BE) :
  `project.created`, `simulation.completed`, `report.generated`.
- Authentification API par clé (organisation) ou OAuth2 (utilisateur délégué),
  quotas et rate-limiting par plan d'abonnement.

## 13.5 Interopérabilité équipements

Les fabricants et bases ouvertes (ex. référentiels CEC, bases publiques de
fiches techniques) peuvent être synchronisés via connecteurs configurables
(mapping de champs déclaratif), avec validation automatique de cohérence
physique avant intégration au catalogue (ex. `Vmpp < Voc`, cohérence des
coefficients de température).
