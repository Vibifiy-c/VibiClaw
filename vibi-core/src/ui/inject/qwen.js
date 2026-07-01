(function() {
    if (window.__vibi_obs) return;
    window.__vibi_obs = true;
    window.__vibi_last = '';
    
    function findLatestResponse() {
        var msgs = document.querySelectorAll('.tongyi-markdown');
        if (msgs.length === 0) msgs = document.querySelectorAll('.chat-content');
        if (msgs.length === 0) msgs = document.querySelectorAll('.markdown');
        if (msgs.length > 0) return msgs[msgs.length - 1].textContent.trim();
        return '';
    }
    
    function toHex(str) {
        var bytes = new TextEncoder().encode(str);
        return Array.from(bytes).map(function(b) { return b.toString(16).padStart(2, '0'); }).join('');
    }
    
    setInterval(function() {
        var text = findLatestResponse();
        if (text.length > 0 && text !== window.__vibi_last) {
            window.__vibi_last = text;
            window.location.hash = 'vibi-' + toHex(text);
        }
    }, 2000);
})();