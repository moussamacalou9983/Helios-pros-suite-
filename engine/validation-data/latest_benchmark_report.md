# HeliosPro — Rapport de performance (moteur de position solaire)

Genere par `tests::benchmarks::generate_and_write_benchmark_report` (`cargo test -p tests --release`), sur un an d'appels horaires (8760 positions), a chaque execution de la suite de tests. Mesures manuelles via `std::time::Instant` et un allocateur global compteur (voir le commentaire de module de ce fichier) -- aucune dependance de benchmarking externe.

| Mesure | SPA | Meeus (reduit) |
|---|---|---|
| Temps total (8760 appels) | 34.53 ms | 3.91 ms |
| Temps moyen par appel | 4.044 us | 0.468 us |
| Ecart-type par appel | 1.406 us | 0.367 us |
| Minimum / maximum par appel | 3.467 / 43.042 us | 0.374 / 17.009 us |
| Appels allocateur (8760 positions) | 0 | 0 |
| Octets alloues (8760 positions) | 0 | 0 |

Ratio de temps moyen SPA/Meeus : 8.65x.
