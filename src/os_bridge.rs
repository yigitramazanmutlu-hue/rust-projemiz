//! Güvenli Windows API Sarmalayıcıları (Safe Wrappers)
//!
//! Tüm `unsafe` Win32 API çağrıları bu modülde güvenli fonksiyonlara sarmalanır.
//! Diğer modüller doğrudan `unsafe` blok yazmak yerine bu fonksiyonları kullanır.

use std::cell::RefCell;
use std::time::Duration;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    BitBlt, ClientToScreen, CreateCompatibleBitmap, CreateCompatibleDC, CreateDCW, DeleteDC,
    DeleteObject, EnumDisplayMonitors, GetDC, GetDIBits, GetDeviceCaps, GetMonitorInfoW, ReleaseDC,
    SelectObject,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ, HMONITOR,
    HORZRES, MONITORINFO, MONITORINFOEXW, SRCCOPY, VERTRES,
};
use windows_sys::Win32::System::Threading::{
    CreateWaitableTimerExW, SetWaitableTimer, WaitForSingleObject,
    CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, INFINITE,
};
use windows_sys::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT,
    KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowA, GetClientRect, GetCursorPos, SetForegroundWindow, ShowWindow, SW_HIDE, SW_SHOW,
};

const ZAMANLAYICI_ERISIM: u32 = 0x001F_0003;

// ─── Süreç ──────────────────────────────────────────────────────────────────────

/// Pencere ve ekran ölçüleri fiziksel piksel olsun diye DPI farkındalığını açar.
pub fn surec_dpi_ayarla() {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

// ─── Tuş Durumu ─────────────────────────────────────────────────────────────────

/// Belirtilen sanal tuşun şu anda basılı olup olmadığını kontrol eder.
pub fn is_key_pressed(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) & 0x8000u16 as i16) != 0 }
}

// ─── Fare İşlemleri ─────────────────────────────────────────────────────────────

/// Fare imlecinin ekran konumunu döndürür.
pub fn cursor_screen_pos() -> Option<(i32, i32)> {
    unsafe {
        let mut nokta = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut nokta) == 0 {
            None
        } else {
            Some((nokta.x, nokta.y))
        }
    }
}

/// Fare imlecini belirtilen delta kadar hareket ettirir.
pub fn move_mouse(dx: i32, dy: i32) {
    send_mouse_input(MOUSEEVENTF_MOVE, dx, dy);
}

/// Sol fare tuşunu basılı tutar (down).
pub fn mouse_left_down() {
    send_mouse_input(MOUSEEVENTF_LEFTDOWN, 0, 0);
}

/// Sol fare tuşunu bırakır (up).
pub fn mouse_left_up() {
    send_mouse_input(MOUSEEVENTF_LEFTUP, 0, 0);
}

/// Sağ fare tuşunu basılı tutar (down).
pub fn mouse_right_down() {
    send_mouse_input(MOUSEEVENTF_RIGHTDOWN, 0, 0);
}

/// Sağ fare tuşunu bırakır (up).
pub fn mouse_right_up() {
    send_mouse_input(MOUSEEVENTF_RIGHTUP, 0, 0);
}

fn send_mouse_input(flags: u32, dx: i32, dy: i32) {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
    }
}

// ─── Klavye İşlemleri ───────────────────────────────────────────────────────────

/// Klavye tuşuna basar (down).
pub fn press_key(vk: u8) {
    send_keyboard_input(vk, 0);
}

/// Klavye tuşunu bırakır (up).
pub fn release_key(vk: u8) {
    send_keyboard_input(vk, KEYEVENTF_KEYUP);
}

fn send_keyboard_input(vk: u8, flags: u32) {
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk as u16,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
    }
}

// ─── Pencere İşlemleri ──────────────────────────────────────────────────────────

/// Pencereyi başlığına göre bulup gösterir veya gizler.
pub fn toggle_window(title: &[u8], show: bool) {
    unsafe {
        let hwnd = FindWindowA(std::ptr::null(), title.as_ptr());
        if !hwnd.is_null() {
            if show {
                ShowWindow(hwnd, SW_SHOW);
                SetForegroundWindow(hwnd);
            } else {
                ShowWindow(hwnd, SW_HIDE);
            }
        }
    }
}

/// Rust penceresinin istemci alanının ekran koordinatındaki merkezi.
pub fn target_window_center() -> Option<(i32, i32)> {
    unsafe {
        let mut hwnd = FindWindowA(b"UnityWndClass\0".as_ptr(), std::ptr::null());
        if hwnd.is_null() {
            hwnd = FindWindowA(std::ptr::null(), b"Rust\0".as_ptr());
        }
        if hwnd.is_null() {
            return None;
        }

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetClientRect(hwnd, &mut rect) == 0 {
            return None;
        }

        let mut sol_ust = POINT {
            x: rect.left,
            y: rect.top,
        };
        let mut sag_alt = POINT {
            x: rect.right,
            y: rect.bottom,
        };
        if ClientToScreen(hwnd, &mut sol_ust) == 0 || ClientToScreen(hwnd, &mut sag_alt) == 0 {
            return None;
        }

        let genislik = sag_alt.x - sol_ust.x;
        let yukseklik = sag_alt.y - sol_ust.y;
        if genislik <= 0 || yukseklik <= 0 {
            return None;
        }
        Some((sol_ust.x + genislik / 2, sol_ust.y + yukseklik / 2))
    }
}

/// Bağlı ekranların her biri için bir aygıt bağlamı açar.
pub fn her_ekran(mut ziyaret: impl FnMut([u16; 32], HDC)) {
    unsafe {
        let mut adlar: Vec<[u16; 32]> = Vec::new();
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(ekran_adi_topla),
            &mut adlar as *mut Vec<[u16; 32]> as isize,
        );

        if adlar.is_empty() {
            let hdc = GetDC(std::ptr::null_mut());
            if !hdc.is_null() {
                ziyaret([0; 32], hdc);
                ReleaseDC(std::ptr::null_mut(), hdc);
            }
            return;
        }

        let surucu: Vec<u16> = "DISPLAY".encode_utf16().chain(std::iter::once(0)).collect();
        for ad in adlar {
            let hdc = CreateDCW(
                surucu.as_ptr(),
                ad.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
            );
            if !hdc.is_null() {
                ziyaret(ad, hdc);
                DeleteDC(hdc);
            }
        }
    }
}

unsafe extern "system" fn ekran_adi_topla(
    monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    data: isize,
) -> i32 {
    let adlar = unsafe { &mut *(data as *mut Vec<[u16; 32]>) };
    let mut info: MONITORINFOEXW = unsafe { std::mem::zeroed() };
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    if unsafe {
        GetMonitorInfoW(
            monitor,
            &mut info as *mut MONITORINFOEXW as *mut MONITORINFO,
        )
    } != 0
    {
        adlar.push(info.szDevice);
    }
    1
}

// ─── Kısa uyku ──────────────────────────────────────────────────────────────────

struct Zamanlayici(HANDLE);

impl Drop for Zamanlayici {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

impl Zamanlayici {
    fn yeni() -> Option<Self> {
        unsafe {
            let mut handle = CreateWaitableTimerExW(
                std::ptr::null(),
                std::ptr::null(),
                CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                ZAMANLAYICI_ERISIM,
            );
            if handle.is_null() {
                handle = CreateWaitableTimerExW(
                    std::ptr::null(),
                    std::ptr::null(),
                    0,
                    ZAMANLAYICI_ERISIM,
                );
            }
            if handle.is_null() {
                None
            } else {
                Some(Self(handle))
            }
        }
    }

    fn uyu(&self, sure: Duration) -> bool {
        let yuz_nanosaniye = (sure.as_nanos() / 100) as i64;
        if yuz_nanosaniye <= 0 {
            return true;
        }
        let vade = -yuz_nanosaniye;
        unsafe {
            if SetWaitableTimer(self.0, &vade, 0, None, std::ptr::null(), 0) == 0 {
                return false;
            }
            WaitForSingleObject(self.0, INFINITE);
        }
        true
    }
}

thread_local! {
    static ZAMANLAYICI: RefCell<Option<Zamanlayici>> = const { RefCell::new(None) };
}

/// İş parçacığına özel yüksek çözünürlüklü zamanlayıcı ile uyur.
///
/// Sistem genelindeki `timeBeginPeriod` çağrısının yerine geçer.
pub fn short_sleep(sure: Duration) {
    let uyudu = ZAMANLAYICI.with(|slot| {
        let mut tutucu = slot.borrow_mut();
        if tutucu.is_none() {
            *tutucu = Zamanlayici::yeni();
        }
        tutucu.as_ref().is_some_and(|zamanlayici| zamanlayici.uyu(sure))
    });
    if !uyudu {
        std::thread::sleep(sure);
    }
}

// ─── Küçük Ekran Bölgesi Yakalama ─────────────────────────────────────────────

/// Birincil ekranın merkezindeki küçük kare alanı yakalamak için GDI kaynaklarını
/// tekrar kullanan nesne. Oyun penceresi bulunursa karenin merkezi o pencerenin
/// istemci alanıdır.
pub struct CenterCapture {
    screen_dc: HDC,
    memory_dc: HDC,
    bitmap: HBITMAP,
    old_bitmap: HGDIOBJ,
    size: i32,
    pixels: Vec<u8>,
    colors: Vec<(u8, u8, u8)>,
}

impl CenterCapture {
    pub fn new() -> Option<Self> {
        unsafe {
            let screen_dc = GetDC(std::ptr::null_mut());
            if screen_dc.is_null() {
                return None;
            }

            let memory_dc = CreateCompatibleDC(screen_dc);
            if memory_dc.is_null() {
                ReleaseDC(std::ptr::null_mut(), screen_dc);
                return None;
            }

            Some(Self {
                screen_dc,
                memory_dc,
                bitmap: std::ptr::null_mut(),
                old_bitmap: std::ptr::null_mut(),
                size: 0,
                pixels: Vec::new(),
                colors: Vec::new(),
            })
        }
    }

    pub fn capture_center(&mut self, requested_size: u32) -> Option<&[(u8, u8, u8)]> {
        let screen_width = unsafe { GetDeviceCaps(self.screen_dc, HORZRES as i32) };
        let screen_height = unsafe { GetDeviceCaps(self.screen_dc, VERTRES as i32) };
        let size = if screen_width > 0 && screen_height > 0 {
            (requested_size as i32).clamp(1, screen_width.min(screen_height))
        } else {
            (requested_size as i32).clamp(1, 64)
        };
        let (merkez_x, merkez_y) = if let Some(merkez) = target_window_center() {
            merkez
        } else if screen_width <= 0 || screen_height <= 0 {
            return None;
        } else {
            (screen_width / 2, screen_height / 2)
        };
        self.capture_at(merkez_x, merkez_y, size as u32)
    }

    /// Karenin merkezi verilen ekran noktasıdır.
    pub fn capture_at(&mut self, center_x: i32, center_y: i32, requested_size: u32) -> Option<&[(u8, u8, u8)]> {
        let screen_width = unsafe { GetDeviceCaps(self.screen_dc, HORZRES as i32) };
        let screen_height = unsafe { GetDeviceCaps(self.screen_dc, VERTRES as i32) };
        let size = if screen_width > 0 && screen_height > 0 {
            (requested_size as i32).clamp(1, screen_width.min(screen_height))
        } else {
            (requested_size as i32).clamp(1, 64)
        };
        if size != self.size && !self.resize_bitmap(size) {
            return None;
        }

        let source_x = center_x - size / 2;
        let source_y = center_y - size / 2;

        let copied = unsafe {
            BitBlt(
                self.memory_dc,
                0,
                0,
                size,
                size,
                self.screen_dc,
                source_x,
                source_y,
                SRCCOPY,
            )
        };
        if copied == 0 {
            return None;
        }

        let byte_count = size as usize * size as usize * 4;
        self.pixels.resize(byte_count, 0);

        let mut bitmap_info: BITMAPINFO = unsafe { std::mem::zeroed() };
        bitmap_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bitmap_info.bmiHeader.biWidth = size;
        bitmap_info.bmiHeader.biHeight = -size;
        bitmap_info.bmiHeader.biPlanes = 1;
        bitmap_info.bmiHeader.biBitCount = 32;
        bitmap_info.bmiHeader.biCompression = BI_RGB;

        let rows = unsafe {
            GetDIBits(
                self.screen_dc,
                self.bitmap,
                0,
                size as u32,
                self.pixels.as_mut_ptr().cast(),
                &mut bitmap_info,
                DIB_RGB_COLORS,
            )
        };
        if rows != size {
            return None;
        }

        self.colors.clear();
        self.colors.reserve(size as usize * size as usize);
        self.colors.extend(
            self.pixels
                .chunks_exact(4)
                .map(|bgra| (bgra[2], bgra[1], bgra[0])),
        );
        Some(&self.colors)
    }

    fn resize_bitmap(&mut self, size: i32) -> bool {
        unsafe {
            if !self.bitmap.is_null() {
                SelectObject(self.memory_dc, self.old_bitmap);
                DeleteObject(self.bitmap as HGDIOBJ);
                self.bitmap = std::ptr::null_mut();
                self.old_bitmap = std::ptr::null_mut();
            }

            let bitmap = CreateCompatibleBitmap(self.screen_dc, size, size);
            if bitmap.is_null() {
                self.size = 0;
                return false;
            }

            let old_bitmap = SelectObject(self.memory_dc, bitmap as HGDIOBJ);
            if old_bitmap.is_null() || old_bitmap == (-1isize as HGDIOBJ) {
                DeleteObject(bitmap as HGDIOBJ);
                self.size = 0;
                return false;
            }

            self.bitmap = bitmap;
            self.old_bitmap = old_bitmap;
            self.size = size;
            true
        }
    }
}

impl Drop for CenterCapture {
    fn drop(&mut self) {
        unsafe {
            if !self.bitmap.is_null() {
                SelectObject(self.memory_dc, self.old_bitmap);
                DeleteObject(self.bitmap as HGDIOBJ);
            }
            if !self.memory_dc.is_null() {
                DeleteDC(self.memory_dc);
            }
            if !self.screen_dc.is_null() {
                ReleaseDC(std::ptr::null_mut(), self.screen_dc);
            }
        }
    }
}
