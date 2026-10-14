//! Ayar Yapıları ve Yapılandırma
//!
//! Silah ayarları, uygulama yapılandırması ve JSON serialization tanımları.
//! Tuş kodu dönüştürme yardımcı fonksiyonlarını da içerir.

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// ─── Varsayılan Değer Fonksiyonları (serde default) ─────────────────────────────
fn default_ilk5() -> f32 {
    1.2
}
fn default_gamma() -> f32 {
    1.8
}
fn default_bool_true() -> bool {
    true
}
fn default_menu_key() -> String {
    "INSERT".to_string()
}
fn default_action_key() -> String {
    "V".to_string()
}
fn default_med_slot() -> u8 {
    5
}
fn default_secondary_key() -> String {
    "X".to_string()
}
fn default_health_threshold() -> usize {
    35
}
fn default_scan_area() -> usize {
    5
}
fn default_confirm_frame() -> usize {
    2
}
fn default_jitter_min() -> usize {
    20
}
fn default_jitter_max() -> usize {
    80
}
fn default_burst_time() -> usize {
    300
}
fn default_hassasiyet() -> f32 {
    1.0
}
fn default_fov() -> f32 {
    90.0
}
fn default_adim_ms() -> u64 {
    22
}

// ─── Silah Ayarı ────────────────────────────────────────────────────────────────
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct ProcessSettings {
    pub dikey: f32, // Y çarpanı (aşağı çekiş gücü)
    pub yatay: f32, // X çarpanı (yatay çekiş gücü)
    #[serde(default = "default_ilk5")]
    pub ilk5_mermi_carpan: f32, // İlk 5 mermi ekstra dikey güç çarpanı
}

// ─── Tüm Ayarlar ───────────────────────────────────────────────────────────────
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct AppConfig {
    #[serde(rename = "ak47", alias = "prof_a")]
    pub prof_a: ProcessSettings,
    #[serde(rename = "mp5", alias = "prof_b")]
    pub prof_b: ProcessSettings,
    #[serde(rename = "thompson", alias = "prof_c")]
    pub prof_c: ProcessSettings,
    #[serde(rename = "hmlmg", alias = "prof_d")]
    pub prof_d: ProcessSettings,
    #[serde(rename = "lr300", alias = "prof_e")]
    pub prof_e: ProcessSettings,
    #[serde(rename = "custom_smg", alias = "prof_f")]
    pub prof_f: ProcessSettings,
    #[serde(rename = "sar", alias = "prof_g")]
    pub prof_g: ProcessSettings,
    #[serde(rename = "m249", alias = "prof_h")]
    pub prof_h: ProcessSettings,
    #[serde(rename = "m39", alias = "prof_i")]
    pub prof_i: ProcessSettings,
    #[serde(default)]
    pub tetik_modu: usize,
    #[serde(default = "default_gamma")]
    pub gamma_seviyesi: f32,
    #[serde(default = "default_bool_true")]
    pub mini_hud: bool,
    #[serde(default = "default_menu_key")]
    pub menu_tusu: String,
    #[serde(default)]
    pub fast_loot: bool,
    #[serde(default)]
    pub fast_med: bool,
    #[serde(default = "default_action_key")]
    pub med_tusu: String,
    #[serde(default = "default_med_slot")]
    pub med_slotu: u8,
    #[serde(default = "default_secondary_key")]
    pub loot_tusu: String,
    #[serde(default)]
    pub trigger_etkin: bool,
    #[serde(default = "default_health_threshold", alias = "trigger_hassasiyet")]
    pub trigger_sensitivity: usize,
    #[serde(default = "default_scan_area")]
    pub trigger_tarama_alan: usize,
    #[serde(default = "default_confirm_frame")]
    pub trigger_onay_frame: usize,
    #[serde(default = "default_jitter_min")]
    pub trigger_humanize_min: usize,
    #[serde(default = "default_jitter_max")]
    pub trigger_humanize_max: usize,
    #[serde(default)]
    pub trigger_renk_filtre: usize,
    #[serde(default = "default_burst_time")]
    pub trigger_burst_suresi: usize,
    #[serde(default)]
    pub anti_afk: bool,
    #[serde(default)]
    pub task_id: usize,
    #[serde(default)]
    pub gamma_aktif: bool,
    /// 1.0, paternin yazıldığı hassasiyettir. Düşük hassasiyet daha büyük fare adımı ister.
    #[serde(default = "default_hassasiyet")]
    pub hassasiyet: f32,
    /// 90, paternin yazıldığı görüş açısıdır.
    #[serde(default = "default_fov")]
    pub gorus_acisi: f32,
    /// Patern adımları arasındaki süre. 22 ms bugünkü tempo.
    #[serde(default = "default_adim_ms")]
    pub adim_ms: u64,
}

pub fn silah_adi(id: usize) -> &'static str {
    match id {
        0 => "AK-47",
        1 => "MP5",
        2 => "Thompson",
        3 => "HMLMG",
        4 => "LR-300",
        5 => "Custom SMG",
        6 => "SAR",
        7 => "M249",
        _ => "M39",
    }
}

/// Fare adımını oyundaki hassasiyet ve görüş açısına göre ölçekler.
///
/// Varsayılan 1.0 ve 90, mevcut paterni değiştirmez.
pub fn cekis_olcegi(hassasiyet: f32, gorus_acisi: f32) -> f64 {
    let sens = (hassasiyet as f64).clamp(0.1, 5.0);
    let fov = (gorus_acisi as f64).clamp(60.0, 110.0);
    (fov / 90.0) / sens
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            prof_a: ProcessSettings {
                dikey: 2.45,
                yatay: 1.85,
                ilk5_mermi_carpan: 1.25,
            },
            prof_b: ProcessSettings {
                dikey: 1.7,
                yatay: 1.8,
                ilk5_mermi_carpan: 1.2,
            },
            prof_c: ProcessSettings {
                dikey: 1.65,
                yatay: 1.7,
                ilk5_mermi_carpan: 1.15,
            },
            prof_d: ProcessSettings {
                dikey: 1.2,
                yatay: 0.6,
                ilk5_mermi_carpan: 1.4,
            },
            prof_e: ProcessSettings {
                dikey: 1.75,
                yatay: 1.55,
                ilk5_mermi_carpan: 1.2,
            },
            prof_f: ProcessSettings {
                dikey: 1.5,
                yatay: 1.5,
                ilk5_mermi_carpan: 1.1,
            },
            prof_g: ProcessSettings {
                dikey: 1.6,
                yatay: 1.4,
                ilk5_mermi_carpan: 1.0,
            },
            prof_h: ProcessSettings {
                dikey: 1.7,
                yatay: 1.5,
                ilk5_mermi_carpan: 1.5,
            },
            prof_i: ProcessSettings {
                dikey: 1.6,
                yatay: 1.4,
                ilk5_mermi_carpan: 1.2,
            },
            tetik_modu: 0,
            gamma_seviyesi: 1.8,
            mini_hud: true,
            menu_tusu: "INSERT".to_string(),
            fast_loot: false,
            fast_med: false,
            med_tusu: "V".to_string(),
            med_slotu: 5,
            loot_tusu: "X".to_string(),
            trigger_etkin: false,
            trigger_sensitivity: 35,
            trigger_tarama_alan: 5,
            trigger_onay_frame: 2,
            trigger_humanize_min: 20,
            trigger_humanize_max: 80,
            trigger_renk_filtre: 0,
            trigger_burst_suresi: 300,
            anti_afk: false,
            task_id: 0,
            gamma_aktif: false,
            hassasiyet: 1.0,
            gorus_acisi: 90.0,
            adim_ms: 22,
        }
    }
}

const GOMULU_AYARLAR: &str = include_str!("../ayarlar.json");

fn config_dir() -> PathBuf {
    let kok = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|yol| yol.parent().map(PathBuf::from))
        })
        .unwrap_or_else(|| PathBuf::from("."));
    kok.join("RustNorecoil")
}

fn dosya_var(yol: &Path) -> bool {
    yol.is_file()
}

fn exe_ustunde_ayarlar() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let mut dizin = exe.parent().map(Path::to_path_buf);
    for _ in 0..5 {
        let dir = dizin?;
        let aday = dir.join("ayarlar.json");
        if dosya_var(&aday) {
            return Some(aday);
        }
        dizin = dir.parent().map(Path::to_path_buf);
    }
    None
}

fn ayarlar_yolunu_bul() -> PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        let aday = cwd.join("ayarlar.json");
        if dosya_var(&aday) {
            return aday;
        }
    }
    if let Some(aday) = exe_ustunde_ayarlar() {
        return aday;
    }
    config_dir().join("ayarlar.json")
}

pub fn ayarlar_yolu() -> PathBuf {
    static YOL: OnceLock<PathBuf> = OnceLock::new();
    YOL.get_or_init(ayarlar_yolunu_bul).clone()
}

fn dosyadan_oku(yol: &PathBuf) -> Option<AppConfig> {
    File::open(yol)
        .ok()
        .and_then(|dosya| serde_json::from_reader(dosya).ok())
}

pub fn load_config() -> AppConfig {
    let yol = ayarlar_yolu();
    if let Some(ayarlar) = dosyadan_oku(&yol).or_else(|| dosyadan_oku(&yol.with_extension("json.bak"))) {
        return ayarlar;
    }

    // Okunamayan mevcut dosyanın üzerine yazma; kayıtlı kalibrasyon kaybolmasın.
    if yol.exists() || yol.with_extension("json.bak").exists() {
        return AppConfig::default();
    }

    let gomulu = serde_json::from_str(GOMULU_AYARLAR).unwrap_or_default();
    let _ = save_config(&gomulu);
    gomulu
}

pub fn save_config(ayarlar: &AppConfig) -> io::Result<()> {
    let path = ayarlar_yolu();
    if let Some(dizin) = path.parent() {
        fs::create_dir_all(dizin)?;
    }
    let temp = path.with_extension("json.tmp");
    let backup = path.with_extension("json.bak");

    let mut file = File::create(&temp)?;
    serde_json::to_writer_pretty(&mut file, ayarlar).map_err(io::Error::other)?;
    file.write_all(b"\n")?;
    file.sync_all()?;

    if path.exists() {
        let _ = fs::remove_file(&backup);
        fs::rename(&path, &backup)?;
    }

    if let Err(error) = fs::rename(&temp, &path) {
        if backup.exists() {
            let _ = fs::rename(&backup, &path);
        }
        return Err(error);
    }

    let _ = fs::remove_file(backup);
    Ok(())
}

// ─── Tuş Kodu Dönüştürme Yardımcıları ──────────────────────────────────────────

/// Tuş adını Windows Virtual Key koduna dönüştürür.
pub fn key_to_vk(key: &str) -> Option<i32> {
    Some(match key.to_uppercase().as_str() {
        "V" => 0x56,
        "C" => 0x43,
        "X" => 0x58,
        "Z" => 0x5A,
        "G" => 0x47,
        "H" => 0x48,
        "B" => 0x42,
        "F" => 0x46,
        "Q" => 0x51,
        "E" => 0x45,
        "R" => 0x52,
        "T" => 0x54,
        "F6" => 0x75,
        "F5" => 0x74,
        "F12" => 0x7B,
        "INSERT" => 0x2D,
        "DELETE" => 0x2E,
        "HOME" => 0x24,
        "END" => 0x23,
        "PAUSE" => 0x13,
        "NUMPAD0" => 0x60,
        _ => return None,
    })
}

/// Envanter slot numarasını Virtual Key koduna dönüştürür.
pub fn slot_to_vk(slot: u8) -> u8 {
    match slot {
        1 => 0x31,
        2 => 0x32,
        3 => 0x33,
        4 => 0x34,
        5 => 0x35,
        6 => 0x36,
        _ => 0x35,
    }
}

#[cfg(test)]
mod tests {
    use super::{cekis_olcegi, key_to_vk, slot_to_vk, AppConfig};

    #[test]
    fn tus_adlarini_buyuk_kucuk_harf_duyarsiz_esler() {
        assert_eq!(key_to_vk("insert"), Some(0x2D));
        assert_eq!(key_to_vk("f12"), Some(0x7B));
        assert_eq!(key_to_vk("v"), Some(0x56));
        assert_eq!(key_to_vk("yok"), None);
    }

    #[test]
    fn slot_araligini_vk_koduna_cevirir() {
        assert_eq!(slot_to_vk(1), 0x31);
        assert_eq!(slot_to_vk(6), 0x36);
        assert_eq!(slot_to_vk(99), 0x35);
    }

    #[test]
    fn eski_alan_adlari_yeni_semaya_duser() {
        let eski = r#"{
            "ak47": {"dikey": 2.45, "yatay": 1.85, "ilk5_mermi_carpan": 1.25},
            "mp5": {"dikey": 1.7, "yatay": 1.8, "ilk5_mermi_carpan": 1.2},
            "thompson": {"dikey": 1.65, "yatay": 1.7, "ilk5_mermi_carpan": 1.15},
            "hmlmg": {"dikey": 1.2, "yatay": 0.6, "ilk5_mermi_carpan": 1.4},
            "lr300": {"dikey": 1.75, "yatay": 1.55, "ilk5_mermi_carpan": 1.2},
            "custom_smg": {"dikey": 1.5, "yatay": 1.5, "ilk5_mermi_carpan": 1.1},
            "sar": {"dikey": 1.6, "yatay": 1.4, "ilk5_mermi_carpan": 1.0},
            "m249": {"dikey": 1.7, "yatay": 1.5, "ilk5_mermi_carpan": 1.5},
            "m39": {"dikey": 1.6, "yatay": 1.4, "ilk5_mermi_carpan": 1.2},
            "trigger_hassasiyet": 40
        }"#;
        let cfg: AppConfig = serde_json::from_str(eski).unwrap();
        assert_eq!(cfg.prof_a.dikey, 2.45);
        assert_eq!(cfg.prof_b.yatay, 1.8);
        assert_eq!(cfg.prof_g.ilk5_mermi_carpan, 1.0);
        assert_eq!(cfg.trigger_sensitivity, 40);
    }

    #[test]
    fn cekis_olcegi_varsayilanda_bir_kalir() {
        assert!((cekis_olcegi(1.0, 90.0) - 1.0).abs() < 1e-9);
        assert!((cekis_olcegi(0.5, 90.0) - 2.0).abs() < 1e-9);
        assert!((cekis_olcegi(1.0, 75.0) - (75.0 / 90.0)).abs() < 1e-9);
    }
}
