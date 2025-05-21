import os
import argparse
from pathlib import Path

# --- Argument Parsing ---
parser = argparse.ArgumentParser()
parser.add_argument("--no-cache", action="store_true", help="Disable header generation cache")
args = parser.parse_args()
USE_CACHE = not args.no_cache

FONT_DIR = Path("src/fonts")
HEADER_DIR = Path("include/fonts")
HEADER_TEMPLATE = '''#pragma once

#include "lvgl.h"

extern const lv_font_t {font_name};
'''

HEADER_DIR.mkdir(parents=True, exist_ok=True)

for c_file in FONT_DIR.glob("*.c"):
    font_basename = c_file.stem
    header_path = HEADER_DIR / f"{font_basename}.h"

    if USE_CACHE and header_path.exists() and header_path.stat().st_mtime >= c_file.stat().st_mtime:
        print(f"✅ Skipping {header_path.name} (cached)")
        continue

    with open(header_path, "w") as f:
        f.write(HEADER_TEMPLATE.format(font_name=font_basename))
    print(f"📝 Created header: {header_path.name}")
