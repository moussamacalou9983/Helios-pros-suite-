import json

with open('pvlib_reference_dataset.json') as f:
    cases = json.load(f)

def rust_str(s):
    return '"' + s.replace('"', '\\"') + '"'

lines = []
lines.append("// AUTO-GENERATED from engine/validation-data/pvlib_reference_dataset.json")
lines.append("// Regenerate with engine/validation-data/generate_pvlib_reference_dataset.py")
lines.append("// (see engine/validation-data/README.md). Do not hand-edit.")
lines.append("")
lines.append("use crate::dataset::PvlibReferenceCase;")
lines.append("")
lines.append(f"pub(crate) static PVLIB_REFERENCE_DATASET: [PvlibReferenceCase; {len(cases)}] = [")
for c in cases:
    lines.append("    PvlibReferenceCase {")
    lines.append(f"        label: {rust_str(c['label'])},")
    lines.append(f"        year: {c['year']}, month: {c['month']}, day: {c['day']}, hour: {c['hour']}, minute: {c['minute']}, second: {c['second']}.0,")
    lines.append(f"        lat_deg: {c['lat']!r}, lon_deg: {c['lon']!r}, elev_m: {c['elev']!r},")
    lines.append(f"        pressure_pa: {c['pressure_pa']!r}, temperature_c: {c['temperature_c']!r}, delta_t_s: {c['delta_t']!r}, atmos_refract_deg: {c['atmos_refract']!r},")
    lines.append(f"        expected_right_ascension_deg: {c['alpha_deg']!r},")
    lines.append(f"        expected_declination_deg: {c['delta_deg']!r},")
    lines.append(f"        expected_equation_of_time_min: {c['equation_of_time_min']!r},")
    lines.append(f"        expected_hour_angle_deg: {c['hour_angle_deg']!r},")
    lines.append(f"        expected_elevation_true_deg: {c['elevation_true_deg']!r},")
    lines.append(f"        expected_elevation_apparent_deg: {c['elevation_apparent_deg']!r},")
    lines.append(f"        expected_zenith_true_deg: {c['zenith_true_deg']!r},")
    lines.append(f"        expected_zenith_apparent_deg: {c['zenith_apparent_deg']!r},")
    lines.append(f"        expected_azimuth_deg: {c['azimuth_deg']!r},")
    lines.append(f"        expected_earth_radius_vector_au: {c['R_au']!r},")
    lines.append("    },")
lines.append("];")

with open('../crates/validation/src/dataset/generated_dataset.rs', 'w') as f:
    f.write("\n".join(lines))

print("cases:", len(cases), "lines:", len(lines))
