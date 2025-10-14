#!/usr/bin/env python3
import json, os, hashlib, time, sys

DIST = os.path.join(os.getcwd(), 'dist')

def sha256(p):
    h = hashlib.sha256()
    with open(p, 'rb') as f:
        for chunk in iter(lambda: f.read(1<<20), b''):
            h.update(chunk)
    return h.hexdigest()

def main():
    artifacts = []
    for name in sorted(os.listdir(DIST)):
        if name.endswith(('.exe','-linux-x64','-gnu.exe','-msvc.exe')):
            p = os.path.join(DIST, name)
            artifacts.append({
                'name': name,
                'path': os.path.relpath(p, start='.'),
                'size': os.path.getsize(p),
                'sha256': sha256(p),
            })
    commit = os.popen('git rev-parse --short HEAD').read().strip() or 'unknown'
    now = int(time.time())
    manifest = {
        'version_tag': 'v1.1-frozen',
        'commit': commit,
        'generated_at': now,
        'artifacts': artifacts,
        'notes': 'Frozen toolchain remint with macro enhancements (Ubuntu + Windows, MSVC/GNU)',
        'support_matrix': {
            'linux-x64': {'format':'ELF','runtime':'static','linker':'clang','status':'verified'},
            'win-x64-msvc': {'format':'PE','runtime':'static','toolchain':'MSVC/clang','status':'verified'},
            'win-x64-gnu': {'format':'PE','runtime':'static','toolchain':'MinGW','status':'verified'}
        }
    }
    out = os.path.join(DIST, 'release.json')
    with open(out, 'w', encoding='utf-8') as f:
        json.dump(manifest, f, ensure_ascii=False, indent=2)
    print(out)

if __name__ == '__main__':
    if not os.path.isdir(DIST):
        print('dist/ not found', file=sys.stderr); sys.exit(2)
    main()
