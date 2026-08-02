"""Regenerate the periodic-term tables in
engine/crates/solar_geometry/src/spa/tables.rs from pvlib-python's
reproduction of the NREL SPA coefficient tables. See
engine/validation-data/README.md for provenance and licensing notes: only
the numeric data (public domain) is used, no pvlib source code or
structure is copied into the generated Rust file.

Usage: python3 generate_spa_tables_rust.py
Fetches pvlib/pvlib-python's spa.py directly from GitHub (no local pvlib
installation required for this script; only `pvlib` Python package is
needed for generate_pvlib_reference_dataset.py, not for this one).
"""
import re
import urllib.request

SPA_PY_URL = (
    "https://raw.githubusercontent.com/pvlib/pvlib-python/main/pvlib/spa.py"
)

with urllib.request.urlopen(SPA_PY_URL) as response:
    text = response.read().decode("utf-8")


def extract_rows(name):
    start = text.index(f"\n{name} = np.array([")
    end = text.index("])", start)
    block = text[start:end]
    rows = re.findall(r'\[([^\[\]]+)\]', block)
    return [[x.strip() for x in row.split(',')] for row in rows]


def fmt_f64(s):
    # ensure it prints as a valid Rust f64 literal
    v = float(s)
    return repr(v)


def emit_lbr_table(rust_name, rows, comment):
    lines = [f"/// {comment}", f"pub(crate) const {rust_name}: [Term; {len(rows)}] = ["]
    for row in rows:
        a, b, c = row
        lines.append(f"    Term {{ a: {fmt_f64(a)}, b: {fmt_f64(b)}, c: {fmt_f64(c)} }},")
    lines.append("];")
    return "\n".join(lines)


def emit_nutation_abcd(rows):
    lines = [
        "/// Nutation periodic term coefficients A, B, C, D (units of 0.0001 arcsecond).",
        "///",
        "/// Source: Reda & Andreas (2004), NREL/TP-560-34302, Table A3.2.",
        f"pub(crate) const NUTATION_ABCD: [NutationCoeff; {len(rows)}] = [",
    ]
    for row in rows:
        a, b, c, d = row
        lines.append(
            f"    NutationCoeff {{ a: {fmt_f64(a)}, b: {fmt_f64(b)}, c: {fmt_f64(c)}, d: {fmt_f64(d)} }},"
        )
    lines.append("];")
    return "\n".join(lines)


def emit_nutation_yterm(rows):
    lines = [
        "/// Nutation periodic term multipliers Y0..Y4 (integer multiples of the five",
        "/// fundamental arguments X0..X4).",
        "///",
        "/// Source: Reda & Andreas (2004), NREL/TP-560-34302, Table A3.2.",
        f"pub(crate) const NUTATION_YTERM: [YTerm; {len(rows)}] = [",
    ]
    for row in rows:
        y0, y1, y2, y3, y4 = (int(float(x)) for x in row)
        lines.append(f"    YTerm {{ y0: {y0}, y1: {y1}, y2: {y2}, y3: {y3}, y4: {y4} }},")
    lines.append("];")
    return "\n".join(lines)


lbr_specs = [
    ("L0_TERMS", "L0", "Heliocentric longitude periodic terms, order 0 (64 terms)."),
    ("L1_TERMS", "L1", "Heliocentric longitude periodic terms, order 1 (34 terms)."),
    ("L2_TERMS", "L2", "Heliocentric longitude periodic terms, order 2 (20 terms)."),
    ("L3_TERMS", "L3", "Heliocentric longitude periodic terms, order 3 (7 terms)."),
    ("L4_TERMS", "L4", "Heliocentric longitude periodic terms, order 4 (3 terms)."),
    ("L5_TERMS", "L5", "Heliocentric longitude periodic terms, order 5 (1 term)."),
    ("B0_TERMS", "B0", "Heliocentric latitude periodic terms, order 0 (5 terms)."),
    ("B1_TERMS", "B1", "Heliocentric latitude periodic terms, order 1 (2 terms)."),
    ("R0_TERMS", "R0", "Heliocentric radius vector periodic terms, order 0 (40 terms)."),
    ("R1_TERMS", "R1", "Heliocentric radius vector periodic terms, order 1 (10 terms)."),
    ("R2_TERMS", "R2", "Heliocentric radius vector periodic terms, order 2 (6 terms)."),
    ("R3_TERMS", "R3", "Heliocentric radius vector periodic terms, order 3 (2 terms)."),
    ("R4_TERMS", "R4", "Heliocentric radius vector periodic terms, order 4 (1 term)."),
]

out = []
out.append("// AUTO-EXTRACTED from the NREL Solar Position Algorithm coefficient tables.")
out.append("// See engine/validation-data/README.md for exact provenance and regeneration steps.")
out.append("")
for rust_name, py_name, comment in lbr_specs:
    rows = extract_rows(py_name)
    out.append(emit_lbr_table(rust_name, rows, comment))
    out.append("")

out.append(emit_nutation_abcd(extract_rows("NUTATION_ABCD_ARRAY")))
out.append("")
out.append(emit_nutation_yterm(extract_rows("NUTATION_YTERM_ARRAY")))
out.append("")

output_path = "../crates/solar_geometry/src/spa/tables_generated_body.rs"
with open(output_path, "w") as f:
    f.write("\n".join(out))

print(f"done, lines: {len(chr(10).join(out).splitlines())}, written to {output_path}")
print(
    "NOTE: tables.rs prepends struct definitions and module documentation "
    "by hand; this script only regenerates the data body. Diff before "
    "replacing the committed file."
)
