#![cfg(windows)]

use wadesk::win32::instance::SingleInstanceGuard;

#[test]
fn test_single_instance_guard_behavior() {
    let profile_a = "test_profile_alpha";
    let profile_b = "test_profile_beta";

    // Instance 1 untuk profile A harus berhasil menjadi primary
    let guard1 = SingleInstanceGuard::acquire(profile_a).expect("acquire guard1 should succeed");
    assert!(guard1.is_primary(), "guard1 should be primary instance");

    // Instance 2 untuk profile A harus mendeteksi bahwa sudah ada instance yang berjalan
    let guard2 = SingleInstanceGuard::acquire(profile_a).expect("acquire guard2 should succeed");
    assert!(!guard2.is_primary(), "guard2 should NOT be primary instance (collision detected)");

    // Instance untuk profile B harus independen dan berhasil menjadi primary
    let guard_b = SingleInstanceGuard::acquire(profile_b).expect("acquire guard_b should succeed");
    assert!(guard_b.is_primary(), "guard_b should be primary for profile B");

    // Di Win32, mutex kernel tetap aktif selama masih ada handle yang terbuka (guard1 dan guard2).
    // Setelah guard1 dan guard2 di-drop, mutex hancur, dan guard3 bisa menjadi primary.
    drop(guard1);
    drop(guard2);
    let guard3 = SingleInstanceGuard::acquire(profile_a).expect("acquire guard3 should succeed");
    assert!(guard3.is_primary(), "guard3 should be primary after all handles are dropped");
}
