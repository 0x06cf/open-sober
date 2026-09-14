#!/bin/bash
# SH146: COMPOSED real-avatar scene — N distinct Roblox .mesh objects rendered in
# ONE frame, each with its own VBO/EBO, per-object compose-MVP (translate+scale
# +yaw) and SH145 diffuse lighting, all driven through the engine's own geometry
# wrapper 0x105b35288. Avatar torso + head + MaterialManager sphere.
# NOTE: --renderframe-mesh + --renderframe-mesh-tex are REQUIRED here because the
# compose block reuses the mesh-uv program those flags build (mesh_uv_mode).
set -u
cd "$(dirname "$0")/.."
AS=/home/hermes-worker/.cache/open-sober/android-env/assets
TORSO="${1:-$AS/content/avatar/compositing/CompositTorsoBase.mesh}"
HEAD="${2:-$AS/content/avatar/heads/head.mesh}"
SPHERE="${3:-$AS/content/models/MaterialManager/smooth_sphere.mesh}"
DDS="$AS/android/textures/studs.dds"
TAG="${4:-sh146-compose}"
LOG=/home/hermes-worker/runs/${TAG}.txt
PNG=/home/hermes-worker/runs/${TAG}.png
RAW=/home/hermes-worker/runs/${TAG}.rgb
rm -f "$LOG" "$PNG" "$RAW"
COMLIST="$TORSO,$HEAD,$SPHERE"
timeout 90 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --renderinit 0x105b3a280 --renderthunk --renderframe \
  --renderframe-drive --renderframe-seedgles --renderframe-drawprobe --renderframe-triangle \
  --renderframe-mesh "$SPHERE" --renderframe-mesh-tex "$DDS" \
  --renderframe-mesh-compose "$COMLIST" \
  --kicker 0x106863af8 > "$LOG" 2>&1 &
PID=$!
for i in $(seq 1 150); do
  if grep -q "mesh-compose\] 3/3" "$LOG" 2>/dev/null; then break; fi
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
echo "=== per-object parse + draw ==="
grep -E "mesh-compose\] obj" "$LOG"
echo "=== composed-frame summary ==="
grep -E "mesh-compose\] 3/3" "$LOG"
echo "SIGSEGV/SIGABRT: $(grep -icE 'SIGSEGV|SIGABRT' "$LOG")"
grep -oE "silhouette=[0-9]+/9" "$LOG"
if [ -s "$RAW" ]; then
  ffmpeg -y -loglevel error -f rawvideo -pix_fmt rgb24 -s 1280x720 -i "$RAW" "$PNG"
  python3 -c "
import statistics
d=open('$RAW','rb').read()
def analyze(x0,x1,y0,y1):
    pts=[]
    for yy in range(y0,y1,2):
        for xx in range(x0,x1,2):
            j=(yy*1280+xx)*3; r,g,b=d[j],d[j+1],d[j+2]
            if abs(r-5)<=3 and abs(g-5)<=3 and abs(b-36)<=3: continue
            pts.append((r,g,b,(r+g+b)//3))
    return len(pts),(statistics.mean(p[3] for p in pts) if pts else 0)
for i,(x0,x1) in enumerate([(0,426),(426,853),(853,1280)]):
    n,ml=analyze(x0,x1,100,620)
    print('third %d: geometry px=%d meanlum=%.1f'%(i,n,ml))
"
fi