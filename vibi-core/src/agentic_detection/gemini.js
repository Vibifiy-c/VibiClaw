(function() {
    setInterval(function() {
        document.title = 'vibi-alive-' + Date.now();
    }, 3000);
    
    if (window.__vibi_detector) return;
    window.__vibi_detector = true;
    
    function findMessages() {
        var msgs = document.querySelectorAll('[data-message-author-role="assistant"]');
        if (msgs.length > 0) return msgs;
        
        var articles = document.querySelectorAll('article');
        if (articles.length > 0) return articles;
        
        return [];
    }
    
    function scanForVibiBlocks() {
        var messages = findMessages();
        if (messages.length === 0) return;
        
        // Scan all messages from newest to oldest
        for (var i = messages.length - 1; i >= 0; i--) {
            var text = messages[i].textContent || messages[i].innerText || '';
            
            var vibiMatch = text.match(/```vibi\s*([\s\S]*?)```/) || text.match(/(<vibi\.claw>[\s\S]*?<\/vibi\.claw>)/);
            
            if (vibiMatch && vibiMatch[1] !== window.__vibi_last_sent) {
                window.__vibi_last_sent = vibiMatch[1];
                var vibiCode = vibiMatch[1].trim();
                
                document.title = 'vibi-dbg-MATCH:' + vibiCode.length;
                
                if (messages[i].innerHTML) {
                    messages[i].innerHTML = messages[i].innerHTML.replace(/```vibi[\s\S]*?```/g, '').replace(/<vibi\.claw>[\s\S]*?<\/vibi\.claw>/g, '');
                }
                
                var bytes = new TextEncoder().encode(vibiCode);
                var hex = '';
                for (var j = 0; j < bytes.length; j++) {
                    hex += bytes[j].toString(16).padStart(2, '0');
                }
                
                var chunkSize = 1800;
                var chunks = [];
                for (var k = 0; k < hex.length; k += chunkSize) {
                    chunks.push(hex.substring(k, k + chunkSize));
                }
                
                var idx = 0;
                function sendChunk() {
                    if (idx < chunks.length) {
                        history.replaceState(null, '', window.location.pathname + window.location.search + '#vibi-action-' + idx + '-' + chunks[idx]);
                        idx++;
                        setTimeout(sendChunk, 80);
                    } else {
                        history.replaceState(null, '', window.location.pathname + window.location.search + '#vibi-action-done');
                    }
                }
                sendChunk();
                return;
            }
        }
    }
    
    setInterval(scanForVibiBlocks, 2000);
})();