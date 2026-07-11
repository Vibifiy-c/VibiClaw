(function() {
    if (window.__vibi_obs) return;
    window.__vibi_obs = true;
    window.__vibi_last = '';
    
    function findLatestResponse() {
        var msgs = document.querySelectorAll('.markdown');
        if (msgs.length === 0) msgs = document.querySelectorAll('.prose');
        if (msgs.length === 0) msgs = document.querySelectorAll('[class*="message"]');
        if (msgs.length === 0) msgs = document.querySelectorAll('[class*="response"]');
        if (msgs.length === 0) msgs = document.querySelectorAll('article');
        if (msgs.length > 0) return msgs[msgs.length - 1].textContent.trim();
        return '';
    }
    
    function toHex(str) {
        var bytes = new TextEncoder().encode(str);
        return Array.from(bytes).map(function(b) { return b.toString(16).padStart(2, '0'); }).join('');
    }
    
    function tryCapture() {
        var text = findLatestResponse();
        if (text.length > 0 && text !== window.__vibi_last) {
            window.__vibi_last = text;
            var hex = toHex(text);
            if (hex.length > 1900) hex = hex.substring(0, 1900);
            history.replaceState(null, '', window.location.pathname + window.location.search + '#vibi-' + hex);
        }
    }
    
    setInterval(tryCapture, 1000);
    new MutationObserver(tryCapture).observe(document.body, { childList: true, subtree: true, characterData: true });
    setTimeout(function() {
        window.location.hash = 'vibi-test-ok';
    }, 3000);
})();