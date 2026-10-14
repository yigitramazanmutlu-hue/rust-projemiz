//! System state verification
//!
//! Ekranın ortasındaki NxN piksel alanını tarayarak frame-to-frame
//! renk değişimi algılar. Eşik aşıldığında otomatik ateş eder.
//!
//! Özellikler:
//! - Ayarlanabilir tarama alanı, onay frame, humanize gecikme
//! - 5 renk filtresi: Kapalı, Kırmızı, Yeşil, Mavi, Sarı
//! - Minimum burst süresi

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::os_bridge::short_sleep;

use crate::state::*;
use crate::os_bridge;

/// Auto Trigger v2 ana döngüsü.
pub fn health_check_loop() {
    let mut prev_colors: Vec<(u8, u8, u8)> = Vec::new();
    let mut holding_down = false;
    let mut onay_sayaci: usize = 0;
    let mut burst_baslangic: Option<Instant> = None;
    let mut capturer = os_bridge::CenterCapture::new();
    let mut son_capture_deneme = Instant::now();
    let mut art_arda_hata: u32 = 0;

    // Basit pseudo-random (thread-local seed)
    let mut rng_seed: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    while !KAPATILIYOR.load(Ordering::Relaxed) {
        if TRIGGER_ETKIN.load(Ordering::Relaxed) {
            // GDI kaynakları geçici olarak alınamazsa iki saniyede bir tekrar dene.
            if capturer.is_none() && son_capture_deneme.elapsed() >= Duration::from_secs(2) {
                capturer = os_bridge::CenterCapture::new();
                son_capture_deneme = Instant::now();
            }

            let mut capture_basarili = false;
            if let Some(ref mut capture) = capturer {
                let alan = TRIGGER_TARAMA_ALAN.load(Ordering::Relaxed) as u32;
                if let Some(current_colors) = capture.capture_center(alan) {
                    capture_basarili = true;
                    let renk_filtre = TRIGGER_RENK_FILTRE.load(Ordering::Relaxed);

                    if prev_colors.len() == current_colors.len() && !prev_colors.is_empty() {
                        let threshold = TRIGGER_HASSASIYET.load(Ordering::Relaxed) as i32;
                        let sag_tik = os_bridge::is_key_pressed(VK_RBUTTON);

                        // Piksel bazlı fark hesapla
                        let mut tetiklenen_piksel = 0usize;
                        let toplam_piksel = current_colors.len();

                        for (i, (cr, cg, cb)) in current_colors.iter().enumerate() {
                            let (pr, pg, pb) = prev_colors[i];
                            let diff = renk_farki_hesapla(renk_filtre, *cr, *cg, *cb, pr, pg, pb);
                            if diff >= threshold {
                                tetiklenen_piksel += 1;
                            }
                        }

                        // Piksellerin en az %30'u tetiklenmeli (gürültü filtresi)
                        let min_tetik = (toplam_piksel as f32 * 0.30).ceil() as usize;
                        let algilandi = sag_tik && tetiklenen_piksel >= min_tetik.max(1);

                        if algilandi {
                            onay_sayaci += 1;
                            let gerekli_onay = TRIGGER_ONAY_FRAME.load(Ordering::Relaxed);

                            if onay_sayaci >= gerekli_onay {
                                if !holding_down {
                                    // Humanize: rastgele gecikme ekle
                                    let h_min = TRIGGER_HUMANIZE_MIN.load(Ordering::Relaxed) as u64;
                                    let h_max = TRIGGER_HUMANIZE_MAX.load(Ordering::Relaxed) as u64;
                                    if h_max > 0 && h_max >= h_min {
                                        let aralik = h_max - h_min + 1;
                                        let gecikme = h_min + (next_rand(&mut rng_seed) % aralik);
                                        short_sleep(Duration::from_millis(gecikme));
                                    }

                                    // Bekleme sırasında kullanıcı nişanı veya özelliği kapatmış olabilir.
                                    if TRIGGER_ETKIN.load(Ordering::Relaxed)
                                        && os_bridge::is_key_pressed(VK_RBUTTON)
                                    {
                                        os_bridge::mouse_left_down();
                                        holding_down = true;
                                    }
                                }

                                // Hedef algılandığı sürece burst süresini tazele
                                if holding_down {
                                    burst_baslangic = Some(Instant::now());
                                }
                                onay_sayaci = 0;
                            }
                        } else {
                            onay_sayaci = 0;
                            if holding_down {
                                let burst_suresi =
                                    TRIGGER_SPREY_SURESI.load(Ordering::Relaxed) as u128;
                                let gecen_sure = burst_baslangic
                                    .map(|t| t.elapsed().as_millis())
                                    .unwrap_or(0);

                                // Hedef ekrandan çıksa bile min. burst süresi kadar basılı tut
                                if gecen_sure >= burst_suresi {
                                    os_bridge::mouse_left_up();
                                    holding_down = false;
                                    burst_baslangic = None;
                                }
                            }
                        }
                    }
                    prev_colors.clear();
                    prev_colors.extend_from_slice(current_colors);
                    art_arda_hata = 0;
                }
            }

            // Tek bir boş kare yakalayıcıyı düşürmesin; art arda hatalarda yeniden kur.
            if !capture_basarili {
                onay_sayaci = 0;
                prev_colors.clear();
                if capturer.is_some() {
                    art_arda_hata = art_arda_hata.saturating_add(1);
                    if art_arda_hata >= 30 {
                        capturer = None;
                        art_arda_hata = 0;
                        son_capture_deneme = Instant::now();
                    }
                }
                if holding_down {
                    os_bridge::mouse_left_up();
                    holding_down = false;
                    burst_baslangic = None;
                }
            }
            short_sleep(Duration::from_millis(4));
        } else {
            if holding_down {
                os_bridge::mouse_left_up();
                holding_down = false;
                burst_baslangic = None;
            }
            prev_colors.clear();
            onay_sayaci = 0;
            short_sleep(Duration::from_millis(50));
        }
    }

    if holding_down {
        os_bridge::mouse_left_up();
    }
}

// ─── Yardımcı Fonksiyonlar ──────────────────────────────────────────────────────

/// Basit XorShift pseudo-random sayı üreteci.
fn next_rand(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

/// Renk filtre moduna göre iki piksel arasındaki farkı hesaplar.
///
/// Filtre modları:
/// - 0: Kapalı (genel RGB ortalaması)
/// - 1: Kırmızı baskınlık farkı
/// - 2: Yeşil baskınlık farkı
/// - 3: Mavi baskınlık farkı
/// - 4: Sarı (kırmızı + yeşil) baskınlık farkı
fn renk_farki_hesapla(filtre: usize, cr: u8, cg: u8, cb: u8, pr: u8, pg: u8, pb: u8) -> i32 {
    match filtre {
        // Kırmızı filtre: kırmızı kanalının baskın olmasına odaklan
        1 => {
            let r_baskinlik = (cr as i32) - ((cg as i32 + cb as i32) / 2);
            let prev_r_baskinlik = (pr as i32) - ((pg as i32 + pb as i32) / 2);
            (r_baskinlik - prev_r_baskinlik).abs()
        }
        // Yeşil filtre
        2 => {
            let g_baskinlik = (cg as i32) - ((cr as i32 + cb as i32) / 2);
            let prev_g_baskinlik = (pg as i32) - ((pr as i32 + pb as i32) / 2);
            (g_baskinlik - prev_g_baskinlik).abs()
        }
        // Mavi filtre
        3 => {
            let b_baskinlik = (cb as i32) - ((cr as i32 + cg as i32) / 2);
            let prev_b_baskinlik = (pb as i32) - ((pr as i32 + pg as i32) / 2);
            (b_baskinlik - prev_b_baskinlik).abs()
        }
        // Sarı filtre (kırmızı + yeşil baskınlığı)
        4 => {
            let rg_baskinlik = ((cr as i32 + cg as i32) / 2) - cb as i32;
            let prev_rg_baskinlik = ((pr as i32 + pg as i32) / 2) - pb as i32;
            (rg_baskinlik - prev_rg_baskinlik).abs()
        }
        // Kapalı (genel RGB farkı)
        _ => {
            let diff_r = (cr as i32 - pr as i32).abs();
            let diff_g = (cg as i32 - pg as i32).abs();
            let diff_b = (cb as i32 - pb as i32).abs();
            (diff_r + diff_g + diff_b) / 3
        }
    }
}

#[cfg(test)]
mod tests {
    use super::renk_farki_hesapla;

    #[test]
    fn genel_rgb_farki_ortalamasini_hesaplar() {
        assert_eq!(renk_farki_hesapla(0, 30, 60, 90, 0, 0, 0), 60);
    }

    #[test]
    fn renk_filtreleri_ilgili_kanal_baskinligini_olcer() {
        assert_eq!(renk_farki_hesapla(1, 100, 20, 20, 20, 20, 20), 80);
        assert_eq!(renk_farki_hesapla(2, 20, 100, 20, 20, 20, 20), 80);
        assert_eq!(renk_farki_hesapla(3, 20, 20, 100, 20, 20, 20), 80);
        assert_eq!(renk_farki_hesapla(4, 100, 100, 20, 20, 20, 20), 80);
    }
}
