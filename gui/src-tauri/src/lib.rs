mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::load_image,
            commands::process_image,
            commands::preview_image,
            commands::process_batch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
