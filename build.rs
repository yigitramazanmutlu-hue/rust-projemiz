fn main() {
    println!("cargo:rerun-if-changed=icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }
    if !std::path::Path::new("icon.ico").exists() {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("icon.ico");
    res.compile().expect("Windows kaynakları derlenemedi");
}
