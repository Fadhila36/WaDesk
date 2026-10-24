// Account switcher overlay UI in Shadow DOM
(function () {
    var CSS = `
        .wadesk-fab {
            position: fixed; left: 14px; bottom: 18px; width: 40px; height: 40px;
            border-radius: 50%; background: #202c33; color: #e9edef; border: 1px solid #2a3942;
            cursor: pointer; font: 600 14px system-ui, sans-serif; display: flex;
            align-items: center; justify-content: center; opacity: .8; box-sizing: border-box;
            z-index: 999999; box-shadow: 0 4px 12px rgba(0,0,0,0.3); transition: all 0.2s;
        }
        .wadesk-fab:hover { opacity: 1; background: #2a3942; transform: scale(1.05); }
        .wadesk-panel {
            position: fixed; left: 14px; bottom: 68px; width: 260px; background: #233138;
            border: 1px solid #2a3942; border-radius: 12px; padding: 8px; color: #e9edef;
            font: 400 13px system-ui, sans-serif; box-shadow: 0 8px 32px rgba(0,0,0,0.5);
            z-index: 999999; display: none; flex-direction: column; gap: 4px;
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
        fab.innerHTML = '⚡';
        fab.title = 'WaDesk Account Switcher';

        var panel = document.createElement('div');
        panel.className = 'wadesk-panel';
        panel.innerHTML = `
            <div class="wadesk-hdr">WaDesk Accounts</div>
            <div id="wadesk-account-list"></div>
            <div class="wadesk-sep"></div>
            <div class="wadesk-act" id="wadesk-add-btn">+ Add Account</div>
        `;

        fab.addEventListener('click', function(e) {
            e.stopPropagation();
            panel.classList.toggle('show');
            if (panel.classList.contains('show')) {
                refresh();
            }
        });

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
                var name = prompt('Enter new account name:');
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
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
