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
        var msgs = document.querySelectorAll('[data-message-author-role="assistant"]');
        if (msgs.length === 0) msgs = document.querySelectorAll('.markdown');
        if (msgs.length === 0) msgs = document.querySelectorAll('article');
        if (msgs.length > 0) return htmlToMarkdown(msgs[msgs.length - 1].innerHTML);
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
        if (isStillGenerating()) return;
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
    
    setInterval(tryCapture, 1000);
    new MutationObserver(tryCapture).observe(document.body, { childList: true, subtree: true, characterData: true });
})();