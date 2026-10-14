//! Klavye/Fare Giriş Dinleyici ve Arka Plan İşçi Thread'leri
//!
//! Ateş tetikleme, task değiştirme (F1-F11), recoil toggle (CapsLock/F5),
//! menü gizleme, gamma toggle (F6), hızlı şırınga, hızlı loot ve anti-AFK
//! döngülerini yönetir.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::settings::{slot_to_vk, AppConfig};
use crate::display;
use crate::state::*;
use crate::os_bridge;

static MED_ISLEM_AKTIF: AtomicBool = AtomicBool::new(false);
static MED_THREAD: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(None);

/// Kapanışta süren şırınga dizisini bekler.
pub fn wait_for_action() {
    let handle = MED_THREAD.lock().ok().and_then(|mut slot| slot.take());
    if let Some(handle) = handle {
        let _ = handle.join();
    }
}

struct MedTemizlik {
    slot_vk: u8,
}

impl Drop for MedTemizlik {
    fn drop(&mut self) {
        os_bridge::mouse_right_up();
        os_bridge::release_key(self.slot_vk);
        os_bridge::release_key(0x31);
        MED_ISLEM_AKTIF.store(false, Ordering::Release);
    }
}

fn short_wait(ms: u64) -> bool {
    let mut kalan = ms;
    while kalan > 0 {
        if KAPATILIYOR.load(Ordering::Relaxed) {
            return false;
        }
        let dilim = kalan.min(5);
        os_bridge::short_sleep(Duration::from_millis(dilim));
        kalan -= dilim;
    }
    !KAPATILIYOR.load(Ordering::Relaxed)
}

/// Klavye/fare dinleyici ana döngüsü (Thread 1).
///
/// 8ms döngü hızında çalışır. Tüm kısayol tuşlarını ve fare olaylarını izler.
pub fn event_loop(performance_mode: Arc<AtomicBool>, ayarlar: Arc<Mutex<AppConfig>>) {
    let mut prev_toggle_pressed = false;
    let mut prev_insert_pressed = false;
    let mut prev_gamma_pressed = false;
    let mut prev_med_pressed = false;
    let mut prev_task_pressed = [false; 9];

    while !KAPATILIYOR.load(Ordering::Relaxed) {
        let sag = os_bridge::is_key_pressed(VK_RBUTTON);
        let sol = os_bridge::is_key_pressed(VK_LBUTTON);

        // ─── Ateş Tetikleme ───────────────────────────────────────────────────
        let tetik_mode = TETIK_MODU.load(Ordering::Relaxed);
        let ates_tetiklendi = if tetik_mode == 0 { sag && sol } else { sol };

        if ates_tetiklendi {
            if !ATES_AKTIF.load(Ordering::Relaxed) {
                ATES_AKTIF.store(true, Ordering::Relaxed);
            }
        } else if ATES_AKTIF.load(Ordering::Relaxed) {
            ATES_AKTIF.store(false, Ordering::Relaxed);
            ATES_ADIM.store(0, Ordering::Relaxed);
        }

        // ─── Açma / Kapatma Kısayol Tuşları (CAPS LOCK, F5) ──────────────────
        let toggle_pressed = os_bridge::is_key_pressed(VK_CAPITAL) || os_bridge::is_key_pressed(VK_F5);
        if toggle_pressed && !prev_toggle_pressed {
            let mevcut = performance_mode.load(Ordering::Relaxed);
            performance_mode.store(!mevcut, Ordering::Relaxed);
        }
        prev_toggle_pressed = toggle_pressed;

        // ─── Menü Gizleme / Gösterme Kısayolu — Dinamik Tuş Ataması ──────────
        let menu_vk = MENU_TOGGLE_VK.load(Ordering::Relaxed) as i32;
        let insert_pressed = os_bridge::is_key_pressed(menu_vk);
        if insert_pressed && !prev_insert_pressed {
            let cur = PANEL_GORUNUR.load(Ordering::Relaxed);
            let yeni = !cur;
            PANEL_GORUNUR.store(yeni, Ordering::Relaxed);
            os_bridge::toggle_window(b"System Diagnostic Tool\0", yeni);
        }
        prev_insert_pressed = insert_pressed;

        // ─── Gece Görüşü / Gamma Toggle (F6) ─────────────────────────────────
        let gamma_pressed = os_bridge::is_key_pressed(VK_F6);
        if gamma_pressed && !prev_gamma_pressed {
            let cur = GAMMA_AKTIF.load(Ordering::Relaxed);
            let yeni = !cur;
            GAMMA_AKTIF.store(yeni, Ordering::Relaxed);
            let level = if let Ok(mut cfg) = ayarlar.lock() {
                cfg.gamma_aktif = yeni;
                cfg.gamma_seviyesi
            } else {
                1.8
            };
            if !display::set_display_color(yeni, level) {
                GAMMA_AKTIF.store(!yeni, Ordering::Relaxed);
                if let Ok(mut cfg) = ayarlar.lock() {
                    cfg.gamma_aktif = !yeni;
                }
            }
        }
        prev_gamma_pressed = gamma_pressed;

        // ─── Hızlı Şırınga / Bandaj (Fast Med) ──────────────────────────────
        let med_vk = FAST_MED_VK.load(Ordering::Relaxed) as i32;
        let med_pressed = os_bridge::is_key_pressed(med_vk);
        if med_pressed
            && !prev_med_pressed
            && FAST_MED_ETKIN.load(Ordering::Relaxed)
            && MED_ISLEM_AKTIF
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
        {
            let slot = FAST_MED_SLOT.load(Ordering::Relaxed) as u8;
            let handle = thread::spawn(move || {
                let slot_vk = slot_to_vk(slot);
                let _temizlik = MedTemizlik { slot_vk };
                if !short_wait(0) {
                    return;
                }
                os_bridge::press_key(slot_vk);
                if !short_wait(15) {
                    return;
                }
                os_bridge::release_key(slot_vk);
                if !short_wait(30) {
                    return;
                }
                os_bridge::mouse_right_down();
                if !short_wait(15) {
                    return;
                }
                os_bridge::mouse_right_up();
                if !short_wait(40) {
                    return;
                }
                os_bridge::press_key(0x31);
                if !short_wait(15) {
                    return;
                }
                os_bridge::release_key(0x31);
            });
            if let Ok(mut slot) = MED_THREAD.lock() {
                if let Some(onceki) = slot.take() {
                    let _ = onceki.join();
                }
                *slot = Some(handle);
            }
        }
        prev_med_pressed = med_pressed;

        // ─── Silah Değiştirme Kısayolları (F1 – F11) ─────────────────────────
        let task_kisayollari: [(i32, usize); 9] = [
            (VK_F1, 0),
            (VK_F2, 1),
            (VK_F3, 2),
            (VK_F4, 3),
            (VK_F7, 4),
            (VK_F8, 5),
            (VK_F9, 6),
            (VK_F10, 7),
            (VK_F11, 8),
        ];
        for (index, (vk, id)) in task_kisayollari.into_iter().enumerate() {
            let pressed = os_bridge::is_key_pressed(vk);
            if pressed && !prev_task_pressed[index] {
                SILAH_ID.store(id, Ordering::Relaxed);
                ATES_ADIM.store(0, Ordering::Relaxed);
                let kopya = ayarlar.lock().ok().map(|mut cfg| {
                    cfg.task_id = id;
                    cfg.clone()
                });
                if let Some(cfg) = kopya {
                    let _ = crate::settings::save_config(&cfg);
                }
            }
            prev_task_pressed[index] = pressed;
        }

        os_bridge::short_sleep(Duration::from_millis(8));
    }
}

/// Hızlı lootlama (Fast Auto-Loot) döngüsü (Thread 3).
///
/// Atanan tuş basılı tutulduğunda seri sağ tık ile hızlı eşya toplama yapar.
pub fn cache_cleaner_loop() {
    while !KAPATILIYOR.load(Ordering::Relaxed) {
        if FAST_LOOT_ETKIN.load(Ordering::Relaxed) {
            let loot_vk = FAST_LOOT_VK.load(Ordering::Relaxed) as i32;
            if os_bridge::is_key_pressed(loot_vk) {
                os_bridge::mouse_right_down();
                os_bridge::short_sleep(Duration::from_millis(15));
                os_bridge::mouse_right_up();
                os_bridge::short_sleep(Duration::from_millis(25));
                continue;
            }
        }
        os_bridge::short_sleep(Duration::from_millis(10));
    }
}

/// Keep-Alive worker döngüsü (Thread 5).
///
/// 2.5 dakikada bir fareyi milimetrik hareket ettirerek
/// GeForce Now üzerinde hareketsizlik atılmasını önler.
pub fn keep_alive_loop() {
    let mut sayac = 0;
    while !KAPATILIYOR.load(Ordering::Relaxed) {
        if ANTI_AFK_ETKIN.load(Ordering::Relaxed) {
            sayac += 1;
            if sayac >= 150 {
                // 150 saniye = 2.5 dakika
                os_bridge::move_mouse(1, 1);
                os_bridge::short_sleep(Duration::from_millis(50));
                os_bridge::move_mouse(-1, -1);
                sayac = 0;
            }
        } else {
            sayac = 0;
        }
        os_bridge::short_sleep(Duration::from_secs(1));
    }
}
