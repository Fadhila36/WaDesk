// Account switcher overlay UI in Shadow DOM with Rename & Delete capabilities
(function () {
    var CSS = `
        /* Floating button disembunyikan agar layar WhatsApp bersih */
        .wadesk-fab {
            display: none !important;
        }
        .wadesk-backdrop {
            position: fixed; inset: 0; background: rgba(0, 0, 0, 0.55);
            backdrop-filter: blur(6px); z-index: 999998; display: none;
            align-items: center; justify-content: center;
        }
        .wadesk-backdrop.show { display: flex; }
        .wadesk-panel {
            width: 360px; max-width: 90vw; background: #233138;
            border: 1px solid #2a3942; border-radius: 14px; padding: 14px; color: #e9edef;
            font: 400 13px system-ui, -apple-system, sans-serif; box-shadow: 0 16px 40px rgba(0,0,0,0.6);
            display: flex; flex-direction: column; gap: 8px;
            animation: wadesk-fadein 0.18s cubic-bezier(0.16, 1, 0.3, 1);
        }
        @keyframes wadesk-fadein {
            from { opacity: 0; transform: scale(0.95); }
            to { opacity: 1; transform: scale(1); }
        }
        .wadesk-hdr-row {
            display: flex; align-items: center; justify-content: space-between;
            padding: 4px 6px 8px 6px; border-bottom: 1px solid #2a3942; margin-bottom: 2px;
        }
        .wadesk-hdr {
            font-size: 11px; letter-spacing: .08em;
            text-transform: uppercase; color: #00a884; font-weight: 700;
        }
        .wadesk-close {
            cursor: pointer; color: #8696a0; font-size: 18px; line-height: 1;
            padding: 2px 6px; border-radius: 4px; transition: all 0.15s;
        }
        .wadesk-close:hover { color: #e9edef; background: #2a3942; }
        .wadesk-account-list {
            display: flex; flex-direction: column; gap: 4px; max-height: 240px; overflow-y: auto;
        }
        .wadesk-row {
            display: flex; align-items: center; justify-content: space-between;
            padding: 8px 10px; border-radius: 8px; cursor: pointer; transition: background 0.15s;
        }
        .wadesk-row:hover { background: #2a3942; }
        .wadesk-row.active { background: #182229; font-weight: 600; cursor: default; }
        .wadesk-row-left {
            display: flex; align-items: center; gap: 10px; flex: 1; min-width: 0;
        }
        .wadesk-dot { width: 8px; height: 8px; border-radius: 50%; background: #00a884; flex-shrink: 0; }
        .wadesk-row-name {
            overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: #e9edef; font-size: 13px;
        }
        .wadesk-row-actions {
            display: flex; align-items: center; gap: 4px; opacity: 0.6; transition: opacity 0.15s;
        }
        .wadesk-row:hover .wadesk-row-actions { opacity: 1; }
        .wadesk-icon-btn {
            background: none; border: none; cursor: pointer; padding: 4px 6px;
            border-radius: 4px; color: #8696a0; font-size: 13px; line-height: 1;
            display: flex; align-items: center; justify-content: center; transition: all 0.15s;
        }
        .wadesk-icon-btn:hover { background: #3b4a54; color: #e9edef; }
        .wadesk-icon-btn.delete:hover { background: rgba(234, 67, 53, 0.2); color: #ea4335; }
        .wadesk-sep { height: 1px; background: #2a3942; margin: 4px 0; }
        .wadesk-act {
            display: flex; align-items: center; gap: 8px; padding: 9px 12px;
            border-radius: 8px; cursor: pointer; color: #00a884; font-weight: 600; font-size: 13px;
            transition: all 0.15s;
        }
        .wadesk-act:hover { background: #2a3942; color: #25d366; }
    `;

    function init() {
        if (document.getElementById('wadesk-overlay-root')) return;

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
            <div class="wadesk-account-list" id="wadesk-account-list"></div>
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

                var leftDiv = document.createElement('div');
                leftDiv.className = 'wadesk-row-left';
                leftDiv.innerHTML = (acct.id === currentId ? '<div class="wadesk-dot"></div>' : '<div style="width:8px"></div>') +
                    '<span class="wadesk-row-name" title="' + acct.name + '">' + acct.name + '</span>';

                var actionsDiv = document.createElement('div');
                actionsDiv.className = 'wadesk-row-actions';

                // Tombol Rename (Pensil)
                var renameBtn = document.createElement('button');
                renameBtn.className = 'wadesk-icon-btn';
                renameBtn.title = 'Ganti nama akun';
                renameBtn.innerHTML = '✏️';
                renameBtn.addEventListener('click', function(e) {
                    e.stopPropagation();
                    var newName = prompt('Ubah nama akun:', acct.name);
                    if (newName && newName.trim() && newName.trim() !== acct.name) {
                        if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                            window.chrome.webview.postMessage({ type: 'rename_account', id: acct.id, name: newName.trim() });
                        }
                    }
                });
                actionsDiv.appendChild(renameBtn);

                // Tombol Hapus (Hanya untuk non-default)
                if (acct.id !== 'default') {
                    var deleteBtn = document.createElement('button');
                    deleteBtn.className = 'wadesk-icon-btn delete';
                    deleteBtn.title = 'Hapus akun ini';
                    deleteBtn.innerHTML = '🗑️';
                    deleteBtn.addEventListener('click', function(e) {
                        e.stopPropagation();
                        if (confirm('Apakah Anda yakin ingin menghapus akun "' + acct.name + '"?')) {
                            if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                                window.chrome.webview.postMessage({ type: 'remove_account', id: acct.id });
                            }
                        }
                    });
                    actionsDiv.appendChild(deleteBtn);
                }

                row.appendChild(leftDiv);
                row.appendChild(actionsDiv);

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
