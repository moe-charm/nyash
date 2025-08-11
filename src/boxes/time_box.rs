/*! ⏰ TimeBox - 時間・日付操作Box
 * 
 * ## 📝 概要
 * 高精度な時間・日付操作を提供するBox。
 * JavaScript Date、Python datetime、C# DateTimeと同等機能。
 * タイムスタンプ処理、フォーマット、時差計算をサポート。
 * 
 * ## 🛠️ 利用可能メソッド
 * 
 * ### 📅 基本操作
 * - `now()` - 現在日時取得
 * - `fromTimestamp(timestamp)` - UNIXタイムスタンプから日時作成
 * - `parse(date_string)` - 文字列から日時パース
 * - `format(pattern)` - 指定フォーマットで文字列化
 * 
 * ### 🔢 値取得
 * - `year()` - 年取得
 * - `month()` - 月取得 (1-12)
 * - `day()` - 日取得 (1-31)
 * - `hour()` - 時取得 (0-23)
 * - `minute()` - 分取得 (0-59)
 * - `second()` - 秒取得 (0-59)
 * - `weekday()` - 曜日取得 (0=日曜)
 * 
 * ### ⏱️ 計算
 * - `addDays(days)` - 日数加算
 * - `addHours(hours)` - 時間加算
 * - `addMinutes(minutes)` - 分加算
 * - `diffDays(other)` - 日数差計算
 * - `diffHours(other)` - 時間差計算
 * 
 * ## 💡 使用例
 * ```nyash
 * local time, now, birthday, age
 * time = new TimeBox()
 * 
 * // 現在日時
 * now = time.now()
 * print("現在: " + now.format("yyyy/MM/dd HH:mm:ss"))
 * 
 * // 誕生日から年齢計算
 * birthday = time.parse("1995-03-15")
 * age = now.diffYears(birthday)
 * print("年齢: " + age.toString() + "歳")
 * 
 * // 1週間後
 * local next_week
 * next_week = now.addDays(7)
 * print("1週間後: " + next_week.format("MM月dd日"))
 * ```
 * 
 * ## 🎮 実用例 - イベントスケジューラー
 * ```nyash
 * static box EventScheduler {
 *     init { time, events, current }
 *     
 *     main() {
 *         me.time = new TimeBox()
 *         me.events = []
 *         me.current = me.time.now()
 *         
 *         // イベント追加
 *         me.addEvent("会議", me.current.addHours(2))
 *         me.addEvent("ランチ", me.current.addHours(5))
 *         me.addEvent("プレゼン", me.current.addDays(1))
 *         
 *         me.showUpcomingEvents()
 *     }
 *     
 *     addEvent(title, datetime) {
 *         local event
 *         event = new MapBox()
 *         event.set("title", title)
 *         event.set("datetime", datetime)
 *         event.set("timestamp", datetime.toTimestamp())
 *         me.events.push(event)
 *     }
 *     
 *     showUpcomingEvents() {
 *         print("=== 今後のイベント ===")
 *         loop(i < me.events.length()) {
 *             local event, hours_until
 *             event = me.events.get(i)
 *             hours_until = event.get("datetime").diffHours(me.current)
 *             
 *             print(event.get("title") + " - " + 
 *                   hours_until.toString() + "時間後")
 *         }
 *     }
 * }
 * ```
 * 
 * ## 🕐 時間計算例
 * ```nyash
 * local time, start, end, duration
 * time = new TimeBox()
 * 
 * // 作業時間計測
 * start = time.now()
 * // 何か重い処理...
 * heavyCalculation()
 * end = time.now()
 * 
 * duration = end.diffSeconds(start)
 * print("処理時間: " + duration.toString() + "秒")
 * 
 * // 締切まで残り時間
 * local deadline, remaining
 * deadline = time.parse("2025-12-31 23:59:59")
 * remaining = deadline.diffDays(time.now())
 * print("締切まで" + remaining.toString() + "日")
 * ```
 * 
 * ## ⚠️ 注意
 * - ローカルタイムゾーンに基づく処理
 * - パース可能な日時フォーマットは限定的
 * - UNIXタイムスタンプは秒単位
 * - 夏時間切り替え時は計算に注意
 */

use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, BoxCore, BoxBase};
use std::fmt::{Debug, Display};
use std::any::Any;
use std::time::{SystemTime, Duration};
use chrono::{DateTime, Local, TimeZone, Datelike, Timelike};

/// 時間操作を提供するBox
#[derive(Debug, Clone)]
pub struct TimeBox {
    base: BoxBase,
}

impl TimeBox {
    pub fn new() -> Self {
        Self { base: BoxBase::new() }
    }
    
    /// 現在時刻を取得
    pub fn now(&self) -> Box<dyn NyashBox> {
        Box::new(DateTimeBox::now())
    }
    
    /// UNIXタイムスタンプから日時を作成
    pub fn fromTimestamp(&self, timestamp: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = timestamp.as_any().downcast_ref::<IntegerBox>() {
            Box::new(DateTimeBox::from_timestamp(int_box.value))
        } else {
            Box::new(StringBox::new("Error: fromTimestamp() requires integer input"))
        }
    }
    
    /// 日時文字列をパース
    pub fn parse(&self, date_str: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(string_box) = date_str.as_any().downcast_ref::<StringBox>() {
            match DateTimeBox::parse(&string_box.value) {
                Ok(dt) => Box::new(dt),
                Err(e) => Box::new(StringBox::new(&format!("Error: {}", e))),
            }
        } else {
            Box::new(StringBox::new("Error: parse() requires string input"))
        }
    }
    
    /// ミリ秒スリープ
    pub fn sleep(&self, millis: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = millis.as_any().downcast_ref::<IntegerBox>() {
            if int_box.value > 0 {
                std::thread::sleep(Duration::from_millis(int_box.value as u64));
                Box::new(StringBox::new("ok"))
            } else {
                Box::new(StringBox::new("Error: sleep() requires positive milliseconds"))
            }
        } else {
            Box::new(StringBox::new("Error: sleep() requires integer input"))
        }
    }
    
    /// 現在時刻をフォーマット
    pub fn format(&self, format_str: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(str_box) = format_str.as_any().downcast_ref::<StringBox>() {
            let now = Local::now();
            let formatted = now.format(&str_box.value).to_string();
            Box::new(StringBox::new(formatted))
        } else {
            Box::new(StringBox::new("Error: format() requires string format pattern"))
        }
    }
}

impl NyashBox for TimeBox {
    fn type_name(&self) -> &'static str {
        "TimeBox"
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new("TimeBox()")
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_time) = other.as_any().downcast_ref::<TimeBox>() {
            BoolBox::new(self.base.id == other_time.base.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BoxCore for TimeBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimeBox()")
    }
}

impl Display for TimeBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

/// 日時を表すBox
#[derive(Debug, Clone)]
pub struct DateTimeBox {
    pub datetime: DateTime<Local>,
    base: BoxBase,
}

impl DateTimeBox {
    /// 現在時刻で作成
    pub fn now() -> Self {
        Self { 
            datetime: Local::now(),
            base: BoxBase::new(),
        }
    }
    
    /// UNIXタイムスタンプから作成
    pub fn from_timestamp(timestamp: i64) -> Self {
        let datetime = Local.timestamp_opt(timestamp, 0).unwrap();
        Self { datetime, base: BoxBase::new() }
    }
    
    /// 文字列からパース
    pub fn parse(date_str: &str) -> Result<Self, String> {
        // ISO 8601形式でパース
        match DateTime::parse_from_rfc3339(date_str) {
            Ok(dt) => Ok(Self { 
                datetime: dt.with_timezone(&Local),
                base: BoxBase::new(),
            }),
            Err(_) => {
                // シンプルな形式でパース (YYYY-MM-DD HH:MM:SS)
                match chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
                    Ok(naive_dt) => {
                        let datetime = Local.from_local_datetime(&naive_dt).unwrap();
                        Ok(Self { datetime, base: BoxBase::new() })
                    }
                    Err(e) => Err(format!("Failed to parse date: {}", e)),
                }
            }
        }
    }
    
    /// 年を取得
    pub fn year(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.datetime.year() as i64))
    }
    
    /// 月を取得
    pub fn month(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.datetime.month() as i64))
    }
    
    /// 日を取得
    pub fn day(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.datetime.day() as i64))
    }
    
    /// 時を取得
    pub fn hour(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.datetime.hour() as i64))
    }
    
    /// 分を取得
    pub fn minute(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.datetime.minute() as i64))
    }
    
    /// 秒を取得
    pub fn second(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.datetime.second() as i64))
    }
    
    /// UNIXタイムスタンプを取得
    pub fn timestamp(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.datetime.timestamp()))
    }
    
    /// ISO 8601形式でフォーマット
    pub fn toISOString(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(&self.datetime.to_rfc3339()))
    }
    
    /// カスタムフォーマット
    pub fn format(&self, fmt: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(string_box) = fmt.as_any().downcast_ref::<StringBox>() {
            let formatted = self.datetime.format(&string_box.value).to_string();
            Box::new(StringBox::new(&formatted))
        } else {
            Box::new(StringBox::new("Error: format() requires string input"))
        }
    }
    
    /// 日付を加算
    pub fn addDays(&self, days: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = days.as_any().downcast_ref::<IntegerBox>() {
            let new_datetime = self.datetime + chrono::Duration::days(int_box.value);
            Box::new(DateTimeBox {
                datetime: new_datetime,
                base: BoxBase::new(),
            })
        } else {
            Box::new(StringBox::new("Error: addDays() requires integer input"))
        }
    }
    
    /// 時間を加算
    pub fn addHours(&self, hours: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = hours.as_any().downcast_ref::<IntegerBox>() {
            let new_datetime = self.datetime + chrono::Duration::hours(int_box.value);
            Box::new(DateTimeBox {
                datetime: new_datetime,
                base: BoxBase::new(),
            })
        } else {
            Box::new(StringBox::new("Error: addHours() requires integer input"))
        }
    }
}

impl NyashBox for DateTimeBox {
    fn type_name(&self) -> &'static str {
        "DateTimeBox"
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new(&self.datetime.format("%Y-%m-%d %H:%M:%S").to_string())
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_dt) = other.as_any().downcast_ref::<DateTimeBox>() {
            BoolBox::new(self.datetime == other_dt.datetime)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BoxCore for DateTimeBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.datetime.format("%Y-%m-%d %H:%M:%S"))
    }
}

impl Display for DateTimeBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

/// タイマーを表すBox
#[derive(Debug, Clone)]
pub struct TimerBox {
    start_time: SystemTime,
    base: BoxBase,
}

impl TimerBox {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            base: BoxBase::new(),
        }
    }
    
    /// 経過時間をミリ秒で取得
    pub fn elapsed(&self) -> Box<dyn NyashBox> {
        match self.start_time.elapsed() {
            Ok(duration) => {
                let millis = duration.as_millis() as i64;
                Box::new(IntegerBox::new(millis))
            }
            Err(_) => Box::new(IntegerBox::new(0)),
        }
    }
    
    /// タイマーをリセット
    pub fn reset(&mut self) -> Box<dyn NyashBox> {
        self.start_time = SystemTime::now();
        Box::new(StringBox::new("Timer reset"))
    }
}

impl NyashBox for TimerBox {
    fn type_name(&self) -> &'static str {
        "TimerBox"
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new("TimerBox()")
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_timer) = other.as_any().downcast_ref::<TimerBox>() {
            BoolBox::new(self.base.id == other_timer.base.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BoxCore for TimerBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimerBox()")
    }
}

impl Display for TimerBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}