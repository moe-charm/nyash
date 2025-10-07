# パスキー管理アプリの設計分析

## 🔐 パスキーの基本知識

### パスキーとは
- FIDO2/WebAuthn標準
- 公開鍵/秘密鍵ペア
- **秘密鍵は絶対に外に出さない**（パスワードと違う！）
- 公開鍵だけをサーバーに登録

### 保存すべきデータ
1. **秘密鍵** ← 絶対に流出させてはいけない
2. **公開鍵** ← サーバーに登録する用
3. **メタデータ** (サイト名、ユーザー名、作成日時等)

## 🌐 保存場所の選択肢と評価

### 1️⃣ ローカルデバイスのみ（現在の主流）

#### 例: Apple Keychain, Google Password Manager
```
保存場所: デバイス内の secure enclave/TPM
同期: iCloud Keychain, Google Sync (暗号化済み)
```

**メリット**:
- ✅ 最高のセキュリティ（秘密鍵がデバイス外に出ない）
- ✅ OS標準機能（Secure Enclave/TPM）
- ✅ バイオメトリクス認証と統合

**デメリット**:
- ❌ デバイス紛失=全ロスト（バックアップ必須）
- ❌ クロスプラットフォーム困難
- ❌ 新デバイス移行が面倒

---

### 2️⃣ E2E暗号化クラウド（推奨）

#### 例: 1Password, Bitwarden
```
保存場所: クラウドサーバー
暗号化: End-to-End (マスターパスワード由来の鍵)
秘密鍵: 暗号化された状態でクラウドに保存
```

**アーキテクチャ**:
```
ローカル:
  秘密鍵 → マスターパスワード派生鍵で暗号化 → 暗号化Blob
           ↓
クラウド:
  暗号化Blob保存（サーバーは復号不可）
           ↓
新デバイス:
  マスターパスワード → 復号 → 秘密鍵取得
```

**メリット**:
- ✅ デバイス間同期可能
- ✅ デバイス紛失しても復旧可能
- ✅ クロスプラットフォーム対応
- ✅ サーバーは平文を見れない（E2E）

**デメリット**:
- ⚠️ マスターパスワード忘れ=全ロスト
- ⚠️ クラウド侵害リスク（暗号化されてるが）
- ⚠️ ネットワーク必須

---

### 3️⃣ 分散ストレージ（将来的）

#### 例: IPFS, 分散ファイルシステム
```
保存場所: 分散ネットワーク
暗号化: E2E
特徴: 単一障害点なし
```

**メリット**:
- ✅ 検閲耐性
- ✅ 単一サーバー依存なし
- ✅ 永続性（複数ノード）

**デメリット**:
- ❌ 技術的複雑さ
- ❌ パフォーマンス不安定
- ❌ まだ実用レベルでない

---

### 4️⃣ ハードウェアキー（補助的）

#### 例: YubiKey, Titan Security Key
```
保存場所: USB/NFCハードウェア
秘密鍵: ハードウェア内（取り出し不可）
```

**メリット**:
- ✅ 物理的セキュリティ最強
- ✅ フィッシング完全防御

**デメリット**:
- ❌ 物理デバイス紛失=ロック
- ❌ バックアップ困難
- ❌ コスト（1本$50-100）

---

## 🎯 Hakoruneアプリの推奨設計

### **Option A: E2E暗号化クラウド（実用的）**

```hakorune
// apps/passkey_manager/vault.hako

box PasskeyVault {
  master_key: KeyDerivationBox
  storage: CloudStorageBox  // E2E encrypted
  
  // 秘密鍵を暗号化して保存
  save_passkey(site, username, private_key) {
    local encrypted = me.master_key.encrypt(private_key)
    local metadata = map({
      site: site,
      username: username,
      created_at: TimerBox.now_ms(),
      encrypted_key: encrypted
    })
    me.storage.save(site + ":" + username, metadata)
  }
  
  // 復号して取得
  get_passkey(site, username) {
    local data = me.storage.load(site + ":" + username)
    return me.master_key.decrypt(data.get("encrypted_key"))
  }
}
```

**クラウドプロバイダー選択肢**:
1. **自己ホスティング** (最高のプライバシー)
   - VPS + Nginx + PostgreSQL
   - コスト: $5-10/月
   
2. **Firebase/Supabase** (簡単)
   - E2E暗号化追加実装
   - 無料枠あり
   
3. **専用パスワードマネージャーAPI** (Bitwarden自己ホスト)
   - 既存実装活用
   - オープンソース

---

### **Option B: ローカル + オプショナル同期（安全重視）**

```hakorune
box PasskeyVault {
  local_storage: SecureStorageBox  // OS Keychain
  sync_enabled: BoolBox
  cloud_backup: CloudStorageBox    // オプション
  
  save_passkey(site, username, private_key) {
    // 必ずローカル保存
    me.local_storage.save_secure(site, private_key)
    
    // オプションでクラウドバックアップ
    if me.sync_enabled {
      local encrypted = me.encrypt_for_cloud(private_key)
      me.cloud_backup.save(site, encrypted)
    }
  }
}
```

**メリット**:
- デフォルトはローカルのみ（最高セキュリティ）
- ユーザーが明示的にクラウド有効化
- 段階的リスク管理

---

## 🛡️ セキュリティ設計の鉄則

### **1. 暗号化層の分離**
```
Layer 1: Transport (HTTPS/TLS)
Layer 2: Storage (AES-256-GCM)
Layer 3: Application (マスターパスワード派生鍵)
```

### **2. 鍵導出（PBKDF2/Argon2）**
```hakorune
box KeyDerivationBox {
  derive_master_key(password, salt) {
    // Argon2id推奨（メモリハード＋GPU耐性）
    return argon2id(
      password,
      salt,
      iterations: 100000,
      memory: 64MB,
      parallelism: 4
    )
  }
}
```

### **3. ゼロ知識証明（Zero-Knowledge）**
```
サーバーは以下を知らない:
- マスターパスワード
- 派生鍵
- 復号後の秘密鍵

サーバーが知っているのは:
- 暗号化されたBlob
- メタデータ（暗号化可能）
```

### **4. 二要素認証（2FA）**
```
マスターパスワード + デバイス固有キー
→ どちらか1つだけでは復号不可
```

---

## 🌟 具体的な実装戦略

### **Phase 1: ローカルのみ（MVP）**
```
- OS Keychainに保存（macOS/iOS/Android）
- Hakoruneからネイティブ呼び出し
- バックアップはユーザー責任
```

**実装時間**: 1-2週間

---

### **Phase 2: E2E暗号化同期**
```
- Firebase/Supabase統合
- Argon2idでマスター鍵導出
- AES-256-GCMで暗号化
- デバイス間同期
```

**実装時間**: 3-4週間

---

### **Phase 3: 高度な機能**
```
- 共有Vault（家族・チーム）
- 緊急アクセス（死亡時）
- パスキー生成ログ
- 侵害検知
```

**実装時間**: 2-3ヶ月

---

## 💡 推奨プロトコル

### **おすすめ: Bitwarden互換API + 自己ホスト**

**理由**:
1. オープンソース（監査済み）
2. E2E暗号化実証済み
3. 自己ホスト可能（プライバシー）
4. クロスプラットフォーム対応

**Hakorune実装**:
```hakorune
// apps/passkey_manager/bitwarden_client.hako
box BitwardenClient {
  api_url: StringBox
  encryption: VaultEncryptionBox
  
  sync() {
    local encrypted_vault = me.api_get("/sync")
    return me.encryption.decrypt_vault(encrypted_vault)
  }
  
  save_passkey(item) {
    local encrypted = me.encryption.encrypt_item(item)
    me.api_post("/cipher", encrypted)
  }
}
```

---

## ⚠️ 絶対にやってはいけないこと

### ❌ **1. 平文でクラウド保存**
```
秘密鍵を暗号化せずサーバーに送る → 即座に侵害リスク
```

### ❌ **2. サーバー側で復号**
```
サーバーが復号鍵を持つ → Zero-Knowledgeでない
```

### ❌ **3. 弱い暗号化**
```
MD5/SHA1 → 既に破られている
AES-128 → 現代では不十分
```

### ❌ **4. 鍵導出の手抜き**
```
単純なSHA256(password) → レインボーテーブル攻撃
正解: Argon2id/PBKDF2 (100k+ iterations)
```

---

## 🎯 結論: 最良の選択

### **推奨アーキテクチャ**

```
┌─────────────────────┐
│ Hakoruneアプリ      │
├─────────────────────┤
│ ローカル優先        │
│ + E2E暗号化同期     │
│ + 自己ホストVPS     │
└─────────────────────┘
         ↓
┌─────────────────────┐
│ 暗号化層            │
│ Argon2id + AES-256  │
└─────────────────────┘
         ↓
┌─────────────────────┐
│ ストレージ選択      │
│ 1. OS Keychain (L1) │
│ 2. VPS (オプション)  │
│ 3. Bitwarden互換     │
└─────────────────────┘
```

### **セキュリティ優先度**
```
1. ローカルKeychain (最高)
2. E2E暗号化 + 自己ホストVPS (高)
3. E2E暗号化 + 信頼できるクラウド (中)
4. 暗号化なしクラウド (絶対NG)
```

### **利便性優先度**
```
1. E2E + マルチクラウド (最高)
2. E2E + 自己ホスト (高)
3. ローカルのみ (中)
4. ハードウェアキーのみ (低)
```

### **バランス型（推奨）**
```
デフォルト: ローカルKeychain
オプション: E2E暗号化 + 自己ホストVPS
バックアップ: QRコード紙保存 + 金庫
```

**コスト**: $5-10/月（VPS）
**セキュリティ**: 9/10
**利便性**: 8/10
**プライバシー**: 10/10
