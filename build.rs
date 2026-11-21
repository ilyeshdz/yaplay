fn main() {
    println!("cargo:rerun-if-env-changed=YAPLAY_UPDATE_PUBLIC_KEY");
    println!("cargo:rerun-if-changed=assets/branding/icon.ico");
    #[cfg(windows)]
    winresource::WindowsResource::new()
        .set_icon("assets/branding/icon.ico")
        .set("ProductName", "Yaplay")
        .set(
            "FileDescription",
            "Yaplay — an unofficial overlay for Yapin",
        )
        .compile()
        .expect("Failed to embed the Yaplay Windows icon");
}
