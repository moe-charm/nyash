# Plugins スイート補足（HostHandleRouter 境界）

- HostHandleRouter 境界スイート（-1/-11/-13/-14）をまとめて実行:

```
tools/smokes/v2/run.sh --profile plugins --filter 'hosthandle_boundary_*'
```

- 返却型不一致（-14）を観測するテストフックを有効化する場合:

```
HAKO_HOSTHANDLE_TEST_RET_MISMATCH=1 \
  tools/smokes/v2/run.sh --profile plugins --filter hosthandle_return_type_mismatch_vm.sh
```

備考
- プラグインのビルドが必要な環境では初回実行時に時間がかかることがあります。
- plugin-only ビルド環境では、一部レガシー依存の境界テストは SKIP になります（仕様）。

