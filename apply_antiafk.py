import os

path = "src/main.rs"
with open(path, "r", encoding="utf-8") as f:
    content = f.read()

# 1. Global Durum
content = content.replace(
    "static TRIGGER_HASSASIYET: AtomicUsize = AtomicUsize::new(35);   // RGB fark eşiği",
    "static TRIGGER_HASSASIYET: AtomicUsize = AtomicUsize::new(35);   // RGB fark eşiği\nstatic ANTI_AFK_ETKIN:      AtomicBool  = AtomicBool::new(false);"
)

# 2. TumAyarlar Struct
content = content.replace(
    "    #[serde(default = \"default_trigger_hassasiyet\")]\n    trigger_hassasiyet: usize,\n}",
    "    #[serde(default = \"default_trigger_hassasiyet\")]\n    trigger_hassasiyet: usize,\n    #[serde(default)]\n    anti_afk: bool,\n}"
)

# 3. Default for TumAyarlar
content = content.replace(
    "            trigger_etkin: false,\n            trigger_hassasiyet: 35,\n        }\n    }\n}",
    "            trigger_etkin: false,\n            trigger_hassasiyet: 35,\n            anti_afk: false,\n        }\n    }\n}"
)

# 4. App::new
content = content.replace(
    "            TRIGGER_HASSASIYET.store(cfg.trigger_hassasiyet, Ordering::Relaxed);\n        }\n        Self { recoil_aktif, ayarlar, aktif_sekme: 0, son_kayit: None }\n    }",
    "            TRIGGER_HASSASIYET.store(cfg.trigger_hassasiyet, Ordering::Relaxed);\n            ANTI_AFK_ETKIN.store(cfg.anti_afk, Ordering::Relaxed);\n        }\n        Self { recoil_aktif, ayarlar, aktif_sekme: 0, son_kayit: None }\n    }"
)

# 5. App::kaydet
content = content.replace(
    "            TRIGGER_HASSASIYET.store(cfg.trigger_hassasiyet, Ordering::Relaxed);\n\n            if let Ok(file) = File::create(\"ayarlar.json\") {",
    "            TRIGGER_HASSASIYET.store(cfg.trigger_hassasiyet, Ordering::Relaxed);\n            ANTI_AFK_ETKIN.store(cfg.anti_afk, Ordering::Relaxed);\n\n            if let Ok(file) = File::create(\"ayarlar.json\") {"
)

# 6. UI Eklenmesi
ui_ek = """                            });

                            ui.add_space(6.0);

                            // 5. Anti-AFK (GeForce Now)
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
                            let save_btn = egui::Button::new("""
content = content.replace(
    "                            });\n\n                            ui.add_space(8.0);\n                            let save_btn = egui::Button::new(",
    ui_ek
)

# 7. Thread Eklenmesi
thread_ek = """        }
    });

    // Thread 5: Anti-AFK (GeForce Now) Thread
    thread::spawn(move || {
        let mut sayac = 0;
        loop {
            if ANTI_AFK_ETKIN.load(Ordering::Relaxed) {
                sayac += 1;
                if sayac >= 150 { // 150 saniye = 2.5 dakika
                    unsafe {
                        mouse_event(MOUSEEVENTF_MOVE, 1, 1, 0, 0);
                        thread::sleep(std::time::Duration::from_millis(50));
                        mouse_event(MOUSEEVENTF_MOVE, -1, -1, 0, 0);
                    }
                    sayac = 0;
                }
            } else {
                sayac = 0;
            }
            thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    // GUI Başlat"""
content = content.replace(
    "        }\n    });\n\n    // GUI Başlat",
    thread_ek
)

with open(path, "w", encoding="utf-8") as f:
    f.write(content)

print("Modifications applied successfully.")
