// WaDesk In-App Professional Update Modal & Background Downloader UI
(function () {
    var CSS = `
        .wadesk-update-backdrop {
            position: fixed; inset: 0;
            background: rgba(11, 20, 26, 0.72);
            backdrop-filter: blur(8px);
            z-index: 999999;
            display: none;
            align-items: center;
            justify-content: center;
            opacity: 0;
            transition: opacity 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        }
        .wadesk-update-backdrop.show {
            display: flex;
            opacity: 1;
        }
        .wadesk-update-modal {
            width: 440px;
            max-width: 90vw;
            background: #1f2c34;
            border: 1px solid #2a3942;
            border-radius: 16px;
            padding: 22px 24px;
            box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
            color: #e9edef;
            font-family: Segoe UI, -apple-system, BlinkMacSystemFont, Roboto, sans-serif;
            display: flex;
            flex-direction: column;
            gap: 16px;
            transform: scale(0.96);
            transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        }
        .wadesk-update-backdrop.show .wadesk-update-modal {
            transform: scale(1);
        }
        .wadesk-modal-hdr {
            display: flex;
            align-items: flex-start;
            justify-content: space-between;
            gap: 12px;
        }
        .wadesk-hdr-left {
            display: flex;
            align-items: center;
            gap: 14px;
        }
        .wadesk-badge-icon {
            width: 44px;
            height: 44px;
            border-radius: 12px;
            background: rgba(0, 168, 132, 0.15);
            border: 1px solid rgba(0, 168, 132, 0.3);
            display: flex;
            align-items: center;
            justify-content: center;
            color: #00a884;
            flex-shrink: 0;
        }
        .wadesk-hdr-title {
            font-size: 17px;
            font-weight: 600;
            color: #e9edef;
            margin: 0 0 4px 0;
            line-height: 1.2;
        }
        .wadesk-versions {
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 12px;
        }
        .wadesk-ver-pill {
            padding: 2px 8px;
            border-radius: 12px;
            background: rgba(134, 150, 160, 0.12);
            color: #8696a0;
            font-weight: 500;
        }
        .wadesk-ver-arrow {
            color: #8696a0;
            font-size: 11px;
        }
        .wadesk-ver-new {
            background: rgba(0, 168, 132, 0.15);
            color: #00a884;
            font-weight: 600;
        }
        .wadesk-close-btn {
            background: none;
            border: none;
            color: #8696a0;
            font-size: 20px;
            line-height: 1;
            cursor: pointer;
            padding: 4px 8px;
            border-radius: 6px;
            transition: all 0.15s;
        }
        .wadesk-close-btn:hover {
            color: #e9edef;
            background: #2a3942;
        }
        .wadesk-changelog-card {
            background: #111b21;
            border: 1px solid #2a3942;
            border-radius: 10px;
            padding: 12px 14px;
            max-height: 140px;
            overflow-y: auto;
        }
        .wadesk-changelog-card::-webkit-scrollbar {
            width: 5px;
        }
        .wadesk-changelog-card::-webkit-scrollbar-thumb {
            background: #2a3942;
            border-radius: 4px;
        }
        .wadesk-changelog-hdr {
            font-size: 11px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.06em;
            color: #8696a0;
            margin-bottom: 8px;
        }
        .wadesk-changelog-list {
            margin: 0;
            padding: 0;
            list-style: none;
            display: flex;
            flex-direction: column;
            gap: 6px;
        }
        .wadesk-changelog-item {
            font-size: 13px;
            color: #d1d7db;
            line-height: 1.4;
            display: flex;
            align-items: flex-start;
            gap: 8px;
        }
        .wadesk-changelog-dot {
            color: #00a884;
            font-size: 12px;
            margin-top: 1px;
        }
        .wadesk-progress-card {
            display: none;
            flex-direction: column;
            gap: 6px;
            background: #111b21;
            border: 1px solid #2a3942;
            border-radius: 10px;
            padding: 12px 14px;
        }
        .wadesk-progress-card.show {
            display: flex;
        }
        .wadesk-progress-labels {
            display: flex;
            justify-content: space-between;
            font-size: 12px;
            color: #8696a0;
        }
        .wadesk-progress-track {
            height: 8px;
            background: #1f2c34;
            border-radius: 999px;
            overflow: hidden;
            position: relative;
        }
        .wadesk-progress-fill {
            height: 100%;
            width: 0%;
            background: linear-gradient(90deg, #00a884, #25d366);
            border-radius: 999px;
            transition: width 0.25s ease-out;
        }
        .wadesk-modal-ftr {
            display: flex;
            align-items: center;
            justify-content: flex-end;
            gap: 10px;
            margin-top: 4px;
        }
        .wadesk-btn {
            font-family: inherit;
            font-size: 13px;
            padding: 9px 18px;
            border-radius: 8px;
            cursor: pointer;
            transition: all 0.15s ease;
            display: inline-flex;
            align-items: center;
            gap: 6px;
        }
        .wadesk-btn-secondary {
            background: transparent;
            border: 1px solid #3b4a54;
            color: #8696a0;
            font-weight: 500;
        }
        .wadesk-btn-secondary:hover {
            background: #2a3942;
            color: #e9edef;
        }
        .wadesk-btn-primary {
            background: #00a884;
            border: none;
            color: #111b21;
            font-weight: 600;
            box-shadow: 0 4px 12px rgba(0, 168, 132, 0.25);
        }
        .wadesk-btn-primary:hover {
            background: #02906f;
        }
        .wadesk-btn:disabled {
            opacity: 0.6;
            cursor: not-allowed;
        }
        .wadesk-toast {
            position: fixed;
            top: 24px;
            left: 50%;
            transform: translateX(-50%) translateY(-10px);
            background: #1f2c34;
            border: 1px solid #2a3942;
            border-radius: 24px;
            padding: 8px 18px;
            color: #00a884;
            font-size: 13px;
            font-weight: 500;
            box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
            z-index: 999999;
            display: flex;
            align-items: center;
            gap: 8px;
            opacity: 0;
            pointer-events: none;
            transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
        }
        .wadesk-toast.show {
            opacity: 1;
            transform: translateX(-50%) translateY(0);
        }
    `;

    function init() {
        if (document.getElementById('wadesk-update-root')) return;

        var host = document.createElement('div');
        host.id = 'wadesk-update-root';
        var root = host.attachShadow({ mode: 'open' });

        var style = document.createElement('style');
        style.textContent = CSS;
        root.appendChild(style);

        // Toast element
        var toast = document.createElement('div');
        toast.className = 'wadesk-toast';
        toast.innerHTML = '<span>⚡</span><span id="wadesk-toast-text">WaDesk</span>';
        root.appendChild(toast);

        // Backdrop & Modal
        var backdrop = document.createElement('div');
        backdrop.className = 'wadesk-update-backdrop';

        var modal = document.createElement('div');
        modal.className = 'wadesk-update-modal';
        modal.innerHTML = `
            <div class="wadesk-modal-hdr">
                <div class="wadesk-hdr-left">
                    <div class="wadesk-badge-icon">
                        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                            <polyline points="7 10 12 15 17 10"></polyline>
                            <line x1="12" y1="15" x2="12" y2="3"></line>
                        </svg>
                    </div>
                    <div>
                        <h3 class="wadesk-hdr-title">Pembaruan Tersedia</h3>
                        <div class="wadesk-versions">
                            <span class="wadesk-ver-pill" id="wadesk-cur-ver">v1.0.7</span>
                            <span class="wadesk-ver-arrow">&rarr;</span>
                            <span class="wadesk-ver-pill wadesk-ver-new" id="wadesk-lat-ver">v1.0.8</span>
                        </div>
                    </div>
                </div>
                <button class="wadesk-close-btn" id="wadesk-btn-close" title="Nanti Saja">&times;</button>
            </div>

            <div class="wadesk-changelog-card" id="wadesk-changelog-box">
                <div class="wadesk-changelog-hdr">Sorotan Perubahan</div>
                <ul class="wadesk-changelog-list" id="wadesk-changelog-items"></ul>
            </div>

            <div class="wadesk-progress-card" id="wadesk-progress-box">
                <div class="wadesk-progress-labels">
                    <span id="wadesk-progress-status">Mengunduh di latar belakang...</span>
                    <span id="wadesk-progress-percent">0%</span>
                </div>
                <div class="wadesk-progress-track">
                    <div class="wadesk-progress-fill" id="wadesk-progress-bar"></div>
                </div>
            </div>

            <div class="wadesk-modal-ftr" id="wadesk-actions-box">
                <button class="wadesk-btn wadesk-btn-secondary" id="wadesk-btn-later">Nanti Saja</button>
                <button class="wadesk-btn wadesk-btn-primary" id="wadesk-btn-download">
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                        <polyline points="7 10 12 15 17 10"></polyline>
                        <line x1="12" y1="15" x2="12" y2="3"></line>
                    </svg>
                    <span>Perbarui Sekarang</span>
                </button>
            </div>
        `;

        backdrop.appendChild(modal);
        root.appendChild(backdrop);
        document.documentElement.appendChild(host);

        var currentUpdateData = null;

        function closeModal() {
            backdrop.classList.remove('show');
        }

        function showModal(data) {
            currentUpdateData = data;
            root.getElementById('wadesk-cur-ver').textContent = 'v' + data.currentVersion;
            root.getElementById('wadesk-lat-ver').textContent = 'v' + data.latestVersion;

            var list = root.getElementById('wadesk-changelog-items');
            list.innerHTML = '';
            if (data.highlights && data.highlights.length > 0) {
                data.highlights.forEach(function (text) {
                    var li = document.createElement('li');
                    li.className = 'wadesk-changelog-item';
                    li.innerHTML = '<span class="wadesk-changelog-dot">•</span><span>' + text.replace(/^[•\-\*]\s*/, '') + '</span>';
                    list.appendChild(li);
                });
            } else {
                var li = document.createElement('li');
                li.className = 'wadesk-changelog-item';
                li.innerHTML = '<span class="wadesk-changelog-dot">•</span><span>Peningkatan stabilitas dan performa sistem.</span>';
                list.appendChild(li);
            }

            // Reset progress box
            root.getElementById('wadesk-progress-box').classList.remove('show');
            root.getElementById('wadesk-progress-bar').style.width = '0%';
            root.getElementById('wadesk-progress-percent').textContent = '0%';
            root.getElementById('wadesk-actions-box').style.display = 'flex';
            root.getElementById('wadesk-btn-download').disabled = false;

            backdrop.classList.add('show');
        }

        root.getElementById('wadesk-btn-close').addEventListener('click', closeModal);
        root.getElementById('wadesk-btn-later').addEventListener('click', closeModal);

        backdrop.addEventListener('click', function (e) {
            if (e.target === backdrop) {
                // Klik backdrop untuk menutup jika belum mengunduh
                var progressBox = root.getElementById('wadesk-progress-box');
                if (!progressBox.classList.contains('show')) {
                    closeModal();
                }
            }
        });

        root.getElementById('wadesk-btn-download').addEventListener('click', function () {
            if (!currentUpdateData) return;

            // Masuk ke mode unduh di latar belakang (tanpa direct ke browser)
            root.getElementById('wadesk-progress-box').classList.add('show');
            root.getElementById('wadesk-btn-download').disabled = true;
            root.getElementById('wadesk-btn-later').style.display = 'none';

            if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                window.chrome.webview.postMessage({
                    type: 'download_update',
                    url: currentUpdateData.downloadUrl || currentUpdateData.releaseUrl,
                    version: currentUpdateData.latestVersion
                });
            }
        });

        // Global functions dipanggil dari Rust
        window.wadeskShowUpdateModal = function (data) {
            showModal(data);
        };

        window.wadeskUpdateProgress = function (percent, statusText) {
            var box = root.getElementById('wadesk-progress-box');
            if (box) box.classList.add('show');

            var bar = root.getElementById('wadesk-progress-bar');
            if (bar) bar.style.width = percent + '%';

            var percEl = root.getElementById('wadesk-progress-percent');
            if (percEl) percEl.textContent = percent + '%';

            var statusEl = root.getElementById('wadesk-progress-status');
            if (statusEl && statusText) statusEl.textContent = statusText;
        };

        window.wadeskUpdateError = function (errorMsg) {
            var statusEl = root.getElementById('wadesk-progress-status');
            if (statusEl) statusEl.textContent = 'Gagal: ' + errorMsg;
            var btnLater = root.getElementById('wadesk-btn-later');
            if (btnLater) {
                btnLater.style.display = 'inline-flex';
                btnLater.textContent = 'Tutup';
            }
        };

        window.wadeskShowToast = function (message) {
            var textEl = root.getElementById('wadesk-toast-text');
            if (textEl) textEl.textContent = message;
            toast.classList.add('show');
            setTimeout(function () {
                toast.classList.remove('show');
            }, 3500);
        };
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
