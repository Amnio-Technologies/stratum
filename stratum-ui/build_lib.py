#!/usr/bin/env python3
import shutil
import subprocess
import sys
import time
import io
from pathlib import Path
import multiprocessing

# Ensure proper UTF-8 stdout for builds
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", line_buffering=True)

# -------- Configuration --------
PROJECT_ROOT = Path(__file__).parent.resolve()
FONT_GEN = "tools/generate_fonts.py"
SHIM_GEN = "tools/generate_shims.py"
GENERATOR = "Ninja" if shutil.which("ninja") else "MinGW Makefiles"

ESP_IDF_PATH = Path.home() / "esp" / "esp-idf"
TOOLCHAIN_FILE = ESP_IDF_PATH / "tools" / "cmake" / "toolchain-esp32.cmake"
EXPORT_SH = Path.home() / "export-esp.sh"


def do_build(
    dynamic: bool, nocache: bool, target: str, release: bool, output_name: str
) -> bool:
    """
    Runs the full build process in-process.
    Returns True on success, False on failure.
    """
    build_type = "Release" if release else "Debug"
    is_dynamic = dynamic
    no_cache = nocache
    final_name = output_name or "stratum-ui"

    # intermediaries:
    intermediary = "stratum-ui-intermediary"
    # extension logic:
    if is_dynamic:
        if sys.platform == "win32":
            ext = "dll"
        elif sys.platform == "darwin":
            ext = "dylib"
        else:
            ext = "so"
    else:
        ext = "a"

    build_start = time.time()

    # -------- Font generation + prep --------
    if target == "firmware":
        if not TOOLCHAIN_FILE.exists() or not EXPORT_SH.exists():
            print("❌ Missing ESP toolchain/export script.")
            return False

    for script in [FONT_GEN, SHIM_GEN]:
        print(f"📁 Running {script}...")
        cmd = [sys.executable, script]
        if no_cache:
            cmd.append("--no-cache")
        if subprocess.run(cmd, cwd=PROJECT_ROOT).returncode != 0:
            print(f"❌ {script} failed.")
            return False

    build_dir = PROJECT_ROOT / "build" / target
    if no_cache and build_dir.exists():
        print("🧹 Cleaning build dir", build_dir)
        shutil.rmtree(build_dir)
    build_dir.mkdir(parents=True, exist_ok=True)

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
        "-DCMAKE_C_COMPILER_LAUNCHER=ccache",
        "-DCMAKE_CXX_COMPILER_LAUNCHER=ccache",
        str(PROJECT_ROOT),
    ]

    cmake_cache = build_dir / "CMakeCache.txt"
    need_cfg = no_cache or not cmake_cache.exists()
    if cmake_cache.exists():
        txt = cmake_cache.read_text()
        toggle = "ON" if is_dynamic else "OFF"
        if (
            f"STRATUM_TARGET:STRING={target}" not in txt
            or f"STRATUM_BUILD_DYNAMIC:BOOL={toggle}" not in txt
        ):
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
        cmake_cmd.insert(2, "-DIDF_TARGET=esp32")
        full = (
            f'export IDF_PATH="{ESP_IDF_PATH}" && source "{EXPORT_SH}" && '
            + " ".join(cmake_cmd)
        )
        if need_cfg:
            print("⚙️  Running CMake config (firmware)...")
            subprocess.run(["bash", "-c", full], cwd=build_dir, check=True)
        else:
            print("⚙️  Skipping CMake config")

    # -------- Build --------
    print(f"🔧 Building ({target}/{build_type}/{'DYN' if is_dynamic else 'STATIC'})...")
    cpu = multiprocessing.cpu_count()
    build_cmd = ["cmake", "--build", ".", "--", f"-j{cpu}"]
    if target == "desktop":
        subprocess.run(build_cmd, cwd=build_dir, check=True)
    else:
        full = (
            f'export IDF_PATH="{ESP_IDF_PATH}" && source "{EXPORT_SH}" && '
            + " ".join(build_cmd)
        )
        subprocess.run(["bash", "-c", full], cwd=build_dir, check=True)

    # -------- Rename intermediary -> final --------
    arb = build_dir / f"lib{intermediary}.{ext}"
    final = build_dir / f"lib{final_name}.{ext}"
    if not arb.exists():
        print("❌ Expected intermediary not found:", arb)
        return False

    if final.exists():
        final.unlink()
    arb.replace(final)

    if is_dynamic and sys.platform == "win32":
        imp = build_dir / f"lib{final_name}.dll.a"
        if imp.exists():
            imp.unlink()

    elapsed = time.time() - build_start
    m, s = divmod(elapsed, 60)
    print(f"✅ Built {final.name} in {int(m)}m {s:.3f}s")
    return True
