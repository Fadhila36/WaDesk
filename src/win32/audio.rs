use windows::core::{Interface, PCWSTR};
use windows::Win32::Media::Audio::{
    eMultimedia, eRender, IAudioSessionControl2, IAudioSessionManager2,
    IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
};

/// Merename label sesi audio di Windows Volume Mixer agar menampilkan nama akun WhatsApp
pub fn rename_audio_session(label: &str) {
    let wide_label: Vec<u16> = label.encode_utf16().chain(std::iter::once(0)).collect();

    std::thread::spawn(move || unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        if let Ok(enumerator) = CoCreateInstance::<_, IMMDeviceEnumerator>(
            &MMDeviceEnumerator,
            None,
            CLSCTX_ALL,
        ) {
            if let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) {
                if let Ok(session_manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) {
                    if let Ok(session_control) = session_manager.GetAudioSessionControl(None, 0) {
                        if let Ok(session_control2) = session_control.cast::<IAudioSessionControl2>() {
                            let _ = session_control2.SetDisplayName(PCWSTR(wide_label.as_ptr()), std::ptr::null());
                        }
                    }
                }
            }
        }

        CoUninitialize();
    });
}
