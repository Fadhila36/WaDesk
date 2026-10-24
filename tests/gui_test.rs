use wadesk::gui::state::WindowState;

#[test]
fn test_window_state_default() {
    let state = WindowState::default();
    assert_eq!(state.width, 1100);
    assert_eq!(state.height, 750);
    assert!(!state.maximized);
}

#[test]
fn test_window_state_roundtrip() {
    let temp_dir = std::env::temp_dir().join(format!("wadesk_gui_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let state_file = temp_dir.join("window_state.json");

    let mut state = WindowState::default();
    state.x = Some(150);
    state.y = Some(200);
    state.width = 1280;
    state.height = 800;
    state.maximized = true;

    state.save(&state_file).expect("save window state should succeed");

    let loaded = WindowState::load_or_default(&state_file);
    assert_eq!(loaded.x, Some(150));
    assert_eq!(loaded.y, Some(200));
    assert_eq!(loaded.width, 1280);
    assert_eq!(loaded.height, 800);
    assert!(loaded.maximized);

    let _ = std::fs::remove_dir_all(&temp_dir);
}
