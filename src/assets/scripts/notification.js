// Native Notification Polyfill untuk WaDesk Windows Tray Balloon
(function() {
    function postHost(msg) {
        if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
            window.chrome.webview.postMessage(msg);
        } else if (typeof window.sendNativeNotification === 'function') {
            window.sendNativeNotification(msg.title, msg.body, msg.icon);
        }
    }

    var SERVICE = { color: '#25d366' };

    function paintIcon(ctx, size, bmp) {
        var cx = size / 2, cy = size / 2, r = size / 2;
        ctx.clearRect(0, 0, size, size);

        ctx.save();
        ctx.beginPath();
        ctx.arc(cx, cy, r - 3, 0, Math.PI * 2);
        ctx.closePath();
        ctx.clip();
        if (bmp) {
            ctx.drawImage(bmp, 0, 0, size, size);
        } else {
            ctx.fillStyle = SERVICE.color;
            ctx.fillRect(0, 0, size, size);
        }
        ctx.restore();

        ctx.beginPath();
        ctx.arc(cx, cy, r - 3, 0, Math.PI * 2);
        ctx.lineWidth = 4;
        ctx.strokeStyle = SERVICE.color;
        ctx.stroke();
    }

    function iconDataURL(url) {
        var size = 64;
        var canvas = document.createElement('canvas');
        canvas.width = size;
        canvas.height = size;
        var ctx = canvas.getContext('2d');
        var fallback = function() {
            paintIcon(ctx, size, null);
            return canvas.toDataURL('image/png');
        };
        if (!url) { return Promise.resolve(fallback()); }
        return fetch(url)
            .then(function(r) { return r.blob(); })
            .then(function(blob) { return createImageBitmap(blob); })
            .then(function(bmp) {
                paintIcon(ctx, size, bmp);
                bmp.close();
                return canvas.toDataURL('image/png');
            })
            .catch(fallback);
    }

    var lastNotification = null;

    function NativeNotification(title, options) {
        options = options || {};
        this.title = title;
        this.body = options.body || '';
        this.icon = options.icon || '';
        this.data = options.data || null;
        this.onclick = null;

        var self = this;
        lastNotification = self;

        iconDataURL(self.icon).then(function(dataUrl) {
            postHost({
                type: 'notification',
                title: self.title,
                body: self.body,
                icon: dataUrl
            });
        });
    }

    NativeNotification.permission = 'granted';
    NativeNotification.requestPermission = function(cb) {
        if (cb) { cb('granted'); }
        return Promise.resolve('granted');
    };

    window.Notification = NativeNotification;

    window.wadeskNotificationClick = function() {
        if (lastNotification && typeof lastNotification.onclick === 'function') {
            try { lastNotification.onclick({ currentTarget: lastNotification }); } catch (e) {}
        }
    };
})();
