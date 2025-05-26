#!/usr/bin/env python3

import shutil
import subprocess
import sys
import time
from pathlib import Path
import multiprocessing
import argparse
import io

# Ensure proper UTF-8 stdout
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")

# -------- Configuration --------
PROJECT_ROOT = Path(__file__).parent.resolve()
FONT_GEN = "tools/generate_fonts.py"
GENERATOR = "Ninja" if shutil.which("ninja") else "MinGW Makefiles"

ESP_IDF_PATH = Path.home() / "esp" / "esp-idf"
TOOLCHAIN_FILE = ESP_IDF_PATH / "tools" / "cmake" / "toolchain-esp32.cmake"
EXPORT_SH = Path.home() / "export-esp.sh"

# -------- Argument Parsing --------
parser = argparse.ArgumentParser()
parser.add_argument("--dynamic", action="store_true", help="Build a shared library")
parser.add_argument("--nocache", action="store_true", help="Force rebuild")
parser.add_argument(
    "--target", type=str, default="desktop", choices=["desktop", "firmware"]
)
parser.add_argument("--release", action="store_true", help="Use Release build")
parser.add_argument("--output-name", type=str, help="Final library name (no rebuild)")
args = parser.parse_args()

target = args.target
build_type = "Release" if args.release else "Debug"
is_dynamic = args.dynamic
no_cache = args.nocache
final_name = args.output_name or "stratum-ui"

# intermediaries:
intermediary = "stratum-ui-intermediary"
# extension logic:
if is_dynamic:
    ext = (
        "dll"
        if sys.platform == "win32"
        else "dylib"
        if sys.platform == "darwin"
        else "so"
    )
else:
    ext = "a"

build_start = time.time()

# -------- Font generation + prep --------
if target == "firmware":
    if not TOOLCHAIN_FILE.exists() or not EXPORT_SH.exists():
        print("❌ Missing ESP toolchain/export script.")
        sys.exit(1)

scripts = [FONT_GEN]
for script in scripts:
    print(f"📁 Running {script}...")
    cmd = ["python3", script]
    if no_cache:
        cmd.append("--no-cache")
    if subprocess.run(cmd, cwd=PROJECT_ROOT).returncode != 0:
        print(f"❌ {script} failed.")
        sys.exit(1)

build_dir = PROJECT_ROOT / "build" / target
if no_cache and build_dir.exists():
    print("🧹 Cleaning build dir", build_dir)
    shutil.rmtree(build_dir)
build_dir.mkdir(parents=True, exist_ok=True)

# remove any stale intermediary
stale = build_dir / f"lib{intermediary}.{ext}"
if stale.exists():
    stale.unlink()

# -------- CMake configure --------
cmake_cmd = [
    "cmake",
    f"-DCMAKE_BUILD_TYPE={build_type}",
    f"-DSTRATUM_TARGET={target}",
    f"-DSTRATUM_BUILD_DYNAMIC={'ON' if is_dynamic else 'OFF'}",
    f"-DSTRATUM_OUTPUT_NAME={intermediary}",
    str(PROJECT_ROOT),
]

# decide rerun CMake or not (only on target/dynamic change)
cmake_cache = build_dir / "CMakeCache.txt"
need_cfg = no_cache or not cmake_cache.exists()
if cmake_cache.exists():
    txt = cmake_cache.read_text()
    if ("STRATUM_TARGET:STRING=" + target) not in txt or (
        "STRATUM_BUILD_DYNAMIC:BOOL=ON"
        if is_dynamic
        else "STRATUM_BUILD_DYNAMIC:BOOL=OFF"
    ) not in txt:
        need_cfg = True

if target == "desktop":
    cmake_cmd[1:1] = ["-G", GENERATOR]
    if need_cfg:
        print("⚙️  Running CMake config...")
        subprocess.run(cmake_cmd, cwd=build_dir, check=True)
    else:
        print("⚙️  Skipping CMake config")
else:
    cmake_cmd.insert(1, f"-DCMAKE_TOOLCHAIN_FILE={TOOLCHAIN_FILE}")
    cmake_cmd.insert(2, f"-DIDF_TARGET=esp32")
    full = f'export IDF_PATH="{ESP_IDF_PATH}" && source "{EXPORT_SH}" && ' + " ".join(
        cmake_cmd
    )
    if need_cfg:
        print("⚙️  Running CMake config (firmware)...")
        subprocess.run(["bash", "-c", full], cwd=build_dir, check=True)
    else:
        print("⚙️  Skipping CMake config")

# -------- Build --------
print(f"🔧 Building ({target}/{build_type}/{'DYN' if is_dynamic else 'STA'})...")
cpu = multiprocessing.cpu_count()
build_cmd = ["cmake", "--build", ".", "--", f"-j{cpu}"]
if target == "desktop":
    subprocess.run(build_cmd, cwd=build_dir, check=True)
else:
    full = f'export IDF_PATH="{ESP_IDF_PATH}" && source "{EXPORT_SH}" && ' + " ".join(
        build_cmd
    )
    subprocess.run(["bash", "-c", full], cwd=build_dir, check=True)

# -------- Rename intermediary -> final --------
arb = build_dir / f"lib{intermediary}.{ext}"
final = build_dir / f"lib{final_name}.{ext}"
if not arb.exists():
    print("❌ Expected intermediary not found:", arb)
    sys.exit(1)

# remove any old final
if final.exists():
    final.unlink()
# atomic move
arb.replace(final)

# cleanup import lib on Windows
if is_dynamic and sys.platform == "win32":
    imp = build_dir / f"lib{final_name}.dll.a"
    if imp.exists():
        imp.unlink()

end = time.time() - build_start
m, s = divmod(int(end), 60)
print(f"✅ Built {final.name} in {m}m {s}s")
