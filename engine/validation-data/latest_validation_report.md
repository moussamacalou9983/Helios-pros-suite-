# HeliosPro — Rapport de validation automatique

Genere par `validation::integration_tests::generate_and_write_full_validation_report` a partir de 91 cas de reference (voir `engine/validation-data/README.md`).

### SPA (Reda & Andreas, 2004, NREL/TP-560-34302) -- primary engine — 91 cas, 4.0 µs/cas (total 0.37 ms)

| Grandeur | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|
| Zenith (apparent) | -0.000005 deg | 0.000034 deg | 0.000038 deg | 0.000064 deg | singapore_20241221_1200 |
| Azimuth | -0.000026 deg | 0.000056 deg | 0.000063 deg | 0.000151 deg | golden_co_20240115_0800 |
| Elevation (apparent) | 0.000005 deg | 0.000034 deg | 0.000038 deg | 0.000064 deg | singapore_20241221_1200 |
| Hour angle (geocentric) | -0.000059 deg | 0.000060 deg | 0.000060 deg | 0.000070 deg | paris_20241221_1600 |
| Declination (geocentric) | 0.000000 deg | 0.000011 deg | 0.000015 deg | 0.000024 deg | paris_20240922_1600 |
| Equation of time | -0.000005 min | 0.000018 min | 0.000020 min | 0.000031 min | paris_20241221_1600 |
| Right ascension (geocentric) | 0.000059 deg | 0.000060 deg | 0.000060 deg | 0.000070 deg | paris_20241221_1600 |
| Earth radius vector | -0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | paris_20240922_1600 |

### Meeus reduced-order (Astronomical Algorithms, 1998) -- secondary validation engine — 91 cas, 0.5 µs/cas (total 0.05 ms)

| Grandeur | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|
| Zenith (apparent) | -0.002144 deg | 0.002541 deg | 0.003066 deg | 0.007576 deg | singapore_20241221_1200 |
| Azimuth | -0.001241 deg | 0.002536 deg | 0.003412 deg | 0.011822 deg | golden_co_20241221_0800 |
| Elevation (apparent) | 0.002144 deg | 0.002541 deg | 0.003066 deg | 0.007576 deg | singapore_20241221_1200 |
| Hour angle (geocentric) | -0.002728 deg | 0.002728 deg | 0.003315 deg | 0.005661 deg | singapore_20241221_1600 |
| Declination (geocentric) | -0.000154 deg | 0.000616 deg | 0.000977 deg | 0.002195 deg | paris_20240922_1600 |
| Equation of time | -0.012048 min | 0.017678 min | 0.023903 min | 0.044577 min | paris_20240922_1600 |
| Right ascension (geocentric) | 0.002727 deg | 0.002727 deg | 0.003313 deg | 0.005661 deg | paris_20241221_1600 |
| Earth radius vector | 0.000010 AU | 0.000019 AU | 0.000022 AU | 0.000039 AU | paris_20240621_1600 |
