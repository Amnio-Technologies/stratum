import subprocess
from pathlib import Path

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

OUTPUT_DIR = Path("src/fonts")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

for ttf_file, prefix, sizes in FONTS:
    for size in sizes:
        out_name = f"{prefix}_{size}"
        cmd = [
            "lv_font_conv",
            "--font", f"ttf/{ttf_file}",
            "--size", str(size),
            "--bpp", "4",
            "--format", "lvgl",
            "--range", "32-127,160-255",
            "--no-compress",
            "--output", str(OUTPUT_DIR / f"{out_name}.c")
        ]
        print("Generating:", " ".join(cmd))
        subprocess.run(" ".join(cmd), check=True, shell=True)