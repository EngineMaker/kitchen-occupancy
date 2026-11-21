//! 投稿の本文から、検出した時刻（日本時間）を取り出す。
//!
//! 本文の例: `15:05、 EMS002Eで\n\n  人物を検出`
//! 本文には時刻（時:分）しか無いので、日付は投稿日時（日本時間）から取る。
//! 本文の時刻が投稿時刻より後なら、日付をまたいで投稿された（前日の検出）とみなす。

use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, NaiveTime};

pub fn jst() -> FixedOffset {
    FixedOffset::east_opt(9 * 3600).unwrap()
}

/// 投稿時刻より本文の時刻が少し後でも、時計のずれとして同じ日に扱う幅。
const CLOCK_SLACK_MIN: i64 = 10;

pub fn detection_time(content: &str, posted_at: &str, camera: &str) -> Option<NaiveDateTime> {
    let text = content.trim();
    if text.starts_with("[テスト]") || !text.contains("人物を検出") {
        return None;
    }
    let (time_part, rest) = text.split_once('、')?;
    if rest.trim_start().split_whitespace().next()? != format!("{camera}で") {
        return None;
    }
    let time = NaiveTime::parse_from_str(time_part.trim(), "%H:%M").ok()?;
    let posted = DateTime::parse_from_rfc3339(posted_at)
        .ok()?
        .with_timezone(&jst())
        .naive_local();
    let mut detected = posted.date().and_time(time);
    if detected > posted + Duration::minutes(CLOCK_SLACK_MIN) {
        detected -= Duration::days(1);
    }
    Some(detected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap()
    }

    #[test]
    fn normal() {
        let got = detection_time(
            "15:05、 EMS002Eで\n\n  人物を検出",
            "2026-11-21T06:06:19.896000+00:00",
            "EMS002E",
        );
        assert_eq!(got, Some(t("2026-11-21 15:05")));
    }

    #[test]
    fn other_camera_is_ignored() {
        let got = detection_time(
            "19:12、 Eufy C220で\n\n  人物を検出",
            "2026-08-31T10:43:19+00:00",
            "EMS002E",
        );
        assert_eq!(got, None);
    }

    #[test]
    fn test_post_is_ignored() {
        let got = detection_time(
            "[テスト]  19:38、 EMS002Eで\n\n  人物を検出",
            "2026-08-31T10:38:58+00:00",
            "EMS002E",
        );
        assert_eq!(got, None);
    }

    #[test]
    fn crosses_midnight() {
        // 23:58 に検出、翌日 00:20 JST に投稿
        let got = detection_time(
            "23:58、 EMS002Eで\n\n  人物を検出",
            "2026-11-20T15:20:00+00:00",
            "EMS002E",
        );
        assert_eq!(got, Some(t("2026-11-20 23:58")));
    }

    #[test]
    fn small_clock_skew_stays_same_day() {
        // 本文 15:07、投稿 15:05（カメラの時計が少し進んでいる）
        let got = detection_time(
            "15:07、 EMS002Eで\n\n  人物を検出",
            "2026-11-21T06:05:00+00:00",
            "EMS002E",
        );
        assert_eq!(got, Some(t("2026-11-21 15:07")));
    }
}
