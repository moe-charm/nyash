# Smokes Index

Purpose
- 軽量なローカル確認やCI向けのスモークを用途別に集約するためのインデックスだよ。

Categories
- pyvm: PyVM 参照実行の代表スモーク
- llvm: llvmlite/ny-llvmc を使った AOT/EXE スモーク
- selfhost: 自己ホスト（Ny→JSON v0→実行）のスモーク

Entry scripts
- `./tools/smokes/fast_local.sh`
  - 手元確認用の最小セット（PyVM 小パック + crate EXE 3ケース + 短絡ブリッジ）
- `./tools/smokes/selfhost_local.sh`
  - 自己ホスト側の簡易確認（parser→JSON→PyVM 実行）

Notes
- 既存の多数のスモークは `tools/` 直下にあるよ（歴史的事情）。
  少しずつ `tools/smokes/` 配下の集約ランナーに寄せていく方針だよ。

