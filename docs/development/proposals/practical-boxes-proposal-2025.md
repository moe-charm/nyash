# 実用ライブラリBox提案 2025

**作成日**: 2025-10-12
**対象**: Hakorune言語の実用性向上
**調査範囲**: Python標準ライブラリ、npm人気パッケージ、Rust crates、Go標準ライブラリ、Ruby gems

---

## 調査結果サマリ

### 他言語で最も使われている機能（優先度順）

1. **HTTP/Web**: 圧倒的に需要が高い（Express, Axios, net/http, reqwest）
2. **データ変換**: JSON以外のフォーマット対応（YAML, CSV, XML）
3. **非同期処理**: Promise/Future系（FutureBoxは既存）
4. **テスト支援**: モック、アサーション、フィクスチャ生成
5. **CLI開発**: 引数パース、カラー出力、プログレスバー
6. **データベース**: ORM、クエリビルダー
7. **セキュリティ**: パスワードハッシュ、JWT、暗号化
8. **バリデーション**: 入力検証、型チェック
9. **日時処理**: TimeBoxの拡張（タイムゾーン、パース、計算）
10. **ロギング**: 構造化ログ、ログレベル

---

## Box提案（優先度High: 1-5）

## Box 1: HTTPClientBox

### 用途
HTTPリクエストの送信と受信。Web API呼び出し、Webスクレイピング、マイクロサービス間通信。

### 他言語での相当物
- **JavaScript**: axios, node-fetch
- **Python**: requests, urllib
- **Rust**: reqwest (38M+ downloads)
- **Go**: net/http (標準ライブラリ)

### 提供メソッド
```hakorune
box HTTPClientBox {
    birth() {
        // デフォルト設定でクライアント初期化
    }

    get(url, headers) {
        // GETリクエスト
        // 戻り値: HTTPResponseBox
    }

    post(url, body, headers) {
        // POSTリクエスト
    }

    put(url, body, headers) {
        // PUTリクエスト
    }

    delete(url, headers) {
        // DELETEリクエスト
    }

    set_timeout(ms) {
        // タイムアウト設定
    }

    set_header(key, value) {
        // デフォルトヘッダー設定
    }
}

box HTTPResponseBox {
    status() { }      // ステータスコード取得 (200, 404等)
    body() { }        // レスポンスボディ取得 (StringBox)
    json() { }        // JSONとしてパース (MapBox/ArrayBox)
    headers() { }     // ヘッダー取得 (MapBox)
    ok() { }          // 成功判定 (200番台 = true)
}
```

### 使用例
```hakorune
using std.http as HTTPClientBox

static box Main {
    main() {
        local client = new HTTPClientBox()
        client.set_timeout(5000)  // 5秒タイムアウト

        local response = client.get("https://api.example.com/users", null)

        if response.ok() {
            local data = response.json()
            return data
        }

        return null
    }
}
```

### Everything is Box との整合性
- HTTPClientBox: クライアント設定と状態を保持
- HTTPResponseBox: レスポンスデータとメタデータをカプセル化
- 各メソッドはBoxを返す（ResultBoxと組み合わせ可）

### 実装方針
Rust plugin実装：
- `reqwest`クレート使用（非同期対応）
- FutureBoxと統合（nowait/await対応）
- TLS/SSL自動対応
- リダイレクト自動追跡

### 実装難易度
**Medium** - reqwestのラッパー、FutureBox統合、エラー処理

### 優先度
**High** - 現代のアプリケーション開発で必須。Web API連携、マイクロサービス、外部サービス統合すべてに必要。

---

## Box 2: YAMLBox

### 用途
YAML形式データの読み書き。設定ファイル（Kubernetes、Docker Compose、CI/CD）、データ交換。

### 他言語での相当物
- **Python**: PyYAML（標準的な設定フォーマット）
- **JavaScript**: js-yaml (npm人気パッケージ)
- **Rust**: serde_yaml
- **Ruby**: YAML（標準ライブラリ）

### 提供メソッド
```hakorune
box YAMLBox {
    birth() { }

    parse(yaml_string) {
        // YAML文字列をパース
        // 戻り値: MapBox または ArrayBox
    }

    stringify(data, indent) {
        // BoxデータをYAML文字列に変換
        // 戻り値: StringBox
    }

    parse_file(filepath) {
        // YAMLファイルを読み込んでパース
    }

    write_file(filepath, data) {
        // データをYAMLファイルに書き込み
    }
}
```

### 使用例
```hakorune
using std.yaml as YAMLBox

static box ConfigLoader {
    load_config() {
        local yaml = new YAMLBox()
        local config = yaml.parse_file("config.yaml")

        // config.yaml:
        // database:
        //   host: localhost
        //   port: 5432

        local db_config = config.get("database")
        local host = db_config.get("host")  // "localhost"

        return config
    }
}
```

### Everything is Box との整合性
- YAMLBox: パーサー/シリアライザーの状態管理
- MapBox/ArrayBoxへ自然に変換
- FileBoxとの連携

### 実装方針
Rust plugin実装：
- `serde_yaml`クレート使用
- MapBox/ArrayBox/StringBox/IntegerBox/BoolBoxへ相互変換
- エラーハンドリング（ResultBox統合）

### 実装難易度
**Easy** - serdeエコシステム活用、型変換は既存パターン

### 優先度
**High** - JSONBoxがあるなら、YAML対応も必須。Dockerコンテナ、K8s設定、CI/CD定義で広く使用。

---

## Box 3: CSVBox

### 用途
CSV形式データの読み書き。データインポート/エクスポート、表計算データ処理、ログファイル解析。

### 他言語での相当物
- **Python**: csv（標準ライブラリ）、pandas
- **JavaScript**: papaparse, csv-parser
- **Rust**: csv (人気クレート)
- **Go**: encoding/csv (標準ライブラリ)

### 提供メソッド
```hakorune
box CSVBox {
    birth() { }

    parse(csv_string, has_header) {
        // CSV文字列をパース
        // 戻り値: ArrayBox（各行がMapBoxまたはArrayBox）
    }

    parse_file(filepath, has_header) {
        // CSVファイルを読み込んでパース
    }

    write(data, headers) {
        // データをCSV文字列に変換
        // 戻り値: StringBox
    }

    write_file(filepath, data, headers) {
        // データをCSVファイルに書き込み
    }

    set_delimiter(char) {
        // 区切り文字設定（デフォルト: ","）
    }
}
```

### 使用例
```hakorune
using std.csv as CSVBox

static box DataProcessor {
    process_sales_data() {
        local csv = new CSVBox()
        local data = csv.parse_file("sales.csv", true)

        // data = [
        //   {"name": "Alice", "amount": "100"},
        //   {"name": "Bob", "amount": "200"}
        // ]

        local total = 0
        local i = 0
        loop(i < data.length()) {
            local row = data.get(i)
            total = total + row.get("amount").to_int()
            i = i + 1
        }

        return total
    }
}
```

### Everything is Box との整合性
- CSVBox: パーサー設定と状態管理
- ArrayBox/MapBoxへ自然に変換
- FileBoxとの連携

### 実装方針
Rust plugin実装：
- `csv`クレート使用
- ヘッダー有無の自動判定オプション
- UTF-8/Shift-JIS等エンコーディング対応

### 実装難易度
**Easy** - csvクレートは安定、型変換は既存パターン

### 優先度
**High** - データ処理アプリケーションで必須。Excel連携、ログ解析、データマイグレーション。

---

## Box 4: LoggerBox

### 用途
構造化ロギング。デバッグ、本番監視、エラー追跡。

### 他言語での相当物
- **JavaScript**: winston, pino
- **Python**: logging（標準ライブラリ）
- **Rust**: tracing, log
- **Go**: logrus, zap

### 提供メソッド
```hakorune
box LoggerBox {
    birth(name) {
        // ロガー名で初期化
    }

    debug(message, context) {
        // DEBUGレベルログ出力
    }

    info(message, context) {
        // INFOレベルログ出力
    }

    warn(message, context) {
        // WARNレベルログ出力
    }

    error(message, context) {
        // ERRORレベルログ出力
    }

    fatal(message, context) {
        // FATALレベルログ出力（プログラム終了）
    }

    set_level(level) {
        // ログレベル設定 ("debug", "info", "warn", "error")
    }

    set_output(target) {
        // 出力先設定 ("console", "file", FileBox)
    }

    add_field(key, value) {
        // すべてのログに追加するフィールド設定
    }
}
```

### 使用例
```hakorune
using std.logging as LoggerBox

static box UserService {
    logger: LoggerBox

    birth() {
        me.logger = new LoggerBox("UserService")
        me.logger.set_level("info")
        me.logger.add_field("version", "1.0.0")
    }

    create_user(name, email) {
        me.logger.info("Creating user", {
            "name": name,
            "email": email
        })

        // ユーザー作成処理...

        if error {
            me.logger.error("Failed to create user", {
                "name": name,
                "error": error.message()
            })
            return null
        }

        me.logger.info("User created successfully", {
            "user_id": user.id
        })

        return user
    }
}
```

### Everything is Box との整合性
- LoggerBox: ロガー設定と状態保持
- MapBoxでコンテキスト情報を渡す
- FileBoxとの連携

### 実装方針
Rust plugin実装：
- `tracing`/`tracing-subscriber`使用
- JSON構造化ログ対応
- タイムスタンプ自動付与
- マルチ出力先対応（stdout, stderr, file）

### 実装難易度
**Medium** - tracing統合、JSON出力、ファイルローテーション

### 優先度
**High** - 本番運用で必須。ConsoleBoxのlog()では不十分（レベル制御なし、構造化なし）。

---

## Box 5: ValidationBox

### 用途
入力値検証。Webフォーム、API入力、設定ファイル検証。

### 他言語での相当物
- **JavaScript**: joi, yup, validator.js
- **Python**: marshmallow, pydantic
- **Rust**: validator
- **Ruby**: ActiveModel::Validations

### 提供メソッド
```hakorune
box ValidationBox {
    birth() { }

    string() {
        // 文字列バリデーションビルダー作成
        // 戻り値: StringValidatorBox
    }

    number() {
        // 数値バリデーションビルダー作成
        // 戻り値: NumberValidatorBox
    }

    email() {
        // メールアドレスバリデーター作成
    }

    url() {
        // URLバリデーター作成
    }

    object(schema) {
        // オブジェクトスキーマバリデーター作成
    }

    array(item_validator) {
        // 配列バリデーター作成
    }
}

box StringValidatorBox {
    min(length) { }       // 最小文字数
    max(length) { }       // 最大文字数
    pattern(regex) { }    // 正規表現パターン
    required() { }        // 必須
    validate(value) { }   // 検証実行（ResultBox<BoolBox, ErrorBox>）
}

box NumberValidatorBox {
    min(value) { }        // 最小値
    max(value) { }        // 最大値
    integer() { }         // 整数のみ
    positive() { }        // 正の数のみ
    required() { }        // 必須
    validate(value) { }   // 検証実行
}
```

### 使用例
```hakorune
using std.validation as ValidationBox

static box UserRegistration {
    validate_input(data) {
        local validator = new ValidationBox()

        // ユーザー名検証
        local name_validator = validator.string()
            .min(3)
            .max(20)
            .required()

        local name_result = name_validator.validate(data.get("name"))
        if name_result.is_error() {
            return name_result  // エラー返却
        }

        // メール検証
        local email_validator = validator.email().required()
        local email_result = email_validator.validate(data.get("email"))
        if email_result.is_error() {
            return email_result
        }

        // 年齢検証
        local age_validator = validator.number()
            .min(18)
            .max(120)
            .integer()

        local age_result = age_validator.validate(data.get("age"))
        if age_result.is_error() {
            return age_result
        }

        return ResultBox.ok(true)
    }
}
```

### Everything is Box との整合性
- ValidationBox: バリデーター生成ファクトリ
- 各ValidatorBox: ルール設定と検証実行
- ResultBoxとの自然な統合
- メソッドチェーンでルール組み立て

### 実装方針
Rust plugin実装：
- `validator`クレート使用
- ビルダーパターンで設定
- ResultBox統合（成功/失敗）
- エラーメッセージ多言語対応

### 実装難易度
**Medium** - ビルダーパターン実装、多様なルール、ResultBox統合

### 優先度
**High** - Web開発で必須。ユーザー入力、API入力、設定ファイル検証すべてに必要。

---

## Box提案（優先度Medium: 6-10）

## Box 6: PasswordBox

### 用途
パスワードハッシュ化、検証。セキュアなユーザー認証。

### 他言語での相当物
- **JavaScript**: bcrypt (75M+ npm downloads)
- **Python**: bcrypt, passlib
- **Rust**: bcrypt, argon2
- **Ruby**: bcrypt-ruby

### 提供メソッド
```hakorune
box PasswordBox {
    birth() { }

    hash(password, cost) {
        // パスワードをハッシュ化
        // cost: 計算コスト（デフォルト12）
        // 戻り値: StringBox（ハッシュ文字列）
    }

    verify(password, hash) {
        // パスワード検証
        // 戻り値: BoolBox
    }

    strength(password) {
        // パスワード強度チェック
        // 戻り値: IntegerBox (0-4: weak to strong)
    }
}
```

### 使用例
```hakorune
using std.security.password as PasswordBox

static box AuthService {
    register_user(username, password) {
        local pw = new PasswordBox()

        // 強度チェック
        if pw.strength(password) < 3 {
            return ResultBox.error("Password too weak")
        }

        // ハッシュ化
        local hash = pw.hash(password, 12)

        // DBに保存...
        return ResultBox.ok(user)
    }

    login(username, password) {
        // DBからハッシュ取得...
        local stored_hash = db.get_password_hash(username)

        local pw = new PasswordBox()
        if pw.verify(password, stored_hash) {
            return ResultBox.ok(user)
        }

        return ResultBox.error("Invalid credentials")
    }
}
```

### Everything is Box との整合性
- PasswordBox: ハッシュアルゴリズム設定と状態管理
- 返り値はすべてBox（String/Bool/Integer）
- ResultBoxと自然に統合

### 実装方針
Rust plugin実装：
- `bcrypt`クレート使用
- argon2対応も検討
- タイミング攻撃対策
- 定数時間比較

### 実装難易度
**Easy** - bcryptクレートのラッパー

### 優先度
**Medium-High** - 認証機能を持つアプリケーションで必須。セキュリティ基盤。

---

## Box 7: JWTBox

### 用途
JWT（JSON Web Token）の生成・検証。認証トークン、APIトークン。

### 他言語での相当物
- **JavaScript**: jsonwebtoken (npm人気パッケージ)
- **Python**: PyJWT
- **Rust**: jsonwebtoken
- **Go**: jwt-go

### 提供メソッド
```hakorune
box JWTBox {
    birth(secret) {
        // 秘密鍵で初期化
    }

    encode(payload, expires_in) {
        // ペイロードをJWTに変換
        // expires_in: 有効期限（秒）
        // 戻り値: StringBox（JWT文字列）
    }

    decode(token) {
        // JWTをデコード＆検証
        // 戻り値: ResultBox<MapBox, ErrorBox>
    }

    verify(token) {
        // JWT検証のみ
        // 戻り値: BoolBox
    }

    set_algorithm(algo) {
        // アルゴリズム設定（HS256, RS256等）
    }
}
```

### 使用例
```hakorune
using std.security.jwt as JWTBox

static box TokenService {
    jwt: JWTBox

    birth() {
        me.jwt = new JWTBox("my-secret-key")
        me.jwt.set_algorithm("HS256")
    }

    create_token(user_id) {
        local payload = new MapBox()
        payload.set("user_id", user_id)
        payload.set("role", "admin")

        // 1時間有効
        local token = me.jwt.encode(payload, 3600)
        return token
    }

    validate_token(token) {
        local result = me.jwt.decode(token)

        if result.is_ok() {
            local payload = result.unwrap()
            return payload.get("user_id")
        }

        return null
    }
}
```

### Everything is Box との整合性
- JWTBox: 秘密鍵とアルゴリズム設定保持
- MapBoxでペイロード表現
- ResultBoxで検証結果表現

### 実装方針
Rust plugin実装：
- `jsonwebtoken`クレート使用
- HS256/RS256/ES256対応
- 有効期限自動検証
- クレーム検証機能

### 実装難易度
**Easy** - jsonwebtokenクレートのラッパー

### 優先度
**Medium** - Web API認証で広く使用。マイクロサービス間認証。

---

## Box 8: CLIBox

### 用途
コマンドライン引数パース、カラー出力、プログレスバー。CLIツール開発。

### 他言語での相当物
- **Rust**: clap (75M+ downloads)
- **Python**: argparse（標準ライブラリ）、click
- **JavaScript**: commander, yargs
- **Go**: cobra, flag

### 提供メソッド
```hakorune
box CLIBox {
    birth(app_name, version) { }

    arg(name, description) {
        // 位置引数追加
    }

    flag(name, short, description, default_value) {
        // フラグ追加（--flag, -f）
    }

    option(name, short, description, required) {
        // オプション追加（--option value）
    }

    parse(args) {
        // 引数パース
        // 戻り値: MapBox（引数名→値）
    }

    help() {
        // ヘルプメッセージ生成
    }

    version() {
        // バージョン情報表示
    }
}

box ColorBox {
    birth() { }

    red(text) { }      // 赤文字
    green(text) { }    // 緑文字
    yellow(text) { }   // 黄文字
    blue(text) { }     // 青文字
    bold(text) { }     // 太字
    reset(text) { }    // リセット
}

box ProgressBox {
    birth(total) { }

    start() { }        // 開始
    advance(n) { }     // 進捗更新
    finish() { }       // 完了
    set_message(msg) { } // メッセージ設定
}
```

### 使用例
```hakorune
using std.cli as CLIBox
using std.cli.color as ColorBox
using std.cli.progress as ProgressBox

static box Main {
    main() {
        local cli = new CLIBox("mytool", "1.0.0")
        cli.arg("input", "Input file path")
        cli.flag("verbose", "v", "Verbose output", false)
        cli.option("output", "o", "Output file path", false)

        local args = cli.parse(SystemBox.args())

        local color = new ColorBox()
        local console = new ConsoleBox()

        console.log(color.green("Starting process..."))

        local progress = new ProgressBox(100)
        progress.start()

        // 処理...
        local i = 0
        loop(i < 100) {
            // 何か処理
            progress.advance(1)
            i = i + 1
        }

        progress.finish()
        console.log(color.green("Done!"))

        return 0
    }
}
```

### Everything is Box との整合性
- CLIBox: 引数定義と設定管理
- ColorBox: 色付け処理
- ProgressBox: プログレスバー状態管理
- MapBoxで引数を返す

### 実装方針
Rust plugin実装：
- `clap`クレート使用（引数パース）
- `colored`クレート使用（カラー出力）
- `indicatif`クレート使用（プログレスバー）

### 実装難易度
**Medium** - 3つのクレート統合、API設計

### 優先度
**Medium** - CLIツール開発に必須。開発者体験向上。

---

## Box 9: TemplateBox

### 用途
テンプレートエンジン。HTML生成、メール本文生成、コード生成。

### 他言語での相当物
- **JavaScript**: handlebars, ejs, mustache
- **Python**: jinja2
- **Rust**: tera, handlebars-rust
- **Ruby**: ERB（標準ライブラリ）

### 提供メソッド
```hakorune
box TemplateBox {
    birth() { }

    compile(template_string) {
        // テンプレートコンパイル
        // 戻り値: CompiledTemplateBox
    }

    compile_file(filepath) {
        // テンプレートファイルをコンパイル
    }

    render(template, context) {
        // テンプレート展開
        // 戻り値: StringBox
    }
}

box CompiledTemplateBox {
    render(context) {
        // コンテキスト渡して展開
        // 戻り値: StringBox
    }
}
```

### 使用例
```hakorune
using std.template as TemplateBox

static box EmailService {
    send_welcome_email(user) {
        local tmpl = new TemplateBox()

        local template = "Hello {{name}},\n\n" +
                        "Welcome to {{app_name}}!\n\n" +
                        "Your email is: {{email}}\n"

        local compiled = tmpl.compile(template)

        local context = new MapBox()
        context.set("name", user.name)
        context.set("app_name", "Hakorune App")
        context.set("email", user.email)

        local body = compiled.render(context)

        // メール送信...
        return body
    }
}
```

### Everything is Box との整合性
- TemplateBox: テンプレートエンジン設定
- CompiledTemplateBox: コンパイル済みテンプレート保持
- MapBoxでコンテキスト渡し

### 実装方針
Rust plugin実装：
- `tera`クレート使用
- Handlebars/Mustache構文対応
- 条件分岐・ループ・フィルタ対応
- ファイルキャッシュ機能

### 実装難易度
**Easy** - teraクレートのラッパー

### 優先度
**Medium** - Web開発、メール送信、レポート生成で有用。

---

## Box 10: DatabaseBox

### 用途
データベース接続・クエリ実行。PostgreSQL, MySQL, SQLite対応。

### 他言語での相当物
- **Rust**: sqlx (38M+ downloads)
- **JavaScript**: sequelize, prisma, typeorm
- **Python**: SQLAlchemy, Django ORM
- **Ruby**: ActiveRecord

### 提供メソッド
```hakorune
box DatabaseBox {
    birth(connection_string) {
        // データベース接続
        // 例: "postgres://user:pass@localhost/dbname"
    }

    query(sql, params) {
        // SQLクエリ実行
        // 戻り値: ResultBox<ArrayBox, ErrorBox>
    }

    execute(sql, params) {
        // INSERT/UPDATE/DELETE実行
        // 戻り値: ResultBox<IntegerBox, ErrorBox> (affected rows)
    }

    transaction(callback) {
        // トランザクション実行
    }

    close() {
        // 接続終了
    }
}
```

### 使用例
```hakorune
using std.database as DatabaseBox

static box UserRepository {
    db: DatabaseBox

    birth() {
        me.db = new DatabaseBox("postgres://localhost/myapp")
    }

    find_user_by_email(email) {
        local sql = "SELECT * FROM users WHERE email = ?"
        local result = me.db.query(sql, [email])

        if result.is_ok() {
            local rows = result.unwrap()
            if rows.length() > 0 {
                return rows.get(0)
            }
        }

        return null
    }

    create_user(name, email) {
        local sql = "INSERT INTO users (name, email) VALUES (?, ?)"
        local result = me.db.execute(sql, [name, email])

        return result
    }
}
```

### Everything is Box との整合性
- DatabaseBox: 接続プール管理
- ResultBoxでエラー処理
- ArrayBox/MapBoxでクエリ結果表現

### 実装方針
Rust plugin実装：
- `sqlx`クレート使用（非同期対応）
- PostgreSQL/MySQL/SQLite対応
- コネクションプール
- プリペアドステートメント

### 実装難易度
**Hard** - 複数DB対応、コネクションプール、非同期処理

### 優先度
**Medium** - 本格的なアプリケーション開発で必須。現状はFileBoxでSQLite代用可能だがスケールしない。

---

## Box提案（優先度Medium: 11-15）

## Box 11: UUIDBox

### 用途
UUID（Universally Unique Identifier）生成。主キー、セッションID、リクエストID。

### 他言語での相当物
- **JavaScript**: uuid (npm人気パッケージ)
- **Python**: uuid（標準ライブラリ）
- **Rust**: uuid
- **Go**: github.com/google/uuid

### 提供メソッド
```hakorune
box UUIDBox {
    birth() { }

    v4() {
        // ランダムUUID生成（v4）
        // 戻り値: StringBox（例: "550e8400-e29b-41d4-a716-446655440000"）
    }

    v7() {
        // タイムスタンプベースUUID生成（v7、最新）
    }

    parse(uuid_string) {
        // UUID文字列をパース・検証
        // 戻り値: ResultBox<UUIDValueBox, ErrorBox>
    }

    is_valid(uuid_string) {
        // UUID文字列の妥当性チェック
        // 戻り値: BoolBox
    }
}
```

### 使用例
```hakorune
using std.uuid as UUIDBox

static box OrderService {
    create_order(user_id, items) {
        local uuid = new UUIDBox()
        local order_id = uuid.v4()

        // order_id = "550e8400-e29b-41d4-a716-446655440000"

        // 注文作成...
        return order_id
    }
}
```

### Everything is Box との整合性
- UUIDBox: UUID生成ロジック
- StringBoxで返却（既存システムとの互換性）

### 実装方針
Rust plugin実装：
- `uuid`クレート使用
- v4（ランダム）とv7（タイムスタンプ）対応
- パース・検証機能

### 実装難易度
**Easy** - uuidクレートのラッパー

### 優先度
**Medium** - 分散システム、マイクロサービスで必須。一意ID生成。

---

## Box 12: EncodingBox

### 用途
Base64/Hex/URL エンコード・デコード。データ変換、API連携。

### 他言語での相当物
- **JavaScript**: Buffer（Node.js標準）、base64-js
- **Python**: base64（標準ライブラリ）
- **Rust**: base64, hex
- **Go**: encoding/base64（標準ライブラリ）

### 提供メソッド
```hakorune
box EncodingBox {
    birth() { }

    base64_encode(data) {
        // Base64エンコード
        // 戻り値: StringBox
    }

    base64_decode(encoded) {
        // Base64デコード
        // 戻り値: ResultBox<StringBox, ErrorBox>
    }

    hex_encode(data) {
        // 16進数エンコード
    }

    hex_decode(encoded) {
        // 16進数デコード
    }

    url_encode(url) {
        // URLエンコード
    }

    url_decode(encoded_url) {
        // URLデコード
    }
}
```

### 使用例
```hakorune
using std.encoding as EncodingBox

static box APIClient {
    send_binary_data(data) {
        local enc = new EncodingBox()
        local encoded = enc.base64_encode(data)

        // API送信...
        return encoded
    }

    receive_binary_data(response) {
        local enc = new EncodingBox()
        local decoded = enc.base64_decode(response.body())

        if decoded.is_ok() {
            return decoded.unwrap()
        }

        return null
    }
}
```

### Everything is Box との整合性
- EncodingBox: エンコーディングユーティリティ
- ResultBoxでデコードエラー処理

### 実装方針
Rust plugin実装：
- `base64`クレート使用
- `hex`クレート使用
- `urlencoding`クレート使用

### 実装難易度
**Easy** - 複数クレートのラッパー

### 優先度
**Medium** - Web API、データ交換で頻繁に使用。

---

## Box 13: CryptoBox

### 用途
暗号化・復号化、ハッシュ生成。データ保護、改ざん検知。

### 他言語での相当物
- **JavaScript**: crypto（Node.js標準）、crypto-js
- **Python**: hashlib, cryptography
- **Rust**: ring, sha2, aes-gcm
- **Go**: crypto（標準ライブラリ）

### 提供メソッド
```hakorune
box CryptoBox {
    birth() { }

    sha256(data) {
        // SHA256ハッシュ生成
        // 戻り値: StringBox（16進数文字列）
    }

    sha512(data) {
        // SHA512ハッシュ生成
    }

    md5(data) {
        // MD5ハッシュ生成（非推奨だが互換性のため）
    }

    encrypt_aes(data, key, iv) {
        // AES-256-GCM 暗号化
        // 戻り値: StringBox（Base64エンコード）
    }

    decrypt_aes(encrypted, key, iv) {
        // AES-256-GCM 復号化
        // 戻り値: ResultBox<StringBox, ErrorBox>
    }

    random_bytes(length) {
        // 暗号学的に安全な乱数生成
    }
}
```

### 使用例
```hakorune
using std.crypto as CryptoBox

static box SecureStorage {
    save_sensitive_data(data, password) {
        local crypto = new CryptoBox()

        // パスワードからキー生成（簡略化）
        local key = crypto.sha256(password)
        local iv = crypto.random_bytes(16)

        local encrypted = crypto.encrypt_aes(data, key, iv)

        // 保存...
        return encrypted
    }

    load_sensitive_data(encrypted, password) {
        local crypto = new CryptoBox()
        local key = crypto.sha256(password)

        local result = crypto.decrypt_aes(encrypted, key, iv)

        if result.is_ok() {
            return result.unwrap()
        }

        return null
    }
}
```

### Everything is Box との整合性
- CryptoBox: 暗号化ユーティリティ
- ResultBoxで復号化エラー処理
- PasswordBoxと組み合わせ

### 実装方針
Rust plugin実装：
- `ring`クレート使用（高速・安全）
- `sha2`クレート使用（ハッシュ）
- `aes-gcm`クレート使用（暗号化）

### 実装難易度
**Medium** - 暗号化は慎重な実装が必要

### 優先度
**Medium** - セキュリティ重視アプリケーションで必須。

---

## Box 14: TestBox

### 用途
ユニットテスト、アサーション、モック。テスト駆動開発。

### 他言語での相当物
- **JavaScript**: jest, mocha, chai
- **Python**: unittest（標準ライブラリ）、pytest
- **Rust**: built-in test framework
- **Ruby**: rspec, minitest

### 提供メソッド
```hakorune
box TestBox {
    birth(test_name) { }

    assert_equal(actual, expected, message) {
        // 等価性アサーション
    }

    assert_true(value, message) {
        // 真偽値アサーション
    }

    assert_false(value, message) {
        // 偽アサーション
    }

    assert_null(value, message) {
        // null確認
    }

    assert_not_null(value, message) {
        // null以外確認
    }

    assert_throws(callback, message) {
        // 例外発生確認
    }

    run() {
        // テスト実行
        // 戻り値: TestResultBox
    }
}

box MockBox {
    birth() { }

    mock_method(box_instance, method_name, return_value) {
        // メソッドのモック作成
    }

    verify_called(method_name, times) {
        // 呼び出し回数検証
    }
}
```

### 使用例
```hakorune
using std.test as TestBox

static box CalculatorTest {
    test_addition() {
        local test = new TestBox("Calculator addition")

        local calc = new Calculator()
        local result = calc.add(2, 3)

        test.assert_equal(result, 5, "2 + 3 should be 5")
        test.assert_true(result > 0, "Result should be positive")

        return test.run()
    }

    test_division_by_zero() {
        local test = new TestBox("Division by zero")

        local calc = new Calculator()

        // エラーが発生することを期待
        test.assert_throws(lambda() {
            calc.divide(10, 0)
        }, "Should throw error on division by zero")

        return test.run()
    }
}
```

### Everything is Box との整合性
- TestBox: テストケース管理
- MockBox: モックオブジェクト管理
- アサーション失敗時にErrorBoxをthrow

### 実装方針
Hakorune実装（Rust pluginではなく）：
- pure Hakoruneで実装
- apps/lib/test/ に配置
- ConsoleBoxでカラー出力（成功=緑、失敗=赤）

### 実装難易度
**Easy** - Hakoruneで実装可能、既存機能の組み合わせ

### 優先度
**Medium** - 品質保証に必須。現状は手動テストのみ。

---

## Box 15: XMLBox

### 用途
XML形式データの読み書き。SOAP API、RSS/Atom、設定ファイル。

### 他言語での相当物
- **JavaScript**: xml2js, fast-xml-parser
- **Python**: xml.etree.ElementTree（標準ライブラリ）、lxml
- **Rust**: quick-xml, serde-xml-rs
- **Go**: encoding/xml（標準ライブラリ）

### 提供メソッド
```hakorune
box XMLBox {
    birth() { }

    parse(xml_string) {
        // XML文字列をパース
        // 戻り値: XMLNodeBox（ルート要素）
    }

    parse_file(filepath) {
        // XMLファイルを読み込んでパース
    }

    stringify(node, pretty) {
        // XMLNodeBoxをXML文字列に変換
        // 戻り値: StringBox
    }
}

box XMLNodeBox {
    name() {
        // タグ名取得
    }

    text() {
        // テキストコンテンツ取得
    }

    attr(name) {
        // 属性取得
    }

    children() {
        // 子要素取得（ArrayBox）
    }

    find(xpath) {
        // XPath検索
    }
}
```

### 使用例
```hakorune
using std.xml as XMLBox

static box RSSReader {
    read_feed(url) {
        local http = new HTTPClientBox()
        local response = http.get(url, null)

        local xml = new XMLBox()
        local doc = xml.parse(response.body())

        // <rss><channel><item> を探す
        local items = doc.find("//item")

        local result = new ArrayBox()
        local i = 0
        loop(i < items.length()) {
            local item = items.get(i)
            local entry = new MapBox()
            entry.set("title", item.find("title").text())
            entry.set("link", item.find("link").text())
            result.push(entry)
            i = i + 1
        }

        return result
    }
}
```

### Everything is Box との整合性
- XMLBox: パーサー設定
- XMLNodeBox: ツリー構造表現
- MapBox/ArrayBoxへ変換可能

### 実装方針
Rust plugin実装：
- `quick-xml`クレート使用
- XPath対応
- 名前空間対応

### 実装難易度
**Medium** - XML複雑性、XPath実装

### 優先度
**Low-Medium** - JSONが主流だが、レガシーシステム連携で必要。

---

## Box提案（優先度Low: 16-20）

## Box 16: CacheBox

### 用途
インメモリキャッシュ。パフォーマンス向上、外部API呼び出し削減。

### 他言語での相当物
- **JavaScript**: node-cache, lru-cache
- **Python**: functools.lru_cache（標準ライブラリ）
- **Rust**: lru, moka
- **Go**: groupcache, bigcache

### 提供メソッド
```hakorune
box CacheBox {
    birth(max_size, ttl) {
        // max_size: 最大エントリ数
        // ttl: 生存時間（秒）
    }

    set(key, value, ttl_override) {
        // キャッシュ設定
    }

    get(key) {
        // キャッシュ取得
        // 戻り値: Box or null
    }

    has(key) {
        // キャッシュ存在確認
    }

    delete(key) {
        // キャッシュ削除
    }

    clear() {
        // 全削除
    }

    size() {
        // エントリ数取得
    }
}
```

### 使用例
```hakorune
using std.cache as CacheBox

static box UserService {
    cache: CacheBox

    birth() {
        me.cache = new CacheBox(1000, 3600)  // 1000エントリ、1時間TTL
    }

    get_user(user_id) {
        // キャッシュチェック
        if me.cache.has(user_id) {
            return me.cache.get(user_id)
        }

        // DBから取得
        local user = db.find_user(user_id)

        // キャッシュ保存
        me.cache.set(user_id, user, null)

        return user
    }
}
```

### Everything is Box との整合性
- CacheBox: キャッシュストレージ管理
- 任意のBoxを保存可能

### 実装方針
Rust plugin実装：
- `lru`クレート使用
- TTL（Time To Live）対応
- LRU（Least Recently Used）アルゴリズム

### 実装難易度
**Easy** - lruクレートのラッパー

### 優先度
**Low** - パフォーマンス最適化に有用だが、初期段階では優先度低。

---

## Box 17: QueueBox

### 用途
FIFO/LIFOキュー、優先度キュー。タスク管理、メッセージキュー。

### 他言語での相当物
- **JavaScript**: bull, bee-queue
- **Python**: queue（標準ライブラリ）
- **Rust**: crossbeam, tokio channels
- **Go**: channels（言語組み込み）

### 提供メソッド
```hakorune
box QueueBox {
    birth(mode) {
        // mode: "fifo", "lifo", "priority"
    }

    enqueue(item, priority) {
        // キューに追加
    }

    dequeue() {
        // キューから取り出し
        // 戻り値: Box or null
    }

    peek() {
        // 先頭要素確認（削除しない）
    }

    size() {
        // キューサイズ取得
    }

    is_empty() {
        // 空確認
    }

    clear() {
        // 全削除
    }
}
```

### 使用例
```hakorune
using std.queue as QueueBox

static box JobProcessor {
    jobs: QueueBox

    birth() {
        me.jobs = new QueueBox("priority")
    }

    add_job(job, priority) {
        me.jobs.enqueue(job, priority)
    }

    process_jobs() {
        loop(me.jobs.is_empty().not()) {
            local job = me.jobs.dequeue()
            // ジョブ処理...
        }
    }
}
```

### Everything is Box との整合性
- QueueBox: キュー状態管理
- 任意のBoxをキューイング可能

### 実装方針
Rust plugin実装：
- `std::collections::VecDeque`使用（FIFO/LIFO）
- `priority-queue`クレート使用（優先度キュー）

### 実装難易度
**Easy** - 標準ライブラリとクレート組み合わせ

### 優先度
**Low** - バックグラウンドジョブ処理に有用だが、ArrayBoxで代用可能。

---

## Box 18: CompressionBox

### 用途
データ圧縮・展開。ファイル圧縮、ネットワーク転送最適化。

### 他言語での相当物
- **JavaScript**: pako, zlib
- **Python**: gzip, zlib（標準ライブラリ）
- **Rust**: flate2, zstd
- **Go**: compress/gzip（標準ライブラリ）

### 提供メソッド
```hakorune
box CompressionBox {
    birth(algorithm) {
        // algorithm: "gzip", "deflate", "brotli", "zstd"
    }

    compress(data, level) {
        // データ圧縮
        // level: 1-9（圧縮レベル）
        // 戻り値: StringBox（圧縮データ）
    }

    decompress(compressed_data) {
        // データ展開
        // 戻り値: ResultBox<StringBox, ErrorBox>
    }
}
```

### 使用例
```hakorune
using std.compression as CompressionBox

static box LogArchiver {
    archive_logs(log_data) {
        local comp = new CompressionBox("gzip")
        local compressed = comp.compress(log_data, 9)

        // ファイル保存...
        return compressed
    }

    read_logs(compressed_file) {
        local comp = new CompressionBox("gzip")
        local result = comp.decompress(compressed_file)

        if result.is_ok() {
            return result.unwrap()
        }

        return null
    }
}
```

### Everything is Box との整合性
- CompressionBox: 圧縮アルゴリズム設定
- ResultBoxでエラー処理

### 実装方針
Rust plugin実装：
- `flate2`クレート使用（gzip, deflate）
- `brotli`クレート使用（brotli）
- `zstd`クレート使用（zstd）

### 実装難易度
**Easy** - 複数クレートのラッパー

### 優先度
**Low** - ストレージ最適化に有用だが、初期段階では優先度低。

---

## Box 19: MarkdownBox

### 用途
Markdown ⇔ HTML変換。ドキュメント生成、CMS、ブログ。

### 他言語での相当物
- **JavaScript**: marked, markdown-it
- **Python**: markdown, mistune
- **Rust**: pulldown-cmark, comrak
- **Ruby**: redcarpet, kramdown

### 提供メソッド
```hakorune
box MarkdownBox {
    birth() { }

    to_html(markdown_text) {
        // Markdown → HTML変換
        // 戻り値: StringBox（HTML）
    }

    parse(markdown_text) {
        // Markdownをパース（AST取得）
        // 戻り値: MarkdownASTBox
    }

    set_options(opts) {
        // オプション設定
        // - gfm: GitHub Flavored Markdown有効化
        // - tables: テーブル有効化
        // - breaks: 改行を<br>に変換
    }
}
```

### 使用例
```hakorune
using std.markdown as MarkdownBox

static box BlogRenderer {
    render_post(markdown_content) {
        local md = new MarkdownBox()
        md.set_options({
            "gfm": true,
            "tables": true,
            "breaks": true
        })

        local html = md.to_html(markdown_content)

        return html
    }
}
```

### Everything is Box との整合性
- MarkdownBox: パーサー設定
- StringBoxで入出力

### 実装方針
Rust plugin実装：
- `pulldown-cmark`クレート使用
- GitHub Flavored Markdown対応
- テーブル、コードブロック、シンタックスハイライト対応

### 実装難易度
**Easy** - pulldown-cmarkのラッパー

### 優先度
**Low** - ドキュメント生成、CMS開発で有用だが、初期段階では優先度低。

---

## Box 20: GeoBox

### 用途
地理情報処理。距離計算、座標変換、ジオコーディング。

### 他言語での相当物
- **JavaScript**: geolib, turf
- **Python**: geopy, shapely
- **Rust**: geo, geodesy
- **Ruby**: geokit

### 提供メソッド
```hakorune
box GeoBox {
    birth() { }

    distance(lat1, lon1, lat2, lon2) {
        // 2点間距離計算（メートル）
        // 戻り値: FloatBox
    }

    bearing(lat1, lon1, lat2, lon2) {
        // 方位角計算（度）
    }

    destination(lat, lon, distance, bearing) {
        // 指定距離・方位から目的地座標計算
        // 戻り値: MapBox ({"lat": ..., "lon": ...})
    }

    is_inside(lat, lon, polygon) {
        // 点がポリゴン内にあるか判定
    }
}
```

### 使用例
```hakorune
using std.geo as GeoBox

static box LocationService {
    find_nearby_stores(user_lat, user_lon, stores) {
        local geo = new GeoBox()
        local nearby = new ArrayBox()

        local i = 0
        loop(i < stores.length()) {
            local store = stores.get(i)
            local dist = geo.distance(
                user_lat, user_lon,
                store.get("lat"), store.get("lon")
            )

            if dist < 5000 {  // 5km以内
                store.set("distance", dist)
                nearby.push(store)
            }

            i = i + 1
        }

        return nearby
    }
}
```

### Everything is Box との整合性
- GeoBox: 地理情報計算ユーティリティ
- FloatBox/MapBoxで座標表現

### 実装方針
Rust plugin実装：
- `geo`クレート使用
- Haversine公式（球面距離計算）
- ポリゴン内判定アルゴリズム

### 実装難易度
**Medium** - 地理情報計算アルゴリズム

### 優先度
**Low** - 位置情報サービス開発で有用だが、ニッチ。

---

## 実装優先順位まとめ

### Phase 1: 必須Box（優先度High）

1. **HTTPClientBox** - Web API連携、マイクロサービス通信
2. **YAMLBox** - 設定ファイル（K8s, Docker Compose, CI/CD）
3. **CSVBox** - データ処理、Excel連携、ログ解析
4. **LoggerBox** - 本番運用、デバッグ、監視
5. **ValidationBox** - 入力検証、API入力、セキュリティ

**理由**: Web開発・データ処理・本番運用で必須。これらがないと実用的なアプリケーションが作れない。

### Phase 2: セキュリティ・認証Box（優先度Medium-High）

6. **PasswordBox** - ユーザー認証基盤
7. **JWTBox** - API認証、マイクロサービス間認証

**理由**: セキュリティは後回しにできない。認証機能を持つアプリケーションで必須。

### Phase 3: 開発者体験向上Box（優先度Medium）

8. **CLIBox** - CLIツール開発、開発者体験向上
9. **TemplateBox** - HTML生成、メール本文、コード生成
10. **DatabaseBox** - 本格的なアプリケーション開発

**理由**: 開発効率を大幅に向上。CLIツール、Web開発、データベース連携。

### Phase 4: ユーティリティBox（優先度Medium）

11. **UUIDBox** - 一意ID生成
12. **EncodingBox** - Base64/Hex/URL変換
13. **CryptoBox** - 暗号化、ハッシュ
14. **TestBox** - ユニットテスト、品質保証
15. **XMLBox** - レガシーシステム連携

**理由**: 頻繁に使用するユーティリティ。開発スピード向上。

### Phase 5: 最適化・特殊用途Box（優先度Low）

16. **CacheBox** - パフォーマンス最適化
17. **QueueBox** - バックグラウンドジョブ
18. **CompressionBox** - ストレージ最適化
19. **MarkdownBox** - ドキュメント生成、CMS
20. **GeoBox** - 位置情報サービス

**理由**: 特定用途、最適化、ニッチな領域。初期段階では優先度低。

---

## 実装戦略

### すぐに実装可能（1-2日/Box）

- **Easy難易度**: YAMLBox, CSVBox, UUIDBox, EncodingBox, PasswordBox, JWTBox, CacheBox, QueueBox, CompressionBox, MarkdownBox
- **戦略**: Rust cratesのラッパー、既存パターン適用

### 設計が必要（3-5日/Box）

- **Medium難易度**: HTTPClientBox, LoggerBox, ValidationBox, CLIBox, CryptoBox, XMLBox, GeoBox
- **戦略**: API設計、FutureBox統合、エラーハンドリング

### 大規模実装（1-2週間/Box）

- **Hard難易度**: DatabaseBox, TestBox
- **戦略**: 段階的実装、コミュニティフィードバック

### Everything is Box 統一原則

すべてのBoxは以下の原則に従う：

1. **状態管理**: Box自身が設定・状態を保持
2. **メソッドチェーン**: 設定メソッドは`me`を返す
3. **ResultBox統合**: エラーハンドリングはResultBox
4. **FutureBox統合**: 非同期処理はFutureBox + nowait/await
5. **既存Box活用**: MapBox, ArrayBox, StringBox等への自然な変換

---

## 次のステップ

1. **コミュニティフィードバック**: どのBoxが最も必要か？
2. **Phase 1実装開始**: HTTPClientBox → YAMLBox → CSVBox → LoggerBox → ValidationBox
3. **ドキュメント整備**: 各Boxの完全なAPI仕様とサンプルコード
4. **テストスイート**: 各Boxの包括的テスト
5. **プラグインテンプレート**: 新規Box開発のためのテンプレート

---

**調査協力**: Task先生（実用ライブラリの達人）
**調査日**: 2025-10-12
**対象言語**: Python, JavaScript/Node.js, Rust, Go, Ruby
