fn main() {
    // Generated from a fixed code-native color mark, not a downloaded asset.
    std::fs::create_dir_all("icons").expect("create icon directory");
    std::fs::write("icons/icon.png", &ICON).expect("write application icon");
    const COMMANDS: &[&str] = &["load_draft", "save_draft", "hide_window", "quit_app", "get_surface", "set_surface", "resize_capsule", "load_provider", "save_provider", "start_chat", "get_chat", "stop_chat", "settle_chat", "get_chat_history", "get_chat_turn", "claim_selection", "acknowledge_selection", "get_attachments", "capture_screenshot", "cancel_screenshot", "remove_attachment", "save_model_secret", "clear_model_secret", "notes_save", "notes_list", "notes_read", "notes_delete", "notes_rebuild"];
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(COMMANDS)
    )).expect("build application manifest");
}
const ICON: [u8; 105] = [137,80,78,71,13,10,26,10,0,0,0,13,73,72,68,82,0,0,0,32,0,0,0,32,8,6,0,0,0,115,122,122,244,0,0,0,48,73,68,65,84,120,156,237,206,33,1,0,0,8,3,48,122,17,132,28,244,23,16,227,102,98,126,213,59,151,84,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,233,192,3,247,81,8,121,93,229,81,79,0,0,0,0,73,69,78,68,174,66,96,130];
