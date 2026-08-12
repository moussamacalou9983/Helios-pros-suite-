"""Generate the global validation campaign dataset: several thousand cases
spanning latitudes -90..+90, longitudes -180..+180, altitudes 0..6000 m,
years 1900..2100 (including leap years, non-leap century years, and the
2000 exception to the century rule), near-sunrise/sunset sweeps, polar
day/polar night, and near-zenith (subsolar) cases -- as required by the
project's global validation campaign requirement.

Output: global_campaign_dataset.csv (one row per case), used by
`validation::global_campaign` (parsed at test time via a small hand-rolled
CSV parser -- CSV rather than a generated Rust literal array specifically
so this campaign can scale to thousands of cases without an unwieldy
multi-megabyte generated .rs source file).

Usage: pip install pvlib==0.15.2 && python3 generate_global_campaign.py
"""
import csv
import random

import numpy as np
from pvlib import spa

SEED = 20260802  # fixed for reproducibility
rng = random.Random(SEED)

FIELDNAMES = [
    "category", "label",
    "year", "month", "day", "hour", "minute", "second",
    "lat_deg", "lon_deg", "elev_m",
    "pressure_pa", "temperature_c", "delta_t_s", "atmos_refract_deg",
    "expected_right_ascension_deg", "expected_declination_deg",
    "expected_equation_of_time_min", "expected_hour_angle_deg",
    "expected_elevation_true_deg", "expected_elevation_apparent_deg",
    "expected_zenith_true_deg", "expected_zenith_apparent_deg",
    "expected_azimuth_deg", "expected_earth_radius_vector_au",
]


def standard_atmosphere_pressure_pa(elevation_m):
    # ISO 2533:1975 standard atmosphere barometric formula (troposphere).
    return 101325.0 * (1.0 - 2.25577e-5 * elevation_m) ** 5.25588


def estimate_delta_t_seconds(year, month):
    # Espenak & Meeus (2006) polynomial, same one used by the Rust engine's
    # default estimator, applied here purely to generate a *plausible*
    # delta_t for cases outside its documented 2005-2050 validity (the
    # Rust-side test harness is told this exact value, so both sides of
    # the comparison always agree on delta_t -- the campaign validates the
    # *position algorithm*, not delta_t estimation, which is a separate,
    # already-documented concern).
    y = year + (month - 0.5) / 12.0 - 2000.0
    return 62.92 + 0.32217 * y + 0.005589 * y * y


def compute_case(category, label, year, month, day, hour, minute, second,
                  lat, lon, elev, pressure_pa=None, temperature_c=15.0,
                  delta_t=None, atmos_refract=0.5667):
    import datetime as _dt

    if pressure_pa is None:
        pressure_pa = standard_atmosphere_pressure_pa(elev)
    if delta_t is None:
        delta_t = estimate_delta_t_seconds(year, month)

    dt = _dt.datetime(year, month, day, hour, minute, int(second),
                       tzinfo=_dt.timezone.utc)
    unixtime = float(dt.timestamp())

    jd = spa.julian_day(unixtime)
    jde = spa.julian_ephemeris_day(jd, delta_t)
    jc = spa.julian_century(jd)
    jce = spa.julian_ephemeris_century(jde)
    jme = spa.julian_ephemeris_millennium(jce)
    R = spa.heliocentric_radius_vector(jme)
    L = spa.heliocentric_longitude(jme)
    B = spa.heliocentric_latitude(jme)
    Theta = spa.geocentric_longitude(L)
    beta = spa.geocentric_latitude(B)
    x0 = spa.mean_elongation(jce)
    x1 = spa.mean_anomaly_sun(jce)
    x2 = spa.mean_anomaly_moon(jce)
    x3 = spa.moon_argument_latitude(jce)
    x4 = spa.moon_ascending_longitude(jce)
    nut = np.zeros(2)
    spa.longitude_obliquity_nutation(jce, x0, x1, x2, x3, x4, nut)
    delta_psi, delta_epsilon = nut[0], nut[1]
    epsilon0 = spa.mean_ecliptic_obliquity(jme)
    epsilon = spa.true_ecliptic_obliquity(epsilon0, delta_epsilon)
    delta_tau = spa.aberration_correction(R)
    lamd = spa.apparent_sun_longitude(Theta, delta_psi, delta_tau)
    v0 = spa.mean_sidereal_time(jd, jc)
    v = spa.apparent_sidereal_time(v0, delta_psi, epsilon)
    alpha = spa.geocentric_sun_right_ascension(lamd, epsilon, beta)
    delta = spa.geocentric_sun_declination(lamd, epsilon, beta)
    m = spa.sun_mean_longitude(jme)
    eot = spa.equation_of_time(m, alpha, delta_psi, epsilon)
    H = spa.local_hour_angle(v, lon, alpha)
    xi = spa.equatorial_horizontal_parallax(R)
    u = spa.uterm(lat)
    x = spa.xterm(u, lat, elev)
    y = spa.yterm(u, lat, elev)
    delta_alpha = spa.parallax_sun_right_ascension(x, xi, H, delta)
    delta_prime = spa.topocentric_sun_declination(delta, x, y, xi, delta_alpha, H)
    H_prime = spa.topocentric_local_hour_angle(H, delta_alpha)
    e0 = spa.topocentric_elevation_angle_without_atmosphere(lat, delta_prime, H_prime)
    delta_e = spa.atmospheric_refraction_correction(
        pressure_pa / 100.0, temperature_c, e0, atmos_refract)
    e = spa.topocentric_elevation_angle(e0, delta_e)
    theta = spa.topocentric_zenith_angle(e)
    theta0 = spa.topocentric_zenith_angle(e0)
    gamma = spa.topocentric_astronomers_azimuth(H_prime, delta_prime, lat)
    phi = spa.topocentric_azimuth_angle(gamma)

    return {
        "category": category, "label": label,
        "year": year, "month": month, "day": day,
        "hour": hour, "minute": minute, "second": float(second),
        "lat_deg": lat, "lon_deg": lon, "elev_m": elev,
        "pressure_pa": pressure_pa, "temperature_c": temperature_c,
        "delta_t_s": delta_t, "atmos_refract_deg": atmos_refract,
        "expected_right_ascension_deg": float(alpha),
        "expected_declination_deg": float(delta),
        "expected_equation_of_time_min": float(eot),
        "expected_hour_angle_deg": float(H),
        "expected_elevation_true_deg": float(e0),
        "expected_elevation_apparent_deg": float(e),
        "expected_zenith_true_deg": float(theta0),
        "expected_zenith_apparent_deg": float(theta),
        "expected_azimuth_deg": float(phi),
        "expected_earth_radius_vector_au": float(R),
    }


def days_in_month(year, month):
    is_leap = (year % 4 == 0 and year % 100 != 0) or year % 400 == 0
    lengths = [31, 29 if is_leap else 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    return lengths[month - 1]


cases = []

# ---------------------------------------------------------------------
# 1. Random stratified sample across the full specified envelope.
# ---------------------------------------------------------------------
for i in range(2200):
    year = rng.randint(1900, 2100)
    month = rng.randint(1, 12)
    day = rng.randint(1, days_in_month(year, month))
    lat = rng.uniform(-90.0, 90.0)
    lon = rng.uniform(-180.0, 180.0)
    elev = rng.uniform(0.0, 6000.0)
    hour = rng.randint(0, 23)
    minute = rng.randint(0, 59)
    cases.append(compute_case(
        "random_stratified", f"random_{i:04d}",
        year, month, day, hour, minute, 0,
        lat, lon, elev))

# ---------------------------------------------------------------------
# 2. Extreme latitude (including the poles themselves).
# ---------------------------------------------------------------------
for i, lat in enumerate([-90.0, -89.999, -89.9, -45.0, 0.0, 45.0, 89.9, 89.999, 90.0]):
    for lon in [-179.999, 0.0, 179.999]:
        cases.append(compute_case(
            "extreme_latitude", f"extreme_lat_{i}_{lon}",
            2024, 6, 21, 12, 0, 0, lat, lon, 100.0))

# ---------------------------------------------------------------------
# 3. Extreme longitude.
# ---------------------------------------------------------------------
for i, lon in enumerate([-180.0, -179.999, 179.999, 180.0]):
    for lat in [-45.0, 0.0, 45.0]:
        cases.append(compute_case(
            "extreme_longitude", f"extreme_lon_{i}_{lat}",
            2024, 3, 20, 12, 0, 0, lat, lon, 100.0))

# ---------------------------------------------------------------------
# 4. Extreme altitude (sea level to above the highest inhabited/PV sites).
# ---------------------------------------------------------------------
for i, elev in enumerate([0.0, 1500.0, 3000.0, 4500.0, 6000.0]):
    for lat in [-45.0, 45.0]:
        cases.append(compute_case(
            "extreme_altitude", f"extreme_elev_{i}_{lat}",
            2024, 9, 22, 12, 0, 0, lat, 0.0, elev))

# ---------------------------------------------------------------------
# 5. Century years (Gregorian leap-year exception: 1900 and 2100 are NOT
#    leap years despite being divisible by 4; 2000 IS a leap year because
#    it is also divisible by 400).
# ---------------------------------------------------------------------
for year in [1900, 2000, 2100]:
    max_feb_day = days_in_month(year, 2)
    for day in [1, max_feb_day]:
        cases.append(compute_case(
            "century_years", f"century_{year}_{day:02d}",
            year, 2, day, 12, 0, 0, 45.0, 5.0, 200.0))
    for month, day in [(1, 1), (7, 1), (12, 31)]:
        cases.append(compute_case(
            "century_years", f"century_{year}_{month:02d}{day:02d}",
            year, month, day, 12, 0, 0, 45.0, 5.0, 200.0))

# ---------------------------------------------------------------------
# 6. Leap years (February 29th specifically).
# ---------------------------------------------------------------------
for year in [1904, 1908, 1996, 2000, 2024, 2028, 2096]:
    for hour in [0, 6, 12, 18]:
        cases.append(compute_case(
            "leap_years", f"leap_{year}_{hour:02d}",
            year, 2, 29, hour, 30, 0, 45.0, 5.0, 200.0))

# ---------------------------------------------------------------------
# 7. Near-midnight UTC (hour rollover boundary).
# ---------------------------------------------------------------------
for lat in [-60.0, 0.0, 60.0]:
    for lon in [-90.0, 0.0, 90.0]:
        for (hour, minute) in [(0, 0), (0, 1), (23, 58), (23, 59)]:
            cases.append(compute_case(
                "near_midnight_utc", f"midnight_{lat}_{lon}_{hour}{minute}",
                2024, 6, 15, hour, minute, 0, lat, lon, 100.0))

# ---------------------------------------------------------------------
# 8 & 9. Polar day and polar night at high latitudes.
# ---------------------------------------------------------------------
for lat in [70.0, 75.0, 80.0, 85.0, 89.0]:
    for sign, summer_month, winter_month in [(1, 6, 12), (-1, 12, 6)]:
        signed_lat = sign * lat
        for hour in [0, 6, 12, 18]:
            cases.append(compute_case(
                "polar_day", f"polar_day_{signed_lat}_{hour:02d}",
                2024, summer_month, 21, hour, 0, 0, signed_lat, 10.0, 50.0))
            cases.append(compute_case(
                "polar_night", f"polar_night_{signed_lat}_{hour:02d}",
                2024, winter_month, 21, hour, 0, 0, signed_lat, 10.0, 50.0))

# ---------------------------------------------------------------------
# 10. Sunrise/sunset sweeps: fine time resolution across a full day at a
#     handful of representative sites and dates, guaranteeing the dataset
#     contains cases very close to the true elevation = 0 crossing.
# ---------------------------------------------------------------------
sweep_sites = [
    ("equator", 0.0, 0.0, 10.0),
    ("midlat_north", 48.85, 2.35, 35.0),
    ("midlat_south", -33.87, 151.21, 50.0),
    ("subarctic", 63.0, 20.0, 100.0),
    ("tropic", 23.44, 90.0, 20.0),
]
for name, lat, lon, elev in sweep_sites:
    for month, day in [(3, 20), (6, 21)]:
        for step in range(48):  # every 30 minutes across the day
            hour = step // 2
            minute = 30 * (step % 2)
            cases.append(compute_case(
                "sunrise_sunset_sweep", f"sweep_{name}_{month}{day}_{hour:02d}{minute:02d}",
                2024, month, day, hour, minute, 0, lat, lon, elev))

# ---------------------------------------------------------------------
# 11. Sun near zenith (subsolar point): latitude close to the declination
#     at the given date, evaluated at local apparent solar noon.
# ---------------------------------------------------------------------
subsolar_cases = [
    (0.0, 0.0, 3, 20, 12),     # equator at equinox
    (23.44, 0.0, 6, 21, 12),   # Tropic of Cancer at June solstice
    (-23.44, 0.0, 12, 21, 12),  # Tropic of Capricorn at December solstice
]
for lat, lon, month, day, hour in subsolar_cases:
    for minute in range(0, 60, 6):
        cases.append(compute_case(
            "subsolar_near_zenith", f"subsolar_{lat}_{hour:02d}{minute:02d}",
            2024, month, day, hour, minute, 0, lat, lon, 500.0))

print(f"Total cases: {len(cases)}")

with open("global_campaign_dataset.csv", "w", newline="") as f:
    writer = csv.DictWriter(f, fieldnames=FIELDNAMES)
    writer.writeheader()
    for c in cases:
        writer.writerow(c)

print("wrote global_campaign_dataset.csv")

from collections import Counter
print(Counter(c["category"] for c in cases))
