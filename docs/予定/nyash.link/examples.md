# nyash.linkシステム使用例

## 🎯 基本的な使用例

### 📁 プロジェクト構造例
```
my-awesome-app/
├── nyash.link              # 依存関係定義
├── src/
│   ├── main.nyash         # メインファイル
│   ├── models/
│   │   └── user.nyash     # ユーザーモデル
│   └── utils/
│       └── helpers.nyash   # ヘルパー関数
├── libs/
│   └── custom_lib.nyash    # カスタムライブラリ
└── stdlib/
    └── nyashstd.nyash      # 標準ライブラリ
```

### 📋 nyash.linkファイル例
```toml
[project]
name = "my-awesome-app"
version = "1.0.0"
description = "Everything is Box philosophy in action!"

[dependencies]
# 標準ライブラリ
nyashstd = { path = "./stdlib/nyashstd.nyash" }

# プロジェクト内モジュール  
user_model = { path = "./src/models/user.nyash" }
helpers = { path = "./src/utils/helpers.nyash" }

# カスタムライブラリ
custom_lib = { path = "./libs/custom_lib.nyash" }

[search_paths]
stdlib = "./stdlib/"
src = "./src/"
libs = "./libs/"

[build]
entry_point = "./src/main.nyash"
```

## 🌟 実用的なコード例

### 1. 基本的なusing使用
```nyash
# ===== src/main.nyash =====
using nyashstd
using helpers

static box Main {
    init { console }
    
    main() {
        me.console = new ConsoleBox()
        
        # 標準ライブラリ使用
        local text = "hello world"
        local upper_text = string.upper(text)  # nyashstd.string.upper
        me.console.log("Upper: " + upper_text)
        
        # ヘルパー関数使用  
        local processed = helpers.process_data("sample data")
        me.console.log("Processed: " + processed)
        
        # 数学関数
        local result = math.sin(3.14159)
        me.console.log("Sin: " + result.toString())
    }
}
```

### 2. 標準ライブラリ定義例
```nyash
# ===== stdlib/nyashstd.nyash =====
namespace nyashstd {
    static box string {
        static upper(str) {
            local string_box = new StringBox(str)
            return string_box.upper()
        }
        
        static lower(str) {
            local string_box = new StringBox(str) 
            return string_box.lower()
        }
        
        static split(str, separator) {
            local string_box = new StringBox(str)
            return string_box.split(separator)
        }
        
        static join(array, separator) {
            local sep_box = new StringBox(separator)
            return sep_box.join(array)
        }
    }
    
    static box math {
        static sin(x) {
            local math_box = new MathBox()
            return math_box.sin(x)
        }
        
        static cos(x) {
            local math_box = new MathBox()
            return math_box.cos(x)
        }
        
        static random() {
            local random_box = new RandomBox()
            return random_box.nextFloat()
        }
        
        static floor(x) {
            local math_box = new MathBox()
            return math_box.floor(x)
        }
    }
    
    static box io {
        static read_file(path) {
            local file_box = new FileBox()
            return file_box.read(path)
        }
        
        static write_file(path, content) {
            local file_box = new FileBox()
            return file_box.write(path, content)
        }
    }
}
```

### 3. ヘルパーモジュール例
```nyash
# ===== src/utils/helpers.nyash =====
using nyashstd

static function process_data(data) {
    # データ処理のヘルパー
    local trimmed = string.trim(data)
    local upper = string.upper(trimmed)
    return "PROCESSED: " + upper
}

static function calculate_score(points, multiplier) {
    local result = points * multiplier
    return math.floor(result)
}

static function format_user_name(first, last) {
    return string.upper(first) + " " + string.upper(last)
}
```

### 4. モデル定義例
```nyash
# ===== src/models/user.nyash =====
using nyashstd
using helpers

box User {
    init { name, email, score }
    
    birth(user_name, user_email) {
        me.name = user_name
        me.email = user_email
        me.score = 0
    }
    
    add_points(points) {
        me.score = me.score + points
        return me.score
    }
    
    get_formatted_name() {
        local parts = string.split(me.name, " ")
        if parts.length() >= 2 {
            return helpers.format_user_name(parts.get(0), parts.get(1))
        } else {
            return string.upper(me.name)
        }
    }
    
    save_to_file() {
        local data = "User: " + me.name + ", Email: " + me.email + ", Score: " + me.score.toString()
        local filename = "user_" + string.lower(me.name) + ".txt"
        io.write_file(filename, data)
    }
}
```

## 🎮 実用アプリケーション例

### 1. シンプルなWebサーバー
```nyash
# ===== web_server.nyash =====
using nyashstd
using custom_lib

static box WebServer {
    init { server, port }
    
    birth(server_port) {
        me.port = server_port
        me.server = new HttpServerBox()
    }
    
    start() {
        me.server.bind("localhost", me.port)
        
        me.server.on("request", me.handle_request)
        
        local console = new ConsoleBox()
        console.log("Server started on port " + me.port.toString())
        
        me.server.listen()
    }
    
    handle_request(request, response) {
        local url = request.getUrl()
        
        if url == "/" {
            local html = io.read_file("./public/index.html")
            response.setStatus(200)
            response.setHeader("Content-Type", "text/html")
            response.send(html)
        } else {
            response.setStatus(404)
            response.send("Not Found")
        }
    }
}

# メイン実行
local server = new WebServer(3000)
server.start()
```

### 2. データ処理パイプライン
```nyash
# ===== data_processor.nyash =====
using nyashstd
using helpers

static box DataProcessor {
    init { input_file, output_file }
    
    birth(input_path, output_path) {
        me.input_file = input_path
        me.output_file = output_path
    }
    
    process() {
        # データ読み込み
        local raw_data = io.read_file(me.input_file)
        local lines = string.split(raw_data, "\n")
        
        # 処理済みデータ配列
        local processed_lines = new ArrayBox()
        
        # 各行を処理
        local i = 0
        loop(i < lines.length()) {
            local line = lines.get(i)
            local processed = helpers.process_data(line)
            processed_lines.push(processed)
            i = i + 1
        }
        
        # 結果をファイルに保存
        local result = string.join(processed_lines, "\n")
        io.write_file(me.output_file, result)
        
        return processed_lines.length()
    }
}

# メイン処理
local processor = new DataProcessor("input.txt", "output.txt")
local count = processor.process()

local console = new ConsoleBox()
console.log("Processed " + count.toString() + " lines")
```

## 🔧 高度な使用パターン

### 1. 条件付きモジュール読み込み（将来拡張）
```nyash
# 開発環境では詳細ログ、本番環境ではシンプルログ
using nyashstd

static function get_logger() {
    local env = os.get_env("NYASH_ENV")
    
    if env == "development" {
        using dev_logger
        return new dev_logger.DetailLogger()
    } else {
        using prod_logger  
        return new prod_logger.SimpleLogger()
    }
}
```

### 2. エイリアス使用例（将来拡張）
```nyash
# 長い名前空間のエイリアス
using very.long.namespace.name as short

local result = short.helper_function("data")

# 複数の類似ライブラリ
using json_v1 as json1
using json_v2 as json2

local data1 = json1.parse(input)
local data2 = json2.parse(input)
```

### 3. 部分インポート（将来拡張）
```nyash
# 名前空間全体ではなく特定機能のみ
using nyashstd.string
using nyashstd.math

# これで直接呼び出せる
local result = upper("hello")  # string.upper不要
local sin_val = sin(3.14)     # math.sin不要
```

## 📊 移行例：既存includeからusingへ

### Before（現在のinclude使用）
```nyash
# ===== 既存のtext_adventure例 =====
include "text_adventure/items.nyash"
include "text_adventure/rooms.nyash"

# アイテム作成
local sword = new Weapon("Sword", 10)
```

### After（新しいusing使用）
```nyash
# ===== nyash.link =====
[dependencies]
game_items = { path = "./text_adventure/items.nyash" }
game_rooms = { path = "./text_adventure/rooms.nyash" }

# ===== main.nyash =====
using game_items
using game_rooms

# アイテム作成（名前空間経由）
local sword = game_items.create_weapon("Sword", 10)
```

## 🎉 期待される開発体験

### IDE補完の改善
```nyash
using nyashstd

# "st" と入力すると...
st → string (補完候補)

# "string." と入力すると...
string. → upper, lower, split, join, trim, ... (全メソッド表示)
```

### エラーメッセージの改善
```nyash
using nyashstd

# 間違った呼び出し
local result = string.uppper("hello")  # typo

# エラー:
# Error: Method 'uppper' not found in nyashstd.string
# Did you mean: 'upper'?
# Available methods: upper, lower, split, join, trim
```

---

**🌟 これらの例でnyash.linkシステムの実用性と美しさが伝わるにゃ！🐱**