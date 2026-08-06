# HeliosPro — Rapport de performance (moteur de position solaire)

Genere par `tests::benchmarks::generate_and_write_benchmark_report` (`cargo test -p tests --release`), sur un an d'appels horaires (8760 positions), a chaque execution de la suite de tests. Mesures manuelles via `std::time::Instant` et un allocateur global compteur (voir le commentaire de module de ce fichier) -- aucune dependance de benchmarking externe.

| Mesure | SPA | Meeus (reduit) |
|---|---|---|
| Temps total (8760 appels) | 49.11 ms | 4.90 ms |
| Temps moyen par appel | 5.331 us | 0.592 us |
| Ecart-type par appel | 2.352 us | 0.712 us |
| Minimum / maximum par appel | 4.729 / 69.138 us | 0.474 / 36.505 us |
| Appels allocateur (8760 positions) | 0 | 0 |
| Octets alloues (8760 positions) | 0 | 0 |

Ratio de temps moyen SPA/Meeus : 9.00x.
