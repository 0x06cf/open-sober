#!/bin/bash
# SH144: real studs-textured smooth_sphere.mesh rendered from an ORBITING camera
# (model yawed about +Y across N frames) through the engine's own geometry
# wrapper 0x105b35288 — a real rotating textured 3D scene, per-object UV + MVP.
set -u
cd "$(dirname "$0")/.."
MESH="${1:-/home/hermes-worker/.cache/open-sober/android-env/assets/content/models/MaterialManager/smooth_sphere.mesh}"
DDS="${2:-/home/hermes-worker/.cache/open-sober/android-env/assets/android/textures/studs.dds}"
TAG="${3:-sh144-orbit}"
N="${4:-4}"
LOG=/home/hermes-worker/runs/${TAG}.txt
PNG0=/home/hermes-worker/runs/${TAG}-f0.png
PNG1=/home/hermes-worker/runs/${TAG}-f1.png
RAW0=/home/hermes-worker/runs/${TAG}-f0.rgb
RAW1=/home/hermes-worker/runs/${TAG}-f1.rgb
rm -f "$LOG" "$PNG0" "$PNG1" "$RAW0" "$RAW1"
test -f "$MESH" || { echo "MESH NOT FOUND"; exit 2; }
test -f "$DDS" || { echo "DDS NOT FOUND"; exit 2; }
timeout 120 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --renderinit 0x105b3a280 --renderthunk --renderframe \
  --renderframe-drive --renderframe-seedgles --renderframe-drawprobe --renderframe-triangle \
  --renderframe-mesh "$MESH" --renderframe-mesh-tex "$DDS" --renderframe-mesh-uv-orbit "$N" \
  --kicker 0x106863af8 > "$LOG" 2>&1 &
PID=$!
# Wait for orbit frame #1 to be presented, then capture two frames ~900ms apart.
for i in $(seq 1 150); do
  if grep -q "frame 1:" "$LOG" 2>/dev/null; then break; fi
  if ! kill -0 "$PID" 2>/dev/null; then break; fi
  sleep 0.4
done
DISPNUM=$(grep -oE "on :[0-9]+" "$LOG" | head -1 | tr -d 'on :')
echo "capturing display :${DISPNUM:-none}"
grab() { # $1 = raw out
  if [ -n "${DISPNUM:-}" ]; then
    ffmpeg -y -loglevel error -f x11grab -video_size 1280x720 -i ":$DISPNUM" \
      -frames:v 1 -f rawvideo -pix_fmt rgb24 - 2>/dev/null > "$1"
    echo "  $(basename $1): $(stat -c%s "$1" 2>/dev/null || echo 0) bytes"
  fi
}
grab "$RAW0"
sleep 0.6
grab "$RAW1"
wait $PID
echo "EXIT=$?"
echo "=== real mesh + texture + orbit markers ==="
grep -E "parsed real mesh|parsed real DDS|real MVP uploaded|mesh-uv-orbit\] frame" "$LOG" | head -12
echo "=== geometry wrapper + swap ==="
grep -E "geometry wrapper 0x5b35288" "$LOG" | tail -2
echo "=== crash check ==="
echo "SIGSEGV/SIGABRT count: $(grep -icE 'SIGSEGV|SIGABRT' "$LOG")"
echo "=== two-frame difference (rotation proof: frames must DIFFER) ==="
if [ -s "$RAW0" ] && [ -s "$RAW1" ]; then
  python3 -c "
d0=open('$RAW0','rb').read(); d1=open('$RAW1','rb').read()
diff=sum(1 for j in range(0,len(d0)-2,3) if abs(d0[j]-d1[j])+abs(d0[j+1]-d1[j+1])+abs(d0[j+2]-d1[j+2])>12)
tot=len(d0)//3
# non-bg sphere pixels per frame
def sphere(d):
    import collections
    c=collections.Counter(); n=0
    for j in range(0,len(d)-2,3):
        r,g,b=d[j],d[j+1],d[j+2]
        if not(abs(r-0)<=2 and abs(g-0)<=2 and abs(b-76)<=2): n+=1
    return n
print('changed px between frames: %d / %d (%.1f%%)'%(diff,tot,100.0*diff/tot))
print('sphere(geometry) px frame0=%d frame1=%d'%(sphere(d0),sphere(d1)))
"
  ffmpeg -y -loglevel error -f rawvideo -pix_fmt rgb24 -s 1280x720 -i "$RAW0" "$PNG0"
  ffmpeg -y -loglevel error -f rawvideo -pix_fmt rgb24 -s 1280x720 -i "$RAW1" "$PNG1"
fi