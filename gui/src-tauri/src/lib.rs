mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::scan_directory,
            commands::load_image,
            commands::process_image,
            commands::preview_image,
            commands::process_batch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
