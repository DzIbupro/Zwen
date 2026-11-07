fn main() {
    println!("cargo:rerun-if-changed=icons/icon.png");
    const COMMANDS: &[&str] = &["load_appearance", "save_appearance", "load_draft", "save_draft", "hide_window", "quit_app", "get_surface", "set_surface", "resize_capsule", "load_provider", "save_provider", "start_chat", "get_chat", "stop_chat", "settle_chat", "get_chat_history", "delete_chat_turn", "get_chat_turn", "claim_selection", "acknowledge_selection", "get_attachments", "capture_screenshot", "cancel_screenshot", "remove_attachment", "save_model_secret", "clear_model_secret", "notes_save", "notes_list", "notes_read", "notes_delete", "notes_update", "notes_rebuild", "organize_run", "organize_topics", "organize_topic_notes", "organize_undo", "note_export", "notes_export_all", "open_external", "notes_update_source", "organize_rename", "organize_set_column", "organize_merge", "organize_split", "organize_set_locked", "organize_delete_topic", "organize_add_relation", "organize_remove_relation", "organize_list_relations", "organize_set_auto", "organize_get_auto", "candidate_set_enabled", "candidate_get_enabled", "freeze_selection", "start_selection_chat", "load_quick_actions", "save_quick_actions", "save_action_document"];
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(COMMANDS)
    )).expect("build application manifest");
}
