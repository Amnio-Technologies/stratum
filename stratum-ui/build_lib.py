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


def run_command(cmd, cwd=None, shell=False):
    """Run a subprocess command and return True on success."""
    return subprocess.run(cmd, cwd=cwd, shell=shell).returncode == 0


def get_lib_extension(is_dynamic):
    """Return file extension based on dynamic vs. static and platform."""
    if is_dynamic:
        if sys.platform == "win32":
            return "dll"
        if sys.platform == "darwin":
            return "dylib"
        return "so"
    return "a"


def run_tool_scripts(no_cache):
    """Run the font and shim generators."""
    for script in (FONT_GEN, SHIM_GEN):
        print(f"📁 Running {script}...")
        cmd = [sys.executable, script] + (["--no-cache"] if no_cache else [])
        if not run_command(cmd, cwd=PROJECT_ROOT):
            print(f"❌ {script} failed.")
            return False
    return True


def prepare_build_dir(build_dir, no_cache, intermediary, ext):
    """Clean and create the build directory, and remove stale libs."""
    if no_cache and build_dir.exists():
        print("🧹 Cleaning build dir", build_dir)
        shutil.rmtree(build_dir)
    build_dir.mkdir(parents=True, exist_ok=True)

    stale = build_dir / f"lib{intermediary}.{ext}"
    if stale.exists():
        stale.unlink()


def need_cmake_config(build_dir, target, is_dynamic, no_cache):
    """Decide if we need to rerun CMake configure."""
    cache = build_dir / "CMakeCache.txt"
    if no_cache or not cache.exists():
        return True
    txt = cache.read_text()
    toggle = "ON" if is_dynamic else "OFF"
    return not (
        f"STRATUM_TARGET:STRING={target}" in txt
        and f"STRATUM_BUILD_DYNAMIC:BOOL={toggle}" in txt
    )


def configure_cmake(build_dir, build_type, target, is_dynamic, intermediary, no_cache):
    """Run (or skip) the CMake configuration step."""
    args = [
        "cmake",
        f"-DCMAKE_BUILD_TYPE={build_type}",
        f"-DSTRATUM_TARGET={target}",
        f"-DSTRATUM_BUILD_DYNAMIC={'ON' if is_dynamic else 'OFF'}",
        f"-DSTRATUM_OUTPUT_NAME={intermediary}",
        "-DCMAKE_C_COMPILER_LAUNCHER=ccache",
        "-DCMAKE_CXX_COMPILER_LAUNCHER=ccache",
        str(PROJECT_ROOT),
    ]
    if target == "desktop":
        args[1:1] = ["-G", GENERATOR]
    else:
        args.insert(1, f"-DCMAKE_TOOLCHAIN_FILE={TOOLCHAIN_FILE}")
        args.insert(2, "-DIDF_TARGET=esp32")

    if need_cmake_config(build_dir, target, is_dynamic, no_cache):
        phase = "CMake config" + ("" if target == "desktop" else " (firmware)")
        print(f"⚙️  Running {phase}...")
        if target == "desktop":
            return run_command(args, cwd=build_dir)
        full = (
            f'export IDF_PATH="{ESP_IDF_PATH}" && source "{EXPORT_SH}" && '
            + " ".join(args)
        )
        return run_command(["bash", "-c", full], cwd=build_dir)
    print("⚙️  Skipping CMake config")
    return True


def build_target(build_dir, target, build_type, is_dynamic):
    """Invoke `cmake --build`."""
    cpu = multiprocessing.cpu_count()
    print(f"🔧 Building ({target}/{build_type}/{'DYN' if is_dynamic else 'STATIC'})...")
    cmd = ["cmake", "--build", ".", "--", f"-j{cpu}"]
    if target == "desktop":
        return run_command(cmd, cwd=build_dir)
    full = f'export IDF_PATH="{ESP_IDF_PATH}" && source "{EXPORT_SH}" && ' + " ".join(
        cmd
    )
    return run_command(["bash", "-c", full], cwd=build_dir)


def rename_output(build_dir, intermediary, final_name, ext, is_dynamic):
    """Rename the intermediary lib to the final name (and clean up)."""
    arb = build_dir / f"lib{intermediary}.{ext}"
    if not arb.exists():
        print("❌ Expected intermediary not found:", arb)
        return False

    final = build_dir / f"lib{final_name}.{ext}"
    if final.exists():
        final.unlink()
    arb.replace(final)

    if is_dynamic and sys.platform == "win32":
        imp = build_dir / f"lib{final_name}.dll.a"
        if imp.exists():
            imp.unlink()
    return True


def do_build(dynamic, nocache, target, release, output_name):
    """Full build orchestration."""
    build_type = "Release" if release else "Debug"
    is_dynamic = dynamic
    final_name = output_name or "stratum-ui"
    intermediary = "stratum-ui-intermediary"
    ext = get_lib_extension(is_dynamic)

    start = time.time()

    if target == "firmware" and (not TOOLCHAIN_FILE.exists() or not EXPORT_SH.exists()):
        print("❌ Missing ESP toolchain/export script.")
        return False

    if not run_tool_scripts(nocache):
        return False

    build_dir = PROJECT_ROOT / "build" / target
    prepare_build_dir(build_dir, nocache, intermediary, ext)

    if not configure_cmake(
        build_dir, build_type, target, is_dynamic, intermediary, nocache
    ):
        return False
    if not build_target(build_dir, target, build_type, is_dynamic):
        return False
    if not rename_output(build_dir, intermediary, final_name, ext, is_dynamic):
        return False

    m, s = divmod(time.time() - start, 60)
    print(f"✅ Built lib{final_name}.{ext} in {int(m)}m {s:.3f}s")
    return True
