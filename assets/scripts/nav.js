// Highlights the current page in the nav bar
(function() {
    const path = window.location.href;
    const origin = window.location.origin + '/';
    document.querySelectorAll('.nav-link').forEach(ele => {
        const matches = (ele.href === origin && path === origin)
            || (ele.href !== origin && path.startsWith(ele.href.endsWith('/') ? ele.href : ele.href + '/'))
            || ele.href === path;

        if (matches) {
            ele.classList.add('active');
        }
    });
})();
