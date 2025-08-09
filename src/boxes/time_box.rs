/*!
 * Nyash Time Box - Time and Date operations
 * 
 * 時間と日付操作を提供するBox型
 * Everything is Box哲学に基づく時間ライブラリ
 */

use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox};
use std::fmt::{Debug, Display};
use std::any::Any;
use std::time::{SystemTime, Duration};
use chrono::{DateTime, Local, TimeZone, Datelike, Timelike};

/// 時間操作を提供するBox
#[derive(Debug, Clone)]
pub struct TimeBox {
    id: u64,
}

impl TimeBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self { id }
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
            BoolBox::new(self.id == other_time.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
}

impl Display for TimeBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimeBox()")
    }
}

/// 日時を表すBox
#[derive(Debug, Clone)]
pub struct DateTimeBox {
    pub datetime: DateTime<Local>,
    id: u64,
}

impl DateTimeBox {
    /// 現在時刻で作成
    pub fn now() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self { 
            datetime: Local::now(),
            id,
        }
    }
    
    /// UNIXタイムスタンプから作成
    pub fn from_timestamp(timestamp: i64) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        let datetime = Local.timestamp_opt(timestamp, 0).unwrap();
        Self { datetime, id }
    }
    
    /// 文字列からパース
    pub fn parse(date_str: &str) -> Result<Self, String> {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        // ISO 8601形式でパース
        match DateTime::parse_from_rfc3339(date_str) {
            Ok(dt) => Ok(Self { 
                datetime: dt.with_timezone(&Local),
                id,
            }),
            Err(_) => {
                // シンプルな形式でパース (YYYY-MM-DD HH:MM:SS)
                match chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
                    Ok(naive_dt) => {
                        let datetime = Local.from_local_datetime(&naive_dt).unwrap();
                        Ok(Self { datetime, id })
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
                id: self.id,
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
                id: self.id,
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
    
    fn box_id(&self) -> u64 {
        self.id
    }
}

impl Display for DateTimeBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.datetime.format("%Y-%m-%d %H:%M:%S"))
    }
}

/// タイマーを表すBox
#[derive(Debug, Clone)]
pub struct TimerBox {
    start_time: SystemTime,
    id: u64,
}

impl TimerBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self {
            start_time: SystemTime::now(),
            id,
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
            BoolBox::new(self.id == other_timer.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
}

impl Display for TimerBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimerBox()")
    }
}