/*! 🔍 DebugBox - デバッグ支援Box
 *
 * ## 📝 概要
 * プロフェッショナル開発向けデバッグ機能を提供するBox。
 * メモリ使用量監視、実行トレース、ブレークポイントなど
 * 高度なデバッグ機能を完備。
 *
 * ## 🛠️ 利用可能メソッド
 *
 * ### 🎯 基本デバッグ
 * - `startTracking()` - デバッグ追跡開始
 * - `stopTracking()` - デバッグ追跡停止
 * - `trackBox(box, name)` - 特定Boxを追跡対象に追加
 * - `watch(box, name)` - リアルタイム監視
 * - `clear()` - 全デバッグ情報クリア
 *
 * ### 📊 レポート・分析
 * - `dumpAll()` - 全追跡データダンプ
 * - `memoryReport()` - メモリ使用量レポート
 * - `showCallStack()` - 関数呼び出しスタック表示
 * - `saveToFile(filename)` - デバッグ情報をファイル保存
 *
 * ### 🎮 高度機能
 * - `setBreakpoint(function)` - ブレークポイント設定
 * - `traceCall(function, args)` - 関数呼び出しトレース
 * - `isTracking()` - 追跡状態確認
 * - `getTrackedCount()` - 追跡中Box数取得
 *
 * ## 💡 使用例
 * ```nyash
 * local debug, user, product
 * debug = new DebugBox()
 *
 * // デバッグ開始
 * debug.startTracking()
 *
 * // オブジェクトを追跡
 * user = new User("Alice", 25)
 * debug.trackBox(user, "user_alice")
 *
 * product = new Product("Book", 1500)
 * debug.trackBox(product, "book_product")
 *
 * // リアルタイム監視
 * debug.watch(user.age, "user_age")
 *
 * // レポート生成
 * print(debug.memoryReport())
 * print(debug.dumpAll())
 *
 * // ファイルに保存
 * debug.saveToFile("debug_report.txt")
 * ```
 *
 * ## 🎮 実用例 - パフォーマンス診断
 * ```nyash
 * static box PerformanceTest {
 *     init { debug, data, results }
 *     
 *     main() {
 *         me.debug = new DebugBox()
 *         me.debug.startTracking()
 *         
 *         // 大量データ処理のテスト
 *         me.data = []
 *         loop(i < 1000) {
 *             me.data.push("item_" + i.toString())
 *         }
 *         me.debug.trackBox(me.data, "large_array")
 *         
 *         // 処理実行
 *         me.processData()
 *         
 *         // 結果分析
 *         print(me.debug.memoryReport())
 *     }
 * }
 * ```
 *
 * ## ⚡ ベストプラクティス
 * ```nyash
 * // エラーハンドリング付きデバッグ
 * local debug
 * debug = new DebugBox()
 *
 * try {
 *     debug.startTracking()
 *     // 問題のあるコード
 *     risky_operation()
 * } catch (error) {
 *     debug.saveToFile("error_dump.txt")
 *     print("Debug info saved to error_dump.txt")
 * }
 * ```
 *
 * ## ⚠️ 注意
 * - 本格運用時はtrackingを無効にしてパフォーマンス向上
 * - 大量データ追跡時はメモリ消費に注意
 * - call stackは直近100件まで自動保持
 */

use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox, VoidBox};
use crate::instance_v2::InstanceBox;
use crate::interpreter::RuntimeError;
use chrono::Local;
use std::any::Any;
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug)]
pub struct DebugBox {
    base: BoxBase,
    tracking_enabled: RwLock<bool>,
    tracked_boxes: RwLock<HashMap<String, TrackedBoxInfo>>,
    breakpoints: RwLock<Vec<String>>,
    call_stack: RwLock<Vec<CallInfo>>,
}

#[derive(Debug, Clone)]
struct TrackedBoxInfo {
    box_type: String,
    created_at: String,
    fields: String,
    value_repr: String,
}

#[derive(Debug, Clone)]
struct CallInfo {
    function_name: String,
    args: Vec<String>,
    timestamp: String,
}

impl DebugBox {
    pub fn new() -> Self {
        DebugBox {
            base: BoxBase::new(),
            tracking_enabled: RwLock::new(false),
            tracked_boxes: RwLock::new(HashMap::new()),
            breakpoints: RwLock::new(Vec::new()),
            call_stack: RwLock::new(Vec::new()),
        }
    }

    pub fn start_tracking(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut enabled = self.tracking_enabled.write().unwrap();
        *enabled = true;
        println!("[DEBUG] Tracking started");
        Ok(Box::new(VoidBox::new()))
    }

    pub fn stop_tracking(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut enabled = self.tracking_enabled.write().unwrap();
        *enabled = false;
        println!("[DEBUG] Tracking stopped");
        Ok(Box::new(VoidBox::new()))
    }

    pub fn track_box(
        &self,
        box_value: &dyn NyashBox,
        name: &str,
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let enabled = self.tracking_enabled.read().unwrap();
        if !*enabled {
            return Ok(Box::new(VoidBox::new()));
        }

        let mut tracked = self.tracked_boxes.write().unwrap();

        let info = TrackedBoxInfo {
            box_type: box_value.type_name().to_string(),
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            fields: self.get_box_fields(box_value),
            value_repr: box_value.to_string_box().value,
        };

        tracked.insert(name.to_string(), info);

        Ok(Box::new(VoidBox::new()))
    }

    fn get_box_fields(&self, box_value: &dyn NyashBox) -> String {
        // Try to downcast to InstanceBox to get fields
        if let Some(instance) = box_value.as_any().downcast_ref::<InstanceBox>() {
            // instance_v2では get_fields()メソッドを使用
            let fields = instance.get_fields();
            let fields_locked = fields.lock().unwrap();
            let field_names: Vec<String> = fields_locked.keys().cloned().collect();
            field_names.join(", ")
        } else {
            "N/A".to_string()
        }
    }

    pub fn dump_all(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let tracked = self.tracked_boxes.read().unwrap();
        let mut output = String::from("=== Box State Dump ===\n");
        output.push_str(&format!(
            "Time: {}\n",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        ));
        output.push_str(&format!("Total tracked boxes: {}\n\n", tracked.len()));

        for (name, info) in tracked.iter() {
            output.push_str(&format!("Box: {}\n", name));
            output.push_str(&format!("  Type: {}\n", info.box_type));
            output.push_str(&format!("  Created: {}\n", info.created_at));
            output.push_str(&format!("  Fields: {}\n", info.fields));
            output.push_str(&format!("  Value: {}\n", info.value_repr));
            output.push_str("\n");
        }

        Ok(Box::new(StringBox::new(output)))
    }

    pub fn save_to_file(&self, filename: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let dump_result = self.dump_all()?;
        let content = dump_result.to_string_box().value;

        // Write to file using std::fs
        std::fs::write(filename, content).map_err(|e| RuntimeError::InvalidOperation {
            message: format!("Failed to write debug file: {}", e),
        })?;

        println!("[DEBUG] Saved debug info to {}", filename);
        Ok(Box::new(VoidBox::new()))
    }

    pub fn watch(
        &self,
        box_value: &dyn NyashBox,
        name: &str,
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let value_str = box_value.to_string_box().value;
        let type_name = box_value.type_name();

        println!("[DEBUG] Watching {} ({}): {}", name, type_name, value_str);
        Ok(Box::new(VoidBox::new()))
    }

    pub fn memory_report(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let tracked = self.tracked_boxes.read().unwrap();
        let mut report = String::from("=== Memory Report ===\n");
        report.push_str(&format!("Tracked boxes: {}\n", tracked.len()));

        // Count by type
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for info in tracked.values() {
            *type_counts.entry(info.box_type.clone()).or_insert(0) += 1;
        }

        report.push_str("\nBoxes by type:\n");
        for (box_type, count) in type_counts.iter() {
            report.push_str(&format!("  {}: {}\n", box_type, count));
        }

        Ok(Box::new(StringBox::new(report)))
    }

    // Advanced features
    pub fn set_breakpoint(&self, function_name: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut breakpoints = self.breakpoints.write().unwrap();
        breakpoints.push(function_name.to_string());
        println!("[DEBUG] Breakpoint set at function: {}", function_name);
        Ok(Box::new(VoidBox::new()))
    }

    pub fn trace_call(
        &self,
        function_name: &str,
        args: Vec<String>,
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut stack = self.call_stack.write().unwrap();
        stack.push(CallInfo {
            function_name: function_name.to_string(),
            args,
            timestamp: Local::now().format("%H:%M:%S.%3f").to_string(),
        });

        // Keep only last 100 calls to prevent memory issues
        if stack.len() > 100 {
            stack.remove(0);
        }

        Ok(Box::new(VoidBox::new()))
    }

    pub fn show_call_stack(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let stack = self.call_stack.read().unwrap();
        let mut output = String::from("=== Call Stack ===\n");

        for (i, call) in stack.iter().enumerate() {
            output.push_str(&format!(
                "{}: [{}] {}({})\n",
                i,
                call.timestamp,
                call.function_name,
                call.args.join(", ")
            ));
        }

        Ok(Box::new(StringBox::new(output)))
    }

    pub fn clear(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut tracked = self.tracked_boxes.write().unwrap();
        tracked.clear();

        let mut stack = self.call_stack.write().unwrap();
        stack.clear();

        println!("[DEBUG] Cleared all debug information");
        Ok(Box::new(VoidBox::new()))
    }

    pub fn is_tracking(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let enabled = self.tracking_enabled.read().unwrap();
        Ok(Box::new(BoolBox::new(*enabled)))
    }

    pub fn get_tracked_count(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let tracked = self.tracked_boxes.read().unwrap();
        Ok(Box::new(crate::box_trait::IntegerBox::new(
            tracked.len() as i64
        )))
    }

    // --- Phase 1: JIT/Plugin shim tracing ---
    pub fn trace_plugin_calls(&self, on: bool) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // ARCHIVED: JIT functionality moved to archive/jit-cranelift/
        println!(
            "[DEBUG] JIT shim trace: {} (JIT ARCHIVED - no-op)",
            if on { "ENABLED" } else { "DISABLED" }
        );
        Ok(Box::new(VoidBox::new()))
    }

    pub fn get_jit_events(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // ARCHIVED: JIT functionality moved to archive/jit-cranelift/
        let s = "[JIT ARCHIVED] No JIT events available - JIT moved to archive/jit-cranelift/";
        Ok(Box::new(StringBox::new(s.to_string())))
    }
}

// Manual Clone implementation for DebugBox (RwLock doesn't auto-derive Clone)
impl Clone for DebugBox {
    fn clone(&self) -> Self {
        let tracked = self.tracked_boxes.read().unwrap();
        let breakpoints = self.breakpoints.read().unwrap();
        let call_stack = self.call_stack.read().unwrap();
        let tracking_enabled = self.tracking_enabled.read().unwrap();

        DebugBox {
            base: BoxBase::new(), // New unique ID for cloned instance
            tracking_enabled: RwLock::new(*tracking_enabled),
            tracked_boxes: RwLock::new(tracked.clone()),
            breakpoints: RwLock::new(breakpoints.clone()),
            call_stack: RwLock::new(call_stack.clone()),
        }
    }
}

// Implement BoxCore trait for DebugBox
impl BoxCore for DebugBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let tracked = self.tracked_boxes.read().unwrap();
        write!(f, "DebugBox[{} tracked]", tracked.len())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Implement Display trait using BoxCore
impl std::fmt::Display for DebugBox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

// Implement NyashBox trait for DebugBox
impl NyashBox for DebugBox {
    fn to_string_box(&self) -> StringBox {
        let tracked = self.tracked_boxes.read().unwrap();
        StringBox::new(format!("DebugBox[{} tracked]", tracked.len()))
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_debug) = other.as_any().downcast_ref::<DebugBox>() {
            BoolBox::new(self.base.id == other_debug.base.id)
        } else {
            BoolBox::new(false)
        }
    }

    fn type_name(&self) -> &'static str {
        "DebugBox"
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}
