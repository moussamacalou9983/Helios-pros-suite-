# HeliosPro — Rapport de performance (moteur de position solaire)

Genere par `tests::benchmarks::generate_and_write_benchmark_report` (`cargo test -p tests --release`), sur un an d'appels horaires (8760 positions), a chaque execution de la suite de tests. Mesures manuelles via `std::time::Instant` et un allocateur global compteur (voir le commentaire de module de ce fichier) -- aucune dependance de benchmarking externe.

| Mesure | SPA | Meeus (reduit) |
|---|---|---|
| Temps total (8760 appels) | 47.01 ms | 4.87 ms |
| Temps moyen par appel | 5.183 us | 0.607 us |
| Ecart-type par appel | 2.081 us | 0.936 us |
| Minimum / maximum par appel | 4.730 / 57.606 us | 0.467 / 65.273 us |
| Appels allocateur (8760 positions) | 0 | 0 |
| Octets alloues (8760 positions) | 0 | 0 |

Ratio de temps moyen SPA/Meeus : 8.53x.
