//! Global Durum Değişkenleri ve Sabit Tuş Kodları
//!
//! Tüm thread'ler arası paylaşılan atomic değişkenler ve
//! Windows Virtual Key sabitleri bu modülde tanımlanır.

use std::sync::atomic::{AtomicBool, AtomicUsize};

// ─── Sabit Tuş Kodları (Virtual Keys) ──────────────────────────────────────────
pub const VK_LBUTTON: i32 = 0x01;
pub const VK_RBUTTON: i32 = 0x02;
pub const VK_CAPITAL: i32 = 0x14; // Caps Lock
pub const VK_F1: i32 = 0x70;
pub const VK_F2: i32 = 0x71;
pub const VK_F3: i32 = 0x72;
pub const VK_F4: i32 = 0x73;
pub const VK_F5: i32 = 0x74;
pub const VK_F6: i32 = 0x75; // Gece Görüşü / Gamma
pub const VK_F7: i32 = 0x76;
pub const VK_F8: i32 = 0x77;
pub const VK_F9: i32 = 0x78;
pub const VK_F10: i32 = 0x79;
pub const VK_F11: i32 = 0x7A;

// ─── Global Durum Değişkenleri ────────────────────────────────────────────────
pub static ATES_AKTIF: AtomicBool = AtomicBool::new(false);
pub static ATES_ADIM: AtomicUsize = AtomicUsize::new(0);
pub static PERFORMANCE_AYAR_SURUMU: AtomicUsize = AtomicUsize::new(0);
pub static PANEL_GORUNUR: AtomicBool = AtomicBool::new(true);
pub static KAPATILIYOR: AtomicBool = AtomicBool::new(false);

// Gamma & Hızlı Loot & Hızlı Med & Trigger Durumları
pub static GAMMA_AKTIF: AtomicBool = AtomicBool::new(false);
pub static MINI_HUD_ETKIN: AtomicBool = AtomicBool::new(true);
pub static MENU_TOGGLE_VK: AtomicUsize = AtomicUsize::new(0x2D); // Varsayılan: INSERT
pub static FAST_LOOT_ETKIN: AtomicBool = AtomicBool::new(false);
pub static FAST_MED_ETKIN: AtomicBool = AtomicBool::new(false);
pub static FAST_MED_VK: AtomicUsize = AtomicUsize::new(0x56); // Varsayılan: 'V'
pub static FAST_MED_SLOT: AtomicUsize = AtomicUsize::new(5); // Varsayılan: Slot 5
pub static FAST_LOOT_VK: AtomicUsize = AtomicUsize::new(0x58); // Varsayılan: 'X'
pub static TRIGGER_ETKIN: AtomicBool = AtomicBool::new(false);
pub static TRIGGER_HASSASIYET: AtomicUsize = AtomicUsize::new(35); // RGB fark eşiği
pub static TRIGGER_TARAMA_ALAN: AtomicUsize = AtomicUsize::new(5); // NxN piksel tarama alanı
pub static TRIGGER_ONAY_FRAME: AtomicUsize = AtomicUsize::new(2); // Ardışık N frame onay
pub static TRIGGER_HUMANIZE_MIN: AtomicUsize = AtomicUsize::new(20); // Min reaksiyon gecikmesi (ms)
pub static TRIGGER_HUMANIZE_MAX: AtomicUsize = AtomicUsize::new(80); // Max reaksiyon gecikmesi (ms)
pub static TRIGGER_RENK_FILTRE: AtomicUsize = AtomicUsize::new(0); // 0=Kapalı, 1=Kırmızı, 2=Yeşil, 3=Mavi, 4=Sarı
pub static TRIGGER_SPREY_SURESI: AtomicUsize = AtomicUsize::new(300); // Minimum burst süresi (ms)
pub static ANTI_AFK_ETKIN: AtomicBool = AtomicBool::new(false);

// Silah seçimi: 0=AK-47 1=MP5 2=Thompson 3=HMLMG 4=LR-300 5=Custom SMG 6=SAR 7=M249 8=M39
pub static SILAH_ID: AtomicUsize = AtomicUsize::new(0);

// Tetikleme Modu: 0 = Sağ Tık + Sol Tık, 1 = Sadece Sol Tık
pub static TETIK_MODU: AtomicUsize = AtomicUsize::new(0);

use std::sync::atomic::Ordering;

use crate::settings::{self, AppConfig};

/// Kayıtlı ayarları iş parçacıklarının okuduğu atomiklere yazar.
///
/// Bilinmeyen bir tuş adı ilgili atomiği değiştirmez.
pub fn load_from_settings(cfg: &AppConfig) {
    TETIK_MODU.store(cfg.tetik_modu, Ordering::Relaxed);
    MINI_HUD_ETKIN.store(cfg.mini_hud, Ordering::Relaxed);
    if let Some(vk) = settings::key_to_vk(&cfg.menu_tusu) {
        MENU_TOGGLE_VK.store(vk as usize, Ordering::Relaxed);
    }
    FAST_LOOT_ETKIN.store(cfg.fast_loot, Ordering::Relaxed);
    FAST_MED_ETKIN.store(cfg.fast_med, Ordering::Relaxed);
    if let Some(vk) = settings::key_to_vk(&cfg.med_tusu) {
        FAST_MED_VK.store(vk as usize, Ordering::Relaxed);
    }
    FAST_MED_SLOT.store(cfg.med_slotu as usize, Ordering::Relaxed);
    if let Some(vk) = settings::key_to_vk(&cfg.loot_tusu) {
        FAST_LOOT_VK.store(vk as usize, Ordering::Relaxed);
    }
    TRIGGER_ETKIN.store(cfg.trigger_etkin, Ordering::Relaxed);
    TRIGGER_HASSASIYET.store(cfg.trigger_sensitivity, Ordering::Relaxed);
    TRIGGER_TARAMA_ALAN.store(cfg.trigger_tarama_alan, Ordering::Relaxed);
    TRIGGER_ONAY_FRAME.store(cfg.trigger_onay_frame, Ordering::Relaxed);
    TRIGGER_HUMANIZE_MIN.store(cfg.trigger_humanize_min, Ordering::Relaxed);
    TRIGGER_HUMANIZE_MAX.store(cfg.trigger_humanize_max, Ordering::Relaxed);
    TRIGGER_RENK_FILTRE.store(cfg.trigger_renk_filtre, Ordering::Relaxed);
    TRIGGER_SPREY_SURESI.store(cfg.trigger_burst_suresi, Ordering::Relaxed);
    ANTI_AFK_ETKIN.store(cfg.anti_afk, Ordering::Relaxed);
    SILAH_ID.store(cfg.task_id, Ordering::Relaxed);
    GAMMA_AKTIF.store(cfg.gamma_aktif, Ordering::Relaxed);
}
