# System Diagnostic Tool (rust-projemiz)

Rust ve egui (`eframe`) kullanılarak geliştirilmiş Windows tabanlı sistem ve performans izleme / yardımcı teşhis aracı.

## Özellikler

- **GUI & Telemetri**: `eframe` (egui) ile modern, hafif ve her zaman üstte kalabilen (always-on-top) arayüz.
- **Performans & Sistem İzleme**: Arka planda çalışan performans analizi ve durum kontrol iş parçacıkları (threads).
- **Girdi & Telemetri İşleme**: Kalman filtresi destekli hareket ve girdi takibi.
- **Yapılandırma**: `ayarlar.json` üzerinden kolayca yapılandırılabilen profil ve hassasiyet ayarları.

## Kurulum ve Çalıştırma

Projeyi derlemek ve çalıştırmak için sisteminizde Rust yüklü olmalıdır.

```bash
# Bağımlılıkları kontrol etme
cargo check

# Hata ayıklama modunda çalıştırma
cargo run

# Optimize edilmiş sürümü derleme
cargo build --release
```

Derlenen çalıştırılabilir dosya `target/release/sys_diag.exe` konumunda oluşturulacaktır.
