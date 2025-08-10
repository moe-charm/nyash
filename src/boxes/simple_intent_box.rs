/*! 🔗 SimpleIntentBox - P2P通信Box
 * 
 * ## 📝 概要  
 * ピア・ツー・ピア通信機能を提供する軽量実装Box。
 * ノード間のメッセージ配信、イベント通知、
 * 分散アプリケーション構築に使用。
 * 
 * ## 🛠️ 利用可能メソッド
 * - `send(target, message)` - メッセージ送信
 * - `on(event, callback)` - イベントリスナー登録
 * - `emit(event, data)` - イベント発火
 * - `connect(nodeId)` - ノード接続
 * - `disconnect(nodeId)` - ノード切断
 * - `getConnectedNodes()` - 接続中ノード一覧
 * - `setNodeId(id)` - 自ノードID設定
 * - `broadcast(message)` - 全ノードにブロードキャスト
 * 
 * ## 💡 使用例
 * ```nyash
 * local intent
 * intent = new SimpleIntentBox()
 * 
 * // ノード設定
 * intent.setNodeId("node1")
 * 
 * // リスナー登録
 * intent.on("message", "handleMessage")
 * intent.on("join", "handleNodeJoin")
 * 
 * // メッセージ送信
 * intent.send("node2", "Hello from node1!")
 * 
 * // ブロードキャスト
 * intent.broadcast("System announcement")
 * ```
 * 
 * ## 🎮 実用例 - チャットアプリ
 * ```nyash
 * static box ChatNode {
 *     init { intent, username, messages }
 *     
 *     main() {
 *         me.intent = new SimpleIntentBox()
 *         me.username = "User1"
 *         me.messages = []
 *         
 *         // ノード初期化
 *         me.intent.setNodeId(me.username)
 *         me.setupEventHandlers()
 *         
 *         // チャットルームに参加
 *         me.joinChatRoom()
 *     }
 *     
 *     setupEventHandlers() {
 *         // メッセージ受信
 *         me.intent.on("chat_message", "onChatMessage")
 *         // ユーザー参加
 *         me.intent.on("user_joined", "onUserJoined")
 *         // ユーザー退出
 *         me.intent.on("user_left", "onUserLeft")
 *     }
 *     
 *     sendMessage(text) {
 *         local msg
 *         msg = new MapBox()
 *         msg.set("from", me.username)
 *         msg.set("text", text)
 *         msg.set("timestamp", new TimeBox().now())
 *         
 *         me.intent.broadcast("chat_message", msg)
 *     }
 *     
 *     onChatMessage(sender, message) {
 *         me.messages.push(message)
 *         print("[" + message.get("from") + "] " + message.get("text"))
 *     }
 * }
 * ```
 * 
 * ## 🌐 分散計算例
 * ```nyash
 * static box DistributedWorker {
 *     init { intent, node_id, tasks }
 *     
 *     main() {
 *         me.intent = new SimpleIntentBox()
 *         me.node_id = "worker_" + RandomBox.randInt(1000, 9999)
 *         me.tasks = []
 *         
 *         me.intent.setNodeId(me.node_id)
 *         me.registerAsWorker()
 *     }
 *     
 *     registerAsWorker() {
 *         // タスク受信リスナー
 *         me.intent.on("task_assign", "processTask")
 *         // 結果送信完了リスナー
 *         me.intent.on("result_received", "onResultReceived")
 *         
 *         // ワーカー登録通知
 *         me.intent.broadcast("worker_ready", me.node_id)
 *     }
 *     
 *     processTask(coordinator, task) {
 *         print("Processing task: " + task.get("id"))
 *         
 *         // 重い計算処理...
 *         local result
 *         result = heavyCalculation(task.get("data"))
 *         
 *         // 結果を送信
 *         me.intent.send(coordinator, result)
 *     }
 * }
 * ```
 * 
 * ## 🎯 ゲーム用マルチプレイヤー
 * ```nyash
 * static box GameClient {
 *     init { intent, player_id, game_state }
 *     
 *     main() {
 *         me.intent = new SimpleIntentBox()
 *         me.player_id = "player_" + me.generateId()
 *         me.game_state = new MapBox()
 *         
 *         me.connectToGame()
 *     }
 *     
 *     connectToGame() {
 *         me.intent.setNodeId(me.player_id)
 *         
 *         // ゲームイベント
 *         me.intent.on("player_move", "onPlayerMove")
 *         me.intent.on("game_update", "onGameUpdate")
 *         me.intent.on("player_joined", "onPlayerJoined")
 *         
 *         // ゲーム参加
 *         me.intent.broadcast("join_game", me.player_id)
 *     }
 *     
 *     movePlayer(x, y) {
 *         local move_data
 *         move_data = new MapBox()
 *         move_data.set("player", me.player_id)
 *         move_data.set("x", x)
 *         move_data.set("y", y)
 *         
 *         me.intent.broadcast("player_move", move_data)
 *     }
 * }
 * ```
 * 
 * ## ⚠️ 注意
 * - 現在は最小限実装（フル機能開発中）
 * - ネットワーク通信は未実装（ローカル通信のみ）
 * - メッセージ配信は同一プロセス内限定
 * - 本格P2P実装は将来バージョンで提供予定
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug)]
pub struct SimpleIntentBox {
    id: u64,
    // ノードID -> コールバック関数のマップ
    listeners: Arc<Mutex<HashMap<String, Vec<String>>>>, // 仮実装
}

impl SimpleIntentBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        SimpleIntentBox {
            id,
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl NyashBox for SimpleIntentBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new("IntentBox")
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_intent) = other.as_any().downcast_ref::<SimpleIntentBox>() {
            BoolBox::new(self.id == other_intent.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "IntentBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // IntentBoxは共有されるので、新しいインスタンスを作らない
        Box::new(SimpleIntentBox {
            id: self.id,
            listeners: self.listeners.clone(),
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
}