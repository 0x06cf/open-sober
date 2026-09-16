#!/usr/bin/env bash
# SH224 — re-derive the genuine RBX::DataModel vtable rows at the SH187-corrected
# vptr base (0x1067162e8/0x1067163a0/0x1067163f8) and reconcile the SH186c +8-slip
# "null-stub row" verdict. Prints all 6 rows + dispatch-target disassembly.
# Expected: corrected primary slot-2 = 0x1057d19bc (REAL method, not null-stub);
#           0x1057d6ef4 = corrected primary vt+0x38 (not vt+0x30); no GuiObject
#           producer slot in any row (Route-B no-GuiObject gate unchanged).
set -euo pipefail
cd "$(dirname "$0")/.."
SO="${1:-/home/hermes-worker/.cache/open-sober/robbox/libroblox.so}"
cargo run --release --example dm_vtable_corrected -- "$SO"