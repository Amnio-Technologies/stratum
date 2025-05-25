import subprocess
from pathlib import Path
import argparse

import io
import sys
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

# --- Argument Parsing ---
parser = argparse.ArgumentParser()
parser.add_argument("--no-cache", action="store_true", help="Disable font generation cache")
args = parser.parse_args()
USE_CACHE = not args.no_cache

# Fonts to convert: (filename, prefix, [sizes])
FONTS = [
    ("JetBrainsMonoNL-Regular.ttf", "jetbrains_mono_nl_regular", [12]),
    ("JetBrainsMonoNL-ExtraBold.ttf", "jetbrains_mono_nl_extrabold", [12]),
    ("AtkinsonHyperlegible-Regular.ttf", "atkinson_regular", [12, 16, 20]),
    ("AtkinsonHyperlegible-Bold.ttf", "atkinson_bold", [16, 20]),
    ("Lexend-Regular.ttf", "lexend_regular", [12, 14]),
    ("Lexend-Bold.ttf", "lexend_bold", [20]),
    ("Lexend-Light.ttf", "lexend_light", [20])
]

TTF_DIR = Path("ttf")
OUTPUT_DIR = Path("src/fonts")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

def is_outdated(ttf: Path, output: Path) -> bool:
    if not output.exists():
        return True
    return ttf.stat().st_mtime > output.stat().st_mtime

for ttf_file, prefix, sizes in FONTS:
    ttf_path = TTF_DIR / ttf_file
    for size in sizes:
        out_path = OUTPUT_DIR / f"{prefix}_{size}.c"

        if USE_CACHE and not is_outdated(ttf_path, out_path):
            print(f"✅ Skipping {out_path.name} (cached)")
            continue

        print(out_path)
        cmd = [
            "lv_font_conv",
            "--font", str(ttf_path),
            "--size", str(size),
            "--bpp", "4",
            "--format", "lvgl",
            "--range", "32-127,160-255",
            "--no-compress",
            "--output", str(out_path)
        ]
        print("🔤 Generating:", " ".join(cmd))
        subprocess.run(cmd, check=True, shell=True)
