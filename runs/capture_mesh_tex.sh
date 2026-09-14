#!/bin/bash
# SH142: real APK TEXTURE (studs.dds R8) rendered on real mesh geometry
# (smooth_sphere.mesh) through the engine's own GLES path. The DDS material map
# is parsed (DDPF_LUMINANCE R8), expanded to RGBA, uploaded via glTexImage2D and
# sampled over the MaterialManager preview sphere.
set -u
cd "$(dirname "$0")/.."
MESH="${1:-/home/hermes-worker/.cache/open-sober/android-env/assets/content/models/MaterialManager/smooth_sphere.mesh}"
DDS="${2:-/home/hermes-worker/.cache/open-sober/android-env/assets/android/textures/studs.dds}"
TAG="${3:-sh142-mesh-tex}"
LOG=/home/hermes-worker/runs/${TAG}.txt
PNG=/home/hermes-worker/runs/${TAG}.png
RAW=/home/hermes-worker/runs/${TAG}.rgb
rm -f "$LOG" "$PNG" "$RAW"
test -f "$MESH" || { echo "MESH NOT FOUND"; exit 2; }
test -f "$DDS" || { echo "DDS NOT FOUND"; exit 2; }
timeout 90 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --renderinit 0x105b3a280 --renderthunk --renderframe \
  --renderframe-drive --renderframe-seedgles --renderframe-drawprobe --renderframe-triangle \
  --renderframe-mesh "$MESH" --renderframe-mesh-tex "$DDS" \
  --kicker 0x106863af8 > "$LOG" 2>&1 &
PID=$!
for i in $(seq 1 150); do
  if grep -q "parsed real DDS" "$LOG" 2>/dev/null && grep -q "silhouette 5x5" "$LOG" 2>/dev/null; then break; fi
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
echo "=== real mesh + real texture parsed? ==="
grep -E "renderframe-mesh\] parsed real|renderframe-mesh-tex\] parsed real DDS" "$LOG"
echo "=== silhouette + textured readback ==="
grep -E "silhouette 5x5|mesh-tex" "$LOG" | tail -4
echo "=== geometry wrapper ==="
grep -E "geometry wrapper 0x5b35288" "$LOG" | tail -2
echo "=== pixel histogram (expect grayscale studs texture over the sphere) ==="
if [ -s "$RAW" ]; then
  ffmpeg -y -loglevel error -f rawvideo -pix_fmt rgb24 -s 1280x720 -i "$RAW" "$PNG"
  python3 -c "
from collections import Counter
d=open('$RAW','rb').read()
c=Counter(); sphere=0; gray=0
for j in range(0,len(d)-2,3):
    r,g,b=d[j],d[j+1],d[j+2]
    c[(r,g,b)]+=1
    if r>60 or g>60 or b>60:
        sphere+=1
        if abs(r-g)<24 and abs(g-b)<24: gray+=1
print('bright-ish px=%d  of those grayscale=%d (%.1f%%)'%(sphere,gray,100.0*gray/max(1,sphere)))
for col,cnt in c.most_common(5):
    print('  RGB%d,%d,%d x%d'%(col[0],col[1],col[2],cnt))
"
fi
echo "=== crash summary ==="
grep -icE "SIGSEGV|SIGABRT" "$LOG"