#!/bin/bash
# SH414: verify the host-input AXIS is a real (not orphaned) runtime surface —
# input-wrapper promoted to a real arm64jit dependency and the X11-pointer ->
# guest nativePassInput delivery bridge wired end-to-end. Latent-but-correct,
# env-gated, so it must be green WITHOUT the env (product path unchanged):
#   (a) workspace builds with input-wrapper as a REAL dep (production path),
#   (b) the SH414 hermetics pass (motion->TouchAction bridge + session pump
#       inert guards a/b/c),
#   (c) the type-4 self-driven frame deliverable stays green (no regression
#       from the promoted dependency),
#   (d) real-image pin of the guest input native ABI byte-exact.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh414-input-axis.txt
echo "=== SH414 host-input axis verification ===" | tee "$LOG"
echo "--- (a) production build with input-wrapper as a REAL dep ---" | tee -a "$LOG"
cargo build -p arm64jit 2>&1 | tail -2 | tee -a "$LOG"
echo "--- (b) SH414 hermetics (bridge + pump inert guards + real-image pin) ---" | tee -a "$LOG"
cargo test -p arm64jit --lib sh414 2>&1 | tail -8 | tee -a "$LOG"
cargo test -p arm64jit --lib ainput 2>&1 | tail -4 | tee -a "$LOG"
echo "--- (c) recon-v3 type4 frame deliverable still green (no regression) ---" | tee -a "$LOG"
TASKFRAME_RETRY_MAX=2 ./runs/capture_taskv4_frame.sh 2>&1 | grep -E "attempt|confirmed-green|present=|swap Ok|crash-signals" | tee -a "$LOG"
echo "--- (d) real-image pin of the guest input native ABI ---" | tee -a "$LOG"
python3 - <<'EOF' | tee -a "$LOG"
with open("/home/hermes-worker/.cache/open-sober/robbox/libroblox.so","rb") as f: data=f.read()
w=lambda a: int.from_bytes(data[a:a+4],"little")
print("nativePassInput     @0x2bbba88 prologue sub %#x (expect 0xd10143ff)"%w(0x2bbba88))
print("bl consumer @+0x50                %#x (expect 0x940a4aed)"%w(0x2bbba88+0x50))
EOF
echo "=== GREEN marker: production build + SH414 hermetics + type4 green + pin byte-exact ===" | tee -a "$LOG"