(function() {
    if (window.__vibi_obs) return;
    window.__vibi_obs = true;
    window.__vibi_last = '';
    
    function htmlToMarkdown(html) {
        var txt = document.createElement('div');
        txt.innerHTML = html;
        var md = txt.innerText || txt.textContent || '';
        return md.trim();
    }
    
    function findLatestResponse() {
        var msgs = document.querySelectorAll('response-element');
        if (msgs.length === 0) msgs = document.querySelectorAll('.md-content');
        if (msgs.length === 0) msgs = document.querySelectorAll('.response-content');
        if (msgs.length === 0) {
            var turns = document.querySelectorAll('model-response, .model-response');
            if (turns.length > 0) {
                var html = turns[turns.length - 1].innerHTML;
                return htmlToMarkdown(html).replace(/^Gemini said\s*/i, '');
            }
        }
        if (msgs.length > 0) {
            var html = msgs[msgs.length - 1].innerHTML;
            return htmlToMarkdown(html).replace(/^Gemini said\s*/i, '');
        }
        return '';
    }
    
    function hasMicIcon() {
        return document.querySelector('button[aria-label*="microphone"]') || 
               document.querySelector('mat-icon[data-mat-icon-name="mic"]');
    }
    
    function hasSendIcon() {
        return document.querySelector('button[aria-label*="Send"]') || 
               document.querySelector('button[aria-label*="send"]');
    }
    
    function hasStopIcon() {
        return document.querySelector('button[aria-label*="Stop"]') || 
               document.querySelector('button[aria-label*="stop"]');
    }
    
    function toHex(str) {
        var bytes = new TextEncoder().encode(str);
        return Array.from(bytes).map(function(b) { return b.toString(16).padStart(2, '0'); }).join('');
    }
    
    var chunkQueue = [];
    var sendingChunks = false;
    
    function sendNextChunk() {
        if (chunkQueue.length === 0) {
            sendingChunks = false;
            history.replaceState(null, '', window.location.pathname + window.location.search + '#vibi-done');
            return;
        }
        sendingChunks = true;
        var chunk = chunkQueue.shift();
        history.replaceState(null, '', window.location.pathname + window.location.search + '#vibi-' + chunk.idx + '-' + chunk.data);
        setTimeout(sendNextChunk, 100);
    }
    
    function tryCapture() {
        if (hasStopIcon()) return;
        var text = findLatestResponse();
        if (text.length > 0 && text !== window.__vibi_last) {
            window.__vibi_last = text;
            var hex = toHex(text);
            var chunkSize = 1800;
            chunkQueue = [];
            for (var i = 0; i < hex.length; i += chunkSize) {
                chunkQueue.push({ idx: Math.floor(i / chunkSize), data: hex.substring(i, i + chunkSize) });
            }
            if (!sendingChunks) sendNextChunk();
        }
    }
    
    window.__vibi_send = function(text) {
        var input = document.querySelector('rich-textarea .ql-editor') || document.querySelector('[contenteditable="true"]');
        if (!input) return false;
        
        input.textContent = text;
        input.dispatchEvent(new Event('input', { bubbles: true }));
        
        var attempts = 0;
        var trySend = setInterval(function() {
            attempts++;
            if (hasSendIcon()) {
                var btn = document.querySelector('button[aria-label*="Send"]') || document.querySelector('button[aria-label*="send"]');
                if (btn) { btn.click(); clearInterval(trySend); }
            }
            if (attempts > 20) clearInterval(trySend);
        }, 200);
        return true;
    };
    
    // Always poll for responses, regardless of mic/send icon state
    setInterval(tryCapture, 1000);
    
    // Signal that Gemini JS is loaded
    history.replaceState(null, '', window.location.pathname + window.location.search + '#vibi-gemini-ready');
})();