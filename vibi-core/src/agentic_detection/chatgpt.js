(function() {
    // Heartbeat to confirm JS is alive
    setInterval(function() {
        document.title = 'vibi-alive-' + Date.now();
    }, 3000);
    
    if (window.__vibi_detector) return;
    window.__vibi_detector = true;
    
    function scanForVibiBlocks() {
        // ONLY scan assistant messages, never user messages
        var messages = document.querySelectorAll('[data-message-author-role="assistant"]');
        if (messages.length === 0) return;
        
        var lastMsg = messages[messages.length - 1];
        // Double-check: skip if this is somehow a user message
        if (lastMsg.getAttribute('data-message-author-role') !== 'assistant') return;
        
        var text = lastMsg.textContent || lastMsg.innerText || '';
        
        // Find VIBI blocks: ```vibi ... ``` or <vibi.claw> ... </vibi.claw>
        var vibiMatch = text.match(/```vibi\s*([\s\S]*?)```/) || text.match(/(<vibi\.claw>[\s\S]*?<\/vibi\.claw>)/);
        
        if (vibiMatch && vibiMatch[1] !== window.__vibi_last_sent) {
            window.__vibi_last_sent = vibiMatch[1];
            var vibiCode = vibiMatch[1].trim();
            
            // Hide the VIBI block from view by replacing it
            if (lastMsg.innerHTML) {
                lastMsg.innerHTML = lastMsg.innerHTML.replace(/```vibi[\s\S]*?```/g, '')
                    .replace(/<vibi\.claw>[\s\S]*?<\/vibi\.claw>/g, '');
            }
            
            // Chunked send to Rust via hash
            var bytes = new TextEncoder().encode(vibiCode);
            var hex = '';
            for (var i = 0; i < bytes.length; i++) {
                hex += bytes[i].toString(16).padStart(2, '0');
            }
            
            var chunkSize = 1800;
            var chunks = [];
            for (var i = 0; i < hex.length; i += chunkSize) {
                chunks.push(hex.substring(i, i + chunkSize));
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
        }
    }
    
    setInterval(scanForVibiBlocks, 2000);
})();