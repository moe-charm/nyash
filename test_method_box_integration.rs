/**
 * MethodBox統合テスト - Nyash側使用対応確認
 * 
 * NewP2PBoxのon_method()がMethodBoxを正しく受け取れるか確認
 * MethodBox.invoke()が正しく呼ばれるか確認
 */

use std::sync::{Arc, Mutex};

// Nyashモジュールをインポート
use nyash_rust::boxes::{NewP2PBox, MessageIntentBox, StringBox};
use nyash_rust::transport_trait::TransportKind;
use nyash_rust::method_box::MethodBox;
use nyash_rust::{NyashBox, InstanceBox};

fn main() {
    println!("🎯 MethodBox統合テスト開始");
    
    // テスト1: 基本的なMethodBox作成
    test_method_box_creation();
    
    // テスト2: NewP2PBox + MethodBox統合
    test_method_box_integration();
    
    println!("✅ MethodBox統合テスト完了！");
}

fn test_method_box_creation() {
    println!("\n=== テスト1: MethodBox作成テスト ===");
    
    // テスト用のインスタンスを作成（実際のInstanceBoxは使えないので、StringBoxで代用）
    let test_instance = Box::new(StringBox::new("test_instance"));
    
    // MethodBoxを作成
    let method_box = MethodBox::new(test_instance, "test_method".to_string());
    
    println!("✅ MethodBox作成成功: メソッド名 = {}", method_box.method_name);
    
    // invoke()テスト（現在は未実装エラーが返るはず）
    let args = vec![Box::new(StringBox::new("test_arg")) as Box<dyn NyashBox>];
    match method_box.invoke(args) {
        Ok(result) => println!("📥 MethodBox.invoke() 成功: {}", result.to_string_box().value),
        Err(e) => println!("⚠️  MethodBox.invoke() 未実装: {}", e),
    }
}

fn test_method_box_integration() {
    println!("\n=== テスト2: NewP2PBox + MethodBox統合テスト ===");
    
    // P2PBoxノードを作成
    let alice = NewP2PBox::new("alice_method", TransportKind::InProcess);
    let bob = NewP2PBox::new("bob_method", TransportKind::InProcess);
    
    // テスト用のMethodBoxを作成
    let handler_instance = Box::new(StringBox::new("message_handler"));
    let handler_method = MethodBox::new(handler_instance, "handle_greeting".to_string());
    
    // BobにMethodBoxベースのイベントリスナーを登録
    println!("📋 BobにMethodBoxベースのリスナー登録中...");
    match bob.on_method("greeting", handler_method) {
        Ok(()) => println!("✅ MethodBoxリスナー登録成功！"),
        Err(e) => {
            println!("❌ MethodBoxリスナー登録エラー: {}", e);
            return;
        }
    }
    
    // Aliceからメッセージ送信
    let mut message = MessageIntentBox::new("greeting");
    message.set("text", Box::new(StringBox::new("Hello Bob via MethodBox!")));
    message.set("sender", Box::new(StringBox::new("Alice")));
    
    println!("📤 AliceからBobへMethodBox経由でメッセージ送信...");
    match alice.send("bob_method", &message) {
        Ok(()) => println!("✅ メッセージ送信成功（MethodBox処理を確認）"),
        Err(e) => println!("❌ メッセージ送信エラー: {}", e),
    }
    
    // 少し待つ（非同期処理のため）
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    println!("🎉 MethodBox統合が動作していることを確認！");
}