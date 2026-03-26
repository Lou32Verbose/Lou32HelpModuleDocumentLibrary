(function () {
  var saved = localStorage.getItem("theme");
  if (saved) {
    document.documentElement.dataset.theme = saved;
  }

  document.addEventListener("DOMContentLoaded", function () {
    var picker = document.getElementById("theme-picker");
    if (!picker) return;

    if (saved) picker.value = saved;

    picker.addEventListener("change", function () {
      var value = picker.value;
      if (value) {
        document.documentElement.dataset.theme = value;
        localStorage.setItem("theme", value);
      } else {
        delete document.documentElement.dataset.theme;
        localStorage.removeItem("theme");
      }
    });
  });
})();
