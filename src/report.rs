//! 検出時刻から「曜日×時間帯ごとに人がいた確率」を集計する。
//!
//! 定義: ある日のある区切り（既定は15分。例: 15:00〜15:14）に検出が1回でもあれば「その時間は人がいた」。
//! 確率 = 人がいた日数 ÷ 数えた日数（曜日ごと）。
//! カメラは一度検出すると次まで約10分あくので、区切りを1時間にするとほぼ全部が9割を超えて差が見えない。
//! 検出が `min_per_day` 件未満の日は、カメラが止まっていた可能性が高いので数えない。

use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Serialize)]
pub struct Report {
    pub generated_at: String,
    pub first_day: String,
    pub last_day: String,
    pub counted_days: usize,
    pub skipped_days: Vec<String>,
    pub detections: usize,
    pub slot_minutes: u32,
    pub min_per_day: usize,
    /// [曜日 0=月..6=日][区切り] = 人がいた日数
    pub occupied: Vec<Vec<u32>>,
    /// [曜日] = 数えた日数
    pub days_per_weekday: Vec<u32>,
    /// [区切り] = 全曜日での人がいた日数
    pub occupied_all: Vec<u32>,
    /// [曜日][区切り] = 検出回数の合計
    pub counts: Vec<Vec<u32>>,
}

/// `today` 当日は途中なので数えない。
pub fn build(
    detections: &[NaiveDateTime],
    today: NaiveDate,
    slot_minutes: u32,
    min_per_day: usize,
    generated_at: String,
) -> Option<Report> {
    let slots = (24 * 60 / slot_minutes) as usize;
    let slot_of = |d: &NaiveDateTime| ((d.hour() * 60 + d.minute()) / slot_minutes) as usize;
    let done: Vec<&NaiveDateTime> = detections.iter().filter(|d| d.date() < today).collect();
    let first = done.iter().map(|d| d.date()).min()?;
    let last = today.pred_opt()?;

    let mut per_day: BTreeMap<NaiveDate, (usize, BTreeSet<usize>)> = BTreeMap::new();
    for d in &done {
        let e = per_day.entry(d.date()).or_default();
        e.0 += 1;
        e.1.insert(slot_of(d));
    }

    let mut occupied = vec![vec![0u32; slots]; 7];
    let mut counts = vec![vec![0u32; slots]; 7];
    let mut days_per_weekday = vec![0u32; 7];
    let mut occupied_all = vec![0u32; slots];
    let mut skipped = Vec::new();
    let mut counted = 0;
    let mut detections = 0;
    let mut day = first;
    while day <= last {
        match per_day.get(&day) {
            Some((n, set)) if *n >= min_per_day => {
                let w = day.weekday().num_days_from_monday() as usize;
                days_per_weekday[w] += 1;
                counted += 1;
                detections += n;
                for &s in set {
                    occupied[w][s] += 1;
                    occupied_all[s] += 1;
                }
            }
            _ => skipped.push(day.to_string()),
        }
        day = day.succ_opt()?;
    }
    for d in done.iter().filter(|d| !skipped.contains(&d.date().to_string())) {
        counts[d.weekday().num_days_from_monday() as usize][slot_of(d)] += 1;
    }

    Some(Report {
        generated_at,
        first_day: first.to_string(),
        last_day: last.to_string(),
        counted_days: counted,
        skipped_days: skipped,
        detections,
        slot_minutes,
        min_per_day,
        occupied,
        days_per_weekday,
        occupied_all,
        counts,
    })
}

pub fn print_table(r: &Report) {
    const WD: [&str; 7] = ["月", "火", "水", "木", "金", "土", "日"];
    println!(
        "期間 {} 〜 {}（数えた日 {} 日、検出 {} 回、数えなかった日 {} 日）",
        r.first_day,
        r.last_day,
        r.counted_days,
        r.detections,
        r.skipped_days.len()
    );
    print!("時刻  ");
    for w in WD {
        print!("  {w} ");
    }
    println!("  全体");
    let slots = r.occupied_all.len();
    for h in 0..slots {
        let m = h as u32 * r.slot_minutes;
        print!("{:02}:{:02} ", m / 60, m % 60);
        for w in 0..7 {
            let n = r.days_per_weekday[w];
            let p = if n == 0 { 0.0 } else { r.occupied[w][h] as f64 * 100.0 / n as f64 };
            print!("{p:5.0}%");
        }
        let total = r.counted_days.max(1) as f64;
        println!("{:6.0}%", r.occupied_all[h] as f64 * 100.0 / total);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap()
    }

    #[test]
    fn counts_hours_and_skips_empty_days() {
        // 2026-11-16 は月曜
        let det = vec![
            t("2026-11-16 08:10"),
            t("2026-11-16 08:40"), // 同じ時間帯は1回として数える
            t("2026-11-16 19:00"),
            // 11-17（火）は検出なし、11-18（水）は1件だけ → どちらも数えない（min_per_day = 2）
            t("2026-11-18 08:05"),
            t("2026-11-20 08:20"),
            t("2026-11-20 08:59"),
            t("2026-11-19 12:00"), // today 当日は数えない
        ];
        let today = NaiveDate::from_ymd_opt(2026, 11, 21).unwrap();
        let r = build(&det, today, 60, 2, String::new()).unwrap();
        assert_eq!(r.counted_days, 2);
        assert_eq!(
            r.skipped_days,
            vec!["2026-11-17".to_string(), "2026-11-18".to_string(), "2026-11-19".to_string()]
        );
        assert_eq!(r.occupied[0][8], 1);
        assert_eq!(r.counts[0][8], 2);
        assert_eq!(r.occupied_all[8], 2);
        assert_eq!(r.days_per_weekday[0], 1);
        assert_eq!(r.days_per_weekday[4], 1);
        assert_eq!(r.detections, 5);
    }

    #[test]
    fn fifteen_minute_slots() {
        let det = vec![t("2026-11-16 08:10"), t("2026-11-16 08:40")];
        let today = NaiveDate::from_ymd_opt(2026, 11, 17).unwrap();
        let r = build(&det, today, 15, 1, String::new()).unwrap();
        assert_eq!(r.occupied_all.len(), 96);
        assert_eq!(r.occupied_all[32], 1); // 08:10 → 08:00〜08:14
        assert_eq!(r.occupied_all[33], 0); // 08:15〜08:29 には無い
        assert_eq!(r.occupied_all[34], 1); // 08:40 → 08:30〜08:44
        assert_eq!(r.occupied[0][34], 1);
    }
}

/// ページ用のデータ（1行 = 曜日×区切り。曜日「全体」は全曜日をまとめたもの）
pub fn page_rows(r: &Report) -> serde_json::Value {
    const WD: [&str; 8] = ["月", "火", "水", "木", "金", "土", "日", "全体"];
    let slots = r.occupied_all.len();
    let mut rows = Vec::new();
    for (w, name) in WD.iter().enumerate() {
        for s in 0..slots {
            let (occ, days, det) = if w < 7 {
                (r.occupied[w][s], r.days_per_weekday[w], r.counts[w][s])
            } else {
                let det: u32 = (0..7).map(|i| r.counts[i][s]).sum();
                (r.occupied_all[s], r.counted_days as u32, det)
            };
            let m = s as u32 * r.slot_minutes;
            rows.push(serde_json::json!({
                "weekday": name,
                "time": format!("{:02}:{:02}", m / 60, m % 60),
                "occupied_days": occ,
                "days": days,
                "probability": if days == 0 { 0.0 } else { (occ as f64 / days as f64 * 1000.0).round() / 1000.0 },
                "detections": det,
            }));
        }
    }
    serde_json::json!({
        "rows": rows,
        "meta": [
            {"label": "集計ツール", "value": "https://github.com/EngineMaker/kitchen-occupancy"},
            {"label": "元データ", "value": "Discord「EngineMakerβ版」#入室ログ の EMS002E「人物を検出」投稿の本文の時刻（写真は使わない）"},
            {"label": "区切り", "value": format!("{} 分", r.slot_minutes)},
            {"label": "数えなかった日", "value": format!("検出が {} 件未満の日（{} 日）", r.min_per_day, r.skipped_days.len())},
            {"label": "作成", "value": r.generated_at},
        ]
    })
}

pub fn period_rows(r: &Report) -> serde_json::Value {
    serde_json::json!({
        "rows": [{
            "first_day": r.first_day,
            "last_day": r.last_day,
            "counted_days": r.counted_days,
            "skipped_days": r.skipped_days.len(),
            "detections": r.detections,
            "slot_minutes": r.slot_minutes,
        }],
        "meta": [{"label": "数えなかった日", "value": r.skipped_days.join(", ")}]
    })
}
