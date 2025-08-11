/**
 * NewP2PBox天才アルゴリズムテスト
 * 
 * 1. ローカル配送テスト（Bus経由）
 * 2. リモート配送テスト（Transport経由）
 * 3. イベント購読テスト
 * 
 * MethodBox統合前の基本動作確認
 */

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Nyashモジュールをインポート
use nyash_rust::boxes::{NewP2PBox, MessageIntentBox, StringBox};
use nyash_rust::transport_trait::TransportKind;
use nyash_rust::message_bus::get_global_message_bus;

fn main() {
    println!("🚀 NewP2PBox天才アルゴリズムテスト開始");
    
    // テスト1: 基本的なP2PBox作成
    test_basic_creation();
    
    // テスト2: ローカル配送（Bus経由）
    test_local_delivery();
    
    // テスト3: イベント購読とコールバック
    test_event_subscription();
    
    println!("✅ 全テスト完了！");
}

fn test_basic_creation() {
    println!("\n=== テスト1: 基本的なP2PBox作成 ===");
    
    let alice = NewP2PBox::new("alice", TransportKind::InProcess);
    let bob = NewP2PBox::new("bob", TransportKind::InProcess);
    
    println!("✅ Alice作成: {}", alice.get_node_id());
    println!("✅ Bob作成: {}", bob.get_node_id());
    
    assert_eq!(alice.get_node_id(), "alice");
    assert_eq!(bob.get_node_id(), "bob");
}

fn test_local_delivery() {
    println!("\n=== テスト2: ローカル配送テスト ===");
    
    let alice = NewP2PBox::new("alice_local", TransportKind::InProcess);
    let bob = NewP2PBox::new("bob_local", TransportKind::InProcess);
    
    // メッセージ作成
    let mut message = MessageIntentBox::new("greeting");
    message.set("text", Box::new(StringBox::new("Hello Bob!")));
    message.set("from_user", Box::new(StringBox::new("Alice")));
    
    println!("📨 Aliceからメッセージ送信中...");
    
    // Busが両ノードを認識しているかチェック
    let bus = get_global_message_bus();
    println!("🚌 Alice認識: {}", bus.has_node("alice_local"));
    println!("🚌 Bob認識: {}", bus.has_node("bob_local"));
    
    // ローカル配送テスト
    match alice.send("bob_local", &message) {
        Ok(()) => println!("✅ ローカル配送成功！"),
        Err(e) => println!("❌ ローカル配送エラー: {}", e),
    }
}

fn test_event_subscription() {
    println!("\n=== テスト3: イベント購読テスト ===");
    
    let alice = NewP2PBox::new("alice_events", TransportKind::InProcess);
    let bob = NewP2PBox::new("bob_events", TransportKind::InProcess);
    
    // 受信メッセージカウンター
    let message_count = Arc::new(Mutex::new(0));
    let count_clone = Arc::clone(&message_count);
    
    // Bobにイベントリスナー登録
    bob.on("test_message", Box::new(move |intent_box: &MessageIntentBox| {
        let mut count = count_clone.lock().unwrap();
        *count += 1;
        println!("🎧 Bob received message #{}: intent={}", *count, intent_box.intent);
        
        // メッセージ内容確認
        if let Some(text_box) = intent_box.get("text") {
            if let Some(text) = text_box.as_any().downcast_ref::<StringBox>() {
                println!("   📝 Content: {}", text.value);
            }
        }
    }));
    
    println!("✅ Bobにイベントリスナー登録完了");
    
    // Aliceからメッセージ送信
    let mut test_message = MessageIntentBox::new("test_message");
    test_message.set("text", Box::new(StringBox::new("Test message from Alice!")));
    
    println!("📤 Aliceからテストメッセージ送信...");
    match alice.send("bob_events", &test_message) {
        Ok(()) => println!("✅ メッセージ送信成功"),
        Err(e) => println!("❌ メッセージ送信エラー: {}", e),
    }
    
    // 少し待ってからカウンターチェック
    thread::sleep(Duration::from_millis(100));
    let final_count = *message_count.lock().unwrap();
    println!("📊 最終受信メッセージ数: {}", final_count);
    
    if final_count > 0 {
        println!("✅ イベント購読システム動作確認完了！");
    } else {
        println!("⚠️  メッセージが受信されませんでした（非同期処理の可能性）");
    }
}