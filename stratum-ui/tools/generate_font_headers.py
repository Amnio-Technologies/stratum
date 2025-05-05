import os

FONT_DIR = "src/fonts"
HEADER_DIR = "include/fonts"
HEADER_TEMPLATE = '''#pragma once

#include "lvgl.h"

extern const lv_font_t {font_name};
'''

os.makedirs(HEADER_DIR, exist_ok=True)

for filename in os.listdir(FONT_DIR):
    if filename.endswith(".c"):
        font_basename = os.path.splitext(filename)[0]
        header_path = os.path.join(HEADER_DIR, f"{font_basename}.h")

        with open(header_path, "w") as f:
            f.write(HEADER_TEMPLATE.format(font_name=font_basename))

        print(f"✅ Created header: {header_path}")
