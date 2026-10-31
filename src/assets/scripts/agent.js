// Injected Agent for WaDesk MCP tools and WhatsApp Web automation
(function () {
    if (window.wadeskAgent) { return; }

    function q(sel, parent) { try { return (parent || document).querySelector(sel); } catch (e) { return null; } }
    function qa(sel, parent) { try { return Array.prototype.slice.call((parent || document).querySelectorAll(sel)); } catch (e) { return []; } }

    function getMediaThumbnail(imgEl, maxDim) {
        if (!imgEl) return null;
        try {
            var src = imgEl.src || '';
            if (src.indexOf('data:image') === 0 && src.length < 50000) {
                return src;
            }
            var canvas = document.createElement('canvas');
            var w = imgEl.naturalWidth || imgEl.width || 300;
            var h = imgEl.naturalHeight || imgEl.height || 300;
            if (w <= 0 || h <= 0) return null;

            maxDim = maxDim || 400;
            if (w > maxDim || h > maxDim) {
                if (w > h) {
                    h = Math.round((h * maxDim) / w);
                    w = maxDim;
                } else {
                    w = Math.round((w * maxDim) / h);
                    h = maxDim;
                }
            }
            canvas.width = w;
            canvas.height = h;
            var ctx = canvas.getContext('2d');
            ctx.drawImage(imgEl, 0, 0, w, h);
            return canvas.toDataURL('image/jpeg', 0.7);
        } catch (e) {
            return null;
        }
    }

    function extractMediaInfo(el, includeMedia, maxDim) {
        // Cek video
        var videoPlay = q('[data-testid="video-play"], [data-icon="video-play"], [data-icon="status-video"], [data-testid="media-video"], [data-testid="video-thumb"], video', el);
        var videoDuration = q('[data-testid="video-duration"]', el);
        if (!videoDuration) {
            var spans = qa('span', el);
            for (var i = 0; i < spans.length; i++) {
                var txt = (spans[i].textContent || '').trim();
                if (/^\d{1,2}:\d{2}$/.test(txt)) {
                    videoDuration = spans[i];
                    break;
                }
            }
        }

        // Cek audio / voice note
        var audioPlay = q('[data-testid="audio-player"], [data-testid="ptt-play-button"], [data-icon="audio-play"], [data-icon="voice-note"], audio', el);
        if (audioPlay && !videoPlay) {
            var audioDuration = q('[data-testid="audio-duration"]', el);
            return {
                type: 'audio',
                duration: audioDuration ? audioDuration.textContent.trim() : (videoDuration ? videoDuration.textContent.trim() : '')
            };
        }

        // Cek document
        var docThumb = q('[data-testid="document-thumb"], [data-icon="document"], [data-testid="msg-document"]', el);
        if (docThumb) {
            var titleEl = q('span[title]', el) || q('[data-testid="document-title"]', el);
            return {
                type: 'document',
                filename: titleEl ? (titleEl.getAttribute('title') || titleEl.textContent || '').trim() : ''
            };
        }

        // Cek sticker
        var stickerEl = q('[data-testid="sticker"], img.sticker', el);
        if (stickerEl) {
            var stickerImg = q('img', el);
            return {
                type: 'sticker',
                data_url: includeMedia ? getMediaThumbnail(stickerImg, maxDim) : null
            };
        }

        // Cek visual image / video
        var imgList = qa('img', el);
        var mediaImg = null;
        for (var j = 0; j < imgList.length; j++) {
            var img = imgList[j];
            if (img.classList.contains('wa-emoji') || img.classList.contains('emoji')) continue;
            if (img.getAttribute('data-plain-text')) continue;
            var nw = img.naturalWidth || img.width || 0;
            var nh = img.naturalHeight || img.height || 0;
            if (nw > 0 && nw <= 32 && nh > 0 && nh <= 32) continue;
            mediaImg = img;
            break;
        }

        if (videoPlay) {
            return {
                type: 'video',
                duration: videoDuration ? videoDuration.textContent.trim() : '',
                data_url: includeMedia ? getMediaThumbnail(mediaImg, maxDim) : null
            };
        }

        if (mediaImg || q('[data-testid="media-image"], [data-testid="image-thumb"]', el)) {
            return {
                type: 'image',
                mime_type: 'image/jpeg',
                data_url: includeMedia ? getMediaThumbnail(mediaImg, maxDim) : null
            };
        }

        return null;
    }

    var AGENT = {
        app_status: function() {
            var composer = q('footer div[contenteditable="true"]') || q('[data-testid="conversation-compose-box-input"]');
            return {
                ready: !!composer,
                service: 'WA',
                title: document.title || 'WhatsApp'
            };
        },

        list_chats: function() {
            var rows = qa('[data-testid="cell-frame-container"]') || qa('#pane-side div[role="listitem"]');
            var result = [];

            rows.forEach(function(r) {
                var nameEl = q('[data-testid="cell-frame-title"] span', r) || q('span[title]', r);
                var previewEl = q('[data-testid="last-msg-status"]', r) || q('span[data-testid="cell-frame-secondary-title"]', r);
                var badgeEl = q('span[data-testid="icon-unread-count"]', r) || q('[aria-label*="unread"]', r);

                var name = nameEl ? (nameEl.getAttribute('title') || nameEl.textContent || '').trim() : '';
                if (!name) return;

                var preview = previewEl ? previewEl.textContent.trim() : '';
                var unread = 0;
                if (badgeEl) {
                    var parsed = parseInt(badgeEl.textContent.trim(), 10);
                    if (!isNaN(parsed)) { unread = parsed; }
                }

                result.push({
                    name: name,
                    preview: preview,
                    unread: unread,
                    active: r.getAttribute('aria-selected') === 'true'
                });
            });

            return result;
        },

        open_chat: function(args) {
            var target = (args.name || '').toLowerCase().trim();
            if (!target) return { opened: false, error: 'name required' };

            var rows = qa('[data-testid="cell-frame-container"]');
            for (var i = 0; i < rows.length; i++) {
                var nameEl = q('[data-testid="cell-frame-title"] span', rows[i]);
                var name = nameEl ? (nameEl.getAttribute('title') || nameEl.textContent || '').toLowerCase().trim() : '';
                if (name === target || name.indexOf(target) !== -1) {
                    rows[i].click();
                    return { opened: true, name: name };
                }
            }

            // Fallback: gunakan kolom pencarian WhatsApp Web
            var searchBox = q('[data-testid="chat-list-search"]') || q('div[contenteditable="true"][data-tab="3"]');
            if (searchBox) {
                searchBox.focus();
                document.execCommand('insertText', false, args.name);
                return { opened: false, searched: true };
            }

            return { opened: false, error: 'chat not found' };
        },

        read_messages: function(args) {
            var limit = (args && args.limit) || 50;
            var includeMedia = !args || args.include_media !== false;
            var maxMediaSize = (args && args.max_media_size) || 400;

            var msgEls = qa('div.message-in, div.message-out');
            var slice = msgEls.slice(-limit);

            var messages = slice.map(function(el) {
                var textEl = q('.selectable-text span', el) || q('.copyable-text', el);
                var timeEl = q('[data-testid="msg-meta"] span', el);
                var outgoing = el.classList.contains('message-out');

                var rawText = textEl ? textEl.textContent : '';
                var mediaInfo = extractMediaInfo(el, includeMedia, maxMediaSize);

                var displayText = rawText;
                if (!displayText && mediaInfo) {
                    if (mediaInfo.type === 'video') {
                        displayText = mediaInfo.duration ? '[Video: ' + mediaInfo.duration + ']' : '[Video]';
                    } else if (mediaInfo.type === 'image') {
                        displayText = '[Image]';
                    } else if (mediaInfo.type === 'audio') {
                        displayText = mediaInfo.duration ? '[Audio: ' + mediaInfo.duration + ']' : '[Audio]';
                    } else if (mediaInfo.type === 'sticker') {
                        displayText = '[Sticker]';
                    } else if (mediaInfo.type === 'document') {
                        displayText = mediaInfo.filename ? '[Document: ' + mediaInfo.filename + ']' : '[Document]';
                    }
                }

                if (mediaInfo && rawText) {
                    mediaInfo.caption = rawText;
                }

                return {
                    text: displayText,
                    time: timeEl ? timeEl.textContent : '',
                    outgoing: outgoing,
                    media: mediaInfo
                };
            });

            var headerTitle = q('header [data-testid="conversation-info-header-chat-title"]') || q('header span[title]');

            return {
                chat: {
                    name: headerTitle ? headerTitle.textContent : ''
                },
                messages: messages
            };
        },

        conversation_summary: function(args) {
            var res = AGENT.read_messages(args);
            var msgs = res.messages || [];
            var incoming = 0;
            var outgoing = 0;
            msgs.forEach(function(m) {
                if (m.outgoing) { outgoing++; } else { incoming++; }
            });
            var last = msgs.length > 0 ? msgs[msgs.length - 1] : null;

            return {
                chat: res.chat,
                total: msgs.length,
                incoming_count: incoming,
                outgoing_count: outgoing,
                last_message: last,
                should_reply: last ? !last.outgoing : false,
                messages: msgs
            };
        },

        send_message: function(args) {
            var text = args.text || '';
            if (!text) return { ok: false, error: 'text required' };

            var composer = q('footer div[contenteditable="true"]') || q('[data-testid="conversation-compose-box-input"]');
            if (!composer) return { ok: false, error: 'composer not found, please open a chat first' };

            composer.focus();
            document.execCommand('insertText', false, text);

            var sendBtn = q('footer button[data-testid="send"]') ||
                          q('footer span[data-icon="send"]') ||
                          q('footer span[data-testid="send"]') ||
                          q('footer button[aria-label="Send"]') ||
                          q('footer button[aria-label="Kirim"]');
            if (sendBtn) {
                var clickTarget = sendBtn.closest ? (sendBtn.closest('button') || sendBtn) : sendBtn;
                clickTarget.click();
                return { ok: true, sent: text };
            }

            // Fallback tekan Enter
            var enterEvent = new KeyboardEvent('keydown', {
                key: 'Enter',
                code: 'Enter',
                keyCode: 13,
                which: 13,
                bubbles: true
            });
            composer.dispatchEvent(enterEvent);
            return { ok: true, sent: text };
        },

        export_chat: function(args) {
            return AGENT.read_messages(args);
        }
    };

    window.wadeskAgent = {
        call: function(id, fn, argsJson) {
            var args = {};
            try { args = JSON.parse(argsJson); } catch(e) {}
            var handler = AGENT[fn];
            var result = null;
            if (typeof handler === 'function') {
                try {
                    result = handler(args);
                } catch(err) {
                    result = { error: err.message };
                }
            } else {
                result = { error: 'unknown agent method: ' + fn };
            }

            var resPayload = {
                type: 'agent_reply',
                id: id,
                result: result
            };

            if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
                window.chrome.webview.postMessage(resPayload);
            }
        }
    };
})();
