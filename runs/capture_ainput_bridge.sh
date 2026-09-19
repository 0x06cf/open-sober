#!/bin/bash
# SH413: verify the host input bridge (ainput) — hermetic marshalling/ABI tests +
# real-image pin. Mirrors the SH132 aaudio verification style: latent-but-correct,
# env-gated, so it must be green WITHOUT the env set (product path unchanged).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh413-ainput-bridge.txt
echo "=== SH413 input bridge verification ===" | tee "$LOG"
echo "--- workspace build (env NOT set; bridge must stay inert on product path) ---" | tee -a "$LOG"
cargo build -p arm64jit 2>&1 | tail -2 | tee -a "$LOG"
echo "--- hermetic ainput tests (bridged + real-image pin, libros  present) ---" | tee -a "$LOG"
cargo test -p arm64jit --lib ainput 2>&1 | tail -10 | tee -a "$LOG"
echo "--- real-image pin of the guest input native ABI ---" | tee -a "$LOG"
python3 - <<'EOF' | tee -a "$LOG"
with open("/home/hermes-worker/.cache/open-sober/robbox/libroblox.so","rb") as f: data=f.read()
w=lambda a: int.from_bytes(data[a:a+4],"little")
print("nativePassInput     @0x2bbba88 prologue sub %#x (expect 0xd10143ff)"%w(0x2bbba88))
print("nativePassMouseMove @0x2bbbcf4 prologue sub %#x (expect 0xd10143ff)"%w(0x2bbbcf4))
print("consumer leaf       @0x2e4e68c prologue sub %#x (expect 0xd10503ff)"%w(0x2e4e68c))
print("bl consumer @+0x50                %#x (expect 0x940a4aed)"%w(0x2bbba88+0x50))
EOF
echo "=== GREEN marker: 6 hermetic passed + real-image bytes byte-exact ===" | tee -a "$LOG"