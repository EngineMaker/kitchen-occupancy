# kitchen-occupancy — 立ち上げの依頼（HQ から、2026-11-21）

## やりたいこと（ユーザーの言葉から）
Discord サーバー「EngineMakerβ版」に、キッチンの人感センサーカメラが撮った写真が投稿されるチャンネルがある。
その投稿を分析して、キッチンに「どの時間帯に人がいる確率が高いか」（時間別の稼働率）を可視化したい。
新しいリポジトリとケースを作って開発する。

## 参考になる既存のケース
- rain-alert（em105 の ~/work/ai/rain-alert、GitHub EngineMaker/rain-alert）：同じ EngineMakerβ版（ID 875792847532621824）に Webhook で通知している。Rust 製、systemd timer で定期実行。一番近いとユーザーが言っている
- mk-discord-bot（~/work/ai/mk-discord-bot）：Discord の Bot トークンの扱いの前例。ただし入っているのは個人サーバー「えむけーのサーバー」で、EngineMakerβ版ではない
- discord-reaction-stats（GitHub EngineMaker/discord-reaction-stats）：EngineMakerβ版の投稿を集計してレポートを出す前例（手元に clone は無い）

## 気をつけること
- ユーザーは日本語しか読めない。返答は短く、質問は1つずつ。業界用語は一行で説明する
- AI が作るツールは、事情がなければ Go か Rust で
- キッチンの写真にはシェアハウスの住人が写る。写真の保存・外部への送信・個人の識別をどこまでやるかは、作る前にユーザーと決める
- GitHub のリポジトリをどこに作るか（EngineMaker org か個人か）、公開か非公開かは、ユーザーに聞いてから作る
- 経緯は WORKLOG.md（冒頭に現在地の要約、下に追記式）に残す

## 最初にやること
1. チャンネルの投稿がどんな形か（画像だけか、時刻や「検知しました」の文があるか、どの Bot が投稿しているか）と、読むための手段（Bot を EngineMakerβ版に入れる必要があるか）を調べる
2. できるかどうかの見立てと、作り方の案をユーザーに短く示す
