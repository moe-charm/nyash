#!/usr/bin/env bash
set -euo pipefail

# Sign dist/ artifacts with GPG (detached ASCII). Idempotent.
# Usage:
#   GPG_SIGN=1 GPG_KEY_ID=<KEYID> tools/release/sign_artifacts.sh
# Env:
#   GPG_SIGN   : 1|true to enable (default off)
#   GPG_KEY_ID : signer key id / fingerprint / uid (required when enabled)

DIST=dist
if [ ! -d "$DIST" ]; then
  echo "[sign] dist/ not found" >&2; exit 2
fi

case "${GPG_SIGN:-}" in
  1|true|TRUE|on|ON) ;; 
  *) echo "[sign] GPG_SIGN is not enabled; skip"; exit 0;;
esac

if ! command -v gpg >/dev/null 2>&1; then
  echo "[sign][ERROR] gpg not found. Install: sudo apt-get install -y gnupg" >&2
  exit 2
fi

if [ -z "${GPG_KEY_ID:-}" ]; then
  echo "[sign][ERROR] GPG_KEY_ID is required (export GPG_KEY_ID=<KEYID>)" >&2
  exit 2
fi

echo "[sign] using key: $GPG_KEY_ID"

# Collect artifacts (exclude .asc)
mapfile -t FILES < <(ls -1 "$DIST" | grep -E '^(hako-frozen-v1|HASHES|release\.json)' | grep -v '\.asc$' | sed "s|^|$DIST/|")
if [ ${#FILES[@]} -eq 0 ]; then
  echo "[sign][WARN] no files to sign in dist/" >&2
  exit 0
fi

# Sign all except release.json first
for f in "${FILES[@]}"; do
  base=$(basename "$f")
  if [ "$base" = "release.json" ]; then continue; fi
  out="${f}.asc"
  if [ -f "$out" ]; then
    echo "[sign] exists: $out (skipping)"
    continue
  fi
  echo "[sign] gpg --detach-sign $base"
  gpg --batch --yes --local-user "$GPG_KEY_ID" --output "$out" --detach-sign --armor "$f"
done

# Update release.json with signatures map
if [ -f "$DIST/release.json" ]; then
  python3 - "$DIST/release.json" <<'PY'
import json, os, sys
p=sys.argv[1]
with open(p,'r',encoding='utf-8') as f:
  m=json.load(f)
sigs={}
dist=os.path.dirname(p)
for name in os.listdir(dist):
  if name.endswith('.asc') and name!='release.json.asc':
    target=name[:-4]
    sigs[target]=os.path.join(dist,name).replace('\\','/')
m['signatures']=sigs
tmp=p+".tmp"
with open(tmp,'w',encoding='utf-8') as f:
  json.dump(m,f,ensure_ascii=False,indent=2)
os.replace(tmp,p)
print('[sign] release.json updated with signatures')
PY
else
  echo "[sign][WARN] $DIST/release.json not found; signatures.json is not generated" >&2
fi

# Finally, sign release.json itself
if [ -f "$DIST/release.json" ]; then
  if [ ! -f "$DIST/release.json.asc" ]; then
    echo "[sign] gpg --detach-sign release.json"
    gpg --batch --yes --local-user "$GPG_KEY_ID" --output "$DIST/release.json.asc" --detach-sign --armor "$DIST/release.json"
  else
    echo "[sign] exists: release.json.asc (skipping)"
  fi
fi

echo "[sign] done"

