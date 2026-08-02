import datetime
import itertools
import json

import numpy as np
import pandas as pd
import pvlib
from pvlib import spa

# ---------------------------------------------------------------------------
# 1) The canonical Reda & Andreas (2004) NREL SPA worked example, as
#    reproduced in pvlib's own test suite (tests/conftest.py `golden_mst`
#    fixture + tests/test_solarposition.py `expected_solpos` fixture).
# ---------------------------------------------------------------------------
canonical = {
    "label": "reda_andreas_2004_golden_co",
    "year": 2003, "month": 10, "day": 17,
    "hour": 19, "minute": 30, "second": 30,  # UTC (12:30:30 MST = UTC-7)
    "lat": 39.742476, "lon": -105.1786, "elev": 1830.14,
    "pressure_pa": 82000.0, "temperature_c": 11.0, "delta_t": 67.0,
    "atmos_refract": 0.5667,
}

# ---------------------------------------------------------------------------
# 2) A broad campaign: 6 dates x 3 UTC times x 5 latitudes/longitudes,
#    generic atmosphere (101325 Pa, 15 degC), delta_t = 69 s (representative
#    of the current era).
# ---------------------------------------------------------------------------
dates = [
    (2024, 3, 20), (2024, 6, 21), (2024, 9, 22), (2024, 12, 21),
    (2024, 1, 15), (2024, 7, 15),
]
times = [(8, 0, 0), (12, 0, 0), (16, 0, 0)]
locations = [
    ("paris", 48.8566, 2.3522, 35.0),
    ("reykjavik", 64.1466, -21.9426, 0.0),
    ("singapore", 1.3521, 103.8198, 15.0),
    ("sydney", -33.8688, 151.2093, 50.0),
    ("golden_co", 39.742476, -105.1786, 1830.14),
]

campaign = []
for (y, mo, d), (h, mi, s), (name, lat, lon, elev) in itertools.product(dates, times, locations):
    campaign.append({
        "label": f"{name}_{y:04d}{mo:02d}{d:02d}_{h:02d}{mi:02d}",
        "year": y, "month": mo, "day": d, "hour": h, "minute": mi, "second": s,
        "lat": lat, "lon": lon, "elev": elev,
        "pressure_pa": 101325.0, "temperature_c": 15.0, "delta_t": 69.0,
        "atmos_refract": 0.5667,
    })

all_cases = [canonical] + campaign


def compute_full(case):
    dt = datetime.datetime(case["year"], case["month"], case["day"],
                            case["hour"], case["minute"], case["second"],
                            tzinfo=datetime.timezone.utc)
    unixtime = float(dt.timestamp())
    delta_t = float(case["delta_t"])

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
    _nutation_out = np.zeros(2)
    spa.longitude_obliquity_nutation(jce, x0, x1, x2, x3, x4, _nutation_out)
    delta_psi, delta_epsilon = _nutation_out[0], _nutation_out[1]
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
    H = spa.local_hour_angle(v, case["lon"], alpha)
    xi = spa.equatorial_horizontal_parallax(R)
    u = spa.uterm(case["lat"])
    x = spa.xterm(u, case["lat"], case["elev"])
    y = spa.yterm(u, case["lat"], case["elev"])
    delta_alpha = spa.parallax_sun_right_ascension(x, xi, H, delta)
    delta_prime = spa.topocentric_sun_declination(delta, x, y, xi, delta_alpha, H)
    H_prime = spa.topocentric_local_hour_angle(H, delta_alpha)
    e0 = spa.topocentric_elevation_angle_without_atmosphere(case["lat"], delta_prime, H_prime)
    delta_e = spa.atmospheric_refraction_correction(
        case["pressure_pa"] / 100.0, case["temperature_c"], e0, case["atmos_refract"])
    e = spa.topocentric_elevation_angle(e0, delta_e)
    theta = spa.topocentric_zenith_angle(e)
    theta0 = spa.topocentric_zenith_angle(e0)
    gamma = spa.topocentric_astronomers_azimuth(H_prime, delta_prime, case["lat"])
    phi = spa.topocentric_azimuth_angle(gamma)

    def f(v):
        return float(v)

    return {
        **case,
        "jd": f(jd), "jde": f(jde), "jc": f(jc), "jce": f(jce), "jme": f(jme),
        "R_au": f(R), "L_deg": f(L), "B_deg": f(B),
        "theta_geo_lon_deg": f(Theta), "beta_geo_lat_deg": f(beta),
        "x0": f(x0), "x1": f(x1), "x2": f(x2), "x3": f(x3), "x4": f(x4),
        "delta_psi_deg": f(delta_psi), "delta_epsilon_deg": f(delta_epsilon),
        "epsilon0_arcsec": f(epsilon0), "epsilon_deg": f(epsilon),
        "delta_tau_deg": f(delta_tau), "lambda_deg": f(lamd),
        "v0_deg": f(v0), "v_deg": f(v),
        "alpha_deg": f(alpha), "delta_deg": f(delta),
        "equation_of_time_min": f(eot), "hour_angle_deg": f(H),
        "xi_deg": f(xi),
        "topo_right_ascension_deg": f(alpha + delta_alpha),
        "topo_declination_deg": f(delta_prime),
        "topo_hour_angle_deg": f(H_prime),
        "elevation_true_deg": f(e0),
        "elevation_apparent_deg": f(e),
        "zenith_true_deg": f(theta0),
        "zenith_apparent_deg": f(theta),
        "azimuth_deg": f(phi),
    }


results = [compute_full(c) for c in all_cases]

with open("/tmp/validation_dataset.json", "w") as fh:
    json.dump(results, fh, indent=2)

print(f"Generated {len(results)} cases")
print(f"pvlib version: {pvlib.__version__}")
print(json.dumps(results[0], indent=2))
