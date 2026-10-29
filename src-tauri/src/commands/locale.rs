#[tauri::command]
pub fn get_locale_hint() -> Option<String> {
    std::env::var("PRINDECK_LOCALE").ok()
}
