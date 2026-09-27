(function () {
  "use strict";

  function setupDrawer() {
    var toggle = document.getElementById("menu-toggle");
    var drawer = document.getElementById("drawer");
    var scrim = document.getElementById("scrim");
    if (!toggle || !drawer) {
      return;
    }
    function setOpen(open) {
      if (open) {
        drawer.classList.add("open");
        document.body.classList.add("drawer-open");
        var active = drawer.querySelector(".projects a.active");
        if (active) {
          active.scrollIntoView({ block: "center" });
        }
      } else {
        drawer.classList.remove("open");
        document.body.classList.remove("drawer-open");
      }
      toggle.setAttribute("aria-expanded", open ? "true" : "false");
      if (scrim) {
        if (open) {
          scrim.classList.add("show");
        } else {
          scrim.classList.remove("show");
        }
      }
    }
    toggle.addEventListener("click", function () {
      setOpen(!drawer.classList.contains("open"));
    });
    if (scrim) {
      scrim.addEventListener("click", function () {
        setOpen(false);
      });
    }
    document.addEventListener("keydown", function (event) {
      if (event.key === "Escape") {
        setOpen(false);
      }
    });
  }

  function setupProjectFilter() {
    var input = document.getElementById("proj-filter");
    var list = document.getElementById("projects");
    if (!input || !list) {
      return;
    }
    input.addEventListener("input", function () {
      var needle = input.value.toLowerCase();
      var items = list.getElementsByTagName("li");
      for (var i = 0; i < items.length; i++) {
        var text = items[i].textContent.toLowerCase();
        items[i].style.display = text.indexOf(needle) === -1 ? "none" : "";
      }
    });
  }

  function clearTargets() {
    var nodes = document.querySelectorAll(".line.target");
    for (var i = 0; i < nodes.length; i++) {
      nodes[i].classList.remove("target");
    }
  }

  function openTargetNode(hash) {
    var node = document.getElementById(hash.slice(1));
    if (node && node.tagName === "DETAILS") {
      node.open = true;
      node.scrollIntoView({ block: "center" });
    }
  }

  function highlightHash() {
    clearTargets();
    var hash = window.location.hash;
    if (!hash || hash.charAt(0) !== "#") {
      return;
    }
    openTargetNode(hash);
    var match = hash.slice(1).match(/^L(\d+)(?:-L(\d+))?$/);
    if (!match) {
      return;
    }
    var start = parseInt(match[1], 10);
    var end = match[2] ? parseInt(match[2], 10) : start;
    var first = document.getElementById("L" + start);
    if (first) {
      first.scrollIntoView({ block: "center" });
    }
    for (var line = start; line <= end && line - start < 400; line++) {
      var node = document.getElementById("L" + line);
      if (node) {
        node.classList.add("target");
      }
    }
  }

  window.addEventListener("hashchange", highlightHash);
  window.addEventListener("load", function () {
    setupDrawer();
    setupProjectFilter();
    highlightHash();
  });
})();
