# Nyash Macro Examples - 世界最強マクロ言語への具体例

**更新日**: 2025-09-18
**ステータス**: 設計完了、実装待ち

## 🎯 マクロ分類と優先度

| 優先度 | マクロ | 実用性 | 実装コスト | 特徴 |
|--------|--------|--------|------------|------|
| 🥇 **MVP** | @derive | ⭐⭐⭐⭐ | ⭐⭐ | ボイラープレート除去 |
| 🥈 **早期** | @validate | ⭐⭐⭐⭐ | ⭐⭐⭐ | 型安全・入力品質 |
| 🥈 **早期** | @config_schema | ⭐⭐⭐⭐ | ⭐⭐ | 実アプリ即効 |
| 🥉 **段階** | @api_client | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | プロダクション |
| 🥉 **段階** | @sql_schema | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | 企業利用 |
| 🏅 **実験** | @html_dsl | ⭐⭐⭐ | ⭐⭐⭐ | 表現力・デモ |

## 🔥 1. @derive系マクロ（MVP最優先）

### 基本例
```nyash
@derive(Equals, ToString, Clone, Json)
box UserBox {
    name: StringBox
    age: IntegerBox
    email: StringBox
}
```

### 自動生成されるメソッド
```nyash
// @derive(Equals) → equals method
method equals(other: UserBox) -> BoolBox {
    return me.name == other.name && 
           me.age == other.age && 
           me.email == other.email
}

// @derive(ToString) → toString method  
method toString() -> StringBox {
    return "UserBox(name=" + me.name + 
           ", age=" + me.age + 
           ", email=" + me.email + ")"
}

// @derive(Clone) → clone method
method clone() -> UserBox {
    return new UserBox(me.name, me.age, me.email)
}

// @derive(Json) → toJson/fromJson methods
method toJson() -> JsonBox {
    return JsonBox.object([
        ["name", me.name],
        ["age", me.age], 
        ["email", me.email]
    ])
}
```

### Property System統合
```nyash
@derive(Equals, ToString)
box AdvancedBox {
    // stored fields
    name: StringBox
    
    // computed fields も自動でderiveに含まれる！
    display_name: StringBox { me.name.toUpperCase() }
    
    // once fields も含まれる
    once uuid: StringBox { generateUUID() }
}

// 生成されるequalsはcomputed/onceも含む
method equals(other: AdvancedBox) -> BoolBox {
    return me.name == other.name &&
           me.display_name == other.display_name &&
           me.uuid == other.uuid  // onceプロパティも比較
}
```

## 🛡️ 2. @validate系マクロ（型安全革命）

### 基本バリデーション
```nyash
@validate
box UserBox {
    @required @email
    email: StringBox
    
    @range(0, 150) @required
    age: IntegerBox
    
    @min_length(8) @optional
    password: StringBox
    
    @pattern("^[a-zA-Z]+$") @required
    name: StringBox
}
```

### 自動生成されるバリデーション
```nyash
// setter methods with validation
method set_email(value: StringBox) {
    if value.length() == 0 {
        throw new ValidationError("email is required")
    }
    if !value.contains("@") {
        throw new ValidationError("invalid email format")
    }
    me.email = value
}

method set_age(value: IntegerBox) {
    if value < 0 || value > 150 {
        throw new ValidationError("age must be between 0 and 150")
    }
    me.age = value
}

method set_name(value: StringBox) {
    if value.length() == 0 {
        throw new ValidationError("name is required")
    }
    if !value.matches("^[a-zA-Z]+$") {
        throw new ValidationError("name must contain only letters")
    }
    me.name = value
}

// bulk validation method
method validate() -> Result<BoolBox, ValidationErrorBox> {
    try {
        me.validate_email()
        me.validate_age()  
        me.validate_name()
        return Ok(true)
    } catch(ValidationError e) {
        return Err(e)
    }
}
```

### Property System統合
```nyash
@validate
box ConfigBox {
    @required @env("DATABASE_URL")
    database_url: StringBox
    
    // computed property でもバリデーション適用
    @range(1, 100)
    max_connections: IntegerBox { me.calculate_connections() }
    
    // validation はcomputed propertyの計算時に実行
}
```

## ⚙️ 3. @config_schema系マクロ（実アプリ即効）

### 環境変数ベース設定
```nyash
@config_schema
box AppConfigBox {
    @env("DATABASE_URL") @required
    database_url: StringBox
    
    @env("REDIS_URL") @default("redis://localhost:6379")
    redis_url: StringBox
    
    @env("DEBUG") @default(false) @parse_bool
    debug_mode: BoolBox
    
    @env("MAX_CONNECTIONS") @default(100) @range(1, 1000) @parse_int
    max_connections: IntegerBox
    
    @env("LOG_LEVEL") @default("INFO") @enum(["DEBUG", "INFO", "WARN", "ERROR"])
    log_level: StringBox
}
```

### 自動生成される設定ローダー
```nyash
// 静的ローダーメソッド
static method load() -> Result<AppConfigBox, ConfigErrorBox> {
    local config = new AppConfigBox()
    
    // required環境変数チェック
    local database_url = EnvBox.get("DATABASE_URL")
    if database_url.is_none() {
        return Err(new ConfigError("DATABASE_URL is required"))
    }
    config.database_url = database_url.unwrap()
    
    // デフォルト値付き設定
    config.redis_url = EnvBox.get_or("REDIS_URL", "redis://localhost:6379")
    config.debug_mode = EnvBox.get_or("DEBUG", "false").parse_bool()
    config.max_connections = EnvBox.get_or("MAX_CONNECTIONS", "100").parse_int()
    
    // バリデーション実行
    if config.max_connections < 1 || config.max_connections > 1000 {
        return Err(new ConfigError("MAX_CONNECTIONS must be between 1 and 1000"))
    }
    
    return Ok(config)
}

// 設定リロードメソッド
method reload() -> Result<BoolBox, ConfigErrorBox> {
    local new_config = AppConfigBox.load()
    if new_config.is_err() {
        return Err(new_config.unwrap_err())
    }
    
    // 現在の設定を更新
    local config = new_config.unwrap()
    me.database_url = config.database_url
    me.redis_url = config.redis_url
    // ... other fields
    
    return Ok(true)
}
```

### Property System統合
```nyash
@config_schema
box LiveConfigBox {
    @env("API_HOST") @required
    api_host: StringBox
    
    @env("API_PORT") @default(8080) @parse_int
    api_port: IntegerBox
    
    // computed: 設定から自動でURL生成
    api_url: StringBox { 
        "http://" + me.api_host + ":" + me.api_port 
    }
    
    // once: 重い初期化処理
    once connection_pool: PoolBox { 
        createPool(me.api_url, me.max_connections) 
    }
}
```

## 🌐 4. @api_client系マクロ（プロダクション級）

### OpenAPI仕様ベース生成
```nyash
@api_client("https://petstore.swagger.io/v2/swagger.json")
box PetStoreApiBox {
    base_url: StringBox = "https://petstore.swagger.io/v2"
    api_key: StringBox
    
    // 以下のメソッドが自動生成される：
    // getPetById(id: IntegerBox) -> Promise<PetBox>
    // addPet(pet: PetBox) -> Promise<PetBox>
    // updatePet(pet: PetBox) -> Promise<PetBox>
    // deletePet(id: IntegerBox) -> Promise<BoolBox>
    // findPetsByStatus(status: StringBox) -> Promise<ArrayBox<PetBox>>
}
```

### 自動生成されるAPIメソッド
```nyash
// GET /pet/{petId}
method getPetById(id: IntegerBox) -> Promise<PetBox> {
    local url = me.base_url + "/pet/" + id.toString()
    local request = HttpRequestBox.new()
        .url(url)
        .method("GET")
        .header("api_key", me.api_key)
    
    return HttpClientBox.send(request)
        .then(|response| {
            if response.status() != 200 {
                throw new ApiError("Failed to get pet: " + response.status())
            }
            return PetBox.fromJson(response.body())
        })
}

// POST /pet
method addPet(pet: PetBox) -> Promise<PetBox> {
    local url = me.base_url + "/pet"
    local request = HttpRequestBox.new()
        .url(url)
        .method("POST")
        .header("Content-Type", "application/json")
        .header("api_key", me.api_key)
        .body(pet.toJson().toString())
    
    return HttpClientBox.send(request)
        .then(|response| {
            if response.status() != 200 {
                throw new ApiError("Failed to add pet: " + response.status())
            }
            return PetBox.fromJson(response.body())
        })
}
```

## 🗄️ 5. @sql_schema系マクロ（企業級）

### データベーススキーマベース生成
```nyash
@sql_schema("database_schema.json")
box UserQueryBox {
    connection: DatabaseBox
    
    // 以下のメソッドが型安全に自動生成される
}
```

### 自動生成される型安全クエリビルダー
```nyash
// SELECT with type safety
method findByAge(min_age: IntegerBox, max_age: IntegerBox) -> Promise<ArrayBox<UserBox>> {
    local query = "SELECT id, name, email, age FROM users WHERE age BETWEEN ? AND ?"
    return me.connection.query(query, [min_age, max_age])
        .then(|rows| {
            return rows.map(|row| {
                return UserBox.new(
                    row.get_int("id"),
                    row.get_string("name"), 
                    row.get_string("email"),
                    row.get_int("age")
                )
            })
        })
}

// Fluent query builder
method where(condition: QueryConditionBox) -> UserQueryBuilderBox {
    return new UserQueryBuilderBox(me.connection)
        .add_condition(condition)
}

// Type-safe usage
local users = await user_query
    .where(UserQuery.age.greater_than(18))
    .where(UserQuery.name.like("%john%"))
    .orderBy(UserQuery.created_at.desc())
    .limit(10)
    .execute()  // Promise<ArrayBox<UserBox>>
```

## 🎨 6. @html_dsl系マクロ（表現力デモ）

### HTML生成DSL
```nyash
@html_dsl
box WebPageBox {
    title: StringBox = "My Page"
    users: ArrayBox<UserBox>
    
    // computed: HTML生成
    content: StringBox {
        html {
            head {
                title { me.title }
                meta(charset="utf-8")
            }
            body {
                div(class="container") {
                    h1 { "User List" }
                    ul(class="user-list") {
                        for user in me.users {
                            li(class="user-item") {
                                span(class="name") { user.name }
                                span(class="age") { "Age: " + user.age }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

### 自動生成されるHTML Builder
```nyash
// HTML builder methods
method html(content: () -> StringBox) -> StringBox {
    return "<html>" + content.call() + "</html>"
}

method div(attributes: MapBox, content: () -> StringBox) -> StringBox {
    local attrs = me.build_attributes(attributes)
    return "<div" + attrs + ">" + content.call() + "</div>"
}

method build_attributes(attrs: MapBox) -> StringBox {
    local result = ""
    attrs.each_pair(|key, value| {
        result = result + " " + key + "=\"" + value + "\""
    })
    return result
}
```

## 🚀 マクロの革新的特徴

### 1. Property System完全統合
- **stored/computed/once/birth_once** 全てでマクロ適用可能
- **リアルタイム更新**: ファイル変更でマクロ再展開

### 2. Box-First一貫性
- **MacroBox**: マクロ自体が一等市民のBox
- **型安全性**: `MacroBox<InputAst, OutputAst>`

### 3. Visual Development
- **`nyash --expand`**: 展開結果の可視化
- **`NYASH_MACRO_TRACE=1`**: ステップバイステップ追跡

### 4. 段階的導入
- **最小MVP**: @derive(Equals)から開始
- **実用拡張**: @validate, @config_schema追加
- **高機能化**: @api_client, @sql_schema実装

---

**これらのマクロ例により、Nyashは日常的な開発から企業級アプリケーションまで、全レベルでの生産性革命を実現する。**

*Property System × Macro System統合により、他言語では不可能な表現力と実用性を両立。*