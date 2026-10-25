// Account switcher overlay UI in Shadow DOM
(function () {
    var CSS = `
        /* Reposisi FAB ke pojok kanan bawah dengan desain subtle pill/badge modern */
        .wadesk-fab {
            position: fixed; right: 20px; bottom: 20px; height: 32px; padding: 0 12px;
            border-radius: 16px; background: rgba(32, 44, 51, 0.75); color: #8696a0; border: 1px solid rgba(42, 57, 66, 0.8);
            backdrop-filter: blur(8px); cursor: pointer; font: 500 12px system-ui, sans-serif; display: flex;
            align-items: center; justify-content: center; gap: 6px; box-sizing: border-box;
            z-index: 999999; box-shadow: 0 4px 16px rgba(0,0,0,0.25); transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
            user-select: none;
        }
        .wadesk-fab:hover {
            background: rgba(42, 57, 66, 0.95); color: #e9edef; transform: translateY(-2px);
            border-color: #00a884; box-shadow: 0 6px 20px rgba(0,0,0,0.4);
        }
        .wadesk-fab-icon {
            font-size: 13px; color: #00a884;
        }
        .wadesk-panel {
            position: fixed; right: 20px; bottom: 60px; width: 250px; background: #233138;
            border: 1px solid #2a3942; border-radius: 12px; padding: 8px; color: #e9edef;
            font: 400 13px system-ui, sans-serif; box-shadow: 0 8px 32px rgba(0,0,0,0.5);
            z-index: 999999; display: none; flex-direction: column; gap: 4px;
            animation: wadesk-fadein 0.15s ease-out;
        }
        @keyframes wadesk-fadein {
            from { opacity: 0; transform: translateY(6px); }
            to { opacity: 1; transform: translateY(0); }
        }
        .wadesk-panel.show { display: flex; }
        .wadesk-hdr {
            padding: 6px 10px; font-size: 11px; letter-spacing: .08em;
            text-transform: uppercase; color: #8696a0; font-weight: 700;
        }
        .wadesk-row {
            display: flex; align-items: center; gap: 8px; padding: 8px 10px;
            border-radius: 6px; cursor: pointer; transition: background 0.15s;
        }
        .wadesk-row:hover { background: #2a3942; }
        .wadesk-row.active { background: #182229; font-weight: 600; cursor: default; }
        .wadesk-dot { width: 8px; height: 8px; border-radius: 50%; background: #00a884; }
        .wadesk-sep { height: 1px; background: #2a3942; margin: 4px 0; }
        .wadesk-act {
            display: flex; align-items: center; gap: 8px; padding: 8px 10px;
            border-radius: 6px; cursor: pointer; color: #00a884; font-weight: 500;
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

        var fab = document.createElement('div');
        fab.className = 'wadesk-fab';
        fab.innerHTML = '<span class="wadesk-fab-icon">⚡</span><span>Akun</span>';
        fab.title = 'WaDesk Account Switcher (Ctrl+Shift+A)';

        var panel = document.createElement('div');
        panel.className = 'wadesk-panel';
        panel.innerHTML = `
            <div class="wadesk-hdr">WaDesk Accounts</div>
            <div id="wadesk-account-list"></div>
            <div class="wadesk-sep"></div>
            <div class="wadesk-act" id="wadesk-add-btn">+ Tambah Akun</div>
        `;

        function togglePanel(e) {
            if (e) e.stopPropagation();
            panel.classList.toggle('show');
            if (panel.classList.contains('show')) {
                refresh();
            }
        }

        fab.addEventListener('click', togglePanel);

        document.addEventListener('click', function() {
            panel.classList.remove('show');
        });

        panel.addEventListener('click', function(e) {
            e.stopPropagation();
        });

        root.appendChild(fab);
        root.appendChild(panel);
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
                            window.chrome.webview.postMessage({ type: 'switch_account', id: acct.id });
                        }
                    });
                }
                list.appendChild(row);
            });
        };

        window.wadeskOverlay = {
            open: function() {
                panel.classList.add('show');
                refresh();
            },
            close: function() {
                panel.classList.remove('show');
            },
            toggle: togglePanel
        };
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
