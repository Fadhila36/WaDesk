use wadesk::assets::{get_init_scripts, Prefs};

#[test]
fn test_init_scripts_integrity() {
    let prefs = Prefs::default();
    let scripts = get_init_scripts(&prefs);

    assert_eq!(scripts.len(), 5, "Must produce user-agent/notification, overlay, privacy, agent, and updater scripts");
    for (i, script) in scripts.iter().enumerate() {
        assert!(!script.trim().is_empty(), "Script {} must not be empty", i);
        assert!(script.contains("function"), "Script {} must contain JS function definitions", i);
    }

    // Verify privacy blur contains CSS rules
    assert!(scripts[2].contains("data-wadesk-privacy") || scripts[2].contains("privacy"));
    // Verify agent contains MCP methods
    assert!(scripts[3].contains("list_chats") || scripts[3].contains("wadeskAgent"));
    // Verify updater contains update modal
    assert!(scripts[4].contains("wadeskShowUpdateModal") || scripts[4].contains("wadesk-update-modal"));
}
