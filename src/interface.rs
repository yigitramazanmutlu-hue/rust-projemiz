//! Arayüz (GUI) Modülü
//!
//! egui/eframe tabanlı Discord/Steam tarzı karanlık tema arayüzü.
//! 4 sekmeli sidebar navigasyonu ile tüm uygulama ayarlarını yönetir.

use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::settings::{self, AppConfig};
use crate::display;
use crate::state::*;
use crate::os_bridge;

// ─── Arayüz Yardımcı Fonksiyonları ─────────────────────────────────────────────

fn card_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(egui::Color32::from_rgb(26, 29, 39))
        .stroke(egui::Stroke::new(
            1.0_f32,
            egui::Color32::from_rgb(40, 45, 59),
        ))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(12))
}

fn pill_toggle_button(ui: &mut egui::Ui, active: bool, text: &str) -> bool {
    let bg = if active {
        egui::Color32::from_rgb(88, 101, 242) // Blurple
    } else {
        egui::Color32::from_rgb(33, 37, 49)
    };
    let stroke = if active {
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(120, 132, 255))
    } else {
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(48, 54, 71))
    };
    let text_color = if active {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_rgb(160, 172, 195)
    };

    let btn = egui::Button::new(
        egui::RichText::new(text)
            .color(text_color)
            .strong()
            .size(12.0),
    )
    .fill(bg)
    .stroke(stroke)
    .corner_radius(egui::CornerRadius::same(6))
    .min_size(egui::vec2(32.0, 24.0));

    ui.add(btn).clicked()
}

fn configure_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(egui::Color32::from_rgb(241, 245, 249));
    visuals.panel_fill = egui::Color32::from_rgb(17, 19, 24);
    visuals.window_fill = egui::Color32::from_rgb(21, 24, 32);
    visuals.faint_bg_color = egui::Color32::from_rgb(26, 29, 39);
    visuals.extreme_bg_color = egui::Color32::from_rgb(13, 14, 18);

    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(26, 29, 39);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(40, 45, 59));
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(7);

    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(31, 35, 47);
    visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(46, 52, 69));
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(7);
    visuals.widgets.inactive.fg_stroke =
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(203, 213, 225));

    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(43, 49, 66);
    visuals.widgets.hovered.bg_stroke =
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(88, 101, 242));
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(7);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);

    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(88, 101, 242);
    visuals.widgets.active.bg_stroke =
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(120, 132, 255));
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(7);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);

    visuals.selection.bg_fill = egui::Color32::from_rgb(88, 101, 242);
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
    ctx.set_visuals(visuals);
}

// ─── Arayüz Uygulaması (App) ──────────────────────────────────────────────────

pub struct App {
    pub performance_mode: Arc<AtomicBool>,
    pub ayarlar: Arc<Mutex<AppConfig>>,
    aktif_sekme: usize,
    son_kayit: Option<Instant>,
    kayit_hatasi: bool,
    gamma_hata: bool,
    cisim_deneme: bool,
    cisim_fare: bool,
    cisim_track: crate::track::ObjectTrack,
    cisim_capture: Option<crate::os_bridge::CenterCapture>,
    cisim_zaman: Option<Instant>,
    cisim_rgb: Vec<u8>,
    cisim_ham: Option<crate::track::ScanHit>,
    cisim_iz: crate::track::TrackView,
    cisim_kare_var: bool,
    cisim_texture: Option<egui::TextureHandle>,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        performance_mode: Arc<AtomicBool>,
        ayarlar: Arc<Mutex<AppConfig>>,
    ) -> Self {
        configure_theme(&cc.egui_ctx);

        if let Ok(cfg) = ayarlar.lock() {
            crate::state::load_from_settings(&cfg);
        }
        Self {
            performance_mode,
            ayarlar,
            aktif_sekme: 0,
            son_kayit: None,
            kayit_hatasi: false,
            gamma_hata: false,
            cisim_deneme: false,
            cisim_fare: false,
            cisim_track: crate::track::ObjectTrack::new(0.033, 2),
            cisim_capture: None,
            cisim_zaman: None,
            cisim_rgb: Vec::new(),
            cisim_ham: None,
            cisim_iz: crate::track::TrackView {
                x: 0.0,
                y: 0.0,
                live: false,
            },
            cisim_kare_var: false,
            cisim_texture: None,
        }
    }

    fn kaydet(&mut self) {
        let sonuc = self.ayarlar.lock().map(|cfg| {
            crate::state::load_from_settings(&cfg);
            settings::save_config(&cfg)
        });
        self.kayit_hatasi = !matches!(sonuc, Ok(Ok(())));
        self.son_kayit = Some(Instant::now());
    }

    fn ayar_yaz(&self, yaz: impl FnOnce(&mut AppConfig)) {
        if let Ok(mut cfg) = self.ayarlar.lock() {
            yaz(&mut cfg);
        }
    }

    fn select_task(&self, id: usize) {
        SILAH_ID.store(id, Ordering::Relaxed);
        ATES_ADIM.store(0, Ordering::Relaxed);
        let kopya = self.ayarlar.lock().ok().map(|mut cfg| {
            cfg.task_id = id;
            cfg.clone()
        });
        if let Some(cfg) = kopya {
            let _ = settings::save_config(&cfg);
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Arka plan durum göstergelerini güncel tutarken boşta sınırsız FPS tüketme.
        ui.ctx().request_repaint_after(Duration::from_millis(33));

        self.cisim_izini_guncelle();

        let secili_task_idx = SILAH_ID.load(Ordering::Relaxed);
        let task_name = settings::silah_adi(secili_task_idx);
        let aktif_durum = self.performance_mode.load(Ordering::Relaxed);

        // ═════════════════════════════════════════════════════════════════════
        // SOL YAN MENÜ (SIDEBAR)
        // ═════════════════════════════════════════════════════════════════════
        self.render_sidebar(ui, task_name, aktif_durum);

        // ═════════════════════════════════════════════════════════════════════
        // SAĞ İÇERİK PANELİ (CENTRAL CONTENT PANEL)
        // ═════════════════════════════════════════════════════════════════════
        self.render_content(ui, task_name, aktif_durum);
    }
}

impl App {
    fn render_sidebar(&mut self, ui: &mut egui::Ui, _task_name: &str, aktif_durum: bool) {
        egui::Panel::left("nav_sidebar")
            .exact_size(185.0)
            .resizable(false)
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(21, 24, 32))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(35, 39, 52),
                    ))
                    .inner_margin(egui::Margin::same(12)),
            )
            .show(ui, |ui| {
                // Logo & Başlık
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("⚡")
                            .size(20.0)
                            .color(egui::Color32::from_rgb(88, 101, 242)),
                    );
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("RUST PRO")
                                .size(15.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );
                        ui.label(
                            egui::RichText::new("SUITE V2.0")
                                .size(9.0)
                                .color(egui::Color32::from_rgb(148, 163, 184)),
                        );
                    });
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // Navigasyon Butonları
                let sekmeler = [
                    (0, "🎯", "Optimization"),
                    (1, "⚡", "Hızlı Araçlar"),
                    (2, "📊", "Mermi Tablosu"),
                    (3, "⚙️", "Sistem & Tuşlar"),
                ];

                for (idx, icon, isim) in sekmeler.iter() {
                    let secili = self.aktif_sekme == *idx;
                    let (bg, stroke, text_color) = if secili {
                        (
                            egui::Color32::from_rgb(40, 46, 68),
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(88, 101, 242)),
                            egui::Color32::WHITE,
                        )
                    } else {
                        (
                            egui::Color32::TRANSPARENT,
                            egui::Stroke::NONE,
                            egui::Color32::from_rgb(148, 163, 184),
                        )
                    };

                    let btn_text = format!("{}  {}", icon, isim);
                    let btn = egui::Button::new(
                        egui::RichText::new(btn_text)
                            .size(13.0)
                            .color(text_color)
                            .strong(),
                    )
                    .fill(bg)
                    .stroke(stroke)
                    .corner_radius(egui::CornerRadius::same(6))
                    .min_size(egui::vec2(160.0, 32.0));

                    if ui.add(btn).clicked() {
                        self.aktif_sekme = *idx;
                    }
                    ui.add_space(3.0);
                }

                // Alt Kısım: Sistem Durumu & Hızlı Toggle
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(4.0);

                    // Alt OSD Bilgi Rozeti
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgb(16, 18, 24))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgb(32, 36, 48),
                        ))
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(10))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                let (durum_renk, durum_yazi, nokta) = if aktif_durum {
                                    (egui::Color32::from_rgb(34, 197, 94), "PERFORMANCE AKTİF", "●")
                                } else {
                                    (egui::Color32::from_rgb(239, 68, 68), "PERFORMANCE PASİF", "○")
                                };

                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(nokta).size(14.0).color(durum_renk),
                                    );
                                    ui.label(
                                        egui::RichText::new(durum_yazi)
                                            .size(12.0)
                                            .strong()
                                            .color(durum_renk),
                                    );
                                });

                                ui.add_space(2.0);
                                ui.label(
                                    egui::RichText::new("[F5 / CapsLock]")
                                        .size(10.0)
                                        .color(egui::Color32::from_rgb(100, 116, 139)),
                                );

                                ui.add_space(6.0);

                                let toggle_btn_text = if aktif_durum {
                                    "Devre Dışı Bırak"
                                } else {
                                    "Aktifleştir"
                                };
                                let toggle_btn_fill = if aktif_durum {
                                    egui::Color32::from_rgb(45, 26, 30)
                                } else {
                                    egui::Color32::from_rgb(24, 45, 32)
                                };
                                let toggle_btn_stroke = if aktif_durum {
                                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(239, 68, 68))
                                } else {
                                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(34, 197, 94))
                                };

                                let t_btn = egui::Button::new(
                                    egui::RichText::new(toggle_btn_text)
                                        .size(11.0)
                                        .color(egui::Color32::WHITE),
                                )
                                .fill(toggle_btn_fill)
                                .stroke(toggle_btn_stroke)
                                .corner_radius(egui::CornerRadius::same(5))
                                .min_size(egui::vec2(138.0, 24.0));

                                if ui.add(t_btn).clicked() {
                                    self.performance_mode.store(!aktif_durum, Ordering::Relaxed);
                                }
                            });
                        });
                });
            });
    }

    fn render_content(&mut self, ui: &mut egui::Ui, task_name: &str, aktif_durum: bool) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(15, 17, 23))
                    .inner_margin(egui::Margin::symmetric(14, 10)),
            )
            .show(ui, |ui| {
                // Üst Bar (Header & Canlı Bilgi Çubuğu)
                ui.horizontal(|ui| {
                    let baslik = match self.aktif_sekme {
                        0 => "Optimization Kalibrasyonu",
                        1 => "Oyun İçi Hızlı Araçlar",
                        2 => "Rust Mermi Düşme Tablosu",
                        _ => "Sistem & Kısayol Ayarları",
                    };
                    ui.label(
                        egui::RichText::new(baslik)
                            .size(16.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Kayıt Bildirimi
                        if let Some(t) = self.son_kayit {
                            if t.elapsed() < Duration::from_secs(3) {
                                let (mesaj, renk) = if self.kayit_hatasi {
                                    ("⚠ Kaydedilemedi", egui::Color32::from_rgb(239, 68, 68))
                                } else {
                                    ("✓ Kaydedildi", egui::Color32::from_rgb(34, 197, 94))
                                };
                                ui.label(egui::RichText::new(mesaj).size(11.0).color(renk));
                            }
                        }

                        // Üst OSD Mini Bilgi Rozeti
                        if MINI_HUD_ETKIN.load(Ordering::Relaxed) {
                            let tetik_mod = TETIK_MODU.load(Ordering::Relaxed);
                            let mod_txt = if tetik_mod == 0 { "ADS" } else { "Sol Tık" };
                            let info_text = format!("{} | {}", task_name, mod_txt);

                            egui::Frame::NONE
                                .fill(egui::Color32::from_rgb(26, 29, 39))
                                .stroke(egui::Stroke::new(
                                    1.0_f32,
                                    egui::Color32::from_rgb(44, 49, 65),
                                ))
                                .corner_radius(egui::CornerRadius::same(5))
                                .inner_margin(egui::Margin::symmetric(8, 3))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(info_text)
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(137, 180, 250)),
                                    );
                                });
                        }
                    });
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // İçerik Alanı
                egui::ScrollArea::vertical().show(ui, |ui| match self.aktif_sekme {
                    0 => self.render_tab_performance(ui, aktif_durum),
                    1 => self.render_tab_utilities(ui),
                    2 => self.render_tab_diagnostics(ui),
                    _ => self.render_tab_system(ui),
                });
            });
    }

    // ─── SEKME 0: NO-PERFORMANCE & KALİBRASYON ────────────────────────────────────
    fn render_tab_performance(&mut self, ui: &mut egui::Ui, aktif_durum: bool) {
        // 1. Tetikleme Modu Seçim Kartı
        card_frame().show(ui, |ui| {
            ui.label(
                egui::RichText::new("TETİKLEME MODU")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(148, 163, 184))
                    .strong(),
            );
            ui.add_space(4.0);

            let mut tetik_m = TETIK_MODU.load(Ordering::Relaxed);
            ui.horizontal(|ui| {
                let btn_ads_selected = tetik_m == 0;
                let btn_hip_selected = tetik_m == 1;

                let (ads_bg, ads_stroke) = if btn_ads_selected {
                    (
                        egui::Color32::from_rgb(38, 44, 66),
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(88, 101, 242)),
                    )
                } else {
                    (
                        egui::Color32::from_rgb(22, 24, 32),
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(38, 42, 54)),
                    )
                };

                let ads_btn = egui::Button::new(
                    egui::RichText::new("🎯 ADS Modu (Sağ + Sol Tık)")
                        .size(12.0)
                        .color(if btn_ads_selected {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::from_rgb(148, 163, 184)
                        }),
                )
                .fill(ads_bg)
                .stroke(ads_stroke)
                .corner_radius(egui::CornerRadius::same(6))
                .min_size(egui::vec2(220.0, 28.0));

                if ui.add(ads_btn).clicked() {
                    tetik_m = 0;
                    TETIK_MODU.store(0, Ordering::Relaxed);
                    if let Ok(mut cfg) = self.ayarlar.lock() {
                        cfg.tetik_modu = 0;
                    }
                }

                let (hip_bg, hip_stroke) = if btn_hip_selected {
                    (
                        egui::Color32::from_rgb(38, 44, 66),
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(88, 101, 242)),
                    )
                } else {
                    (
                        egui::Color32::from_rgb(22, 24, 32),
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(38, 42, 54)),
                    )
                };

                let hip_btn = egui::Button::new(
                    egui::RichText::new("🔫 Serbest Mod (Sadece Sol Tık)")
                        .size(12.0)
                        .color(if btn_hip_selected {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::from_rgb(148, 163, 184)
                        }),
                )
                .fill(hip_bg)
                .stroke(hip_stroke)
                .corner_radius(egui::CornerRadius::same(6))
                .min_size(egui::vec2(220.0, 28.0));

                if ui.add(hip_btn).clicked() {
                    tetik_m = 1;
                    TETIK_MODU.store(1, Ordering::Relaxed);
                    if let Ok(mut cfg) = self.ayarlar.lock() {
                        cfg.tetik_modu = 1;
                    }
                }
            });
        });

        ui.add_space(6.0);

        // 2. Profile Selection Kartı
        card_frame().show(ui, |ui| {
            ui.label(
                egui::RichText::new("SİLAH SEÇİMİ (KISAYOLLAR: F1 - F11)")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(148, 163, 184))
                    .strong(),
            );
            ui.add_space(5.0);

            let secili = SILAH_ID.load(Ordering::Relaxed);

            // 1. Satır Silahlar
            ui.horizontal_wrapped(|ui| {
                let row1 = [
                    (0, settings::silah_adi(0), "F1"),
                    (1, settings::silah_adi(1), "F2"),
                    (2, settings::silah_adi(2), "F3"),
                    (3, settings::silah_adi(3), "F4"),
                ];
                for (id, isim, key) in row1.iter() {
                    if self.select_taskim_butonu(ui, secili, *id, isim, key) {
                        self.select_task(*id);
                    }
                }
            });

            ui.add_space(3.0);

            // 2. Satır Silahlar
            ui.horizontal_wrapped(|ui| {
                let row2 = [
                    (4, settings::silah_adi(4), "F7"),
                    (5, settings::silah_adi(5), "F8"),
                    (6, settings::silah_adi(6), "F9"),
                    (7, settings::silah_adi(7), "F10"),
                    (8, settings::silah_adi(8), "F11"),
                ];
                for (id, isim, key) in row2.iter() {
                    if self.select_taskim_butonu(ui, secili, *id, isim, key) {
                        self.select_task(*id);
                    }
                }
            });
        });

        ui.add_space(6.0);

        card_frame().show(ui, |ui| {
            let (mut sens, mut fov, mut adim) = self
                .ayarlar
                .lock()
                .map(|cfg| (cfg.hassasiyet, cfg.gorus_acisi, cfg.adim_ms))
                .unwrap_or((1.0, 90.0, 22));
            let mut degisti = false;
            ui.label(
                egui::RichText::new("OYUN AYARI")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(148, 163, 184))
                    .strong(),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Hassasiyet:").size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Slider::new(&mut sens, 0.1..=3.0).step_by(0.01))
                        .changed()
                    {
                        degisti = true;
                    }
                });
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Görüş açısı:").size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Slider::new(&mut fov, 60.0..=110.0).step_by(1.0))
                        .changed()
                    {
                        degisti = true;
                    }
                });
            });
            let mut adim_f = adim as f64;
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Adım süresi:").size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Slider::new(&mut adim_f, 8.0..=40.0).step_by(1.0).suffix(" ms"))
                        .changed()
                    {
                        adim = adim_f.round() as u64;
                        degisti = true;
                    }
                });
            });
            if degisti {
                self.ayar_yaz(|cfg| {
                    cfg.hassasiyet = sens;
                    cfg.gorus_acisi = fov;
                    cfg.adim_ms = adim;
                });
                PERFORMANCE_AYAR_SURUMU.fetch_add(1, Ordering::Release);
            }
            ui.add_space(3.0);
            ui.label(
                egui::RichText::new("1.00 hassasiyet, 90 görüş açısı ve 22 ms bugünkü paterni olduğu gibi bırakır.")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(100, 116, 139)),
            );
        });

        ui.add_space(6.0);

        // 3. Kalibrasyon Kaydırıcıları Kartı
        card_frame().show(ui, |ui| {
            let secili = SILAH_ID.load(Ordering::Relaxed);
            let (isim, mut ayar) = self
                .ayarlar
                .lock()
                .ok()
                .map(|cfg| match secili {
                    0 => (settings::silah_adi(0), cfg.prof_a),
                    1 => (settings::silah_adi(1), cfg.prof_b),
                    2 => (settings::silah_adi(2), cfg.prof_c),
                    3 => (settings::silah_adi(3), cfg.prof_d),
                    4 => (settings::silah_adi(4), cfg.prof_e),
                    5 => (settings::silah_adi(5), cfg.prof_f),
                    6 => (settings::silah_adi(6), cfg.prof_g),
                    7 => (settings::silah_adi(7), cfg.prof_h),
                    _ => (settings::silah_adi(8), cfg.prof_i),
                })
                .unwrap_or((
                    settings::silah_adi(0),
                    crate::settings::ProcessSettings {
                        dikey: 1.0,
                        yatay: 1.0,
                        ilk5_mermi_carpan: 1.0,
                    },
                ));
            let mut ayar_degisti = false;

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("⚙️  {} KALİBRASYON PARAMETRELERİ", isim))
                        .size(12.0)
                        .strong()
                        .color(egui::Color32::from_rgb(137, 180, 250)),
                );
            });

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Dikey (Aşağı) Çekiş Gücü:").size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Slider::new(&mut ayar.dikey, 0.1..=3.0).step_by(0.01))
                        .changed()
                    {
                        ayar_degisti = true;
                    }
                });
            });

            ui.add_space(3.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Yatay (Sol/Sağ) Çekiş Gücü:").size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Slider::new(&mut ayar.yatay, 0.1..=3.0).step_by(0.01))
                        .changed()
                    {
                        ayar_degisti = true;
                    }
                });
            });

            ui.add_space(3.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("İlk 5 Mermi Ekstra Güç Çarpanı:").size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Slider::new(&mut ayar.ilk5_mermi_carpan, 0.5..=2.5).step_by(0.05),
                        )
                        .changed()
                    {
                        ayar_degisti = true;
                    }
                });
            });

            if ayar_degisti {
                self.ayar_yaz(|cfg| match secili {
                    0 => cfg.prof_a = ayar,
                    1 => cfg.prof_b = ayar,
                    2 => cfg.prof_c = ayar,
                    3 => cfg.prof_d = ayar,
                    4 => cfg.prof_e = ayar,
                    5 => cfg.prof_f = ayar,
                    6 => cfg.prof_g = ayar,
                    7 => cfg.prof_h = ayar,
                    _ => cfg.prof_i = ayar,
                });
                PERFORMANCE_AYAR_SURUMU.fetch_add(1, Ordering::Release);
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let save_btn = egui::Button::new(
                    egui::RichText::new("💾 Ayarları Kaydet")
                        .size(12.0)
                        .strong()
                        .color(egui::Color32::WHITE),
                )
                .fill(egui::Color32::from_rgb(88, 101, 242))
                .corner_radius(egui::CornerRadius::same(6))
                .min_size(egui::vec2(130.0, 28.0));

                if ui.add(save_btn).clicked() {
                    self.kaydet();
                }

                let test_btn = egui::Button::new(
                    egui::RichText::new("🎯 Fare Testi (Aşağı Çek)")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(203, 213, 225)),
                )
                .fill(egui::Color32::from_rgb(33, 37, 49))
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    egui::Color32::from_rgb(50, 56, 74),
                ))
                .corner_radius(egui::CornerRadius::same(6))
                .min_size(egui::vec2(150.0, 28.0));

                if ui.add(test_btn).clicked() {
                    os_bridge::move_mouse(0, 15);
                }
            });
        });

        ui.add_space(6.0);

        // 4. Canlı Çalışma Durumu Gösterge Kartı
        card_frame().show(ui, |ui| {
            let ates = ATES_AKTIF.load(Ordering::Relaxed);
            let adim = ATES_ADIM.load(Ordering::Relaxed);
            let tetik_m = TETIK_MODU.load(Ordering::Relaxed);

            if ates && aktif_durum {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("🔥 ATEŞ AKTİF")
                            .size(13.0)
                            .strong()
                            .color(egui::Color32::from_rgb(249, 115, 22)),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("Patern Adımı: #{}", adim))
                            .size(12.0)
                            .color(egui::Color32::from_rgb(250, 204, 21)),
                    );
                });
            } else {
                let trigger_text = if tetik_m == 0 {
                    "Sağ Tık + Sol Tık"
                } else {
                    "Sol Tık"
                };
                ui.label(
                    egui::RichText::new(format!(
                        "⚡ Beklemede ({}) — Ateş edildiğinde geri tepme anında dengelenir.",
                        trigger_text
                    ))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(148, 163, 184)),
                );
            }
        });
    }

    // ─── SEKME 1: OYUN İÇİ HIZLI ARAÇLAR ─────────────────────────────────────
    fn render_tab_utilities(&mut self, ui: &mut egui::Ui) {
        // 1. Hızlı Şırınga / Bandaj (Fast Med)
        let mut f_med = FAST_MED_ETKIN.load(Ordering::Relaxed);
        card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("💉 HIZLI ŞIRINGA / BANDAJ (FAST MED)").size(12.0).strong().color(egui::Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let f_med_lbl = if f_med { "AÇIK" } else { "KAPALI" };
                    if ui.checkbox(&mut f_med, f_med_lbl).changed() {
                        FAST_MED_ETKIN.store(f_med, Ordering::Relaxed);
                        if let Ok(mut cfg) = self.ayarlar.lock() { cfg.fast_med = f_med; }
                    }
                });
            });

            ui.add_space(4.0);

            let mut med_tusu = self
                .ayarlar
                .lock()
                .map(|cfg| cfg.med_tusu.clone())
                .unwrap_or_else(|_| "V".to_string());
            let mut med_slotu = self.ayarlar.lock().map(|cfg| cfg.med_slotu).unwrap_or(5);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Tetikleme Tuşu:").size(12.0).color(egui::Color32::from_rgb(148, 163, 184)));
                for key in ["V", "C", "G", "X", "B", "Z", "Q"].iter() {
                    let is_sel = med_tusu == *key;
                    if pill_toggle_button(ui, is_sel, key) {
                        med_tusu = (*key).to_string();
                        if let Some(vk) = settings::key_to_vk(key) {
                            FAST_MED_VK.store(vk as usize, Ordering::Relaxed);
                        }
                        let secilen = med_tusu.clone();
                        self.ayar_yaz(|cfg| cfg.med_tusu = secilen);
                    }
                }
            });

            ui.add_space(3.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Şırınga Slotu:").size(12.0).color(egui::Color32::from_rgb(148, 163, 184)));
                for slot in [4, 5, 6].iter() {
                    let is_sel = med_slotu == *slot;
                    let label = format!("Slot {}", slot);
                    if pill_toggle_button(ui, is_sel, &label) {
                        med_slotu = *slot;
                        FAST_MED_SLOT.store(*slot as usize, Ordering::Relaxed);
                        self.ayar_yaz(|cfg| cfg.med_slotu = med_slotu);
                    }
                }
            });

            ui.add_space(3.0);
            ui.label(
                egui::RichText::new("ℹ️ Tuşa bastığınızda slottaki şırıngayı salisede basıp tekrar 1. slottaki taska geri döner.")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(100, 116, 139))
            );
        });

        ui.add_space(6.0);

        // 2. Hızlı Kutu / Eşya Lootlama (Fast Loot)
        let mut f_loot = FAST_LOOT_ETKIN.load(Ordering::Relaxed);
        card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("📦 HIZLI LOOT (FAST AUTO-LOOT)").size(12.0).strong().color(egui::Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let f_loot_lbl = if f_loot { "AÇIK" } else { "KAPALI" };
                    if ui.checkbox(&mut f_loot, f_loot_lbl).changed() {
                        FAST_LOOT_ETKIN.store(f_loot, Ordering::Relaxed);
                        if let Ok(mut cfg) = self.ayarlar.lock() { cfg.fast_loot = f_loot; }
                    }
                });
            });

            ui.add_space(4.0);

            let mut loot_tusu = self
                .ayarlar
                .lock()
                .map(|cfg| cfg.loot_tusu.clone())
                .unwrap_or_else(|_| "X".to_string());
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Loot Tuşu (Basılı Tut):").size(12.0).color(egui::Color32::from_rgb(148, 163, 184)));
                for key in ["X", "H", "C", "Z", "V"].iter() {
                    let is_sel = loot_tusu == *key;
                    if pill_toggle_button(ui, is_sel, key) {
                        loot_tusu = (*key).to_string();
                        if let Some(vk) = settings::key_to_vk(key) {
                            FAST_LOOT_VK.store(vk as usize, Ordering::Relaxed);
                        }
                        let secilen = loot_tusu.clone();
                        self.ayar_yaz(|cfg| cfg.loot_tusu = secilen);
                    }
                }
            });

            ui.add_space(3.0);
            ui.label(
                egui::RichText::new("ℹ️ Tuşa basılı tutarak sandıktaki veya ölü oyuncudaki eşyaları tek tıkla seri olarak çantanıza çeker.")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(100, 116, 139))
            );
        });

        ui.add_space(6.0);

        // 3. Gece Parlaklığı (Gamma Boost)
        let mut gamma_on = GAMMA_AKTIF.load(Ordering::Relaxed);
        card_frame().show(ui, |ui| {
            let mut g_level = self.ayarlar.lock().map(|cfg| cfg.gamma_seviyesi).unwrap_or(1.8);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🌙 GECE PARLAKLIĞI (GAMMA BOOST) [F6]").size(12.0).strong().color(egui::Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let gamma_lbl = if gamma_on { "AÇIK" } else { "KAPALI" };
                    if ui.checkbox(&mut gamma_on, gamma_lbl).changed() {
                        if display::set_display_color(gamma_on, g_level) {
                            GAMMA_AKTIF.store(gamma_on, Ordering::Relaxed);
                            self.gamma_hata = false;
                            self.ayar_yaz(|cfg| cfg.gamma_aktif = gamma_on);
                        } else {
                            gamma_on = !gamma_on;
                            self.gamma_hata = true;
                        }
                    }
                });
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Gamma Gücü:").size(12.0).color(egui::Color32::from_rgb(148, 163, 184)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let slider = egui::Slider::new(&mut g_level, 1.2..=2.8).step_by(0.1);
                    if ui.add(slider).changed() {
                        self.ayar_yaz(|cfg| cfg.gamma_seviyesi = g_level);
                        if gamma_on && !display::set_display_color(true, g_level) {
                            self.gamma_hata = true;
                        }
                    }
                });
            });

            if self.gamma_hata {
                ui.add_space(3.0);
                ui.label(
                    egui::RichText::new("Gamma rampası yazılamadı. HDR veya sürücü bu çağrıyı reddediyor olabilir.")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(239, 68, 68)),
                );
            }

            ui.add_space(3.0);
            ui.label(
                egui::RichText::new("ℹ️ Gece vakti ekran gamma eğrisini donanımsal artırarak düşmanları ve çevreyi net görmenizi sağlar.")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(100, 116, 139))
            );
        });

        ui.add_space(6.0);

        // 4. Mikro Piksel Algılayıcı (Auto Trigger)
        let mut trig_on = TRIGGER_ETKIN.load(Ordering::Relaxed);
        card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🎯 GELİŞMİŞ PİKSEL ALGILAYICI (AUTO TRIGGER v2)").size(12.0).strong().color(egui::Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let trig_lbl = if trig_on { "AÇIK" } else { "KAPALI" };
                    if ui.checkbox(&mut trig_on, trig_lbl).changed() {
                        TRIGGER_ETKIN.store(trig_on, Ordering::Relaxed);
                        if let Ok(mut cfg) = self.ayarlar.lock() { cfg.trigger_etkin = trig_on; }
                    }
                });
            });

            ui.add_space(4.0);

            let mut cfg = self.ayarlar.lock().map(|c| c.clone()).unwrap_or_default();
            let mut ayar_degisti = false;
            {
                // Tolerans Eşiği
                let mut sens = cfg.trigger_sensitivity;
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Tolerans Eşiği (Düşük = Çok Hassas):").size(12.0).color(egui::Color32::from_rgb(148, 163, 184)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let slider = egui::Slider::new(&mut sens, 10..=80);
                        if ui.add(slider).changed() {
                            TRIGGER_HASSASIYET.store(sens, Ordering::Relaxed);
                            cfg.trigger_sensitivity = sens;
                            ayar_degisti = true;
                        }
                    });
                });

                ui.add_space(2.0);

                // Tarama Alanı (NxN)
                let mut tarama = cfg.trigger_tarama_alan;
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("Tarama Alanı ({}x{} piksel):", tarama, tarama)).size(12.0).color(egui::Color32::from_rgb(148, 163, 184)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let slider = egui::Slider::new(&mut tarama, 1..=21).step_by(2.0);
                        if ui.add(slider).changed() {
                            // Tek sayı olmasını garanti et
                            if tarama % 2 == 0 { tarama += 1; }
                            TRIGGER_TARAMA_ALAN.store(tarama, Ordering::Relaxed);
                            cfg.trigger_tarama_alan = tarama;
                            ayar_degisti = true;
                        }
                    });
                });

                ui.add_space(2.0);

                // Onay Frame Sayısı
                let mut onay = cfg.trigger_onay_frame;
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Onay Frame (Yüksek = Az False-Positive):").size(12.0).color(egui::Color32::from_rgb(148, 163, 184)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let slider = egui::Slider::new(&mut onay, 1..=8);
                        if ui.add(slider).changed() {
                            TRIGGER_ONAY_FRAME.store(onay, Ordering::Relaxed);
                            cfg.trigger_onay_frame = onay;
                            ayar_degisti = true;
                        }
                    });
                });

                ui.add_space(2.0);

                // Humanize Gecikme
                let mut h_min = cfg.trigger_humanize_min;
                let mut h_max = cfg.trigger_humanize_max;
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("Humanize Gecikme: {}ms - {}ms", h_min, h_max)).size(12.0).color(egui::Color32::from_rgb(148, 163, 184)));
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("  Min:").size(11.0).color(egui::Color32::from_rgb(120, 135, 155)));
                    let slider_min = egui::Slider::new(&mut h_min, 0..=100).suffix("ms");
                    if ui.add(slider_min).changed() {
                        if h_min > h_max { h_max = h_min; }
                        TRIGGER_HUMANIZE_MIN.store(h_min, Ordering::Relaxed);
                        TRIGGER_HUMANIZE_MAX.store(h_max, Ordering::Relaxed);
                        cfg.trigger_humanize_min = h_min;
                        cfg.trigger_humanize_max = h_max;
                        ayar_degisti = true;
                    }
                    ui.label(egui::RichText::new("  Max:").size(11.0).color(egui::Color32::from_rgb(120, 135, 155)));
                    let slider_max = egui::Slider::new(&mut h_max, 0..=200).suffix("ms");
                    if ui.add(slider_max).changed() {
                        if h_max < h_min { h_min = h_max; }
                        TRIGGER_HUMANIZE_MIN.store(h_min, Ordering::Relaxed);
                        TRIGGER_HUMANIZE_MAX.store(h_max, Ordering::Relaxed);
                        cfg.trigger_humanize_min = h_min;
                        cfg.trigger_humanize_max = h_max;
                        ayar_degisti = true;
                    }
                });

                ui.add_space(2.0);

                // Renk Filtresi
                let renk_secenekler = ["Kapalı", "Kırmızı", "Yeşil", "Mavi", "Sarı"];
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Renk Filtresi:").size(12.0).color(egui::Color32::from_rgb(148, 163, 184)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        for (i, isim) in renk_secenekler.iter().enumerate().rev() {
                            let renk_aktif = cfg.trigger_renk_filtre == i;
                            let renk_renk = match i {
                                1 => egui::Color32::from_rgb(239, 68, 68),   // Kırmızı
                                2 => egui::Color32::from_rgb(34, 197, 94),    // Yeşil
                                3 => egui::Color32::from_rgb(59, 130, 246),   // Mavi
                                4 => egui::Color32::from_rgb(234, 179, 8),    // Sarı
                                _ => egui::Color32::from_rgb(148, 163, 184),  // Kapalı
                            };
                            let btn_bg = if renk_aktif {
                                egui::Color32::from_rgb(88, 101, 242)
                            } else {
                                egui::Color32::from_rgb(33, 37, 49)
                            };
                            let btn = egui::Button::new(
                                egui::RichText::new(*isim).size(10.0).strong().color(if renk_aktif { egui::Color32::WHITE } else { renk_renk })
                            )
                            .fill(btn_bg)
                            .corner_radius(egui::CornerRadius::same(4))
                            .min_size(egui::vec2(24.0, 18.0));
                            if ui.add(btn).clicked() {
                                cfg.trigger_renk_filtre = i;
                                TRIGGER_RENK_FILTRE.store(i, Ordering::Relaxed);
                                ayar_degisti = true;
                            }
                        }
                    });
                });

                ui.add_space(2.0);

                // Sprey Süresi
                let mut burst = cfg.trigger_burst_suresi;
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Min. Sprey Süresi:").size(12.0).color(egui::Color32::from_rgb(148, 163, 184)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let slider = egui::Slider::new(&mut burst, 50..=1000).step_by(50.0).suffix("ms");
                        if ui.add(slider).changed() {
                            TRIGGER_SPREY_SURESI.store(burst, Ordering::Relaxed);
                            cfg.trigger_burst_suresi = burst;
                            ayar_degisti = true;
                        }
                    });
                });
                if ayar_degisti {
                    self.ayar_yaz(|hedef| {
                        hedef.trigger_sensitivity = cfg.trigger_sensitivity;
                        hedef.trigger_tarama_alan = cfg.trigger_tarama_alan;
                        hedef.trigger_onay_frame = cfg.trigger_onay_frame;
                        hedef.trigger_humanize_min = cfg.trigger_humanize_min;
                        hedef.trigger_humanize_max = cfg.trigger_humanize_max;
                        hedef.trigger_renk_filtre = cfg.trigger_renk_filtre;
                        hedef.trigger_burst_suresi = cfg.trigger_burst_suresi;
                    });
                }
            }

            ui.add_space(3.0);
            ui.label(
                egui::RichText::new("ℹ️ Dürbün (Sağ tık) açıkken crosshair çevresinde çoklu piksel tarar. Onay frame ile false-positive azalır, humanize ile doğal gecikme eklenir.")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(100, 116, 139))
            );
        });

        ui.add_space(6.0);

        // 5. Keep-Alive worker
        let mut afk_on = ANTI_AFK_ETKIN.load(Ordering::Relaxed);
        card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⏳ GEFORCE NOW ANTİ-AFK").size(12.0).strong().color(egui::Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let afk_lbl = if afk_on { "AÇIK" } else { "KAPALI" };
                    if ui.checkbox(&mut afk_on, afk_lbl).changed() {
                        ANTI_AFK_ETKIN.store(afk_on, Ordering::Relaxed);
                        if let Ok(mut cfg) = self.ayarlar.lock() { cfg.anti_afk = afk_on; }
                    }
                });
            });

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("ℹ️ Uzun süre hareketsiz kaldığınızda GeForce Now üzerinden atılmamak için 2.5 dakikada bir farenizi milimetrik hareket ettirir.")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(100, 116, 139))
            );
        });

        ui.add_space(8.0);
        let save_btn = egui::Button::new(
            egui::RichText::new("💾 Ayarları Kaydet")
                .size(12.0)
                .strong()
                .color(egui::Color32::WHITE),
        )
        .fill(egui::Color32::from_rgb(88, 101, 242))
        .corner_radius(egui::CornerRadius::same(6))
        .min_size(egui::vec2(130.0, 28.0));

        if ui.add(save_btn).clicked() {
            self.kaydet();
        }
    }

    // ─── SEKME 2: MERMİ DÜŞME TABLOSU ────────────────────────────────────────
    fn render_tab_diagnostics(&self, ui: &mut egui::Ui) {
        card_frame().show(ui, |ui| {
            ui.label(
                egui::RichText::new("🎯 RUST SİLAHLARI MERMİ DÜŞME (BULLET DROP) DEĞERLERİ")
                    .size(12.0)
                    .strong()
                    .color(egui::Color32::WHITE),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Sabit referans tablosu. Değerler oyundan okunmaz; yama sonrası elle güncellenmesi gerekir.")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(148, 163, 184)),
            );
            ui.add_space(6.0);

            egui::Grid::new("bullet_drop_grid_modern")
                .striped(true)
                .min_col_width(58.0)
                .spacing(egui::vec2(8.0, 6.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Silah")
                            .strong()
                            .color(egui::Color32::from_rgb(148, 163, 184)),
                    );
                    ui.label(
                        egui::RichText::new("Mermi")
                            .strong()
                            .color(egui::Color32::from_rgb(148, 163, 184)),
                    );
                    ui.label(
                        egui::RichText::new("Namlu Hızı")
                            .strong()
                            .color(egui::Color32::from_rgb(148, 163, 184)),
                    );
                    ui.label(
                        egui::RichText::new("50m")
                            .strong()
                            .color(egui::Color32::from_rgb(148, 163, 184)),
                    );
                    ui.label(
                        egui::RichText::new("100m")
                            .strong()
                            .color(egui::Color32::from_rgb(148, 163, 184)),
                    );
                    ui.label(
                        egui::RichText::new("150m")
                            .strong()
                            .color(egui::Color32::from_rgb(148, 163, 184)),
                    );
                    ui.label(
                        egui::RichText::new("200m")
                            .strong()
                            .color(egui::Color32::from_rgb(148, 163, 184)),
                    );
                    ui.end_row();

                    let veriler = [
                        ("AK-47", "5.56", "375 m/s", "0 cm", "14 cm", "35 cm", "68 cm"),
                        ("MP5", "9mm", "240 m/s", "3 cm", "32 cm", "80 cm", "160 cm"),
                        ("Thompson", "9mm", "240 m/s", "3 cm", "32 cm", "80 cm", "160 cm"),
                        ("HMLMG", "5.56", "488 m/s", "0 cm", "8 cm", "20 cm", "38 cm"),
                        ("LR-300", "5.56", "375 m/s", "0 cm", "14 cm", "35 cm", "68 cm"),
                        ("Custom SMG", "9mm", "240 m/s", "3 cm", "32 cm", "80 cm", "160 cm"),
                        ("SAR", "5.56", "375 m/s", "0 cm", "14 cm", "35 cm", "68 cm"),
                        ("M249", "5.56", "488 m/s", "0 cm", "8 cm", "20 cm", "38 cm"),
                        ("M39", "5.56", "469 m/s", "0 cm", "9 cm", "22 cm", "42 cm"),
                        ("Bolt Action", "5.56", "656 m/s", "0 cm", "4 cm", "11 cm", "22 cm"),
                        ("L96 Sniper", "5.56", "1125 m/s", "0 cm", "1 cm", "3 cm", "7 cm"),
                    ];

                    for (task, mermi, hiz, d50, d100, d150, d200) in veriler.iter() {
                        ui.label(
                            egui::RichText::new(*task)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );
                        ui.label(
                            egui::RichText::new(*mermi)
                                .color(egui::Color32::from_rgb(160, 174, 192)),
                        );
                        ui.label(
                            egui::RichText::new(*hiz).color(egui::Color32::from_rgb(56, 189, 248)),
                        );
                        ui.label(
                            egui::RichText::new(*d50).color(egui::Color32::from_rgb(203, 213, 225)),
                        );
                        ui.label(
                            egui::RichText::new(*d100)
                                .color(egui::Color32::from_rgb(203, 213, 225)),
                        );
                        ui.label(
                            egui::RichText::new(*d150).color(egui::Color32::from_rgb(251, 146, 60)),
                        );
                        ui.label(
                            egui::RichText::new(*d200)
                                .color(egui::Color32::from_rgb(239, 68, 68))
                                .strong(),
                        );
                        ui.end_row();
                    }
                });
        });

        ui.add_space(6.0);

        // İpuçları Kartı
        card_frame().show(ui, |ui| {
            ui.label(egui::RichText::new("💡 BALİSTİK & NİŞAN İPUÇLARI").size(11.0).strong().color(egui::Color32::from_rgb(250, 204, 21)));
            ui.add_space(4.0);
            ui.label(egui::RichText::new("• HV (High Velocity) Mermi: Mermi hızını %20 artırır, düşmeyi ciddi oranda sıfıra yaklaştırır.").size(11.0).color(egui::Color32::from_rgb(203, 213, 225)));
            ui.label(egui::RichText::new("• Susturucu (Silencer): Mermi hızını %20 düşürür, daha yukarı nişan almanız gerekir.").size(11.0).color(egui::Color32::from_rgb(203, 213, 225)));
            ui.label(egui::RichText::new("• 8x / 16x Dürbün: Uzak mesafelerde düşme payını artı çizgilerindeki alt çentiklerle dengeleyin.").size(11.0).color(egui::Color32::from_rgb(203, 213, 225)));
        });
    }

    // ─── SEKME 3: SİSTEM & KISAYOL AYARLARI ──────────────────────────────────
    fn cisim_izini_guncelle(&mut self) {
        if !self.cisim_deneme {
            return;
        }
        let simdi = Instant::now();
        let dt = self
            .cisim_zaman
            .map(|onceki| simdi.duration_since(onceki).as_secs_f64().clamp(0.001, 0.25))
            .unwrap_or(0.033);
        self.cisim_zaman = Some(simdi);

        if self.cisim_capture.is_none() {
            self.cisim_capture = os_bridge::CenterCapture::new();
        }

        const BOYUT: usize = 64;
        self.cisim_kare_var = false;
        self.cisim_ham = None;
        if let Some(capture) = self.cisim_capture.as_mut() {
            let kare = if self.cisim_fare {
                os_bridge::cursor_screen_pos()
                    .and_then(|(x, y)| capture.capture_at(x, y, BOYUT as u32))
            } else {
                capture.capture_center(BOYUT as u32)
            };
            if let Some(colors) = kare {
                self.cisim_rgb.clear();
                self.cisim_rgb
                    .extend(colors.iter().flat_map(|(r, g, b)| [*r, *g, *b]));
                self.cisim_kare_var = self.cisim_rgb.len() == BOYUT * BOYUT * 3;
                if self.cisim_kare_var {
                    self.cisim_ham = crate::track::scan_contrast(colors, BOYUT, BOYUT);
                }
            }
        }
        self.cisim_iz = self.cisim_track.update(self.cisim_ham, dt);
        if self.cisim_fare && !ATES_AKTIF.load(Ordering::Relaxed) {
            let (dx, dy) = crate::track::fare_adimi(self.cisim_iz, BOYUT as f64);
            if dx != 0 || dy != 0 {
                os_bridge::move_mouse(dx, dy);
            }
        }
    }

    fn cisim_denemeyi_kapat(&mut self) {
        self.cisim_deneme = false;
        self.cisim_fare = false;
        self.cisim_capture = None;
        self.cisim_track = crate::track::ObjectTrack::new(0.033, 2);
        self.cisim_zaman = None;
        self.cisim_rgb.clear();
        self.cisim_ham = None;
        self.cisim_iz = crate::track::TrackView {
            x: 0.0,
            y: 0.0,
            live: false,
        };
        self.cisim_kare_var = false;
        self.cisim_texture = None;
    }

    fn render_tab_system(&mut self, ui: &mut egui::Ui) {
        // 1. Arayüz & OSD Ayarları
        let mut hud_on = MINI_HUD_ETKIN.load(Ordering::Relaxed);
        card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🖥️ MİNİ OSD DURUM PANELİ (HUD)").size(12.0).strong().color(egui::Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let hud_lbl = if hud_on { "AÇIK" } else { "KAPALI" };
                    if ui.checkbox(&mut hud_on, hud_lbl).changed() {
                        MINI_HUD_ETKIN.store(hud_on, Ordering::Relaxed);
                        if let Ok(mut cfg) = self.ayarlar.lock() { cfg.mini_hud = hud_on; }
                    }
                });
            });
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("Arayüzün üst barında ve oyun içi kontrollerde seçili task ve tetik modunu canlı olarak gösterir.")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(100, 116, 139))
            );
        });

        ui.add_space(6.0);

        // 2. Menü Gizleme / Çağırma Tuşu
        card_frame().show(ui, |ui| {
            ui.label(egui::RichText::new("⌨️ MENÜ GİZLEME / ARKA PLANA ALMA TUŞU").size(12.0).strong().color(egui::Color32::WHITE));
            ui.add_space(4.0);

            let mut menu_tusu = self
                .ayarlar
                .lock()
                .map(|cfg| cfg.menu_tusu.clone())
                .unwrap_or_else(|_| "INSERT".to_string());
            ui.horizontal(|ui| {
                for key in ["INSERT", "DELETE", "HOME", "END", "F12", "PAUSE", "NUMPAD0"].iter() {
                    let is_sel = menu_tusu == *key;
                    if pill_toggle_button(ui, is_sel, key) {
                        menu_tusu = (*key).to_string();
                        if let Some(vk) = settings::key_to_vk(key) {
                            MENU_TOGGLE_VK.store(vk as usize, Ordering::Relaxed);
                        }
                        let secilen = menu_tusu.clone();
                        self.ayar_yaz(|cfg| cfg.menu_tusu = secilen);
                    }
                }
            });

            ui.add_space(3.0);
            ui.label(
                egui::RichText::new("ℹ️ Rust oyunundayken bu tuşa basarak menüyü anında gizleyebilir veya geri ekrana getirebilirsiniz.")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(100, 116, 139))
            );
        });

        ui.add_space(6.0);

        card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("🔍 CİSİM İZİ DENEMESİ")
                        .size(12.0)
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut acik = self.cisim_deneme;
                    let yazi = if acik { "AÇIK" } else { "KAPALI" };
                    if ui.checkbox(&mut acik, yazi).changed() {
                        if acik {
                            self.cisim_deneme = true;
                        } else {
                            self.cisim_denemeyi_kapat();
                        }
                    }
                });
            });
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(
                    "Parlak bir adanın merkezini gösterir. Fare kaydırma açıkken kare imlecin etrafından alınır ve imleç adaya doğru gider.",
                )
                .size(11.0)
                .color(egui::Color32::from_rgb(100, 116, 139)),
            );

            if self.cisim_deneme {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let mut fare = self.cisim_fare;
                    if ui.checkbox(&mut fare, "Fareyi adaya kaydır").changed() {
                        self.cisim_fare = fare;
                    }
                });
                ui.add_space(6.0);
                if self.cisim_kare_var {
                    let image = egui::ColorImage::from_rgb([64, 64], &self.cisim_rgb);
                    if let Some(texture) = &mut self.cisim_texture {
                        texture.set(image, egui::TextureOptions::NEAREST);
                    } else {
                        self.cisim_texture = Some(ui.ctx().load_texture(
                            "cisim-onizleme",
                            image,
                            egui::TextureOptions::NEAREST,
                        ));
                    }
                    if let Some(texture) = &self.cisim_texture {
                        ui.image((texture.id(), egui::vec2(128.0, 128.0)));
                    }
                    let ham = self
                        .cisim_ham
                        .map(|hit| {
                            format!(
                                "Ham merkez: {:.1}, {:.1}  ({} piksel)",
                                hit.x, hit.y, hit.pixels
                            )
                        })
                        .unwrap_or_else(|| "Ham merkez: ada yok".to_string());
                    let iz = if self.cisim_iz.live {
                        format!("Kalman izi: {:.1}, {:.1}", self.cisim_iz.x, self.cisim_iz.y)
                    } else {
                        "Kalman izi: kapalı".to_string()
                    };
                    ui.label(egui::RichText::new(ham).size(12.0).color(egui::Color32::from_rgb(203, 213, 225)));
                    ui.label(
                        egui::RichText::new(iz)
                            .size(12.0)
                            .color(egui::Color32::from_rgb(56, 189, 248)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("Ekran karesi alınamadı.")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(239, 68, 68)),
                    );
                }
            }
        });

        ui.add_space(6.0);

        // 3. Kalıcı Ayarlar & Dosya Yönetimi
        card_frame().show(ui, |ui| {
            ui.label(egui::RichText::new("💾 YAPILANDIRMAYI KAYDET").size(12.0).strong().color(egui::Color32::WHITE));
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("Tüm task kalibrasyonları, tuş atamaları ve tercihleriniz ayarlar.json dosyasına kaydedilir.")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(148, 163, 184))
            );

            ui.add_space(8.0);

            let save_all_btn = egui::Button::new(
                egui::RichText::new("💾 Tüm Ayarları Kaydet").size(13.0).strong().color(egui::Color32::WHITE)
            )
            .fill(egui::Color32::from_rgb(88, 101, 242))
            .corner_radius(egui::CornerRadius::same(6))
            .min_size(egui::vec2(160.0, 32.0));

            if ui.add(save_all_btn).clicked() {
                self.kaydet();
            }
        });
    }

    // ─── Yardımcı: Silah Seçim Butonu ─────────────────────────────────────────
    fn select_taskim_butonu(
        &self,
        ui: &mut egui::Ui,
        secili: usize,
        id: usize,
        isim: &str,
        key: &str,
    ) -> bool {
        let is_active = secili == id;
        let (bg, stroke, text_color) = if is_active {
            (
                egui::Color32::from_rgb(38, 44, 68),
                egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(88, 101, 242)),
                egui::Color32::WHITE,
            )
        } else {
            (
                egui::Color32::from_rgb(20, 22, 29),
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(38, 42, 54)),
                egui::Color32::from_rgb(160, 174, 192),
            )
        };

        let label = format!("{} [{}]", isim, key);
        let btn = egui::Button::new(
            egui::RichText::new(label)
                .size(12.0)
                .color(text_color)
                .strong(),
        )
        .fill(bg)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(6))
        .min_size(egui::vec2(86.0, 28.0));

        ui.add(btn).clicked()
    }
}
