# なんでもAPI計画：実世界での具体例

## 🌟 革命的開発体験の実例

### 🎮 ゲーム開発例：Nyashブラウザゲーム
```nyash
# === nyash.link ===
[dependencies]
nyashstd = { builtin = true }
canvas_api = { bid = "./apis/canvas.yaml" }
dom_api = { bid = "./apis/dom.yaml" }
audio_api = { bid = "./apis/webaudio.yaml" }

# === game.nyash ===
using nyashstd
using canvas_api
using dom_api  
using audio_api

static box Game {
    init { canvas_id, score, player_x, player_y, enemies }
    
    main() {
        me.canvas_id = "game-canvas"
        me.score = 0
        me.player_x = 200
        me.player_y = 300
        me.enemies = new ArrayBox()
        
        # DOMイベント設定（FFI-ABI経由）
        dom.addEventListener("keydown", me.handleKeyDown)
        
        # ゲームループ開始
        me.gameLoop()
    }
    
    gameLoop() {
        loop(true) {
            me.update()
            me.render()
            
            # ブラウザのrequestAnimationFrame（FFI-ABI）
            dom.requestAnimationFrame(me.gameLoop)
        }
    }
    
    update() {
        # 敵の移動（組み込み標準ライブラリ）
        local i = 0
        loop(i < array.length(me.enemies)) {
            local enemy = array.get(me.enemies, i)
            enemy.y = enemy.y + enemy.speed
            i = i + 1
        }
        
        # 当たり判定（組み込み数学関数）
        local distance = math.sqrt(
            math.pow(me.player_x - enemy.x, 2) + 
            math.pow(me.player_y - enemy.y, 2)
        )
        
        if distance < 30 {
            me.gameOver()
        }
    }
    
    render() {
        # 画面クリア（Canvas API - FFI-ABI）
        canvas.fillRect(me.canvas_id, 0, 0, 800, 600, "black")
        
        # プレイヤー描画
        canvas.fillRect(me.canvas_id, me.player_x, me.player_y, 20, 20, "blue")
        
        # 敵描画
        local i = 0
        loop(i < array.length(me.enemies)) {
            local enemy = array.get(me.enemies, i)
            canvas.fillRect(me.canvas_id, enemy.x, enemy.y, 15, 15, "red")
            i = i + 1
        }
        
        # スコア表示
        local score_text = "Score: " + string.toString(me.score)
        canvas.fillText(me.canvas_id, score_text, 10, 30, "20px Arial", "white")
    }
    
    handleKeyDown(event) {
        # キーボード入力処理（DOM API経由）
        local key = dom.getEventKey(event)
        
        if key == "ArrowLeft" {
            me.player_x = me.player_x - 10
        } else if key == "ArrowRight" {
            me.player_x = me.player_x + 10
        } else if key == " " {  # スペースキー
            me.shoot()
        }
    }
    
    shoot() {
        # 効果音再生（Web Audio API - FFI-ABI）
        audio.playSound("shoot.wav")
        
        # 弾の生成・発射処理
        # ...
    }
    
    gameOver() {
        # ゲームオーバー処理
        audio.playSound("gameover.wav")
        dom.alert("Game Over! Score: " + string.toString(me.score))
    }
}
```

### 🔬 データサイエンス例：画像処理アプリ
```nyash
# === nyash.link ===
[dependencies]
nyashstd = { builtin = true }
opencv_api = { bid = "./apis/opencv.yaml", library = "./libs/opencv.so" }
numpy_api = { bid = "./apis/numpy.yaml", library = "./libs/numpy.so" }
matplotlib_api = { bid = "./apis/matplotlib.yaml", library = "./libs/matplotlib.so" }
file_api = { bid = "./apis/file.yaml" }

# === image_processor.nyash ===
using nyashstd
using opencv_api
using numpy_api
using matplotlib_api
using file_api

static box ImageProcessor {
    init { input_path, output_path, processed_data }
    
    main() {
        me.input_path = "./images/input.jpg"
        me.output_path = "./images/output.jpg"
        
        # 画像読み込み（OpenCV - FFI-ABI）
        local image = opencv.imread(me.input_path)
        
        # 前処理
        local gray = opencv.cvtColor(image, "BGR2GRAY")
        local blurred = opencv.gaussianBlur(gray, 5, 5)
        
        # エッジ検出
        local edges = opencv.canny(blurred, 50, 150)
        
        # NumPy配列操作（NumPy - FFI-ABI）
        local edge_array = numpy.fromOpenCV(edges)
        local normalized = numpy.normalize(edge_array, 0, 255)
        
        # 統計計算（組み込み標準ライブラリ）
        local edge_count = me.countEdgePixels(normalized)
        local percentage = (edge_count * 100) / (image.width * image.height)
        
        # 結果表示
        io.println("Edge pixels: " + string.toString(edge_count))
        io.println("Edge percentage: " + string.toString(percentage) + "%")
        
        # 結果画像保存（OpenCV）
        opencv.imwrite(me.output_path, edges)
        
        # グラフ生成（Matplotlib - FFI-ABI）
        me.generateHistogram(normalized)
    }
    
    countEdgePixels(image_array) {
        local count = 0
        local height = numpy.shape(image_array, 0)
        local width = numpy.shape(image_array, 1)
        
        local y = 0
        loop(y < height) {
            local x = 0
            loop(x < width) {
                local pixel = numpy.get(image_array, y, x)
                if pixel > 0 {
                    count = count + 1
                }
                x = x + 1
            }
            y = y + 1
        }
        
        return count
    }
    
    generateHistogram(image_array) {
        # ヒストグラム計算（NumPy）
        local histogram = numpy.histogram(image_array, 256)
        
        # グラフ描画（Matplotlib）
        matplotlib.figure(800, 600)
        matplotlib.plot(histogram.bins, histogram.values)
        matplotlib.title("Edge Pixel Histogram")
        matplotlib.xlabel("Pixel Intensity")
        matplotlib.ylabel("Frequency")
        matplotlib.savefig("./images/histogram.png")
        matplotlib.show()
    }
}
```

### 🌐 Webサーバー例：RESTful API
```nyash
# === nyash.link ===
[dependencies]
nyashstd = { builtin = true }
http_server_api = { bid = "./apis/http_server.yaml" }
sqlite_api = { bid = "./apis/sqlite.yaml", library = "./libs/sqlite.so" }
json_api = { bid = "./apis/json.yaml" }
crypto_api = { bid = "./apis/crypto.yaml", library = "./libs/openssl.so" }

# === api_server.nyash ===
using nyashstd
using http_server_api
using sqlite_api
using json_api
using crypto_api

static box ApiServer {
    init { server, database, port }
    
    main() {
        me.port = 8080
        me.server = http_server.create()
        me.database = sqlite.open("./data/app.db")
        
        # データベース初期化
        me.initDatabase()
        
        # ルート設定
        http_server.route(me.server, "GET", "/api/users", me.getUsers)
        http_server.route(me.server, "POST", "/api/users", me.createUser)
        http_server.route(me.server, "PUT", "/api/users/:id", me.updateUser)
        http_server.route(me.server, "DELETE", "/api/users/:id", me.deleteUser)
        
        # サーバー開始
        io.println("Server starting on port " + string.toString(me.port))
        http_server.listen(me.server, me.port)
    }
    
    initDatabase() {
        local sql = "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )"
        
        sqlite.exec(me.database, sql)
    }
    
    getUsers(request, response) {
        # クエリ実行（SQLite - FFI-ABI）
        local sql = "SELECT id, name, email, created_at FROM users"
        local results = sqlite.query(me.database, sql)
        
        # JSON変換（JSON API - FFI-ABI）
        local json_response = json.stringify(results)
        
        # レスポンス送信（HTTP Server API）
        http_server.setHeader(response, "Content-Type", "application/json")
        http_server.setStatus(response, 200)
        http_server.send(response, json_response)
    }
    
    createUser(request, response) {
        # リクエストボディ解析
        local body = http_server.getBody(request)
        local user_data = json.parse(body)
        
        # バリデーション（組み込み標準ライブラリ）
        if string.length(user_data.name) < 2 {
            me.sendError(response, 400, "Name must be at least 2 characters")
            return
        }
        
        if not me.isValidEmail(user_data.email) {
            me.sendError(response, 400, "Invalid email format")
            return
        }
        
        # パスワードハッシュ化（Crypto API - FFI-ABI）
        local password_hash = crypto.hashPassword(user_data.password)
        
        # データベース挿入
        local sql = "INSERT INTO users (name, email, password_hash) VALUES (?, ?, ?)"
        local params = [user_data.name, user_data.email, password_hash]
        
        try {
            local user_id = sqlite.insert(me.database, sql, params)
            
            # 作成されたユーザー情報を返す
            local created_user = map.create()
            map.set(created_user, "id", user_id)
            map.set(created_user, "name", user_data.name)
            map.set(created_user, "email", user_data.email)
            
            local json_response = json.stringify(created_user)
            
            http_server.setHeader(response, "Content-Type", "application/json")
            http_server.setStatus(response, 201)
            http_server.send(response, json_response)
            
        } catch error {
            io.println("Database error: " + error.message)
            me.sendError(response, 500, "Failed to create user")
        }
    }
    
    isValidEmail(email) {
        # 簡単なメール検証（組み込み文字列関数）
        local at_pos = string.indexOf(email, "@")
        local dot_pos = string.lastIndexOf(email, ".")
        
        return at_pos > 0 and dot_pos > at_pos and dot_pos < string.length(email) - 1
    }
    
    sendError(response, status, message) {
        local error_obj = map.create()
        map.set(error_obj, "error", message)
        
        local json_error = json.stringify(error_obj)
        
        http_server.setHeader(response, "Content-Type", "application/json")
        http_server.setStatus(response, status)
        http_server.send(response, json_error)
    }
}
```

### 🔧 システムプログラミング例：ファイル監視ツール
```nyash
# === nyash.link ===
[dependencies]
nyashstd = { builtin = true }
libc_api = { bid = "./apis/libc.yaml", library = "system" }
inotify_api = { bid = "./apis/inotify.yaml", library = "system" }
filesystem_api = { bid = "./apis/filesystem.yaml" }

# === file_monitor.nyash ===
using nyashstd
using libc_api
using inotify_api
using filesystem_api

static box FileMonitor {
    init { watch_path, inotify_fd, watch_descriptors, callbacks }
    
    main() {
        me.watch_path = "./watched_directory"
        me.watch_descriptors = new ArrayBox()
        me.callbacks = map.create()
        
        # inotify初期化（Linux inotify - FFI-ABI）
        me.inotify_fd = inotify.init()
        
        if me.inotify_fd < 0 {
            io.println("Failed to initialize inotify")
            return
        }
        
        # ディレクトリ監視設定
        me.addWatch(me.watch_path)
        
        # コールバック設定
        me.setupCallbacks()
        
        io.println("File monitor started. Watching: " + me.watch_path)
        
        # メインループ
        me.eventLoop()
    }
    
    addWatch(path) {
        # 監視フラグ（inotify constants）
        local flags = inotify.IN_CREATE or inotify.IN_DELETE or 
                     inotify.IN_MODIFY or inotify.IN_MOVED_FROM or 
                     inotify.IN_MOVED_TO
        
        local wd = inotify.addWatch(me.inotify_fd, path, flags)
        
        if wd >= 0 {
            array.push(me.watch_descriptors, wd)
            io.println("Added watch for: " + path)
        } else {
            io.println("Failed to add watch for: " + path)
        }
    }
    
    setupCallbacks() {
        # ファイル作成コールバック
        map.set(me.callbacks, "CREATE", static function(event) {
            io.println("File created: " + event.name)
            
            # ファイル情報取得（Filesystem API）
            local file_info = filesystem.stat(event.path)
            local size = file_info.size
            local permissions = file_info.permissions
            
            io.println("  Size: " + string.toString(size) + " bytes")
            io.println("  Permissions: " + permissions)
        })
        
        # ファイル変更コールバック
        map.set(me.callbacks, "MODIFY", static function(event) {
            io.println("File modified: " + event.name)
            
            # 変更時刻記録
            local timestamp = time.now()
            local formatted_time = time.format(timestamp, "%Y-%m-%d %H:%M:%S")
            io.println("  Modified at: " + formatted_time)
        })
        
        # ファイル削除コールバック
        map.set(me.callbacks, "DELETE", static function(event) {
            io.println("File deleted: " + event.name)
            
            # ログファイルに記録
            me.logEvent("DELETE", event.name, time.now())
        })
    }
    
    eventLoop() {
        local buffer_size = 4096
        local buffer = libc.malloc(buffer_size)
        
        loop(true) {
            # inotify eventsを読み取り（blocking read）
            local bytes_read = libc.read(me.inotify_fd, buffer, buffer_size)
            
            if bytes_read > 0 {
                me.processEvents(buffer, bytes_read)
            } else if bytes_read == 0 {
                # EOF
                break
            } else {
                # エラー
                local error_code = libc.errno()
                io.println("Read error: " + string.toString(error_code))
                break
            }
        }
        
        libc.free(buffer)
    }
    
    processEvents(buffer, bytes_read) {
        local offset = 0
        
        loop(offset < bytes_read) {
            # inotify_event構造体解析（libc memory operations）
            local event = inotify.parseEvent(buffer, offset)
            
            # イベントタイプ判定
            local event_type = me.getEventType(event.mask)
            
            # 対応するコールバック実行
            if map.has(me.callbacks, event_type) {
                local callback = map.get(me.callbacks, event_type)
                callback(event)
            }
            
            # 次のイベントへ
            offset = offset + event.size
        }
    }
    
    getEventType(mask) {
        if mask and inotify.IN_CREATE {
            return "CREATE"
        } else if mask and inotify.IN_MODIFY {
            return "MODIFY"
        } else if mask and inotify.IN_DELETE {
            return "DELETE"
        } else if mask and inotify.IN_MOVED_FROM {
            return "MOVE_FROM"
        } else if mask and inotify.IN_MOVED_TO {
            return "MOVE_TO"
        } else {
            return "UNKNOWN"
        }
    }
    
    logEvent(event_type, filename, timestamp) {
        local log_entry = time.format(timestamp, "%Y-%m-%d %H:%M:%S") + 
                         " [" + event_type + "] " + filename + "\n"
        
        # ログファイルに追記（Filesystem API）
        filesystem.appendFile("./file_monitor.log", log_entry)
    }
}
```

## 📊 MIR同時拡張による最適化効果

### 🚀 最適化前後の比較

#### **従来の実装（最適化なし）**
```mir
; 非効率：毎回関数呼び出し
%1 = ExternCall env.canvas.fillRect ["canvas", 10, 10, 100, 100, "red"]
%2 = ExternCall env.canvas.fillRect ["canvas", 110, 10, 100, 100, "blue"]  
%3 = ExternCall env.canvas.fillRect ["canvas", 220, 10, 100, 100, "green"]
```

#### **MIR最適化後（バッチ処理）**
```mir
; 効率化：バッチ処理
%rects = ArrayConstruct [
    {x: 10, y: 10, w: 100, h: 100, color: "red"},
    {x: 110, y: 10, w: 100, h: 100, color: "blue"},
    {x: 220, y: 10, w: 100, h: 100, color: "green"}
]
%1 = ExternCall env.canvas.fillRectBatch ["canvas", %rects]
```

#### **Effect Systemによる並列化**
```mir
; pure関数は並列実行可能
%1 = BuiltinCall string.upper ["hello"]    ; effect: pure
%2 = BuiltinCall math.sin [3.14]           ; effect: pure  
%3 = BuiltinCall string.lower ["WORLD"]    ; effect: pure
; ↑ これらは並列実行される

%4 = ExternCall env.console.log [%1]       ; effect: io
%5 = ExternCall env.console.log [%2]       ; effect: io
; ↑ これらは順序保持される
```

### 🎯 バックエンド別最適化

#### **WASM最適化**
```wasm
;; BIDから自動生成された最適化WASM
(func $optimized_canvas_batch
  (param $canvas_id i32) (param $canvas_id_len i32)
  (param $rects_ptr i32) (param $rect_count i32)
  
  ;; ループ展開による高速化
  (local $i i32)
  (local $rect_ptr i32)
  
  loop $rect_loop
    ;; 直接メモリアクセス（境界チェック済み）
    local.get $rect_ptr
    i32.load  ;; x
    local.get $rect_ptr
    i32.load offset=4  ;; y
    ;; ... 高速描画処理
    
    local.get $rect_ptr
    i32.const 20
    i32.add
    local.set $rect_ptr
    
    local.get $i
    i32.const 1
    i32.add
    local.tee $i
    local.get $rect_count
    i32.lt_u
    br_if $rect_loop
  end
)
```

#### **AOT最適化（LLVM IR）**
```llvm
; LLVM IRレベルでの最適化
define void @optimized_image_processing(i8* %image_data, i32 %width, i32 %height) {
entry:
  ; ベクトル化された画像処理
  %0 = bitcast i8* %image_data to <16 x i8>*
  
  ; SIMD命令による並列処理
  br label %loop.header

loop.header:
  %i = phi i32 [ 0, %entry ], [ %i.next, %loop.body ]
  %cmp = icmp ult i32 %i, %height
  br i1 %cmp, label %loop.body, label %exit

loop.body:
  ; 16ピクセル同時処理（AVX2/NEON活用）
  %pixel_ptr = getelementptr <16 x i8>, <16 x i8>* %0, i32 %i
  %pixels = load <16 x i8>, <16 x i8>* %pixel_ptr
  
  ; ベクトル化されたエッジ検出
  %edges = call <16 x i8> @vectorized_edge_detection(<16 x i8> %pixels)
  
  store <16 x i8> %edges, <16 x i8>* %pixel_ptr
  
  %i.next = add i32 %i, 1
  br label %loop.header

exit:
  ret void
}
```

## 🌟 革命的効果

### 🚀 開発者体験の向上
- **学習コスト**: 一つの構文ですべてのAPIが使える
- **IDE統合**: 全APIの統一補完・エラー検出
- **デバッグ**: 統一エラーモデルによる一貫したデバッグ体験

### ⚡ パフォーマンス向上
- **MIRレベル最適化**: すべてのAPIで同じ最適化技術
- **Effect System**: 安全な並列化・順序最適化
- **バックエンド最適化**: WASM/AOT固有の最適化

### 🌍 エコシステム拡大
- **ライブラリ統合**: 既存C/Rustライブラリの簡単統合
- **クロスプラットフォーム**: 同じコードが全環境で動作
- **標準化**: BIDによる外部API標準化

---

**🎉 これが「なんでもAPI計画」の真の実力だにゃ！あらゆる開発が統一された美しい構文で実現できるにゃ！🚀🐱**