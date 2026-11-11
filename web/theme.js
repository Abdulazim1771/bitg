// Mavzu va alifbo tanlovi birinchi chizishdan oldin (CSP: inline script yo'q).
(function () {
  var d = document.documentElement, t = null, a = null;
  try { t = localStorage.getItem('bitg-theme'); a = localStorage.getItem('bitg-abc'); } catch (e) {}
  if (t === 'light' || t === 'dark') d.setAttribute('data-theme', t);
  if (a === 'new') d.setAttribute('data-abc', 'new');
})();
