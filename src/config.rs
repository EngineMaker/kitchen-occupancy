//! .env と環境変数から設定を読む（環境変数が優先）。

use std::collections::HashMap;
use std::fs;

pub struct Config {
    pub token: String,
    pub channel_id: String,
    pub camera: String,
    /// 何分ごとに区切って数えるか（1440 を割り切れる数）
    pub slot_minutes: u32,
    /// 検出がこの件数未満の日は数えない
    pub min_per_day: usize,
}

fn read_dotenv(path: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(text) = fs::read_to_string(path) else {
        return map;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let v = v.trim().trim_matches('"').trim_matches('\'');
            map.insert(k.trim().to_string(), v.to_string());
        }
    }
    map
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let file = read_dotenv(".env");
        let get = |key: &str| -> Option<String> {
            std::env::var(key)
                .ok()
                .or_else(|| file.get(key).cloned())
                .filter(|v| !v.is_empty())
        };
        Ok(Config {
            token: get("DISCORD_BOT_TOKEN").ok_or("DISCORD_BOT_TOKEN が設定されていません")?,
            channel_id: get("CHANNEL_ID").ok_or("CHANNEL_ID が設定されていません")?,
            camera: get("CAMERA").unwrap_or_else(|| "EMS002E".to_string()),
            slot_minutes: get("SLOT_MINUTES")
                .and_then(|v| v.parse().ok())
                .filter(|m: &u32| *m > 0 && 1440 % m == 0)
                .unwrap_or(15),
            min_per_day: get("MIN_PER_DAY").and_then(|v| v.parse().ok()).unwrap_or(10),
        })
    }
}
