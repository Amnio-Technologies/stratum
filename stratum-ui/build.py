#!/usr/bin/env python3
import sys
import argparse
from build_lib import do_build

parser = argparse.ArgumentParser()
parser.add_argument("--dynamic", action="store_true", help="Build a shared library")
parser.add_argument("--nocache", action="store_true", help="Force rebuild")
parser.add_argument(
    "--target", type=str, default="desktop", choices=["desktop", "firmware"]
)
parser.add_argument("--release", action="store_true", help="Use Release build")
parser.add_argument("--output-name", type=str, help="Final library name (no rebuild)")
args = parser.parse_args()

success = do_build(
    dynamic=args.dynamic,
    nocache=args.nocache,
    target=args.target,
    release=args.release,
    output_name=args.output_name or "stratum-ui",
)

sys.exit(0 if success else 1)
