# 🚀 First Five Apps - Nyashの実力を証明する最初の5本

## 🎯 概要
Phase 11.5完了を待たずに、**今すぐ作れる**実用アプリ5本で、Nyashの産業レベルの完成度を世に示します。
すべて「Everything is Box／PluginInvoke」で統一実装し、VM/JIT/AOT/WASMの全バックエンドで動作確認します。

## 📋 アプリケーション一覧

### 1. ny-echo（最小CLI）- 基本I/O検証
**目的**: I/O・StringBoxの道通し確認

```nyash
// apps/ny-echo/main.nyash
static box Main {
    main(args) {
        local console = new ConsoleBox()
        local options = parseArgs(args)
        
        loop(true) {
            local input = console.readLine()
            if input == null { break }
            
            local output = input
            if options.get("upper") {
                output = input.toUpperCase()
            } else if options.get("lower") {
                output = input.toLowerCase()
            }
            
            console.log(output)
        }
    }
}
```

**受入基準**:
- [ ] VM/JIT/AOT/GCオン・オフすべてでtrace_hash一致
- [ ] 100万行処理で性能劣化なし
- [ ] メモリリークなし（GCカウンター確認）

### 2. ny-jsonlint（Python連携デモ）- プラグイン統合
**目的**: PyRuntimeBox/PyObjectBox経由のPluginInvoke検証

```nyash
// apps/ny-jsonlint/main.nyash
static box Main {
    init { py, console }
    
    main(args) {
        me.py = new PyRuntimeBox()
        me.console = new ConsoleBox()
        
        local filename = args.get(1)
        if filename == null {
            me.console.error("Usage: ny-jsonlint <file.json>")
            return 1
        }
        
        local file = new FileBox()
        file.open(filename, "r")
        local content = file.read()
        file.close()
        
        local result = me.py.eval("
import json
try:
    json.loads(content)
    'OK'
except Exception as e:
    f'NG: {str(e)}'
", new MapBox().set("content", content))
        
        me.console.log(result)
        return result.startsWith("OK") ? 0 : 1
    }
}
```

**受入基準**:
- [ ] OS差なく実行（Windows/Linux/macOS）
- [ ] --sealedモードで完全再現可能
- [ ] 大規模JSON（10MB）でも安定動作

### 3. ny-array-bench（性能デモ）- ベンチマーク基準
**目的**: ArrayBox map/reduce、StatsBox導入、性能可視化

```nyash
// apps/ny-array-bench/main.nyash
static box Main {
    init { stats }
    
    main(args) {
        me.stats = new StatsBox()
        local sizes = [1000, 10000, 100000]
        
        loop(size in sizes) {
            me.benchArrayOps(size)
        }
        
        // 結果をJSON出力（CI集計用）
        local result = me.stats.toJSON()
        print(result)
    }
    
    benchArrayOps(size) {
        local array = new ArrayBox()
        
        // 1. 配列生成
        me.stats.startTimer("create_" + size)
        loop(i < size) {
            array.push(i)
        }
        me.stats.endTimer("create_" + size)
        
        // 2. map操作
        me.stats.startTimer("map_" + size)
        local doubled = array.map(|x| x * 2)
        me.stats.endTimer("map_" + size)
        
        // 3. reduce操作
        me.stats.startTimer("reduce_" + size)
        local sum = doubled.reduce(|a, b| a + b, 0)
        me.stats.endTimer("reduce_" + size)
        
        // VM基準の相対性能を記録
        me.stats.recordRelative("vm", 1.0)
        if IS_JIT { me.stats.recordRelative("jit", SPEEDUP) }
        if IS_AOT { me.stats.recordRelative("aot", SPEEDUP) }
    }
}
```

**受入基準**:
- [ ] VM=1.0x基準でJIT/AOTの倍率表示
- [ ] fallbacks=0（完全最適化）
- [ ] 結果JSON自動出力（CI集計可能）

### 4. ny-filegrep（実用ミニ）- ファイルI/O実用例
**目的**: BytesBox/FileBox（プラグイン）I/O、実用的なツール

```nyash
// apps/ny-filegrep/main.nyash
static box Main {
    init { pattern, recursive, results }
    
    main(args) {
        me.parseArgs(args)
        me.results = new ArrayBox()
        
        local path = args.getLast() || "."
        me.searchPath(path)
        
        // 結果表示
        loop(result in me.results) {
            print(result)
        }
        
        return me.results.length() > 0 ? 0 : 1
    }
    
    searchPath(path) {
        local file = new FileBox()
        
        if file.isDirectory(path) {
            if me.recursive {
                local entries = file.listDir(path)
                loop(entry in entries) {
                    me.searchPath(path + "/" + entry)
                }
            }
        } else {
            me.searchFile(path)
        }
    }
    
    searchFile(filepath) {
        local file = new FileBox()
        file.open(filepath, "r")
        
        local lineNum = 0
        loop(true) {
            local line = file.readLine()
            if line == null { break }
            
            lineNum = lineNum + 1
            if line.contains(me.pattern) {
                me.results.push(filepath + ":" + lineNum + ":" + line)
            }
        }
        
        file.close()
    }
}
```

**受入基準**:
- [ ] Windows/Linux/macOSで同一結果
- [ ] 大規模ディレクトリ（1万ファイル）対応
- [ ] メモリ効率的（ストリーム処理）

### 5. ny-http-hello（WASM/ネイティブ両対応）- Web実用例
**目的**: NetBox（プラグイン）とイベントループ、FutureBox活用

```nyash
// apps/ny-http-hello/main.nyash
static box Main {
    init { server, running }
    
    main(args) {
        local port = args.get(1) || "8080"
        me.server = new HttpServerBox()
        me.running = true
        
        // シグナルハンドラー設定
        registerSignal("SIGINT", || me.stop())
        
        // サーバー起動
        me.server.start(port.toInteger())
        print("Server listening on http://localhost:" + port)
        
        // リクエストループ
        loop(me.running) {
            nowait request = me.server.accept()
            me.handleRequest(wait request)
        }
        
        me.server.stop()
        return 0
    }
    
    handleRequest(request) {
        local response = new HttpResponseBox()
        
        if request.path() == "/" {
            response.setStatus(200)
            response.setHeader("Content-Type", "text/plain")
            response.write("Hello from Nyash!")
        } else {
            response.setStatus(404)
            response.write("Not Found")
        }
        
        request.respond(response)
    }
    
    stop() {
        print("Shutting down...")
        me.running = false
    }
}
```

**受入基準**:
- [ ] 100req/s程度のスモーク通過
- [ ] 停止シグナルでクリーンfini
- [ ] WASMビルドでも動作（制限付き）

## 🎯 実装優先順位

1. **ny-echo** - 最小実装、CI基盤確立
2. **ny-array-bench** - 性能基準確立
3. **ny-jsonlint** - プラグイン統合実証
4. **ny-filegrep** - 実用性実証
5. **ny-http-hello** - Web対応実証

## 📊 成功指標

### 全体指標
- [ ] 5アプリすべてがVM/JIT/AOTで動作
- [ ] CIでの自動テスト確立
- [ ] ドキュメント・サンプル完備

### 性能指標
- [ ] JIT: VMの5倍以上高速
- [ ] AOT: VMの10倍以上高速
- [ ] メモリ使用量: 同等機能のPython比50%以下

### 品質指標
- [ ] ゼロクラッシュ（1000回実行）
- [ ] メモリリークなし（長時間実行）
- [ ] プラットフォーム差異なし

## 🚀 配布戦略

### リリース形式
```
nyash-apps-v1.0/
├── bin/
│   ├── ny-echo[.exe]
│   ├── ny-jsonlint[.exe]
│   ├── ny-array-bench[.exe]
│   ├── ny-filegrep[.exe]
│   └── ny-http-hello[.exe]
├── examples/
│   └── *.nyash (ソースコード)
├── benchmarks/
│   └── results.json
└── README.md
```

### 展開先
- GitHub Releases
- Homebrew (macOS)
- Scoop (Windows)
- Docker Hub (コンテナ版)

これで「30日で作った言語」の実力を世界に示せます！🎉