# Phase 30: Hakorune Passkey Manager - セキュアなパスキー管理アプリ

Date: 2025-10-07
Status: **PLANNED** - Phase 20完了後
Prerequisite: Phase 20 (マクロシステム完成), Phase 15.7 (セルフホスティング)

## 🎯 **目的**

Hakoruneで実装する、E2E暗号化対応の次世代パスキー管理アプリケーション。

FIDO2/WebAuthn標準に準拠し、プライバシーとセキュリティを最優先しつつ、マルチデバイス対応の利便性を実現する。

## 🔐 **パスキーとは**

### **基本概念**
- FIDO2/WebAuthn標準の認証方式
- 公開鍵/秘密鍵ペアによる認証
- **秘密鍵は絶対に外部に出さない**（パスワードとの根本的違い）
- フィッシング耐性・リプレイ攻撃耐性

### **保存すべきデータ**
1. **秘密鍵** (絶対に流出させてはいけない)
2. **公開鍵** (サーバー登録用)
3. **メタデータ** (サイト名、ユーザー名、作成日時、最終使用日時)

## 🏗️ **アーキテクチャ設計**

### **3層アーキテクチャ**

```
┌─────────────────────────────────┐
│ Hakorune Application Layer      │
│ - UI (TUI/GUI)                  │
│ - Business Logic                │
└─────────────────────────────────┘
           ↓
┌─────────────────────────────────┐
│ Encryption Layer (E2E)          │
│ - Argon2id (鍵導出)              │
│ - AES-256-GCM (暗号化)           │
│ - Zero-Knowledge Proof          │
└─────────────────────────────────┘
           ↓
┌─────────────────────────────────┐
│ Storage Layer                   │
│ - Local: OS Keychain            │
│ - Cloud: Self-hosted VPS (opt)  │
│ - Backup: QR Code + Paper       │
└─────────────────────────────────┘
```

## 🚀 **実装フェーズ**

### **Phase 30.1: ローカルストレージ (MVP)** - 2週間

#### **目標**
OS Keychainを使用した基本的なパスキー保存・取得機能

#### **実装内容**
```hakorune
// apps/passkey_manager/vault_local.hako
box PasskeyVaultLocal {
  storage: OSKeychainBox

  save_passkey(site, username, private_key) {
    local key_id = site + ":" + username
    me.storage.save_secure(key_id, private_key)
  }

  load_passkey(site, username) {
    local key_id = site + ":" + username
    return me.storage.load_secure(key_id)
  }

  list_all() {
    return me.storage.list_keys()
  }

  delete_passkey(site, username) {
    local key_id = site + ":" + username
    me.storage.delete_secure(key_id)
  }
}
```

#### **成功条件**
- [x] OS Keychain統合
- [x] CRUD操作完全実装
- [x] TUIインターフェース
- [x] 基本テスト完備

---

### **Phase 30.2: E2E暗号化システム** - 3週間

#### **目標**
マスターパスワードベースのEnd-to-End暗号化実装

#### **実装内容**
```hakorune
// apps/passkey_manager/encryption.hako
box VaultEncryptionBox {
  kdf: KeyDerivationBox
  cipher: AES256GCMBox

  // マスターパスワードから鍵導出
  derive_master_key(password, salt) {
    return me.kdf.argon2id(
      password: password,
      salt: salt,
      iterations: 100000,
      memory: 64 * 1024,      // 64MB
      parallelism: 4,
      output_len: 32          // 256bit
    )
  }

  // 秘密鍵を暗号化
  encrypt_private_key(private_key, master_key) {
    local nonce = me.generate_nonce()
    local ciphertext = me.cipher.encrypt(
      plaintext: private_key,
      key: master_key,
      nonce: nonce
    )
    return map({
      ciphertext: ciphertext,
      nonce: nonce,
      algorithm: "AES-256-GCM",
      kdf: "Argon2id"
    })
  }

  // 復号
  decrypt_private_key(encrypted_data, master_key) {
    return me.cipher.decrypt(
      ciphertext: encrypted_data.get("ciphertext"),
      key: master_key,
      nonce: encrypted_data.get("nonce")
    )
  }
}
```

#### **成功条件**
- [x] Argon2id実装（またはFFI）
- [x] AES-256-GCM実装
- [x] Nonce生成（CSPRNG）
- [x] 暗号化/復号テスト
- [x] セキュリティ監査

---

### **Phase 30.3: クラウド同期（オプション）** - 4週間

#### **目標**
自己ホストVPSへのE2E暗号化同期機能

#### **実装内容**
```hakorune
// apps/passkey_manager/vault_cloud.hako
box PasskeyVaultCloud {
  local: PasskeyVaultLocal
  encryption: VaultEncryptionBox
  vps: SelfHostedVPSBox
  sync_enabled: BoolBox

  // クラウド同期
  sync() {
    if !me.sync_enabled { return }

    // ローカルデータ取得
    local local_data = me.local.export_all()

    // E2E暗号化
    local master_key = me.encryption.derive_master_key(
      me.get_master_password(),
      me.get_salt()
    )
    local encrypted_vault = me.encrypt_vault(local_data, master_key)

    // VPSへアップロード
    me.vps.upload("/sync", encrypted_vault)

    // VPSからダウンロード
    local remote_encrypted = me.vps.download("/sync")
    local remote_data = me.decrypt_vault(remote_encrypted, master_key)

    // マージ（最新タイムスタンプ優先）
    me.merge_vaults(local_data, remote_data)
  }

  encrypt_vault(vault_data, master_key) {
    local json = JSON.stringify(vault_data)
    return me.encryption.encrypt_private_key(json, master_key)
  }

  decrypt_vault(encrypted, master_key) {
    local json = me.encryption.decrypt_private_key(encrypted, master_key)
    return JSON.parse(json)
  }
}
```

#### **VPS側実装（Hakoruneサーバー）**
```hakorune
// apps/passkey_manager/server/api.hako
box PasskeyServerAPI {
  db: PostgreSQLBox
  auth: JWTAuthBox

  // 暗号化Vaultアップロード（サーバーは復号不可）
  post_sync(user_id, encrypted_vault) {
    me.auth.verify_token(user_id)
    me.db.save(user_id, encrypted_vault)
    return map({ status: "ok" })
  }

  // 暗号化Vaultダウンロード
  get_sync(user_id) {
    me.auth.verify_token(user_id)
    return me.db.load(user_id)
  }
}
```

#### **成功条件**
- [x] VPS API実装（Hakoruneサーバー）
- [x] HTTPS/TLS必須
- [x] JWT認証
- [x] 同期競合解決
- [x] マルチデバイステスト

---

### **Phase 30.4: UI/UX実装** - 3週間

#### **TUIバージョン**
```hakorune
// apps/passkey_manager/ui/tui.hako
box PasskeyManagerTUI {
  vault: PasskeyVaultCloud

  main_menu() {
    print("=== Hakorune Passkey Manager ===")
    print("1. List all passkeys")
    print("2. Add new passkey")
    print("3. Search passkey")
    print("4. Delete passkey")
    print("5. Sync with cloud")
    print("6. Settings")
    print("0. Exit")
  }

  add_passkey_flow() {
    local site = me.prompt("Site name: ")
    local username = me.prompt("Username: ")

    // FIDO2パスキー生成
    local keypair = FIDO2Box.generate_keypair()

    me.vault.save_passkey(site, username, keypair.private_key)
    print("Passkey saved successfully!")

    // 公開鍵をクリップボードへ
    ClipboardBox.copy(keypair.public_key)
    print("Public key copied to clipboard")
  }
}
```

#### **GUIバージョン（将来）**
- Egui統合（Phase 15.8 GUI基盤使用）
- ドラッグ&ドロップ
- QRコード表示/スキャン
- ブラウザ拡張機能連携

#### **成功条件**
- [x] TUI完全実装
- [x] ユーザビリティテスト
- [x] エラーハンドリング完備
- [x] ヘルプドキュメント

---

### **Phase 30.5: 高度な機能** - 継続的

#### **セキュリティ強化**
- [ ] 2FA (TOTP/HOTP)
- [ ] バイオメトリクス統合
- [ ] ハードウェアキー対応（YubiKey）
- [ ] セキュリティ監査ログ

#### **利便性向上**
- [ ] ブラウザ拡張機能
- [ ] 自動入力（browser automation）
- [ ] パスワード→パスキー移行ツール
- [ ] 共有Vault（家族・チーム）

#### **エンタープライズ機能**
- [ ] 組織管理
- [ ] ロールベースアクセス制御
- [ ] 監査ログ
- [ ] コンプライアンスレポート

---

## 🛡️ **セキュリティ設計**

### **暗号化スタック**

| Layer | 技術 | 目的 |
|-------|------|------|
| Transport | HTTPS/TLS 1.3 | 通信暗号化 |
| Storage | AES-256-GCM | データ暗号化 |
| KDF | Argon2id | 鍵導出（耐ブルートフォース） |
| Auth | JWT + Refresh Token | 認証 |
| Integrity | HMAC-SHA256 | 改ざん検知 |

### **Zero-Knowledge Architecture**

```
サーバーが知らないこと:
❌ マスターパスワード
❌ 鍵導出パラメータ（Salt除く）
❌ 復号後の秘密鍵
❌ メタデータ（暗号化推奨）

サーバーが知っていること:
✅ 暗号化Blob
✅ Salt（公開OK）
✅ タイムスタンプ（同期用）
```

### **脅威モデル**

| 脅威 | 対策 | 重要度 |
|------|------|--------|
| フィッシング | パスキー自体がフィッシング耐性 | 高 |
| MITM攻撃 | HTTPS/TLS 1.3必須 | 高 |
| サーバー侵害 | E2E暗号化（サーバーは復号不可） | 高 |
| クライアント侵害 | OS Keychain + Biometrics | 中 |
| ブルートフォース | Argon2id (100k iterations) | 高 |
| リプレイ攻撃 | Nonce + Timestamp | 中 |

---

## 💾 **ストレージ選択肢**

### **Option A: 自己ホストVPS（推奨）**

**プロバイダー**: DigitalOcean, Linode, Hetzner
**構成**:
```
- VPS ($5-10/月)
- Docker Compose
  - PostgreSQL (暗号化Blob)
  - Nginx (HTTPS/Let's Encrypt)
  - Hakorune API Server
```

**メリット**:
- ✅ 完全なプライバシー
- ✅ コントロール可能
- ✅ カスタマイズ自由

**デメリット**:
- ⚠️ 自己管理必要
- ⚠️ 月額コスト

---

### **Option B: Supabase/Firebase**

**構成**:
```
- Supabase PostgreSQL
- Firebase Authentication
- Hakorune Client (E2E暗号化追加)
```

**メリット**:
- ✅ 無料枠あり
- ✅ セットアップ簡単
- ✅ スケーラブル

**デメリット**:
- ⚠️ サードパーティ依存
- ⚠️ プライバシー懸念

---

### **Option C: Bitwarden互換API**

**構成**:
```
- Vaultwarden (Bitwarden自己ホスト実装)
- Hakorune Client（Bitwarden APIクライアント実装）
```

**メリット**:
- ✅ 実績あり（監査済み）
- ✅ オープンソース
- ✅ エコシステム活用

**デメリット**:
- ⚠️ API仕様に縛られる

---

## 📊 **マイルストーン**

### **Phase 30.1: MVP（2週間）**
- ローカルストレージ
- 基本CRUD
- TUI

### **Phase 30.2: 暗号化（3週間）**
- E2E暗号化
- Argon2id/AES-256-GCM
- セキュリティテスト

### **Phase 30.3: 同期（4週間）**
- VPS統合
- 同期ロジック
- マルチデバイステスト

### **Phase 30.4: UI/UX（3週間）**
- TUI完成
- ドキュメント
- ユーザビリティテスト

### **Phase 30.5: 継続改善**
- 2FA
- ブラウザ拡張
- エンタープライズ機能

---

## 🎯 **成功指標**

### **技術指標**
- [ ] セキュリティ監査合格
- [ ] パフォーマンス: 同期 < 1秒
- [ ] 可用性: 99.9% uptime
- [ ] 暗号化強度: 256bit AES-GCM

### **ユーザー指標**
- [ ] セットアップ時間 < 5分
- [ ] 学習コスト < 30分
- [ ] パスキー追加 < 30秒
- [ ] ユーザー満足度 > 8/10

### **開発指標**
- [ ] コードカバレッジ > 80%
- [ ] ドキュメント完備
- [ ] 実装規模 < 3,000行

---

## 🌟 **なぜHakoruneで実装するのか**

### **1. セキュリティ**
```
Everything is Box
→ メモリ安全
→ 型安全
→ バッファオーバーフロー無し
```

### **2. クロスプラットフォーム**
```
Hakorune VM/LLVM/WASM
→ Windows/macOS/Linux/Web対応
→ 単一コードベース
```

### **3. パフォーマンス**
```
LLVM最適化
→ ネイティブ並み速度
→ 低メモリフットプリント
```

### **4. メンテナンス性**
```
Box統一設計
→ シンプルなコード
→ 高い保守性
```

### **5. 拡張性**
```
プラグインシステム
→ ハードウェアキー対応
→ カスタム暗号化
```

---

## 🔗 **関連Phase**

- **Phase 15.7**: セルフホスティング基盤
- **Phase 20**: マクロシステム（@derive便利）
- **Phase 23**: 型システム（型安全性強化）
- **Phase 50**: エコシステム（ブラウザ拡張統合）

---

## 📚 **参考資料**

### **標準仕様**
- [FIDO2/WebAuthn Specification](https://www.w3.org/TR/webauthn/)
- [Argon2 RFC 9106](https://datatracker.ietf.org/doc/html/rfc9106)
- [NIST SP 800-63B Digital Identity Guidelines](https://pages.nist.gov/800-63-3/sp800-63b.html)

### **実装参考**
- [Bitwarden](https://github.com/bitwarden) - オープンソースパスワードマネージャー
- [1Password Security Design](https://1password.com/security/) - セキュリティ設計
- [KeePassXC](https://keepassxc.org/) - ローカル優先設計

### **セキュリティ監査**
- [OWASP ASVS](https://owasp.org/www-project-application-security-verification-standard/)
- [Mozilla Web Security Guidelines](https://infosec.mozilla.org/guidelines/web_security)

---

**Next**: [実装ガイド](./IMPLEMENTATION.md) | [セキュリティ仕様](./SECURITY.md) | [API仕様](./API.md)
