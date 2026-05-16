use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::time::{Duration, Instant};

// --- 1. データモデル層 ---
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Card {
    name: String,
    cost: i32,
    power: i32,
    desc: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SaveData {
    wins: i32,
    deck: Vec<Card>,
}

// 画面の状態を管理する列挙型（勝敗画面を追加）
#[derive(PartialEq)]
enum Scene {
    Title,
    Deck,
    Battle,
    GameClear,
    GameOver,
}

// --- 2. メインアプリ構造体 ---
struct CardGameApp {
    scene: Scene,
    save_data: SaveData,
    
    // バトル用ステータス
    player_hp: i32,
    enemy_hp: i32,
    mana: i32,
    max_mana: i32,
    
    // アニメーション・演出用フラグ
    target_player_hp: i32,
    target_enemy_hp: i32,
    last_tick: Instant,
    message_modal: Option<String>,
}

impl CardGameApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 日本語フォントの設定
        let mut fonts = egui::FontDefinitions::default();
        if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\meiryo.ttc") {
            fonts.font_data.insert("japanese_font".to_owned(), egui::FontData::from_owned(font_data));
            fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "japanese_font".to_owned());
            fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "japanese_font".to_owned());
            cc.egui_ctx.set_fonts(fonts);
        }

        let save_data = Self::load_game();
        Self {
            scene: Scene::Title,
            save_data,
            player_hp: 100,
            enemy_hp: 100,
            mana: 1,
            max_mana: 1,
            target_player_hp: 100,
            target_enemy_hp: 100,
            last_tick: Instant::now(),
            message_modal: None,
        }
    }

    fn load_game() -> SaveData {
        if let Ok(mut file) = File::open("save.json") {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if let Ok(sd) = serde_json::from_str(&contents) {
                    return sd;
                }
            }
        }
        // 初期デッキ（通常攻撃と火炎斬り）
        SaveData {
            wins: 0,
            deck: vec![
                Card { name: "通常攻撃".to_string(), cost: 1, power: 10, desc: "敵に10ダメージ".to_string() },
                Card { name: "火炎斬り".to_string(), cost: 2, power: 20, desc: "敵に20ダメージ".to_string() },
            ],
        }
    }

    fn save_game(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.save_data) {
            if let Ok(mut file) = File::create("save.json") {
                let _ = file.write_all(json.as_bytes());
            }
        }
    }

    // 毎フレームHPをターゲット値に近づけ、0になったら勝敗判定
    fn update_animation(&mut self) {
        if self.player_hp > self.target_player_hp {
            self.player_hp -= 1;
        }
        if self.enemy_hp > self.target_enemy_hp {
            self.enemy_hp -= 1;
        }

        // バトル中のみ勝敗判定を行う
        if self.scene == Scene::Battle {
            if self.enemy_hp <= 0 {
                self.save_data.wins += 1; // 勝利数をカウントアップ
                self.save_game();         // 自動セーブ
                self.scene = Scene::GameClear;
            } else if self.player_hp <= 0 {
                self.scene = Scene::GameOver;
            }
        }
    }

    fn cpu_turn(&mut self) {
        self.message_modal = Some("敵のターン... 15ダメージ受けた！".to_string());
        self.target_player_hp = (self.target_player_hp - 15).max(0);
        if self.max_mana < 10 {
            self.max_mana += 1;
        }
        self.mana = self.max_mana;
    }
}

// --- 3. GUI描画ループ ---
impl eframe::App for CardGameApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_tick.elapsed() >= Duration::from_millis(50) {
            self.update_animation();
            self.last_tick = Instant::now();
        }
        ctx.request_repaint();

        // モーダルダイアログ
        if let Some(msg) = self.message_modal.clone() {
            egui::Window::new("通知").anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
                ui.label(&msg);
                if ui.button("OK").clicked() {
                    self.message_modal = None;
                }
            });
            return;
        }

        match self.scene {
            // --- タイトル画面 ---
            Scene::Title => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.heading(egui::RichText::new("CARD BATTLE SAGA").size(50.0).color(egui::Color32::from_rgb(255, 215, 0)));
                        
                        // 勝利数の表示を追加
                        ui.label(format!("通算勝利数: {} 勝", self.save_data.wins));
                        ui.add_space(100.0);

                        ui.horizontal(|ui| {
                            ui.columns(2, |columns| {
                                if columns[0].button("BATTLE START").clicked() {
                                    self.player_hp = 100;
                                    self.enemy_hp = 100;
                                    self.target_player_hp = 100;
                                    self.target_enemy_hp = 100;
                                    self.mana = 1;
                                    self.max_mana = 1;
                                    self.scene = Scene::Battle;
                                }
                                if columns[1].button("DECK EDIT").clicked() {
                                    self.scene = Scene::Deck;
                                }
                            });
                        });
                    });
                });
            }
            
            // --- デッキ編成画面 ---
            Scene::Deck => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("デッキ編成（簡易版）");
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for card in &self.save_data.deck {
                            ui.label(format!("{} (Cost:{}) - {}", card.name, card.cost, card.desc));
                        }
                    });

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        if ui.button("保存して戻る").clicked() {
                            self.save_game();
                            self.scene = Scene::Title;
                        }
                    });
                });
            }
            
            // --- バトル画面 ---
            Scene::Battle => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label("敵HP");
                        let enemy_progress = self.enemy_hp as f32 / 100.0;
                        ui.add(egui::ProgressBar::new(enemy_progress).text(format!("{}/100", self.enemy_hp)));

                        ui.label("自分HP");
                        let player_progress = self.player_hp as f32 / 100.0;
                        ui.add(egui::ProgressBar::new(player_progress).text(format!("{}/100", self.player_hp)));
                    });
                    ui.separator();

                    ui.label("手札:");
                    ui.horizontal(|ui| {
                        let deck_cards = self.save_data.deck.clone();
                        for card in deck_cards {
                            if ui.button(format!("{}(C:{})", card.name, card.cost)).clicked() {
                                if self.mana >= card.cost {
                                    self.mana -= card.cost;
                                    self.target_enemy_hp = (self.target_enemy_hp - card.power).max(0);
                                } else {
                                    self.message_modal = Some("マナ不足！".to_string());
                                }
                            }
                        }
                    });

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("MANA: {}/{}", self.mana, self.max_mana));
                            if ui.button("ターン終了").clicked() {
                                self.cpu_turn();
                            }
                            if ui.button("降伏する").clicked() {
                                self.scene = Scene::Title;
                            }
                        });
                    });
                });
            }

            // --- ゲームクリア画面 ---
            Scene::GameClear => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(200.0);
                        ui.heading(egui::RichText::new("VICTORY!").size(60.0).color(egui::Color32::GREEN));
                        ui.label("敵を撃破した！");
                        ui.add_space(50.0);
                        if ui.button("タイトルへ戻る").clicked() {
                            self.scene = Scene::Title;
                        }
                    });
                });
            }

            // --- ゲームオーバー画面 ---
            Scene::GameOver => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(200.0);
                        ui.heading(egui::RichText::new("GAME OVER").size(60.0).color(egui::Color32::RED));
                        ui.label("あなたのHPが0になった...");
                        ui.add_space(50.0);
                        if ui.button("タイトルへ戻る").clicked() {
                            self.scene = Scene::Title;
                        }
                    });
                });
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native("Rust Card Battle Saga", options, Box::new(|cc| Box::new(CardGameApp::new(cc))))
}