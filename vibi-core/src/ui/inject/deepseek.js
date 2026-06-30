(function() {
    if (window.__vibi_obs) return;
    window.__vibi_obs = true;
    window.__vibi_last = '';
    setInterval(function() {
        var msgs = document.querySelectorAll('.ds-markdown');
        if (msgs.length === 0) msgs = document.querySelectorAll('.message-content');
        if (msgs.length > 0) {
            var last = msgs[msgs.length - 1].textContent.trim();
            if (last !== window.__vibi_last && last.length > 0) {
                window.__vibi_last = last;
            }
        }
    }, 1500);
})();