#!/usr/bin/env python3
"""Regenerates engine/src/services/vsop87_earth.rs from the `astro` crate's source.

The engine's ephemeris (engine/src/services/ephemeris.rs) uses only the Earth's
VSOP87 heliocentric longitude series. This script extracts those six series from
astro's src/planet/VSOPD_87/earth.rs (the module is private in astro, so its
source is the only way to get at the data) and writes them as static tables.

VSOP87 is a fixed published theory, so the output should never change. The script
exists so the table is reproducible, and CI runs it with --check to prove the
committed file still matches what astro ships.

    python3 scripts/gen-vsop87-earth.py           # rewrite the file
    python3 scripts/gen-vsop87-earth.py --check   # exit 1 if it would change

Needs cargo (to locate astro, the version pinned in engine/Cargo.lock as a
dev-dependency) and rustfmt.
"""

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ENGINE = ROOT / "engine"
OUT = ENGINE / "src" / "services" / "vsop87_earth.rs"

HEADER = """\
// GENERATED from the `astro` {version} crate (MIT licence), src/planet/VSOPD_87/earth.rs.
// Only the heliocentric longitude (L) series of the Earth are kept: they are all
// the engine needs to compute the Sun's geocentric longitude. The full crate also
// links B and R for the Earth and every series for seven other planets, built as
// heap-allocated Vecs on each call, which dominated binary and WASM size.
//
// Each row is [A, B, C] for the term A * cos(B + C * t), t in Julian millennia from J2000.
// Do not edit by hand.

// The literals are the published VSOP87 values (some round to π); they must stay
// exactly as printed for bit-identical results.
#![allow(clippy::approx_constant, clippy::excessive_precision)]
"""


def astro_source() -> tuple[str, Path]:
    """Version and earth.rs path of the astro crate cargo resolved for the engine."""
    meta = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--locked"],
            cwd=ENGINE, check=True, capture_output=True, text=True,
        ).stdout
    )
    for pkg in meta["packages"]:
        if pkg["name"] == "astro":
            crate_dir = Path(pkg["manifest_path"]).parent
            return pkg["version"], crate_dir / "src" / "planet" / "VSOPD_87" / "earth.rs"
    sys.exit("astro not found in cargo metadata; is it still a dev-dependency of the engine?")


def longitude_series(source: str) -> list[list[str]]:
    """The six L series: astro nests vec![ L[T^0..T^5], B[...], R[...] ] of [A, B, C] rows."""
    body = source[source.index("vec!["):]
    token = re.compile(r"vec!\[|\[([^\[\]]+)\]|\]|,|\s+")
    path: list[int] = []      # index of each open vec! within its parent
    counters: list[int] = []  # children seen so far in each open vec!
    series: list[list[str]] = [[] for _ in range(6)]
    pos = 0
    while pos < len(body):
        m = token.match(body, pos)
        if not m:
            sys.exit(f"cannot parse earth.rs near: {body[pos:pos + 40]!r}")
        tok = m.group(0)
        if tok == "vec![":
            if counters:
                counters[-1] += 1
            path.append(counters[-1] - 1 if counters else 0)
            counters.append(0)
        elif m.group(1) is not None:
            coordinate, power = path[1], path[2]  # path[0] is the outermost vec!
            if coordinate == 0:  # 0 = L, 1 = B, 2 = R
                series[power].append(m.group(1).strip())
        elif tok == "]":
            counters.pop()
            path.pop()
            if not counters:
                break
        pos = m.end()
    if any(not s for s in series):
        sys.exit("expected six non-empty L series in earth.rs; did astro change its layout?")
    return series


def render(version: str, series: list[list[str]]) -> str:
    out = [HEADER.format(version=version)]
    for n, rows in enumerate(series):
        out.append(f"pub(crate) static EARTH_L{n}: [[f64; 3]; {len(rows)}] = [")
        out.extend(f"    [{row}]," for row in rows)
        out.append("];\n")
    names = ", ".join(f"&EARTH_L{n}" for n in range(len(series)))
    out.append(f"pub(crate) static EARTH_L: [&[[f64; 3]]; {len(series)}] = [{names}];")
    code = "\n".join(out) + "\n"
    return subprocess.run(
        ["rustfmt", "--edition", "2021", "--emit", "stdout"],
        input=code, check=True, capture_output=True, text=True,
    ).stdout


def main() -> None:
    check = "--check" in sys.argv[1:]
    version, earth_rs = astro_source()
    generated = render(version, longitude_series(earth_rs.read_text()))
    current = OUT.read_text() if OUT.exists() else ""
    if check:
        if generated != current:
            sys.exit(f"{OUT.relative_to(ROOT)} is out of date: run scripts/gen-vsop87-earth.py")
        print(f"{OUT.relative_to(ROOT)} matches astro {version}")
        return
    OUT.write_text(generated)
    print(f"wrote {OUT.relative_to(ROOT)} from astro {version}")


if __name__ == "__main__":
    main()
