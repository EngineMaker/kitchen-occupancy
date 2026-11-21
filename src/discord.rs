//! Discord から投稿を読む（読むだけ。書き込みはしない）。
//! 写真（添付）は URL も含めて保存しない。使うのは ID・投稿時刻・本文だけ。

use serde::{Deserialize, Serialize};
use std::thread::sleep;
use std::time::Duration;
use ureq::Agent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub timestamp: String,
    pub content: String,
}

#[derive(Deserialize)]
struct RateLimited {
    retry_after: f64,
}

/// `after_id` より新しい投稿をすべて取る（古い順に並べて返す）。
pub fn fetch_after(token: &str, channel_id: &str, after_id: u64) -> Result<Vec<Message>, String> {
    let agent: Agent = Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into();
    let mut after = after_id;
    let mut all: Vec<Message> = Vec::new();
    loop {
        let url = format!(
            "https://discord.com/api/v10/channels/{channel_id}/messages?after={after}&limit=100"
        );
        let mut resp = agent
            .get(&url)
            .header("Authorization", &format!("Bot {token}"))
            .header("User-Agent", "kitchen-occupancy (https://github.com/EngineMaker/kitchen-occupancy, 0.1)")
            .call()
            .map_err(|e| format!("Discord への接続に失敗: {e}"))?;
        let status = resp.status().as_u16();
        if status == 429 {
            let wait: RateLimited = resp
                .body_mut()
                .read_json()
                .map_err(|e| format!("429 の応答を読めません: {e}"))?;
            sleep(Duration::from_secs_f64(wait.retry_after + 0.5));
            continue;
        }
        if status != 200 {
            let body = resp.body_mut().read_to_string().unwrap_or_default();
            return Err(format!("Discord が {status} を返しました: {body}"));
        }
        let batch: Vec<Message> = resp
            .body_mut()
            .read_json()
            .map_err(|e| format!("投稿の JSON を読めません: {e}"))?;
        if batch.is_empty() {
            break;
        }
        let max_id = batch
            .iter()
            .filter_map(|m| m.id.parse::<u64>().ok())
            .max()
            .unwrap_or(after);
        let n = batch.len();
        all.extend(batch);
        eprintln!("  取り込み中… {} 件", all.len());
        if n < 100 || max_id <= after {
            break;
        }
        after = max_id;
        sleep(Duration::from_millis(300));
    }
    all.sort_by_key(|m| m.id.parse::<u64>().unwrap_or(0));
    Ok(all)
}
