# Rust Card Battle Saga

JavaのSwingで書かれたカードゲームを、Rust + egui (`eframe`) 環境に移植したデスクトップアプリケーションです。

## 特徴
- **イミディエイト・モードGUI (`egui`)**: サクサク動く軽量なUI。
- **データ永続化**: セーブデータを `serde_json` を使って `save.json` に自動保存。
- **アニメーション**: HPバーが滑らかに減少する演出。
- **勝敗判定**: 通算勝利数のカウントや、ゲームクリア・ゲームオーバー画面の実装。

## 実行方法
```bash
cargo run

<img width="1118" height="907" alt="スクリーンショット 2026-05-16 103127" src="https://github.com/user-attachments/assets/e9621471-3fef-427b-935f-95802b4c22c8" />
