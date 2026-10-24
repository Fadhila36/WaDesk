use wadesk::webhooks::watcher::{detect_unread_changes, ChatItem};

#[test]
fn test_detect_unread_changes() {
    let baseline = vec![
        ChatItem { name: "Alice".to_string(), unread: 0, preview: "old msg".to_string() },
        ChatItem { name: "Bob".to_string(), unread: 1, preview: "need report".to_string() },
    ];

    let current = vec![
        // Alice unread bertambah dari 0 ke 1 -> EVENT!
        ChatItem { name: "Alice".to_string(), unread: 1, preview: "pagi".to_string() },
        // Bob unread tetap 1 -> NO EVENT
        ChatItem { name: "Bob".to_string(), unread: 1, preview: "need report".to_string() },
        // Charlie chat baru dengan unread 2 -> EVENT!
        ChatItem { name: "Charlie".to_string(), unread: 2, preview: "meeting yuk".to_string() },
    ];

    let events = detect_unread_changes(&baseline, &current, "Account 1", "default");
    assert_eq!(events.len(), 2, "Harus menghasilkan 2 event unread bertambah");

    assert_eq!(events[0].chat, "Alice");
    assert_eq!(events[0].unread, 1);
    assert_eq!(events[0].text, "pagi");
    assert_eq!(events[0].account, "Account 1");
    assert_eq!(events[0].profile, "default");
    assert_eq!(events[0].event, "message.incoming");

    assert_eq!(events[1].chat, "Charlie");
    assert_eq!(events[1].unread, 2);
}

#[test]
fn test_no_event_on_read_or_decrease() {
    let baseline = vec![
        ChatItem { name: "Alice".to_string(), unread: 2, preview: "pagi".to_string() },
    ];

    let current = vec![
        // User membaca chat, unread turun ke 0 -> NO EVENT
        ChatItem { name: "Alice".to_string(), unread: 0, preview: "pagi".to_string() },
    ];

    let events = detect_unread_changes(&baseline, &current, "Account 1", "default");
    assert_eq!(events.len(), 0, "Tidak boleh ada event saat unread berkurang");
}
