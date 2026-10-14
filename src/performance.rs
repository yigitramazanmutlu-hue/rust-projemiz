//! Recoil Dengeleme Sistemi
//!
//! Silah başına tanımlı geri tepme paternlerini uygulayarak fareyi
//! dengeleyici yönde hareket ettirir. Her taskın benzersiz (dx, dy)
//! patern vektörü bulunur.
//!
//! **DİKKAT:** Patern değerleri dikkatli kalibrasyon sonucu belirlenmiştir.
//! Değiştirmeden önce mutlaka yedek alın.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use nalgebra::SVector;

use crate::settings::{cekis_olcegi, ProcessSettings, AppConfig};
use crate::state::*;
use crate::os_bridge;
use crate::kalman::KalmanFilter2d;

/// Piksel uzayında patern kıvrımını izleyebilecek süreç gürültüsü.
const KALMAN_IVME_STD: f64 = 8_000.0;
/// Patern bilinen komut olduğu için ölçüme yakın durur.
const KALMAN_OLCUM_STD: f64 = 0.75;

/// Geri tepme adımlarını biriken bir yola çevirip Kalman ile izler.
///
/// Fareye giden değer mutlak konum değil, son gönderimden sonraki yer değiştirmedir.
struct RecoilSmoother {
    kf: KalmanFilter2d,
    hedef: SVector<f64, 2>,
    gonderilen: SVector<f64, 2>,
}

impl RecoilSmoother {
    fn new(dt: f64) -> Self {
        Self {
            kf: KalmanFilter2d::new(dt, KALMAN_IVME_STD, KALMAN_OLCUM_STD),
            hedef: SVector::zeros(),
            gonderilen: SVector::zeros(),
        }
    }

    fn reset(&mut self, dt: f64) {
        *self = Self::new(dt);
    }

    fn next_delta(&mut self, raw_x: f64, raw_y: f64, dt: f64) -> (i32, i32) {
        self.hedef[0] += raw_x;
        self.hedef[1] += raw_y;
        let delta = if self.kf.step(&self.hedef, dt).is_ok() {
            let pos = self.kf.position();
            (pos[0] - self.gonderilen[0], pos[1] - self.gonderilen[1])
        } else {
            (raw_x, raw_y)
        };
        let dx = delta.0.clamp(-127.0, 127.0).round() as i32;
        let dy = delta.1.clamp(-127.0, 127.0).round() as i32;
        self.gonderilen[0] += dx as f64;
        self.gonderilen[1] += dy as f64;
        (dx, dy)
    }
}

const PROF_A: &[(i8, i8)] = &[
    (0, 18),
    (-2, 18),
    (-4, 17),
    (-5, 16),
    (-6, 15),
    (-7, 14),
    (-8, 13),
    (-8, 13),
    (-8, 12),
    (-9, 12),
    (-9, 12),
    (-9, 11),
    (-9, 11),
    (-9, 11),
    (-8, 10),
    (-8, 10),
    (-8, 10),
    (-8, 10),
    (-8, 10),
    (-8, 10),
    (-8, 10),
    (-8, 10),
    (-8, 10),
    (-8, 10),
    (-8, 10),
    (-8, 10),
    (-8, 10),
    (-8, 10),
    (-8, 10),
    (-8, 10),
];
const PROF_B: &[(i8, i8)] = &[
    (0, 12),
    (1, 11),
    (-1, 11),
    (2, 10),
    (-2, 10),
    (1, 9),
    (-2, 9),
    (2, 8),
    (-1, 8),
    (1, 8),
    (-2, 7),
    (1, 7),
    (-1, 7),
    (2, 6),
    (-2, 6),
    (1, 6),
    (-1, 6),
    (2, 5),
    (-2, 5),
    (1, 5),
    (-2, 5),
    (1, 4),
    (1, 4),
    (-2, 4),
    (2, 4),
    (-1, 3),
    (1, 3),
    (-2, 3),
    (1, 3),
    (0, 3),
];
const PROF_C: &[(i8, i8)] = &[
    (0, 10),
    (-2, 9),
    (-4, 8),
    (2, 7),
    (3, 6),
    (4, 5),
    (2, 4),
    (-2, 4),
    (-3, 3),
    (-4, 3),
    (0, 2),
];
const PROF_D: &[(i8, i8)] = &[
    (0, 22),
    (-4, 20),
    (-6, 18),
    (-4, 16),
    (3, 14),
    (5, 12),
    (6, 12),
    (2, 10),
    (-4, 9),
    (-6, 8),
    (-4, 7),
    (0, 6),
];
const PROF_E: &[(i8, i8)] = &[
    (0, 14),
    (-1, 13),
    (-2, 12),
    (-1, 11),
    (0, 10),
    (1, 9),
    (2, 8),
    (1, 7),
    (0, 6),
    (-1, 5),
];
const PROF_F: &[(i8, i8)] = &[(0, 9), (-1, 8), (-3, 8), (1, 7), (2, 6), (1, 5), (0, 4)];
const PROF_G: &[(i8, i8)] = &[(0, 15), (0, 14), (0, 13), (0, 12)];
const PROF_H: &[(i8, i8)] = &[
    (0, 25),
    (-3, 23),
    (-5, 21),
    (2, 19),
    (4, 17),
    (2, 15),
    (-2, 13),
];
const PROF_I: &[(i8, i8)] = &[(0, 16), (0, 15), (0, 14), (0, 13)];

struct TaskPattern {
    adimlar: &'static [(i8, i8)],
    #[allow(dead_code)]
    adim_ms: u64,
    /// Tam otomatik tasksda patern bitince son vektör tekrarlanır.
    /// Yarı otomatiklerde patern bir kez uygulanır.
    son_adimi_tekrarla: bool,
}

const SILAHLAR: [TaskPattern; 9] = [
    TaskPattern { adimlar: PROF_A, adim_ms: 22, son_adimi_tekrarla: true },
    TaskPattern { adimlar: PROF_B, adim_ms: 22, son_adimi_tekrarla: true },
    TaskPattern { adimlar: PROF_C, adim_ms: 22, son_adimi_tekrarla: true },
    TaskPattern { adimlar: PROF_D, adim_ms: 22, son_adimi_tekrarla: true },
    TaskPattern { adimlar: PROF_E, adim_ms: 22, son_adimi_tekrarla: true },
    TaskPattern { adimlar: PROF_F, adim_ms: 22, son_adimi_tekrarla: true },
    TaskPattern { adimlar: PROF_G, adim_ms: 22, son_adimi_tekrarla: false },
    TaskPattern { adimlar: PROF_H, adim_ms: 22, son_adimi_tekrarla: true },
    TaskPattern { adimlar: PROF_I, adim_ms: 22, son_adimi_tekrarla: false },
];

/// Recoil dengeleme ana döngüsü.
///
/// Ateş aktifken seçili taskın adım süresi kadar bekler. Ateş yokken
/// kısayol ve bırakma tepkisi için daha kısa aralıkla uyanır.
pub fn perf_loop(performance_mode: Arc<AtomicBool>, ayarlar: Arc<Mutex<AppConfig>>) {
    let mut cached_ayarlar = ayarlar.lock().map(|cfg| cfg.clone()).unwrap_or_default();
    let mut cached_surum = PERFORMANCE_AYAR_SURUMU.load(Ordering::Relaxed);
    let mut smoother = RecoilSmoother::new(0.022);

    while !KAPATILIYOR.load(Ordering::Relaxed) {
        let guncel_surum = PERFORMANCE_AYAR_SURUMU.load(Ordering::Acquire);
        if guncel_surum != cached_surum {
            if let Ok(cfg) = ayarlar.try_lock() {
                cached_ayarlar = cfg.clone();
                cached_surum = guncel_surum;
            }
        }

        let mut bekle = Duration::from_millis(8);
        if ATES_AKTIF.load(Ordering::Relaxed) && performance_mode.load(Ordering::Relaxed) {
            let adim = ATES_ADIM.load(Ordering::Relaxed);
            let task = SILAH_ID.load(Ordering::Relaxed);
            let patern = &SILAHLAR[task.min(SILAHLAR.len() - 1)];
            let ayar: ProcessSettings = match task {
                0 => cached_ayarlar.prof_a,
                1 => cached_ayarlar.prof_b,
                2 => cached_ayarlar.prof_c,
                3 => cached_ayarlar.prof_d,
                4 => cached_ayarlar.prof_e,
                5 => cached_ayarlar.prof_f,
                6 => cached_ayarlar.prof_g,
                7 => cached_ayarlar.prof_h,
                _ => cached_ayarlar.prof_i,
            };

            let olcek = cekis_olcegi(cached_ayarlar.hassasiyet, cached_ayarlar.gorus_acisi);
            let adim_ms = cached_ayarlar.adim_ms.clamp(8, 40);
            if adim == 0 {
                smoother.reset(adim_ms as f64 / 1000.0);
            }
            let devam = adim < patern.adimlar.len() || patern.son_adimi_tekrarla;
            if devam {
                let idx = adim.min(patern.adimlar.len() - 1);
                let ham = patern.adimlar[idx];
                let raw_x = ham.0 as f64 * ayar.yatay as f64 * olcek;
                let dikey_mult = if idx <= 4 {
                    ayar.dikey * ayar.ilk5_mermi_carpan
                } else {
                    ayar.dikey
                };
                let raw_y = ham.1 as f64 * dikey_mult as f64 * olcek;

                let dt = adim_ms as f64 / 1000.0;
                let (dx, dy) = smoother.next_delta(raw_x, raw_y, dt);
                if dx != 0 || dy != 0 {
                    os_bridge::move_mouse(dx, dy);
                }
                ATES_ADIM.fetch_add(1, Ordering::Relaxed);
            }
            bekle = Duration::from_millis(adim_ms);
        }

        let baslangic = Instant::now();
        os_bridge::short_sleep(bekle.saturating_sub(baslangic.elapsed()));
    }
}

#[cfg(test)]
mod tests {
    use super::RecoilSmoother;

    #[test]
    fn kalman_patern_toplamini_korur_ve_adim_deltasi_gonderir() {
        let adimlar = [
            (0.0, 18.0),
            (-2.0, 18.0),
            (-4.0, 17.0),
            (-5.0, 16.0),
            (-6.0, 15.0),
            (-9.0, 11.0),
            (-8.0, 10.0),
            (-8.0, 10.0),
            (0.0, 8.0),
            (2.0, 6.0),
        ];
        let mut smoother = RecoilSmoother::new(0.022);
        let mut gonderilen = (0i32, 0i32);
        let mut ham = (0.0f64, 0.0f64);
        for (x, y) in adimlar {
            let (dx, dy) = smoother.next_delta(x, y, 0.022);
            assert!(
                dx.abs() <= 40 && dy.abs() <= 40,
                "adim birikimli konumu yolladi: {dx},{dy}"
            );
            gonderilen.0 += dx;
            gonderilen.1 += dy;
            ham.0 += x;
            ham.1 += y;
        }
        assert!(
            (gonderilen.0 as f64 - ham.0).abs() < 4.0,
            "x toplam {} ham {}",
            gonderilen.0,
            ham.0
        );
        assert!(
            (gonderilen.1 as f64 - ham.1).abs() < 4.0,
            "y toplam {} ham {}",
            gonderilen.1,
            ham.1
        );
    }
}
