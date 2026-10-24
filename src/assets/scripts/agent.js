// Injected Agent for WaDesk MCP tools and WhatsApp Web automation
(function () {
    if (window.wadeskAgent) { return; }

    function q(sel, parent) { try { return (parent || document).querySelector(sel); } catch (e) { return null; } }
    function qa(sel, parent) { try { return Array.prototype.slice.call((parent || document).querySelectorAll(sel)); } catch (e) { return []; } }

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
            var msgEls = qa('div.message-in, div.message-out');
            var slice = msgEls.slice(-limit);

            var messages = slice.map(function(el) {
                var textEl = q('.selectable-text span', el) || q('.copyable-text', el);
                var timeEl = q('[data-testid="msg-meta"] span', el);
                var outgoing = el.classList.contains('message-out');

                return {
                    text: textEl ? textEl.textContent : '',
                    time: timeEl ? timeEl.textContent : '',
                    outgoing: outgoing
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

            var sendBtn = q('footer span[data-testid="send"]') || q('footer button[aria-label="Send"]');
            if (sendBtn) {
                sendBtn.click();
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
