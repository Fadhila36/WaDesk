// Account switcher overlay UI in Shadow DOM
(function () {
    var CSS = `
        /* Floating button disembunyikan agar layar WhatsApp bersih */
        .wadesk-fab {
            display: none !important;
        }
        .wadesk-backdrop {
            position: fixed; inset: 0; background: rgba(0, 0, 0, 0.45);
            backdrop-filter: blur(4px); z-index: 999998; display: none;
            align-items: center; justify-content: center;
        }
        .wadesk-backdrop.show { display: flex; }
        .wadesk-panel {
            width: 320px; background: #233138;
            border: 1px solid #2a3942; border-radius: 12px; padding: 12px; color: #e9edef;
            font: 400 13px system-ui, sans-serif; box-shadow: 0 12px 40px rgba(0,0,0,0.6);
            display: flex; flex-direction: column; gap: 6px;
            animation: wadesk-fadein 0.18s cubic-bezier(0.16, 1, 0.3, 1);
        }
        @keyframes wadesk-fadein {
            from { opacity: 0; transform: scale(0.95); }
            to { opacity: 1; transform: scale(1); }
        }
        .wadesk-hdr-row {
            display: flex; align-items: center; justify-content: space-between;
            padding: 4px 6px 8px 6px; border-bottom: 1px solid #2a3942; margin-bottom: 4px;
        }
        .wadesk-hdr {
            font-size: 11px; letter-spacing: .08em;
            text-transform: uppercase; color: #00a884; font-weight: 700;
        }
        .wadesk-close {
            cursor: pointer; color: #8696a0; font-size: 16px; line-height: 1;
            padding: 2px 6px; border-radius: 4px;
        }
        .wadesk-close:hover { color: #e9edef; background: #2a3942; }
        .wadesk-row {
            display: flex; align-items: center; gap: 10px; padding: 10px 12px;
            border-radius: 8px; cursor: pointer; transition: background 0.15s;
        }
        .wadesk-row:hover { background: #2a3942; }
        .wadesk-row.active { background: #182229; font-weight: 600; cursor: default; }
        .wadesk-dot { width: 8px; height: 8px; border-radius: 50%; background: #00a884; }
        .wadesk-sep { height: 1px; background: #2a3942; margin: 4px 0; }
        .wadesk-act {
            display: flex; align-items: center; gap: 8px; padding: 10px 12px;
            border-radius: 8px; cursor: pointer; color: #00a884; font-weight: 500;
        }
        .wadesk-act:hover { background: #2a3942; }
    `;

    function init() {
        var host = document.createElement('div');
        host.id = 'wadesk-overlay-root';
        var root = host.attachShadow({ mode: 'open' });

        var style = document.createElement('style');
        style.textContent = CSS;
        root.appendChild(style);

        var backdrop = document.createElement('div');
        backdrop.className = 'wadesk-backdrop';

        var panel = document.createElement('div');
        panel.className = 'wadesk-panel';
        panel.innerHTML = `
            <div class="wadesk-hdr-row">
                <span class="wadesk-hdr">⚡ WaDesk Multi-Akun</span>
                <span class="wadesk-close" id="wadesk-close-btn">&times;</span>
            </div>
            <div id="wadesk-account-list"></div>
            <div class="wadesk-sep"></div>
            <div class="wadesk-act" id="wadesk-add-btn">+ Tambah Akun Baru</div>
        `;

        function openModal() {
            backdrop.classList.add('show');
            refresh();
        }

        function closeModal() {
            backdrop.classList.remove('show');
        }

        backdrop.addEventListener('click', function(e) {
            if (e.target === backdrop) {
                closeModal();
            }
        });

        panel.addEventListener('click', function(e) {
            e.stopPropagation();
        });

        var closeBtn = panel.querySelector('#wadesk-close-btn');
        if (closeBtn) {
            closeBtn.addEventListener('click', closeModal);
        }

        backdrop.appendChild(panel);
        root.appendChild(backdrop);
        document.documentElement.appendChild(host);

        function refresh() {
            if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                window.chrome.webview.postMessage({ type: 'get_accounts' });
            }
        }

        var addBtn = panel.querySelector('#wadesk-add-btn');
        if (addBtn) {
            addBtn.addEventListener('click', function() {
                var name = prompt('Masukkan nama akun baru:');
                if (name && name.trim()) {
                    if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                        window.chrome.webview.postMessage({ type: 'add_account', name: name.trim() });
                    }
                }
            });
        }

        window.wadeskRenderAccounts = function(accounts, currentId) {
            var list = panel.querySelector('#wadesk-account-list');
            if (!list) return;
            list.innerHTML = '';
            accounts.forEach(function(acct) {
                var row = document.createElement('div');
                row.className = 'wadesk-row' + (acct.id === currentId ? ' active' : '');
                row.innerHTML = (acct.id === currentId ? '<div class="wadesk-dot"></div>' : '<div style="width:8px"></div>') +
                    '<span>' + acct.name + '</span>';
                if (acct.id !== currentId) {
                    row.addEventListener('click', function() {
                        if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                            closeModal();
                            window.chrome.webview.postMessage({ type: 'switch_account', id: acct.id });
                        }
                    });
                }
                list.appendChild(row);
            });
        };

        window.wadeskOverlay = {
            open: openModal,
            close: closeModal,
            toggle: function() {
                if (backdrop.classList.contains('show')) {
                    closeModal();
                } else {
                    openModal();
                }
            }
        };
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
