#![allow(dead_code)]

//! Kontrast adası taraması ve Kalman izi.
//!
//! Tarama bir renk tamponunda arka plandan ayrışan piksel kümesinin merkezini bulur.
//! İz, bu merkezi ölçüm kabul eder. Ölçüm yokken kısa süre hızıyla devam eder.

use nalgebra::SVector;

use crate::kalman::KalmanFilter2d;

const IVME_STD: f64 = 2_000.0;
const OLCUM_STD: f64 = 1.5;
const MIN_PIKSEL: usize = 4;
const ESİK: i32 = 40;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScanHit {
    pub x: f64,
    pub y: f64,
    pub pixels: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackView {
    pub x: f64,
    pub y: f64,
    pub live: bool,
}

/// İz, karenin ortasından ne kadar uzaktaysa fare o kadar kayar.
///
/// Adım kare başına en fazla 12 pikseldir. İz kapalıysa hareket yoktur.
pub fn fare_adimi(iz: TrackView, boyut: f64) -> (i32, i32) {
    if !iz.live || boyut <= 1.0 {
        return (0, 0);
    }
    let orta = (boyut - 1.0) / 2.0;
    let dx = ((iz.x - orta) * 0.35).round().clamp(-12.0, 12.0) as i32;
    let dy = ((iz.y - orta) * 0.35).round().clamp(-12.0, 12.0) as i32;
    (dx, dy)
}

pub struct ObjectTrack {
    kf: KalmanFilter2d,
    misses: u32,
    max_misses: u32,
    live: bool,
}

impl ObjectTrack {
    pub fn new(dt: f64, max_misses: u32) -> Self {
        Self {
            kf: KalmanFilter2d::new(dt, IVME_STD, OLCUM_STD),
            misses: 0,
            max_misses,
            live: false,
        }
    }

    pub fn update(&mut self, hit: Option<ScanHit>, dt: f64) -> TrackView {
        match hit {
            Some(hit) => {
                let z = SVector::<f64, 2>::new(hit.x, hit.y);
                if self.kf.step(&z, dt).is_ok() {
                    self.live = true;
                    self.misses = 0;
                }
            }
            None if self.live => {
                self.kf.set_dt(dt);
                self.kf.predict();
                self.misses = self.misses.saturating_add(1);
                if self.misses > self.max_misses {
                    self.live = false;
                    self.kf = KalmanFilter2d::new(dt, IVME_STD, OLCUM_STD);
                }
            }
            None => {}
        }

        let pos = self.kf.position();
        TrackView {
            x: pos[0],
            y: pos[1],
            live: self.live,
        }
    }
}

/// Satır öncelikli renk tamponunda arka plandan ayrışan adanın merkezini döndürür.
pub fn scan_contrast(colors: &[(u8, u8, u8)], width: usize, height: usize) -> Option<ScanHit> {
    if width == 0 || height == 0 || colors.len() != width * height {
        return None;
    }

    let (bg_r, bg_g, bg_b) = border_mean(colors, width, height);
    let mut mask = vec![false; colors.len()];
    for (i, pixel) in colors.iter().enumerate() {
        let diff = (pixel.0 as i32 - bg_r).abs()
            + (pixel.1 as i32 - bg_g).abs()
            + (pixel.2 as i32 - bg_b).abs();
        mask[i] = diff >= ESİK;
    }

    let mut weight = 0i64;
    let mut sum_x = 0i64;
    let mut sum_y = 0i64;
    for y in 0..height {
        for x in 0..width {
            let i = y * width + x;
            if !mask[i] || !komsu_var(&mask, width, height, x, y) {
                continue;
            }
            weight += 1;
            sum_x += x as i64;
            sum_y += y as i64;
        }
    }

    if (weight as usize) < MIN_PIKSEL {
        return None;
    }

    Some(ScanHit {
        x: sum_x as f64 / weight as f64,
        y: sum_y as f64 / weight as f64,
        pixels: weight as usize,
    })
}

fn komsu_var(mask: &[bool], width: usize, height: usize, x: usize, y: usize) -> bool {
    let sol = x > 0 && mask[y * width + x - 1];
    let sag = x + 1 < width && mask[y * width + x + 1];
    let ust = y > 0 && mask[(y - 1) * width + x];
    let alt = y + 1 < height && mask[(y + 1) * width + x];
    sol || sag || ust || alt
}

fn border_mean(colors: &[(u8, u8, u8)], width: usize, height: usize) -> (i32, i32, i32) {
    let mut n = 0i64;
    let mut r = 0i64;
    let mut g = 0i64;
    let mut b = 0i64;
    let mut ekle = |p: (u8, u8, u8)| {
        r += p.0 as i64;
        g += p.1 as i64;
        b += p.2 as i64;
        n += 1;
    };
    for x in 0..width {
        ekle(colors[x]);
        if height > 1 {
            ekle(colors[(height - 1) * width + x]);
        }
    }
    for y in 1..height.saturating_sub(1) {
        ekle(colors[y * width]);
        if width > 1 {
            ekle(colors[y * width + width - 1]);
        }
    }
    if n == 0 {
        return (0, 0, 0);
    }
    ((r / n) as i32, (g / n) as i32, (b / n) as i32)
}

#[cfg(test)]
mod tests {
    use super::{scan_contrast, ObjectTrack};

    fn kare(origin_x: usize, origin_y: usize, speck: bool) -> Vec<(u8, u8, u8)> {
        let mut pixels = vec![(10, 12, 14); 32 * 32];
        for y in origin_y..origin_y + 4 {
            for x in origin_x..origin_x + 4 {
                pixels[y * 32 + x] = (230, 230, 230);
            }
        }
        if speck {
            pixels[32 + 2] = (255, 0, 0);
        }
        pixels
    }

    #[test]
    fn kontrast_adasi_merkezini_bulur_benek_izi_bozmaz() {
        let hit = scan_contrast(&kare(10, 12, true), 32, 32).unwrap();
        assert_eq!(hit.pixels, 16);
        assert!((hit.x - 11.5).abs() < 0.2, "x {}", hit.x);
        assert!((hit.y - 13.5).abs() < 0.2, "y {}", hit.y);
    }

    #[test]
    fn tek_benek_cisim_sayilmaz() {
        let mut pixels = vec![(10, 12, 14); 32 * 32];
        pixels[40] = (255, 255, 255);
        assert!(scan_contrast(&pixels, 32, 32).is_none());
    }

    #[test]
    fn kalman_izi_gurultuyu_azaltir_ve_kacak_karede_duser() {
        let dt = 0.033;
        let mut track = ObjectTrack::new(dt, 2);
        let mut ham_hata = 0.0;
        let mut iz_hata = 0.0;
        for i in 0..12 {
            let gercek = 10.0 + i as f64 * 1.2;
            let olcum = gercek + if i % 2 == 0 { 2.5 } else { -2.5 };
            let origin = (olcum - 1.5).round().clamp(1.0, 26.0) as usize;
            let hit = scan_contrast(&kare(origin, 12, false), 32, 32).unwrap();
            let view = track.update(Some(hit), dt);
            assert!(view.live);
            ham_hata += (hit.x - gercek).abs();
            iz_hata += (view.x - gercek).abs();
        }
        assert!(
            iz_hata < ham_hata,
            "iz {iz_hata} ham olcumden {ham_hata} daha kotu"
        );

        let mut son = track.update(None, dt);
        assert!(son.live, "ilk kacak kare izi dusurmemeli");
        son = track.update(None, dt);
        assert!(son.live);
        son = track.update(None, dt);
        assert!(!son.live, "ust uste kacista iz kapanmali");
    }

    #[test]
    fn fare_adimi_ortadayken_durur_kenarda_kayar() {
        let orta = super::TrackView {
            x: 31.5,
            y: 31.5,
            live: true,
        };
        assert_eq!(super::fare_adimi(orta, 64.0), (0, 0));
        let sag = super::TrackView {
            x: 48.0,
            y: 31.5,
            live: true,
        };
        let (dx, dy) = super::fare_adimi(sag, 64.0);
        assert!(dx > 0 && dy == 0, "{dx},{dy}");
        let kapali = super::TrackView {
            x: 48.0,
            y: 10.0,
            live: false,
        };
        assert_eq!(super::fare_adimi(kapali, 64.0), (0, 0));
    }
}
