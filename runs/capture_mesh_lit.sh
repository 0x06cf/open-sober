#!/bin/bash
# SH145: diffuse LIGHTING on the real studs-textured sphere. Per-vertex normals
# (3rd attrib, fmt2=size3 GL_FLOAT) uploaded now; VS computes world-space normal
# via mat3(uModelRot)*aNormal; FS = tex * (0.30 + 0.70*dot(N,L)). The lit sphere
# must show a brightness gradient (light-pole bright, terminator dark) instead
# of SH143's flat gray.
set -u
cd "$(dirname "$0")/.."
MESH="${1:-/home/hermes-worker/.cache/open-sober/android-env/assets/content/models/MaterialManager/smooth_sphere.mesh}"
DDS="${2:-/home/hermes-worker/.cache/open-sober/android-env/assets/android/textures/studs.dds}"
TAG="${3:-sh145-lit}"
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
  if grep -q "uModelRot" "$LOG" 2>/dev/null && grep -q "silhouette 5x5" "$LOG" 2>/dev/null; then break; fi
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
echo "=== mesh + texture + MVP + uModelRot uploaded? ==="
grep -E "renderframe-mesh\] parsed real|renderframe-mesh-tex\] parsed real DDS|uModelRot uploaded|geometry wrapper 0x5b35288" "$LOG"
echo "=== silhouette ==="
grep -E "silhouette 5x5" "$LOG"
echo "=== LIGHTING gradient (light-pole must be BRIGHTER than terminator) ==="
if [ -s "$RAW" ]; then
  ffmpeg -y -loglevel error -f rawvideo -pix_fmt rgb24 -s 1280x720 -i "$RAW" "$PNG"
  python3 -c "
d=open('$RAW','rb').read()
# the sphere: read column x=640 (center) top->bottom, sample 5 vertical points
# on a surface band. Light dir (0.4,0.7,0.6): upper band (light-pole ~ N pointing up-ish) should be brighter.
def px(x,y): j=(y*1280+x)*3; return d[j],d[j+1],d[j+2]
import statistics
def band_avg(yc, dy=40):
    vals=[]
    for dy0 in range(-dy,dy+1,2):
        r,g,b=px(540, yc+dy0); vals.append((r+g+b)/3.0)
    return statistics.mean(vals)
# upper sphere band (y~180) vs lower sphere band (y~540): light at +Y-ish so upper>lower
upper=band_avg(180); lower=band_avg(540)
print('upper-band avg luminance=%.1f  lower-band=%.1f  diff=%.1f'%(upper,lower,upper-lower))
# 5-point vertical gradient within the sphere
for y in [200,260,320,380,440]:
    print('  y=%d center lum=%.1f'%(y, band_avg(y,12)))
print('VERDICT:', 'LIT (upper brighter than lower)' if upper>lower else 'FLAT/FAIL')
"
fi
echo "=== crash check ==="
grep -icE "SIGSEGV|SIGABRT" "$LOG"