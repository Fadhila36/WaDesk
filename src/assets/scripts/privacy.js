// Privacy blur mode, Media blur, and Auto-Lock for WhatsApp Web
(function () {
    var CSS = `
        /* Text & Chat Content Privacy Blur */
        html[data-wadesk-privacy="on"] [data-testid="cell-frame-title"],
        html[data-wadesk-privacy="on"] [data-testid="last-msg-status"],
        html[data-wadesk-privacy="on"] span[title]:not([data-testid]),
        html[data-wadesk-privacy="on"] .message-in .selectable-text,
        html[data-wadesk-privacy="on"] .message-out .selectable-text {
            filter: blur(6px) !important;
            transition: filter 0.2s ease !important;
            user-select: none !important;
        }

        html[data-wadesk-privacy="on"][data-wadesk-reveal="hover"] [data-testid="cell-frame-title"]:hover,
        html[data-wadesk-privacy="on"][data-wadesk-reveal="hover"] [data-testid="last-msg-status"]:hover,
        html[data-wadesk-privacy="on"][data-wadesk-reveal="hover"] span[title]:not([data-testid]):hover,
        html[data-wadesk-privacy="on"][data-wadesk-reveal="hover"] .message-in:hover .selectable-text,
        html[data-wadesk-privacy="on"][data-wadesk-reveal="hover"] .message-out:hover .selectable-text {
            filter: none !important;
            user-select: text !important;
        }

        /* Media Blur (Hover-to-Reveal) */
        html[data-wadesk-media-blur="on"] img:not([alt*="avatar"]):not([src*="avatar"]),
        html[data-wadesk-media-blur="on"] video,
        html[data-wadesk-media-blur="on"] [data-testid="image-thumb"],
        html[data-wadesk-media-blur="on"] [data-testid="video-thumb"],
        html[data-wadesk-media-blur="on"] [data-testid="audio-player"] {
            filter: blur(14px) !important;
            transition: filter 0.25s ease !important;
        }

        html[data-wadesk-media-blur="on"] img:not([alt*="avatar"]):not([src*="avatar"]):hover,
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

        function apply(on, reveal) {
            if (on) { html.setAttribute('data-wadesk-privacy', 'on'); }
            else { html.removeAttribute('data-wadesk-privacy'); }
            html.setAttribute('data-wadesk-reveal', reveal === 'hover' ? 'hover' : 'hard');
        }

        window.wadeskPrivacyApply = apply;

        // Auto-blur on inactivity / blur
        var autoBlurTimer = null;
        var autoBlurActive = false;
        var autoBlurSeconds = 30;

        function resetAutoBlurTimer() {
            if (autoBlurTimer) {
                clearTimeout(autoBlurTimer);
                autoBlurTimer = null;
            }
        }

        window.addEventListener('blur', function () {
            if (!autoBlurActive) return;
            resetAutoBlurTimer();
            autoBlurTimer = setTimeout(function () {
                apply(true, 'hover');
                if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                    window.chrome.webview.postMessage({ type: 'privacy_toggle', enabled: true });
                }
            }, autoBlurSeconds * 1000);
        });

        window.addEventListener('focus', function () {
            resetAutoBlurTimer();
        });

        window.wadeskSetAutoBlur = function (enabled, secs) {
            autoBlurActive = !!enabled;
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
                var next = html.getAttribute('data-wadesk-privacy') !== 'on';
                apply(next, 'hover');
                if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                    window.chrome.webview.postMessage({ type: 'privacy_toggle', enabled: next });
                }
                return next;
            },
            set: function (on) {
                apply(on, 'hover');
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
