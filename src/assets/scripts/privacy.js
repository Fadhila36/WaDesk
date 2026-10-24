// Privacy blur mode for WhatsApp Web
(function () {
    var CSS = `
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

        // Pintasan keyboard Ctrl + Shift + B untuk toggle privacy blur
        document.addEventListener('keydown', function (ev) {
            if (!ev.ctrlKey || !ev.shiftKey) { return; }
            if (ev.key !== 'B' && ev.key !== 'b') { return; }
            ev.preventDefault();
            ev.stopPropagation();
            var next = html.getAttribute('data-wadesk-privacy') !== 'on';
            apply(next, 'hover');
            if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                window.chrome.webview.postMessage({ type: 'privacy_toggle', enabled: next });
            }
        }, true);
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
