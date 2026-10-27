(function () {
  var root = document.documentElement, key = "gtv-theme";
  function store(v) { try { v ? localStorage.setItem(key, v) : localStorage.getItem(key); } catch (e) {} }
  var saved = null; try { saved = localStorage.getItem(key); } catch (e) {}
  if (saved === "light" || saved === "dark") root.setAttribute("data-theme", saved);
  document.getElementById("theme").addEventListener("click", function () {
    var dark = root.getAttribute("data-theme") === "dark" ||
      (!root.getAttribute("data-theme") && window.matchMedia("(prefers-color-scheme: dark)").matches);
    var next = dark ? "light" : "dark";
    root.setAttribute("data-theme", next); store(next);
  });
  document.querySelectorAll(".copy").forEach(function (b) {
    b.addEventListener("click", function () {
      var text = b.previousElementSibling.innerText.replace(/^# .*\n/gm, "");
      (navigator.clipboard ? navigator.clipboard.writeText(text) : Promise.reject()).then(function () {
        b.textContent = "Đã chép"; setTimeout(function () { b.textContent = "Sao chép"; }, 1500);
      }, function () { b.textContent = "Chọn và chép tay"; });
    });
  });
})();
