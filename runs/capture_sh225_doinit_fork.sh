#!/usr/bin/env bash
# SH225 — disassemble the GlobalInit do-init DM-construction dispatch fork (the ONE
# reachable DM-touching path SH186 mapped but judged "same-difficulty, never built").
# Fresh resolved disasm: do-init 0x102206c40 acquires the once-guard [0x106a68410]
# (ldar+tbz -> 0x102206d10 -> __call_once 0x10284ce54), then bls closure-build
# 0x102206db8 whose dispatch reads x0=[x19,#4] (binder obj) -> x8=[x0] -> x1=[x8,#0x30]
# -> `br x1`. Expected: once-guard cell + the 9 pinned words decode exactly; branch
# targets resolve to 0x10284ce54 (call_once) / 0x102206db8 / 0x10221942c / 0x102206d10.
# Read-only (load_elf_image + decode); does not drive the harness.
set -euo pipefail
cd "$(dirname "$0")/.."
SO="${1:-/home/hermes-worker/.cache/open-sober/robbox/libroblox.so}"
cargo run --release --example dm_construction_fork -- "$SO"