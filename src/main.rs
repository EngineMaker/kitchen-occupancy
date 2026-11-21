//! kitchen-occupancy: #入室ログ の「人物を検出」投稿から、キッチンの時間帯別の稼働率を出す。
//!
//! 使い方:
//!   kitchen-occupancy fetch    新しい投稿を data/messages.jsonl に取り込む
//!   kitchen-occupancy report   集計して表を出し、data/report.json を書く
//!   kitchen-occupancy          fetch と report の両方

mod config;
mod discord;
mod parse;
mod report;

use discord::Message;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::process::ExitCode;

const MESSAGES: &str = "data/messages.jsonl";
const REPORT_JSON: &str = "data/report.json";

fn load_messages() -> Vec<Message> {
    let Ok(f) = fs::File::open(MESSAGES) else {
        return Vec::new();
    };
    BufReader::new(f)
        .lines()
        .map_while(Result::ok)
        .filter_map(|l| serde_json::from_str(&l).ok())
        .collect()
}

fn fetch(cfg: &config::Config) -> Result<(), String> {
    let known = load_messages();
    let last = known
        .iter()
        .filter_map(|m| m.id.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    eprintln!("新しい投稿を取り込みます（手元に {} 件）", known.len());
    let new = discord::fetch_after(&cfg.token, &cfg.channel_id, last)?;
    fs::create_dir_all("data").map_err(|e| e.to_string())?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(MESSAGES)
        .map_err(|e| e.to_string())?;
    // 住人の書き込みは保存しない。カメラの「人物を検出」の投稿だけを残す
    let kept: Vec<&Message> = new.iter().filter(|m| m.content.contains("人物を検出")).collect();
    for m in &kept {
        let line = serde_json::to_string(m).map_err(|e| e.to_string())?;
        writeln!(f, "{line}").map_err(|e| e.to_string())?;
    }
    eprintln!(
        "{} 件を読み、検出の投稿 {} 件を追加しました（合計 {} 件）",
        new.len(),
        kept.len(),
        known.len() + kept.len()
    );
    Ok(())
}

fn run_report(cfg: &config::Config) -> Result<(), String> {
    let msgs = load_messages();
    let mut det: Vec<_> = msgs
        .iter()
        .filter_map(|m| parse::detection_time(&m.content, &m.timestamp, &cfg.camera))
        .collect();
    det.sort();
    let now = chrono::Utc::now().with_timezone(&parse::jst());
    let r = report::build(
        &det,
        now.date_naive(),
        cfg.slot_minutes,
        cfg.min_per_day,
        now.format("%Y-%m-%d %H:%M").to_string(),
    )
        .ok_or("集計できる検出がまだありません")?;
    report::print_table(&r);
    let json = serde_json::to_string_pretty(&r).map_err(|e| e.to_string())?;
    fs::write(REPORT_JSON, json).map_err(|e| e.to_string())?;
    fs::write("data/occupancy.json", report::page_rows(&r).to_string()).map_err(|e| e.to_string())?;
    fs::write("data/period.json", report::period_rows(&r).to_string()).map_err(|e| e.to_string())?;
    eprintln!("{REPORT_JSON}・data/occupancy.json・data/period.json を書きました");
    Ok(())
}

fn main() -> ExitCode {
    let cmd = std::env::args().nth(1).unwrap_or_default();
    let result = config::Config::load().and_then(|cfg| match cmd.as_str() {
        "fetch" => fetch(&cfg),
        "report" => run_report(&cfg),
        "" => fetch(&cfg).and_then(|_| run_report(&cfg)),
        other => Err(format!("知らないコマンドです: {other}")),
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("エラー: {e}");
            ExitCode::FAILURE
        }
    }
}
