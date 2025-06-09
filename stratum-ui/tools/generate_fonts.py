#!/usr/bin/env python3

import subprocess
from pathlib import Path
import argparse
import io
import sys

# Ensure UTF-8 stdout
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")

# --- Argument Parsing ---
parser = argparse.ArgumentParser()
parser.add_argument(
    "--no-cache", action="store_true", help="Disable font generation cache"
)
args = parser.parse_args()
USE_CACHE = not args.no_cache

# --- Font Configuration ---
FONTS = [
    ("JetBrainsMonoNL-Regular.ttf", "jetbrains_mono_nl_regular", [12]),
    ("JetBrainsMonoNL-ExtraBold.ttf", "jetbrains_mono_nl_extrabold", [12]),
    ("AtkinsonHyperlegible-Regular.ttf", "atkinson_regular", [12, 16, 20]),
    ("AtkinsonHyperlegible-Bold.ttf", "atkinson_bold", [16, 20]),
    ("Lexend-Regular.ttf", "lexend_regular", [12, 14]),
    ("Lexend-Bold.ttf", "lexend_bold", [20]),
    ("Lexend-Light.ttf", "lexend_light", [20]),
]

TTF_DIR = Path("fonts")
C_OUT_DIR = Path("src/fonts")
H_OUT_DIR = Path("include/fonts")
C_OUT_DIR.mkdir(parents=True, exist_ok=True)
H_OUT_DIR.mkdir(parents=True, exist_ok=True)

HEADER_TEMPLATE = """#pragma once

#include "lvgl.h"

extern const lv_font_t {font_name};
"""


def is_outdated(source: Path, target: Path) -> bool:
    return not target.exists() or source.stat().st_mtime > target.stat().st_mtime


# --- Generate .c and .h for each font/size ---
for ttf_file, prefix, sizes in FONTS:
    ttf_path = TTF_DIR / ttf_file
    for size in sizes:
        base_name = f"{prefix}_{size}"
        c_file = C_OUT_DIR / f"{base_name}.c"
        h_file = H_OUT_DIR / f"{base_name}.h"

        regenerate_c = not USE_CACHE or is_outdated(ttf_path, c_file)
        regenerate_h = not USE_CACHE or is_outdated(c_file, h_file)

        if regenerate_c:
            cmd = [
                "lv_font_conv",
                "--font",
                str(ttf_path),
                "--size",
                str(size),
                "--bpp",
                "4",
                "--format",
                "lvgl",
                "--range",
                "32-127,160-255",
                "--no-compress",
                "--output",
                str(c_file),
            ]
            print(f"🔤 Generating: {c_file.name}")
            subprocess.run(cmd, check=True, shell=True)
        else:
            print(f"✅ Skipping {c_file.name} (cached)")

        if regenerate_h:
            h_file.write_text(HEADER_TEMPLATE.format(font_name=base_name))
            print(f"📝 Created header: {h_file.name}")
        else:
            print(f"✅ Skipping {h_file.name} (cached)")
