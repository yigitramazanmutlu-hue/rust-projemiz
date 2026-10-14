//! Ekran Gamma Kontrolü
//!
//! Windows GDI gamma rampasını kaydeder ve geri yükler. Çağrı başarısız olursa
//! (HDR, sürücü reddi) `false` döner; önceki rampa korunur.

use std::sync::Mutex;

use windows_sys::Win32::Graphics::Gdi::HDC;

use crate::os_bridge;

#[link(name = "gdi32")]
unsafe extern "system" {
    fn GetDeviceGammaRamp(hdc: HDC, lp_ramp: *mut core::ffi::c_void) -> i32;
    fn SetDeviceGammaRamp(hdc: HDC, lp_ramp: *const core::ffi::c_void) -> i32;
}

struct EkranKaydi {
    ad: [u16; 32],
    rampa: [u16; 768],
}

static KAYITLAR: Mutex<Vec<EkranKaydi>> = Mutex::new(Vec::new());

/// Ekran gamma eğrisini ayarlar.
///
/// `true`, en az bir ekranda rampa yazılabildiğinde döner.
pub fn set_display_color(boost: bool, level: f32) -> bool {
    let Ok(mut kayitlar) = KAYITLAR.lock() else {
        return false;
    };
    let mut basarili = false;

    os_bridge::her_ekran(|ad, hdc| {
        if !kayitlar.iter().any(|kayit| kayit.ad == ad) {
            let mut rampa = [0u16; 768];
            let okundu = unsafe { GetDeviceGammaRamp(hdc, rampa.as_mut_ptr().cast()) } != 0;
            if okundu {
                kayitlar.push(EkranKaydi { ad, rampa });
            }
        }

        let yazilacak = if boost {
            egri(level)
        } else if let Some(kayit) = kayitlar.iter().find(|kayit| kayit.ad == ad) {
            kayit.rampa
        } else {
            kimlik()
        };

        if unsafe { SetDeviceGammaRamp(hdc, yazilacak.as_ptr().cast()) } != 0 {
            basarili = true;
        }
    });

    basarili
}

fn egri(level: f32) -> [u16; 768] {
    let mut ramp = [0u16; 768];
    let p = 1.0 / level.clamp(1.0, 3.0);
    for i in 0..256 {
        let norm = (i as f32) / 255.0;
        let val = (norm.powf(p) * 65535.0).clamp(0.0, 65535.0) as u16;
        ramp[i] = val;
        ramp[i + 256] = val;
        ramp[i + 512] = val;
    }
    ramp
}

fn kimlik() -> [u16; 768] {
    let mut ramp = [0u16; 768];
    for i in 0..256 {
        let val = ((i as u32 * 65535) / 255) as u16;
        ramp[i] = val;
        ramp[i + 256] = val;
        ramp[i + 512] = val;
    }
    ramp
}
