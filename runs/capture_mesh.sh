#!/bin/bash
# SH141: the engine's OWN coherent geometry draw path renders a REAL Roblox
# in-world/avatar mesh asset — the first non-host-authored 3D geometry through the
# GLES bridge. parse_roblox_mesh_v2 decodes a Roblox `version 2.00` binary mesh
# (content/avatar/compositing/*.mesh), centers+scales its model-space positions to
# NDC, uploads them as a real VBO/EBO through the JIT GLES bridge, then drives the
# engine's own geometry wrapper 0x5b35288 (primitive-setup 0x5b353d0 -> real
# indexed glDrawElements with the mesh's full index list). A 5x5 grid probe reads
# back the rendered silhouette.
#
# Usage: capture_mesh.sh [mesh-path] [out-tag]
#   default mesh = CompositTorsoBase.mesh (664 verts / 416 faces) -> recognizable avatar torso.
# Requires: cargo build -p arm64jit --example elfjit, real libroblox.so, Xvfb+ffmpeg.
set -u
cd "$(dirname "$0")/.."
MESH="${1:-/home/hermes-worker/.cache/open-sober/android-env/assets/content/avatar/compositing/CompositTorsoBase.mesh}"
TAG="${2:-sh141-mesh}"
LOG=/home/hermes-worker/runs/${TAG}.txt
PNG=/home/hermes-worker/runs/${TAG}.png
RAW=/home/hermes-worker/runs/${TAG}.rgb
rm -f "$LOG" "$PNG" "$RAW"
test -f "$MESH" || { echo "MESH NOT FOUND: $MESH"; exit 2; }
timeout 90 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --renderinit 0x105b3a280 --renderthunk --renderframe \
  --renderframe-drive --renderframe-seedgles --renderframe-drawprobe --renderframe-triangle \
  --renderframe-mesh "$MESH" \
  --kicker 0x106863af8 > "$LOG" 2>&1 &
PID=$!
for i in $(seq 1 140); do
  if grep -q "silhouette 5x5" "$LOG" 2>/dev/null; then break; fi
  if ! kill -0 "$PID" 2>/dev/null; then break; fi
  sleep 0.5
done
sleep 1
DISPNUM=$(grep -oE "on :[0-9]+" "$LOG" | head -1 | tr -d 'on :')
echo "capturing display :${DISPNUM:-none}"
if [ -n "$DISPNUM" ]; then
  ffmpeg -y -loglevel error -f x11grab -video_size 1280x720 -i ":$DISPNUM" \
    -frames:v 1 -f rawvideo -pix_fmt rgb24 - 2>/dev/null > "$RAW"
  echo "RAW_BYTES=$(stat -c%s "$RAW" 2>/dev/null || echo 0)"
fi
wait $PID
echo "EXIT=$?"
echo "=== real mesh parsed + geometry wrapper through bridge? ==="
grep -E "renderframe-mesh\] parsed real|coherent renderer.*mesh=|geometry wrapper 0x5b35288" "$LOG" | tail -5
echo "=== silhouette 5x5 (drawn points = the real mesh footprint) ==="
grep -E "silhouette 5x5" "$LOG"
echo "=== hostcall@gl draw/upload reached Mesa? ==="
grep -E "hostcall@gl(DrawElements|BindBuffer|BufferData|VertexAttribPointer|UseProgram)" "$LOG" | grep -vE "VECLD" | tail -8
echo "=== pixel analysis (expect a real silhouette, NOT background) ==="
if [ -s "$RAW" ]; then
  ffmpeg -y -loglevel error -f rawvideo -pix_fmt rgb24 -s 1280x720 -i "$RAW" "$PNG"
  python3 -c "
import sys
from collections import Counter
d=open('$RAW','rb').read()
c=Counter(); drawn=0; total=0
for j in range(0,len(d)-2,3):
    r,g,b=d[j],d[j+1],d[j+2]
    c[(r,g,b)]+=1; total+=1
    if r>150 and g<90 and b<90: drawn+=1
print('total px=%d  red-silhouette px=%d (%.2f%%)'%(total,drawn,100.0*drawn/total))
for col,cnt in c.most_common(4):
    print('  RGB%d,%d,%d x%d'%(col[0],col[1],col[2],cnt))
"
fi
echo "=== crash summary ==="
grep -icE "SIGSEGV|SIGABRT" "$LOG"