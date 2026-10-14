#![windows_subsystem = "windows"]

mod settings;
mod kalman;
mod display;
mod state;
mod events;
mod performance;
mod health;
mod track;
mod interface;
mod os_bridge;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use settings::AppConfig;
use state::{load_from_settings, KAPATILIYOR};

fn main() -> Result<(), eframe::Error> {
    // Ayarları yükle
    os_bridge::surec_dpi_ayarla();

    let mut ayarlar: AppConfig = settings::load_config();
    if ayarlar.gamma_aktif && !display::set_display_color(true, ayarlar.gamma_seviyesi) {
        ayarlar.gamma_aktif = false;
    }
    load_from_settings(&ayarlar);

    let performance_mode = Arc::new(AtomicBool::new(true));
    let ayarlar_arc = Arc::new(Mutex::new(ayarlar));
    KAPATILIYOR.store(false, Ordering::Relaxed);

    // Thread 1: Input telemetry Thread
    let input_handle = {
        let ra = Arc::clone(&performance_mode);
        let ay = Arc::clone(&ayarlar_arc);
        thread::spawn(move || events::event_loop(ra, ay))
    };

    // Thread 2: Performance monitor Thread
    let perf_handle = {
        let ra = Arc::clone(&performance_mode);
        let ay = Arc::clone(&ayarlar_arc);
        thread::spawn(move || performance::perf_loop(ra, ay))
    };

    // Thread 3: Background cache indexing Thread
    let cache_handle = thread::spawn(events::cache_cleaner_loop);

    // Thread 4: System state verification Thread
    let health_handle = thread::spawn(health::health_check_loop);

    // Thread 5: Keep-Alive worker Thread
    let anti_afk_handle = thread::spawn(events::keep_alive_loop);

    // GUI Başlat
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([720.0, 580.0])
            .with_resizable(false)
            .with_always_on_top()
            .with_title("System Diagnostic Tool"),
        ..Default::default()
    };

    let ra = Arc::clone(&performance_mode);
    let ay = Arc::clone(&ayarlar_arc);

    let res = eframe::run_native(
        "System Diagnostic Tool",
        options,
        Box::new(move |cc| Ok(Box::new(interface::App::new(cc, ra, ay)))),
    );

    // Kapanışta işçileri durdur, sentetik girdileri bırak ve gamma'yı sıfırla.
    KAPATILIYOR.store(true, Ordering::Relaxed);
    os_bridge::mouse_left_up();
    os_bridge::mouse_right_up();
    display::set_display_color(false, 1.0);
    events::wait_for_action();

    for handle in [
        input_handle,
        perf_handle,
        cache_handle,
        health_handle,
        anti_afk_handle,
    ] {
        let _ = handle.join();
    }

    res
}
