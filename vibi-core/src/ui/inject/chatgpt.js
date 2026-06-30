(function() {
    if (window.__vibi_obs) return;
    window.__vibi_obs = true;
    window.__vibi_last = '';
    setInterval(function() {
        var msgs = document.querySelectorAll('[data-message-author-role="assistant"]');
        if (msgs.length > 0) {
            var last = msgs[msgs.length - 1].textContent.trim();
            if (last !== window.__vibi_last && last.length > 0) {
                window.__vibi_last = last;
            }
        }
    }, 1500);
})();