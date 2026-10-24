fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        // Hanya compile resources jika icon ada
        if std::path::Path::new("assets/icon.ico").exists() {
            res.set_icon("assets/icon.ico");
        }
        let _ = res.compile();
    }
}
