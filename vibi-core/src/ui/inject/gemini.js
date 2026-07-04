(function() {
    if (window.__vibi_obs) return;
    window.__vibi_obs = true;
    window.__vibi_last = '';
    window.__vibi_ready = false;
    
    function findLatestResponse() {
        var msgs = document.querySelectorAll('response-element');
        if (msgs.length === 0) msgs = document.querySelectorAll('.md-content');
        if (msgs.length === 0) msgs = document.querySelectorAll('.response-content');
        if (msgs.length === 0) {
            var turns = document.querySelectorAll('model-response, .model-response');
            if (turns.length > 0) {
                return turns[turns.length - 1].textContent.trim().replace(/^Gemini said\s*/i, '');
            }
        }
        if (msgs.length > 0) {
            return msgs[msgs.length - 1].textContent.trim().replace(/^Gemini said\s*/i, '');
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
    
    setInterval(function() {
        if (hasStopIcon()) {
            window.__vibi_ready = false;
            return;
        }
        if (hasMicIcon() && !hasStopIcon()) {
            window.__vibi_ready = true;
            var text = findLatestResponse();
            if (text.length > 0 && text !== window.__vibi_last) {
                window.__vibi_last = text;
                window.location.hash = 'vibi-' + toHex(text);
            }
        }
    }, 1000);
})();