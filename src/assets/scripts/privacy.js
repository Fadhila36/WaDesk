// Privacy blur mode, Media blur, and Auto-Lock for WhatsApp Web
(function () {
    var CSS = `
        /* Message Text & Chat Content Privacy Blur */
        /* Hanya mengaburkan teks pesan obrolan dan preview pesan terakhir, bukan nama atau avatar kontak */
        html[data-wadesk-privacy="on"] [data-testid="last-msg-status"],
        html[data-wadesk-privacy="on"] .message-in .selectable-text,
        html[data-wadesk-privacy="on"] .message-out .selectable-text,
        html[data-wadesk-privacy="on"] [data-testid="msg-container"] .selectable-text,
        html[data-wadesk-privacy="on"] div[data-testid="conversation-panel-messages"] .copyable-text:not([data-testid="cell-frame-title"]) {
            filter: blur(6px) !important;
            transition: filter 0.2s ease !important;
            user-select: none !important;
        }

        html[data-wadesk-privacy="on"][data-wadesk-reveal="hover"] [data-testid="last-msg-status"]:hover,
        html[data-wadesk-privacy="on"][data-wadesk-reveal="hover"] .message-in:hover .selectable-text,
        html[data-wadesk-privacy="on"][data-wadesk-reveal="hover"] .message-out:hover .selectable-text,
        html[data-wadesk-privacy="on"][data-wadesk-reveal="hover"] [data-testid="msg-container"]:hover .selectable-text,
        html[data-wadesk-privacy="on"][data-wadesk-reveal="hover"] div[data-testid="conversation-panel-messages"] .copyable-text:hover {
            filter: none !important;
            user-select: text !important;
        }

        /* Pastikan nama kontak di daftar chat, header chat, dan avatar TIDAK ter-blur sama sekali */
        [data-testid="cell-frame-title"],
        [data-testid="conversation-info-header"],
        [data-testid="chat-title"],
        [data-testid="avatar"],
        img[alt*="avatar"],
        img[src*="avatar"] {
            filter: none !important;
        }

        /* Media Blur (Hover-to-Reveal) */
        html[data-wadesk-media-blur="on"] img:not([alt*="avatar"]):not([src*="avatar"]):not([data-testid="avatar"]),
        html[data-wadesk-media-blur="on"] video,
        html[data-wadesk-media-blur="on"] [data-testid="image-thumb"],
        html[data-wadesk-media-blur="on"] [data-testid="video-thumb"],
        html[data-wadesk-media-blur="on"] [data-testid="audio-player"] {
            filter: blur(14px) !important;
            transition: filter 0.25s ease !important;
        }

        html[data-wadesk-media-blur="on"] img:not([alt*="avatar"]):not([src*="avatar"]):not([data-testid="avatar"]):hover,
        html[data-wadesk-media-blur="on"] video:hover,
        html[data-wadesk-media-blur="on"] [data-testid="image-thumb"]:hover,
        html[data-wadesk-media-blur="on"] [data-testid="video-thumb"]:hover,
        html[data-wadesk-media-blur="on"] [data-testid="audio-player"]:hover {
            filter: none !important;
        }
    `;

    function init() {
        var html = document.documentElement;
        var style = document.createElement('style');
        style.id = 'wadesk-privacy';
        style.textContent = CSS;
        html.appendChild(style);

        // State tracking
        var manualPrivacyActive = false;
        var autoBlurTimer = null;
        var autoBlurEnabled = false;
        var autoBlurSeconds = 30;

        function apply(on, reveal) {
            if (on) {
                html.setAttribute('data-wadesk-privacy', 'on');
            } else {
                html.removeAttribute('data-wadesk-privacy');
            }
            html.setAttribute('data-wadesk-reveal', reveal === 'hover' ? 'hover' : 'hard');
        }

        window.wadeskPrivacyApply = apply;

        function resetAutoBlurTimer() {
            if (autoBlurTimer) {
                clearTimeout(autoBlurTimer);
                autoBlurTimer = null;
            }
        }

        // Dipanggil saat jendela kehilangan fokus (Alt+Tab atau klik aplikasi lain)
        function handleBlur() {
            if (manualPrivacyActive) {
                // Jika user mengaktifkan manual blur, pastikan tetap aktif
                apply(true, 'hover');
                return;
            }
            if (!autoBlurEnabled) return;
            resetAutoBlurTimer();
            autoBlurTimer = setTimeout(function () {
                apply(true, 'hover');
            }, autoBlurSeconds * 1000);
        }

        // Dipanggil saat jendela kembali fokus
        function handleFocus() {
            resetAutoBlurTimer();
            if (manualPrivacyActive) {
                // Manual privacy blur tetap menyala!
                apply(true, 'hover');
            }
        }

        window.addEventListener('blur', handleBlur);
        window.addEventListener('focus', handleFocus);

        // Bridge dipanggil langsung dari Win32 WM_ACTIVATE
        window.wadeskOnWindowBlur = handleBlur;
        window.wadeskOnWindowFocus = handleFocus;

        window.wadeskSetAutoBlur = function (enabled, secs) {
            autoBlurEnabled = !!enabled;
            if (secs && secs > 0) autoBlurSeconds = secs;
            resetAutoBlurTimer();
        };

        window.wadeskSetMediaBlur = function (enabled) {
            if (enabled) {
                html.setAttribute('data-wadesk-media-blur', 'on');
            } else {
                html.removeAttribute('data-wadesk-media-blur');
            }
        };

        window.wadeskPrivacy = {
            toggle: function () {
                manualPrivacyActive = !manualPrivacyActive;
                apply(manualPrivacyActive, 'hover');
                if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                    window.chrome.webview.postMessage({ type: 'privacy_toggle', enabled: manualPrivacyActive });
                }
                return manualPrivacyActive;
            },
            set: function (on) {
                manualPrivacyActive = !!on;
                apply(manualPrivacyActive, 'hover');
            }
        };

        // Pintasan keyboard Ctrl + Shift + P atau Ctrl + Shift + B untuk toggle privacy blur
        document.addEventListener('keydown', function (ev) {
            if (!ev.ctrlKey || !ev.shiftKey) { return; }
            if (ev.key !== 'P' && ev.key !== 'p' && ev.key !== 'B' && ev.key !== 'b') { return; }
            ev.preventDefault();
            ev.stopPropagation();
            window.wadeskPrivacy.toggle();
        }, true);
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
