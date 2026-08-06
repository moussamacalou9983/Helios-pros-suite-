# HeliosPro — Campagne de validation mondiale

Genere par `validation::integration_tests::generate_and_write_global_campaign_report` a partir de 2918 cas de reference couvrant l'enveloppe complete latitude/longitude/altitude/annee (voir `engine/validation-data/README.md`).

## Resume (toutes categories confondues)

### SPA (Reda & Andreas, 2004) — 2918 cas, 6.5 µs/cas (total 18.95 ms)

| Grandeur | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|
| Right ascension (geocentric) | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | century_2100_01 |
| Declination (geocentric) | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | leap_1904_18 |
| Equation of time | -0.000000 min | 0.000000 min | 0.000000 min | 0.000000 min | random_0544 |
| Hour angle (geocentric) | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_0430 |
| Elevation (apparent) | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_0363 |
| Zenith (apparent) | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_0363 |
| Azimuth | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_1334 |
| Earth radius vector | 0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | century_1900_01 |

### Meeus (1998, reduced order) — 2918 cas, 0.8 µs/cas (total 2.42 ms)

| Grandeur | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|
| Right ascension (geocentric) | 0.002114 deg | 0.002671 deg | 0.003213 deg | 0.009501 deg | random_1444 |
| Declination (geocentric) | 0.000111 deg | 0.000682 deg | 0.000926 deg | 0.003544 deg | random_2180 |
| Equation of time | -0.005500 min | 0.016700 min | 0.020566 min | 0.059883 min | random_1241 |
| Hour angle (geocentric) | -0.002116 deg | 0.002671 deg | 0.003214 deg | 0.009485 deg | random_1444 |
| Elevation (apparent) | 0.003522 deg | 0.003871 deg | 0.026565 deg | 0.623881 deg | sweep_subarctic_621_0030 |
| Zenith (apparent) | -0.003522 deg | 0.003871 deg | 0.026565 deg | 0.623881 deg | sweep_subarctic_621_0030 |
| Azimuth | -0.000183 deg | 0.002751 deg | 0.005441 deg | 0.188552 deg | random_0903 |
| Earth radius vector | 0.000007 AU | 0.000025 AU | 0.000029 AU | 0.000080 AU | random_0374 |

## Detail par categorie

### SPA (Reda & Andreas, 2004) — détail par catégorie (2918 cas)


#### Right ascension (geocentric)

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | century_2100_01 |
| extreme_altitude | 10 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_elev_0_-45.0 |
| extreme_latitude | 27 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_lat_0_-179.999 |
| extreme_longitude | 12 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_lon_0_-45.0 |
| leap_years | 28 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | leap_2096_00 |
| near_midnight_utc | 36 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | midnight_-60.0_-90.0_2358 |
| polar_day | 40 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | polar_day_-70.0_06 |
| polar_night | 40 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | polar_night_70.0_06 |
| random_stratified | 2200 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_0002 |
| subsolar_near_zenith | 30 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | subsolar_-23.44_1224 |
| sunrise_sunset_sweep | 480 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | sweep_equator_320_0030 |
| __all__ | 2918 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | century_2100_01 |

#### Declination (geocentric)

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | century_2000_29 |
| extreme_altitude | 10 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_elev_0_-45.0 |
| extreme_latitude | 27 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_lat_0_-179.999 |
| extreme_longitude | 12 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_lon_0_-45.0 |
| leap_years | 28 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | leap_1904_18 |
| near_midnight_utc | 36 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | midnight_-60.0_-90.0_00 |
| polar_day | 40 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | polar_day_-70.0_18 |
| polar_night | 40 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | polar_night_70.0_18 |
| random_stratified | 2200 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_0171 |
| subsolar_near_zenith | 30 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | subsolar_-23.44_1212 |
| sunrise_sunset_sweep | 480 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | sweep_equator_320_0030 |
| __all__ | 2918 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | leap_1904_18 |

#### Equation of time

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | 0.000000 min | 0.000000 min | 0.000000 min | 0.000000 min | century_2000_29 |
| extreme_altitude | 10 | 0.000000 min | 0.000000 min | 0.000000 min | 0.000000 min | extreme_elev_0_-45.0 |
| extreme_latitude | 27 | 0.000000 min | 0.000000 min | 0.000000 min | 0.000000 min | extreme_lat_0_-179.999 |
| extreme_longitude | 12 | 0.000000 min | 0.000000 min | 0.000000 min | 0.000000 min | extreme_lon_0_-45.0 |
| leap_years | 28 | 0.000000 min | 0.000000 min | 0.000000 min | 0.000000 min | leap_1996_18 |
| near_midnight_utc | 36 | 0.000000 min | 0.000000 min | 0.000000 min | 0.000000 min | midnight_-60.0_-90.0_00 |
| polar_day | 40 | 0.000000 min | 0.000000 min | 0.000000 min | 0.000000 min | polar_day_70.0_00 |
| polar_night | 40 | 0.000000 min | 0.000000 min | 0.000000 min | 0.000000 min | polar_night_70.0_00 |
| random_stratified | 2200 | -0.000000 min | 0.000000 min | 0.000000 min | 0.000000 min | random_0544 |
| subsolar_near_zenith | 30 | 0.000000 min | 0.000000 min | 0.000000 min | 0.000000 min | subsolar_0.0_1200 |
| sunrise_sunset_sweep | 480 | 0.000000 min | 0.000000 min | 0.000000 min | 0.000000 min | sweep_equator_320_0000 |
| __all__ | 2918 | -0.000000 min | 0.000000 min | 0.000000 min | 0.000000 min | random_0544 |

#### Hour angle (geocentric)

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | century_2100_01 |
| extreme_altitude | 10 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_elev_0_-45.0 |
| extreme_latitude | 27 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_lat_0_179.999 |
| extreme_longitude | 12 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_lon_1_-45.0 |
| leap_years | 28 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | leap_2024_18 |
| near_midnight_utc | 36 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | midnight_-60.0_90.0_2359 |
| polar_day | 40 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | polar_day_-70.0_06 |
| polar_night | 40 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | polar_night_70.0_06 |
| random_stratified | 2200 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_0430 |
| subsolar_near_zenith | 30 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | subsolar_0.0_1200 |
| sunrise_sunset_sweep | 480 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | sweep_equator_320_1130 |
| __all__ | 2918 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_0430 |

#### Elevation (apparent)

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | century_2100_01 |
| extreme_altitude | 10 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_elev_2_-45.0 |
| extreme_latitude | 27 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_lat_4_0.0 |
| extreme_longitude | 12 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_lon_1_0.0 |
| leap_years | 28 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | leap_2024_18 |
| near_midnight_utc | 36 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | midnight_0.0_90.0_2359 |
| polar_day | 40 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | polar_day_-70.0_06 |
| polar_night | 40 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | polar_night_70.0_06 |
| random_stratified | 2200 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_0363 |
| subsolar_near_zenith | 30 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | subsolar_-23.44_1224 |
| sunrise_sunset_sweep | 480 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | sweep_equator_320_0300 |
| __all__ | 2918 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_0363 |

#### Zenith (apparent)

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | century_2100_01 |
| extreme_altitude | 10 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_elev_4_-45.0 |
| extreme_latitude | 27 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_lat_3_-179.999 |
| extreme_longitude | 12 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_lon_2_-45.0 |
| leap_years | 28 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | leap_2024_18 |
| near_midnight_utc | 36 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | midnight_0.0_90.0_2359 |
| polar_day | 40 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | polar_day_-75.0_06 |
| polar_night | 40 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | polar_night_70.0_06 |
| random_stratified | 2200 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_0363 |
| subsolar_near_zenith | 30 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | subsolar_-23.44_1224 |
| sunrise_sunset_sweep | 480 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | sweep_equator_320_0030 |
| __all__ | 2918 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_0363 |

#### Azimuth

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | century_2100_01 |
| extreme_altitude | 10 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_elev_0_45.0 |
| extreme_latitude | 27 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_lat_4_-179.999 |
| extreme_longitude | 12 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | extreme_lon_0_0.0 |
| leap_years | 28 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | leap_2096_00 |
| near_midnight_utc | 36 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | midnight_-60.0_-90.0_2358 |
| polar_day | 40 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | polar_day_-85.0_06 |
| polar_night | 40 | -0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | polar_night_85.0_06 |
| random_stratified | 2200 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_1334 |
| subsolar_near_zenith | 30 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | subsolar_-23.44_1206 |
| sunrise_sunset_sweep | 480 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | sweep_equator_320_0030 |
| __all__ | 2918 | 0.000000 deg | 0.000000 deg | 0.000000 deg | 0.000000 deg | random_1334 |

#### Earth radius vector

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | 0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | century_1900_01 |
| extreme_altitude | 10 | 0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | extreme_elev_0_-45.0 |
| extreme_latitude | 27 | 0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | extreme_lat_0_-179.999 |
| extreme_longitude | 12 | 0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | extreme_lon_0_-45.0 |
| leap_years | 28 | 0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | leap_1904_00 |
| near_midnight_utc | 36 | 0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | midnight_-60.0_-90.0_00 |
| polar_day | 40 | 0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | polar_day_70.0_00 |
| polar_night | 40 | 0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | polar_night_70.0_00 |
| random_stratified | 2200 | 0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | random_0000 |
| subsolar_near_zenith | 30 | 0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | subsolar_0.0_1200 |
| sunrise_sunset_sweep | 480 | 0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | sweep_equator_320_0000 |
| __all__ | 2918 | 0.000000 AU | 0.000000 AU | 0.000000 AU | 0.000000 AU | century_1900_01 |

### Meeus (1998, reduced order) — détail par catégorie (2918 cas)


#### Right ascension (geocentric)

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | 0.001428 deg | 0.002773 deg | 0.003271 deg | 0.006514 deg | century_2100_0701 |
| extreme_altitude | 10 | 0.004639 deg | 0.004639 deg | 0.004639 deg | 0.004639 deg | extreme_elev_0_-45.0 |
| extreme_latitude | 27 | 0.002095 deg | 0.002095 deg | 0.002095 deg | 0.002095 deg | extreme_lat_0_-179.999 |
| extreme_longitude | 12 | 0.002207 deg | 0.002207 deg | 0.002207 deg | 0.002207 deg | extreme_lon_0_-45.0 |
| leap_years | 28 | 0.001496 deg | 0.002542 deg | 0.003070 deg | 0.004885 deg | leap_2024_18 |
| near_midnight_utc | 36 | 0.000564 deg | 0.000564 deg | 0.000566 deg | 0.000613 deg | midnight_-60.0_-90.0_2359 |
| polar_day | 40 | 0.003801 deg | 0.003801 deg | 0.004185 deg | 0.005601 deg | polar_day_-70.0_18 |
| polar_night | 40 | 0.003801 deg | 0.003801 deg | 0.004185 deg | 0.005601 deg | polar_night_70.0_18 |
| random_stratified | 2200 | 0.002055 deg | 0.002771 deg | 0.003389 deg | 0.009501 deg | random_1444 |
| subsolar_near_zenith | 30 | 0.003295 deg | 0.003295 deg | 0.003668 deg | 0.005575 deg | subsolar_-23.44_1254 |
| sunrise_sunset_sweep | 480 | 0.002150 deg | 0.002150 deg | 0.002152 deg | 0.002309 deg | sweep_equator_320_2330 |
| __all__ | 2918 | 0.002114 deg | 0.002671 deg | 0.003213 deg | 0.009501 deg | random_1444 |

#### Declination (geocentric)

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | 0.000213 deg | 0.000524 deg | 0.000706 deg | 0.001776 deg | century_2000_29 |
| extreme_altitude | 10 | -0.002152 deg | 0.002152 deg | 0.002152 deg | 0.002152 deg | extreme_elev_0_-45.0 |
| extreme_latitude | 27 | 0.000175 deg | 0.000175 deg | 0.000175 deg | 0.000175 deg | extreme_lat_0_-179.999 |
| extreme_longitude | 12 | 0.000824 deg | 0.000824 deg | 0.000824 deg | 0.000824 deg | extreme_lon_0_-45.0 |
| leap_years | 28 | 0.000630 deg | 0.001011 deg | 0.001238 deg | 0.002085 deg | leap_2024_18 |
| near_midnight_utc | 36 | 0.000037 deg | 0.000037 deg | 0.000041 deg | 0.000054 deg | midnight_-60.0_-90.0_2359 |
| polar_day | 40 | 0.000061 deg | 0.000114 deg | 0.000130 deg | 0.000176 deg | polar_day_70.0_06 |
| polar_night | 40 | 0.000061 deg | 0.000114 deg | 0.000130 deg | 0.000176 deg | polar_night_-70.0_06 |
| random_stratified | 2200 | 0.000025 deg | 0.000753 deg | 0.001003 deg | 0.003544 deg | random_2180 |
| subsolar_near_zenith | 30 | 0.000319 deg | 0.000348 deg | 0.000488 deg | 0.000828 deg | subsolar_0.0_1254 |
| sunrise_sunset_sweep | 480 | 0.000500 deg | 0.000500 deg | 0.000596 deg | 0.000875 deg | sweep_equator_320_2330 |
| __all__ | 2918 | 0.000111 deg | 0.000682 deg | 0.000926 deg | 0.003544 deg | random_2180 |

#### Equation of time

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | -0.010132 min | 0.016745 min | 0.019543 min | 0.041778 min | century_2000_01 |
| extreme_altitude | 10 | -0.044528 min | 0.044528 min | 0.044528 min | 0.044528 min | extreme_elev_0_-45.0 |
| extreme_latitude | 27 | -0.006580 min | 0.006580 min | 0.006580 min | 0.006580 min | extreme_lat_0_-179.999 |
| extreme_longitude | 12 | 0.014872 min | 0.014872 min | 0.014872 min | 0.014872 min | extreme_lon_0_-45.0 |
| leap_years | 28 | -0.012231 min | 0.014427 min | 0.016293 min | 0.025201 min | leap_2024_00 |
| near_midnight_utc | 36 | -0.002198 min | 0.002198 min | 0.002198 min | 0.002212 min | midnight_-60.0_-90.0_00 |
| polar_day | 40 | -0.020533 min | 0.020533 min | 0.024914 min | 0.034961 min | polar_day_-70.0_00 |
| polar_night | 40 | -0.020533 min | 0.020533 min | 0.024914 min | 0.034961 min | polar_night_70.0_00 |
| random_stratified | 2200 | -0.006871 min | 0.018114 min | 0.022049 min | 0.059883 min | random_1241 |
| subsolar_near_zenith | 30 | -0.008753 min | 0.018665 min | 0.022034 min | 0.034541 min | subsolar_-23.44_1200 |
| sunrise_sunset_sweep | 480 | 0.004146 min | 0.010716 min | 0.011493 min | 0.014982 min | sweep_equator_320_0000 |
| __all__ | 2918 | -0.005500 min | 0.016700 min | 0.020566 min | 0.059883 min | random_1241 |

#### Hour angle (geocentric)

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | -0.001438 deg | 0.002776 deg | 0.003276 deg | 0.006516 deg | century_2100_0701 |
| extreme_altitude | 10 | -0.004635 deg | 0.004635 deg | 0.004635 deg | 0.004635 deg | extreme_elev_0_-45.0 |
| extreme_latitude | 27 | -0.002078 deg | 0.002078 deg | 0.002078 deg | 0.002078 deg | extreme_lat_0_-179.999 |
| extreme_longitude | 12 | -0.002247 deg | 0.002247 deg | 0.002247 deg | 0.002247 deg | extreme_lon_2_-45.0 |
| leap_years | 28 | -0.001510 deg | 0.002543 deg | 0.003071 deg | 0.004879 deg | leap_2024_18 |
| near_midnight_utc | 36 | -0.000582 deg | 0.000582 deg | 0.000584 deg | 0.000628 deg | midnight_-60.0_-90.0_2359 |
| polar_day | 40 | -0.003793 deg | 0.003793 deg | 0.004182 deg | 0.005600 deg | polar_day_-70.0_18 |
| polar_night | 40 | -0.003793 deg | 0.003793 deg | 0.004182 deg | 0.005600 deg | polar_night_70.0_18 |
| random_stratified | 2200 | -0.002055 deg | 0.002769 deg | 0.003388 deg | 0.009485 deg | random_1444 |
| subsolar_near_zenith | 30 | -0.003303 deg | 0.003303 deg | 0.003673 deg | 0.005575 deg | subsolar_-23.44_1254 |
| sunrise_sunset_sweep | 480 | -0.002161 deg | 0.002161 deg | 0.002165 deg | 0.002345 deg | sweep_equator_320_2330 |
| __all__ | 2918 | -0.002116 deg | 0.002671 deg | 0.003214 deg | 0.009485 deg | random_1444 |

#### Elevation (apparent)

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | 0.002229 deg | 0.002229 deg | 0.002392 deg | 0.003857 deg | century_2000_29 |
| extreme_altitude | 10 | 0.001864 deg | 0.002150 deg | 0.002846 deg | 0.004016 deg | extreme_elev_4_-45.0 |
| extreme_latitude | 27 | 0.001924 deg | 0.001924 deg | 0.002005 deg | 0.002416 deg | extreme_lat_5_179.999 |
| extreme_longitude | 12 | 0.001993 deg | 0.001993 deg | 0.002123 deg | 0.002626 deg | extreme_lon_2_45.0 |
| leap_years | 28 | 0.002563 deg | 0.002628 deg | 0.003146 deg | 0.007319 deg | leap_2024_18 |
| near_midnight_utc | 36 | 0.001996 deg | 0.001996 deg | 0.002067 deg | 0.002649 deg | midnight_0.0_-90.0_2358 |
| polar_day | 40 | 0.002324 deg | 0.002324 deg | 0.002408 deg | 0.004184 deg | polar_day_-70.0_18 |
| polar_night | 40 | 0.002106 deg | 0.002106 deg | 0.002208 deg | 0.004126 deg | polar_night_70.0_18 |
| random_stratified | 2200 | 0.003411 deg | 0.003837 deg | 0.024187 deg | 0.535312 deg | random_1743 |
| subsolar_near_zenith | 30 | 0.002901 deg | 0.003334 deg | 0.003644 deg | 0.005658 deg | subsolar_-23.44_1254 |
| sunrise_sunset_sweep | 480 | 0.004664 deg | 0.004793 deg | 0.040068 deg | 0.623881 deg | sweep_subarctic_621_0030 |
| __all__ | 2918 | 0.003522 deg | 0.003871 deg | 0.026565 deg | 0.623881 deg | sweep_subarctic_621_0030 |

#### Zenith (apparent)

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | -0.002229 deg | 0.002229 deg | 0.002392 deg | 0.003857 deg | century_2000_29 |
| extreme_altitude | 10 | -0.001864 deg | 0.002150 deg | 0.002846 deg | 0.004016 deg | extreme_elev_4_-45.0 |
| extreme_latitude | 27 | -0.001924 deg | 0.001924 deg | 0.002005 deg | 0.002416 deg | extreme_lat_5_179.999 |
| extreme_longitude | 12 | -0.001993 deg | 0.001993 deg | 0.002123 deg | 0.002626 deg | extreme_lon_2_45.0 |
| leap_years | 28 | -0.002563 deg | 0.002628 deg | 0.003146 deg | 0.007319 deg | leap_2024_18 |
| near_midnight_utc | 36 | -0.001996 deg | 0.001996 deg | 0.002067 deg | 0.002649 deg | midnight_0.0_-90.0_2358 |
| polar_day | 40 | -0.002324 deg | 0.002324 deg | 0.002408 deg | 0.004184 deg | polar_day_-70.0_18 |
| polar_night | 40 | -0.002106 deg | 0.002106 deg | 0.002208 deg | 0.004126 deg | polar_night_70.0_18 |
| random_stratified | 2200 | -0.003411 deg | 0.003837 deg | 0.024187 deg | 0.535312 deg | random_1743 |
| subsolar_near_zenith | 30 | -0.002901 deg | 0.003334 deg | 0.003644 deg | 0.005658 deg | subsolar_-23.44_1254 |
| sunrise_sunset_sweep | 480 | -0.004664 deg | 0.004793 deg | 0.040068 deg | 0.623881 deg | sweep_subarctic_621_0030 |
| __all__ | 2918 | -0.003522 deg | 0.003871 deg | 0.026565 deg | 0.623881 deg | sweep_subarctic_621_0030 |

#### Azimuth

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | -0.002347 deg | 0.003820 deg | 0.005264 deg | 0.015917 deg | century_2100_0701 |
| extreme_altitude | 10 | -0.000100 deg | 0.006549 deg | 0.006550 deg | 0.006649 deg | extreme_elev_0_45.0 |
| extreme_latitude | 27 | -0.000060 deg | 0.002722 deg | 0.002981 deg | 0.005195 deg | extreme_lat_3_179.999 |
| extreme_longitude | 12 | 0.006712 deg | 0.008799 deg | 0.011860 deg | 0.020054 deg | extreme_lon_1_0.0 |
| leap_years | 28 | -0.001610 deg | 0.002724 deg | 0.003577 deg | 0.008071 deg | leap_2024_00 |
| near_midnight_utc | 36 | -0.000110 deg | 0.000531 deg | 0.000655 deg | 0.001451 deg | midnight_0.0_0.0_2359 |
| polar_day | 40 | 0.001761 deg | 0.003800 deg | 0.004203 deg | 0.006963 deg | polar_day_-70.0_12 |
| polar_night | 40 | -0.001760 deg | 0.003793 deg | 0.004194 deg | 0.006865 deg | polar_night_70.0_00 |
| random_stratified | 2200 | -0.000124 deg | 0.002772 deg | 0.005721 deg | 0.188552 deg | random_0903 |
| subsolar_near_zenith | 30 | 0.004222 deg | 0.007256 deg | 0.013580 deg | 0.053534 deg | subsolar_0.0_1212 |
| sunrise_sunset_sweep | 480 | -0.000793 deg | 0.002105 deg | 0.002985 deg | 0.025748 deg | sweep_equator_320_0000 |
| __all__ | 2918 | -0.000183 deg | 0.002751 deg | 0.005441 deg | 0.188552 deg | random_0903 |

#### Earth radius vector

| Catégorie | N | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |
|---|---|---|---|---|---|---|
| century_years | 15 | -0.000013 AU | 0.000023 AU | 0.000026 AU | 0.000050 AU | century_1900_1231 |
| extreme_altitude | 10 | 0.000008 AU | 0.000008 AU | 0.000008 AU | 0.000008 AU | extreme_elev_0_-45.0 |
| extreme_latitude | 27 | 0.000039 AU | 0.000039 AU | 0.000039 AU | 0.000039 AU | extreme_lat_0_-179.999 |
| extreme_longitude | 12 | 0.000023 AU | 0.000023 AU | 0.000023 AU | 0.000023 AU | extreme_lon_0_-45.0 |
| leap_years | 28 | 0.000006 AU | 0.000023 AU | 0.000028 AU | 0.000057 AU | leap_1904_18 |
| near_midnight_utc | 36 | 0.000017 AU | 0.000017 AU | 0.000017 AU | 0.000020 AU | midnight_-60.0_-90.0_2359 |
| polar_day | 40 | 0.000024 AU | 0.000024 AU | 0.000028 AU | 0.000039 AU | polar_day_70.0_18 |
| polar_night | 40 | 0.000024 AU | 0.000024 AU | 0.000028 AU | 0.000039 AU | polar_night_-70.0_18 |
| random_stratified | 2200 | 0.000001 AU | 0.000023 AU | 0.000029 AU | 0.000080 AU | random_0374 |
| subsolar_near_zenith | 30 | 0.000023 AU | 0.000023 AU | 0.000026 AU | 0.000039 AU | subsolar_23.44_1254 |
| sunrise_sunset_sweep | 480 | 0.000031 AU | 0.000031 AU | 0.000032 AU | 0.000039 AU | sweep_equator_621_1600 |
| __all__ | 2918 | 0.000007 AU | 0.000025 AU | 0.000029 AU | 0.000080 AU | random_0374 |
