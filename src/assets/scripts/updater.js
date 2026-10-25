// WaDesk In-App Professional Update Modal & Background Downloader UI
(function () {
    var CSS = `
        .wadesk-update-backdrop {
            position: fixed; inset: 0;
            background: rgba(11, 20, 26, 0.75);
            backdrop-filter: blur(4px);
            z-index: 999999;
            display: none;
            align-items: center;
            justify-content: center;
            opacity: 0;
            transition: opacity 0.15s ease;
        }
        .wadesk-update-backdrop.show {
            display: flex;
            opacity: 1;
        }
        .wadesk-update-modal {
            width: 440px;
            max-width: 90vw;
            background: #202c33;
            border: 1px solid rgba(134, 150, 160, 0.2);
            border-radius: 10px;
            padding: 22px;
            box-shadow: 0 16px 40px rgba(0, 0, 0, 0.55);
            color: #e9edef;
            font-family: Segoe UI, -apple-system, BlinkMacSystemFont, Roboto, sans-serif;
            display: flex;
            flex-direction: column;
            gap: 14px;
            transform: scale(0.97);
            transition: transform 0.15s ease;
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
        .wadesk-hdr-title {
            font-size: 16px;
            font-weight: 600;
            color: #e9edef;
            margin: 0 0 4px 0;
            line-height: 1.3;
        }
        .wadesk-subtext {
            font-size: 13px;
            color: #8696a0;
            margin: 0;
            line-height: 1.4;
        }
        .wadesk-close-btn {
            background: none;
            border: none;
            color: #8696a0;
            font-size: 18px;
            line-height: 1;
            cursor: pointer;
            padding: 2px 6px;
            border-radius: 4px;
            transition: color 0.15s, background 0.15s;
        }
        .wadesk-close-btn:hover {
            color: #e9edef;
            background: #2a3942;
        }
        .wadesk-changelog-card {
            background: #111b21;
            border: 1px solid #2a3942;
            border-radius: 6px;
            padding: 12px 14px;
            max-height: 130px;
            overflow-y: auto;
        }
        .wadesk-changelog-card::-webkit-scrollbar {
            width: 4px;
        }
        .wadesk-changelog-card::-webkit-scrollbar-thumb {
            background: #2a3942;
            border-radius: 2px;
        }
        .wadesk-changelog-hdr {
            font-size: 12px;
            font-weight: 600;
            color: #d1d7db;
            margin-bottom: 6px;
        }
        .wadesk-changelog-list {
            margin: 0;
            padding: 0 0 0 16px;
            list-style-type: disc;
            color: #aebac1;
            font-size: 12px;
            line-height: 1.5;
        }
        .wadesk-changelog-item {
            padding: 2px 0;
        }
        .wadesk-progress-card {
            display: none;
            flex-direction: column;
            gap: 6px;
            background: #111b21;
            border: 1px solid #2a3942;
            border-radius: 6px;
            padding: 10px 12px;
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
            height: 6px;
            background: #222e35;
            border-radius: 3px;
            overflow: hidden;
        }
        .wadesk-progress-fill {
            height: 100%;
            width: 0%;
            background: #00a884;
            transition: width 0.2s ease;
        }
        .wadesk-modal-ftr {
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 12px;
            margin-top: 4px;
        }
        .wadesk-ftr-left {
            display: flex;
            align-items: center;
            gap: 12px;
        }
        .wadesk-link-action {
            font-size: 12px;
            color: #8696a0;
            background: none;
            border: none;
            cursor: pointer;
            padding: 2px 0;
            transition: color 0.15s;
        }
        .wadesk-link-action:hover {
            color: #d1d7db;
        }
        .wadesk-ftr-right {
            display: flex;
            align-items: center;
            gap: 8px;
        }
        .wadesk-btn {
            font-family: inherit;
            font-size: 13px;
            padding: 7px 16px;
            border-radius: 6px;
            cursor: pointer;
            transition: background 0.15s, border-color 0.15s;
            display: inline-flex;
            align-items: center;
            justify-content: center;
        }
        .wadesk-btn-secondary {
            background: transparent;
            border: 1px solid #3b4a54;
            color: #d1d7db;
            font-weight: 500;
        }
        .wadesk-btn-secondary:hover {
            background: #2a3942;
            color: #ffffff;
        }
        .wadesk-btn-primary {
            background: #00a884;
            border: 1px solid #00a884;
            color: #111b21;
            font-weight: 600;
        }
        .wadesk-btn-primary:hover {
            background: #02906f;
            border-color: #02906f;
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
            background: #202c33;
            border: 1px solid #2a3942;
            border-radius: 8px;
            padding: 8px 16px;
            color: #e9edef;
            font-size: 13px;
            font-weight: 500;
            box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
            z-index: 999999;
            display: flex;
            align-items: center;
            gap: 8px;
            opacity: 0;
            pointer-events: none;
            transition: all 0.2s ease;
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
        toast.innerHTML = '<span id="wadesk-toast-text">WaDesk</span>';
        root.appendChild(toast);

        // Backdrop & Modal
        var backdrop = document.createElement('div');
        backdrop.className = 'wadesk-update-backdrop';

        var modal = document.createElement('div');
        modal.className = 'wadesk-update-modal';
        modal.innerHTML = `
            <div class="wadesk-modal-hdr">
                <div>
                    <h3 class="wadesk-hdr-title" id="wadesk-title-text">Pembaruan Tersedia</h3>
                    <p class="wadesk-subtext" id="wadesk-subtext">Versi baru siap dipasang pada WaDesk.</p>
                </div>
                <button class="wadesk-close-btn" id="wadesk-btn-close" title="Tutup (Esc)">&times;</button>
            </div>

            <div class="wadesk-changelog-card" id="wadesk-changelog-box">
                <div class="wadesk-changelog-hdr">Catatan Perubahan</div>
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
                <div class="wadesk-ftr-left">
                    <button class="wadesk-link-action" id="wadesk-btn-snooze" title="Jangan ingatkan selama 24 jam">Tunda 24 Jam</button>
                    <button class="wadesk-link-action" id="wadesk-btn-skip" title="Lewati notifikasi versi ini">Lewati Versi Ini</button>
                </div>
                <div class="wadesk-ftr-right">
                    <button class="wadesk-btn wadesk-btn-secondary" id="wadesk-btn-later">Nanti Saja</button>
                    <button class="wadesk-btn wadesk-btn-primary" id="wadesk-btn-download">
                        <span id="wadesk-btn-text">Perbarui Sekarang</span>
                    </button>
                </div>
            </div>
        `;

        backdrop.appendChild(modal);
        root.appendChild(backdrop);
        document.documentElement.appendChild(host);

        var currentUpdateData = null;
        var restartTimer = null;

        function closeModal() {
            if (restartTimer) {
                clearInterval(restartTimer);
                restartTimer = null;
            }
            backdrop.classList.remove('show');
        }

        function showModal(data) {
            currentUpdateData = data;
            if (restartTimer) {
                clearInterval(restartTimer);
                restartTimer = null;
            }

            var list = root.getElementById('wadesk-changelog-items');
            list.innerHTML = '';
            if (data.highlights && data.highlights.length > 0) {
                data.highlights.forEach(function (text) {
                    var clean = text.replace(/^[•\-\*]\s*/, '').trim();
                    if (!clean) return;
                    var li = document.createElement('li');
                    li.className = 'wadesk-changelog-item';
                    li.textContent = clean;
                    list.appendChild(li);
                });
            } else {
                var li = document.createElement('li');
                li.className = 'wadesk-changelog-item';
                li.textContent = 'Peningkatan performa, keamanan, dan stabilitas obrolan.';
                list.appendChild(li);
            }

            var titleText = root.getElementById('wadesk-title-text');
            var subText = root.getElementById('wadesk-subtext');
            var btnText = root.getElementById('wadesk-btn-text');
            var btnLater = root.getElementById('wadesk-btn-later');

            if (data.isReady) {
                titleText.textContent = 'Pembaruan Siap Dipasang';
                subText.textContent = 'Versi ' + data.latestVersion + ' telah diunduh di latar belakang. Anda dapat memasangnya saat aplikasi ditutup agar tidak mengganggu aktivitas Anda, atau mulai ulang sekarang.';
                btnText.textContent = 'Mulai Ulang Sekarang';
                btnLater.textContent = 'Pasang Saat Keluar';
            } else {
                titleText.textContent = 'Pembaruan Tersedia';
                subText.textContent = 'Versi ' + data.latestVersion + ' kini tersedia (versi saat ini: ' + data.currentVersion + ').';
                btnText.textContent = 'Unduh di Latar Belakang';
                btnLater.textContent = 'Nanti Saja';
            }

            // Reset progress box
            root.getElementById('wadesk-progress-box').classList.remove('show');
            root.getElementById('wadesk-progress-bar').style.width = '0%';
            root.getElementById('wadesk-progress-percent').textContent = '0%';
            root.getElementById('wadesk-actions-box').style.display = 'flex';
            root.getElementById('wadesk-btn-download').disabled = false;
            btnLater.style.display = 'inline-flex';

            backdrop.classList.add('show');
        }

        root.getElementById('wadesk-btn-close').addEventListener('click', function () {
            if (currentUpdateData && currentUpdateData.isReady) {
                if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                    window.chrome.webview.postMessage({
                        type: 'schedule_install_on_exit',
                        version: currentUpdateData.latestVersion
                    });
                }
            }
            closeModal();
        });

        root.getElementById('wadesk-btn-later').addEventListener('click', function () {
            // Jika dalam mode hitung mundur Mulai Ulang, tombol ini berfungsi sebagai "Batal"
            if (restartTimer) {
                clearInterval(restartTimer);
                restartTimer = null;
                root.getElementById('wadesk-btn-download').disabled = false;
                root.getElementById('wadesk-btn-text').textContent = 'Mulai Ulang Sekarang';
                root.getElementById('wadesk-btn-later').textContent = 'Pasang Saat Keluar';
                window.wadeskShowToast('Mulai ulang dibatalkan.');
                return;
            }

            if (currentUpdateData && currentUpdateData.isReady) {
                if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                    window.chrome.webview.postMessage({
                        type: 'schedule_install_on_exit',
                        version: currentUpdateData.latestVersion
                    });
                }
                closeModal();
                window.wadeskShowToast('Pembaruan akan dipasang otomatis saat WaDesk ditutup. Anda dapat mengetik dengan aman.');
            } else {
                closeModal();
            }
        });

        // Snooze 24 Jam
        root.getElementById('wadesk-btn-snooze').addEventListener('click', function () {
            closeModal();
            if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                window.chrome.webview.postMessage({ type: 'snooze_update' });
            }
            window.wadeskShowToast('Pemberitahuan pembaruan ditunda 24 jam.');
        });

        // Skip Versi Ini
        root.getElementById('wadesk-btn-skip').addEventListener('click', function () {
            if (!currentUpdateData) return;
            var ver = currentUpdateData.latestVersion;
            closeModal();
            if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                window.chrome.webview.postMessage({ type: 'skip_update', version: ver });
            }
            window.wadeskShowToast('Pembaruan v' + ver + ' dilewati.');
        });

        backdrop.addEventListener('click', function (e) {
            if (e.target === backdrop) {
                var progressBox = root.getElementById('wadesk-progress-box');
                if (!progressBox.classList.contains('show')) {
                    if (currentUpdateData && currentUpdateData.isReady) {
                        if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                            window.chrome.webview.postMessage({
                                type: 'schedule_install_on_exit',
                                version: currentUpdateData.latestVersion
                            });
                        }
                    }
                    closeModal();
                }
            }
        });

        // Tombol Escape menutup modal
        document.addEventListener('keydown', function (e) {
            if (e.key === 'Escape' && backdrop.classList.contains('show')) {
                var progressBox = root.getElementById('wadesk-progress-box');
                if (!progressBox.classList.contains('show')) {
                    if (currentUpdateData && currentUpdateData.isReady) {
                        if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                            window.chrome.webview.postMessage({
                                type: 'schedule_install_on_exit',
                                version: currentUpdateData.latestVersion
                            });
                        }
                    }
                    closeModal();
                }
            }
        });

        root.getElementById('wadesk-btn-download').addEventListener('click', function () {
            if (!currentUpdateData) return;

            // Jika installer sudah di-preload di background, mulai hitung mundur 2 detik agar aman dan bisa dibatalkan
            if (currentUpdateData.isReady) {
                var secondsLeft = 2;
                var btnDownload = root.getElementById('wadesk-btn-download');
                var btnLater = root.getElementById('wadesk-btn-later');
                var btnText = root.getElementById('wadesk-btn-text');

                btnLater.textContent = 'Batal';
                btnText.textContent = 'Menutup WaDesk (' + secondsLeft + 's)...';

                restartTimer = setInterval(function () {
                    secondsLeft -= 1;
                    if (secondsLeft > 0) {
                        btnText.textContent = 'Menutup WaDesk (' + secondsLeft + 's)...';
                    } else {
                        clearInterval(restartTimer);
                        restartTimer = null;
                        btnDownload.disabled = true;
                        btnText.textContent = 'Memulai pemasangan...';

                        if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                            window.chrome.webview.postMessage({
                                type: 'install_update',
                                version: currentUpdateData.latestVersion
                            });
                        }

                        // Safety fallback timeout: jika setelah 5 detik aplikasi belum keluar (misal ditolak Windows UAC)
                        setTimeout(function () {
                            btnDownload.disabled = false;
                            btnText.textContent = 'Mulai Ulang Sekarang';
                            btnLater.textContent = 'Pasang Saat Keluar';
                        }, 5000);
                    }
                }, 1000);

                return;
            }

            // Jika belum di-preload, jalankan download di background dengan progress
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

        // Global functions dipanggil dari Rust host
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
