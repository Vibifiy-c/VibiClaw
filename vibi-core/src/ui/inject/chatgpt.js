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
        if (document.querySelector('[data-testid="stop-button"]')) return true;
        if (document.querySelector('.result-streaming')) return true;
        if (document.querySelector('[class*="streaming"]')) return true;
        return false;
    }
    
    function toHex(str) {
        var bytes = new TextEncoder().encode(str);
        return Array.from(bytes).map(function(b) { return b.toString(16).padStart(2, '0'); }).join('');
    }
    
    var captureCount = 0;
    function tryCapture() {
        var generating = isStillGenerating();
        var text = findLatestResponse();
        window.__vibi_debug = JSON.stringify({
            count: captureCount++,
            generating: generating,
            textLen: text.length,
            lastLen: window.__vibi_last.length,
            same: text === window.__vibi_last
        });
        if (generating) return;
        if (text.length > 0 && text !== window.__vibi_last) {
            window.__vibi_last = text;
            history.replaceState(null, '', window.location.pathname + window.location.search + '#vibi-' + toHex(text));
        }
    }
    
    setInterval(tryCapture, 1000);
    
    new MutationObserver(tryCapture).observe(document.body, { childList: true, subtree: true, characterData: true });
        window.location.hash = 'vibi-test-ok';
})();