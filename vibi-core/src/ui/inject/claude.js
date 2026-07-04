(function() {
    if (window.__vibi_obs) return;
    window.__vibi_obs = true;
    window.__vibi_last = '';
    
    function findLatestResponse() {
        var msgs = document.querySelectorAll('[data-message-author-role="assistant"]');
        if (msgs.length === 0) msgs = document.querySelectorAll('.markdown');
        if (msgs.length === 0) msgs = document.querySelectorAll('article');
        if (msgs.length > 0) return msgs[msgs.length - 1].textContent.trim();
        return '';
    }
    
    function isStillGenerating() {
        var stopBtn = document.querySelector('[data-testid="stop-button"]');
        if (stopBtn) return true;
        var resultStreaming = document.querySelector('.result-streaming');
        if (resultStreaming) return true;
        var streaming = document.querySelector('[class*="streaming"]');
        if (streaming) return true;
        return false;
    }
    
    function toHex(str) {
        var bytes = new TextEncoder().encode(str);
        return Array.from(bytes).map(function(b) { return b.toString(16).padStart(2, '0'); }).join('');
    }
    
    var checkInterval = setInterval(function() {
        if (isStillGenerating()) return;
        
        var text = findLatestResponse();
        if (text.length > 0 && text !== window.__vibi_last) {
            window.__vibi_last = text;
            window.location.hash = 'vibi-' + toHex(text);
        }
    }, 1000);
    
    var observer = new MutationObserver(function() {
        if (!isStillGenerating()) {
            var text = findLatestResponse();
            if (text.length > 0 && text !== window.__vibi_last) {
                window.__vibi_last = text;
                window.location.hash = 'vibi-' + toHex(text);
            }
        }
    });
    observer.observe(document.body, { childList: true, subtree: true, characterData: true });
})();