"""Write Volume suggestions from `render --calibrate` into presets.rs.

usage: cargo run -p oxide-dsp --release --example render -- --calibrate | python3 tools/apply_calibration.py
"""
import re, sys

path = "crates/core/src/presets.rs"
src = open(path).read()
for line in sys.stdin:
    if "|" not in line:
        continue
    name, vol = line.rstrip("\n").rsplit("|", 1)
    pat = re.compile(r'(preset!\("' + re.escape(name) + r'", "[^"]+")(.*?)\),\n')
    m = pat.search(src)
    if not m:
        sys.exit(f"preset not found: {name}")
    body = re.sub(r", Volume=-?[\d.]+", "", m.group(2))
    src = src[: m.start()] + f"{m.group(1)}{body}, Volume={vol}),\n" + src[m.end():]
open(path, "w").write(src)
