# HeliosPro — Rapport de performance (moteur de position solaire)

Genere par `tests::benchmarks::generate_and_write_benchmark_report` (`cargo test -p tests --release`), sur un an d'appels horaires (8760 positions), a chaque execution de la suite de tests. Mesures manuelles via `std::time::Instant` et un allocateur global compteur (voir le commentaire de module de ce fichier) -- aucune dependance de benchmarking externe.

| Mesure | SPA | Meeus (reduit) |
|---|---|---|
| Temps total (8760 appels) | 35.67 ms | 4.00 ms |
| Temps moyen par appel | 4.107 us | 0.493 us |
| Ecart-type par appel | 1.497 us | 0.403 us |
| Minimum / maximum par appel | 3.594 / 54.411 us | 0.381 / 21.135 us |
| Appels allocateur (8760 positions) | 0 | 0 |
| Octets alloues (8760 positions) | 0 | 0 |

Ratio de temps moyen SPA/Meeus : 8.34x.
