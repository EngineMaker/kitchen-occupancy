# 作業ログ — kitchen-occupancy

## 🔖 現在地（2026-11-21）

- リポジトリ: https://github.com/EngineMaker/kitchen-occupancy（public、2026-11-21 本人決定）。作業場所は em105（ホスト名 em105-claw）
- Rust ツールできた（`cargo test` 7件）。`kitchen-occupancy fetch` で取り込み、`report` で集計。9/23〜11/20 の57日分
- 見るページ（自分だけ）: https://claude.ai/artifact/1YsNDyaqji9w4FSj85E8vr（Data 型。data/occupancy.json・period.json を添付）
- 次: ページの見た目の確認 → 定期更新（systemd timer）をするか決める
- 対象チャンネル: 「入室ログ」（ID 1543931132389761034、サーバー ID 875792847532621824）
- 読む Bot は **EM105-Claw を使い回す**（2026-11-21 決定。一度「新しく作る」を選んだが本人が訂正）。鍵は lifelog の `.env` に前例あり
- 投稿に出てくる「EMS002E」がキッチンのカメラ（ユーザー談）。他のカメラの投稿も混ざっている可能性あり
- 投稿の形を確認済み（下の「投稿の形」）。EM105-Claw で読める。鍵は lifelog の `.env` から本人の許可を得てコピー（`.env`、git 管理外）
- 写真の扱い: **写真は一切ダウンロード・保存しない。使うのは投稿の文字（カメラ名・検出時刻）だけ。個人の識別はしない**（2026-11-21 本人了承）

---

## 2026-11-21 立ち上げ調査

### 読む手段
- rain-alert は Webhook（書き込み専用）なので読むのには使えない
- mk-discord-bot の Bot は個人サーバーのみ。トークンで所属サーバーを確かめようとしたが、Claude 側の権限確認で止められたので未確認（BRIEF の情報どおりなら EngineMakerβ版には入っていない）
- discord-reaction-stats（public）は EngineMakerβ版に Bot を入れて履歴を読んでいる。権限は「チャンネルを表示」「メッセージ履歴を読む」のみ（66560）。トークンは GitHub Actions の Secrets にある

### 見立て
- 「人がいた時間帯」を出すだけなら、投稿の**時刻**だけで足りる。写真を開く必要は無い
- Discord API では、Message Content Intent が無くても投稿時刻と投稿者は取れる（本文・添付は空になる）
  → 写真を一切ダウンロードしない作りにできる

### 補足（EMS002E）
- カメラ名が投稿者名に出ていれば、本文を読む権限が無くても絞り込める
- 本文や埋め込みに出ているなら Message Content Intent が要る（写真はダウンロードしなくて済む）

### 投稿の形（2026-11-21 確認。EM105-Claw で最新5件と最古3件を読んだ。写真はダウンロードしていない）
- 投稿者: Bot「さばがに105」
- 本文: `15:05、 EMS002Eで\n\n  人物を検出`。添付: JPEG 1枚（`eufy_ems002e-<数字>.jpg`）
- 一番古い投稿は 2026-08-31。そのころは「Eufy C220」「Atom Cam 2」や `[テスト]` 付きの投稿もある → カメラ名 EMS002E で絞り、`[テスト]` は除く
- 本文の時刻（検出時刻）と投稿時刻がずれることがある（例: 本文 19:12、投稿 19:43 JST）→ 本文の時刻を使う方がよさそう。日付は投稿日時から取る（日付をまたぐ場合に注意）
- Discord の時刻は UTC。日本時間は +9 時間

### 2026-11-21 ツールとページ
- 「入室ログ」には住人の書き込みも混ざる → 保存するのは「人物を検出」の投稿だけにした（一度全部保存したものも消した）
- 他のカメラ: EMS101E・EMS105A・EMS001E・EMS301E もこのチャンネルに投稿している。数えるのは EMS002E だけ
- カメラは一度検出すると約10分あく。1時間ごとだと全部9割前後で差が見えない → **15分ごと**に区切った（`SLOT_MINUTES` で変更可）
- 検出が10件未満の日は数えない（`MIN_PER_DAY`）。今は 10/12・10/22 の2日
- 使い方: `~/.cargo/bin/cargo build --release` → `./target/release/kitchen-occupancy`（fetch と report の両方）
- ページの更新は今は手作業: data/occupancy.json・period.json を artifact に asset で上げ直し、datasets の url を差し替える
