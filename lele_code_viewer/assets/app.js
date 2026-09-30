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
    if (window.__lcvTargetTimer) {
      clearTimeout(window.__lcvTargetTimer);
      window.__lcvTargetTimer = null;
    }
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
    window.__lcvTargetTimer = setTimeout(clearTargets, 1000);
  }

  var LIVE_KEY = "lcv-live:";

  function storageGet(key) {
    try {
      return window.sessionStorage.getItem(key);
    } catch (err) {
      return null;
    }
  }

  function storageSet(key, value) {
    try {
      if (value === null) {
        window.sessionStorage.removeItem(key);
      } else {
        window.sessionStorage.setItem(key, value);
      }
    } catch (err) {
      return;
    }
  }

  function liveReload() {
    var open = [];
    var nodes = document.querySelectorAll("details[data-key]");
    for (var i = 0; i < nodes.length; i++) {
      open.push([nodes[i].getAttribute("data-key"), nodes[i].open]);
    }
    storageSet(
      LIVE_KEY + window.location.pathname,
      JSON.stringify({ y: window.scrollY, open: open })
    );
    window.location.reload();
  }

  function restoreLiveState() {
    var key = LIVE_KEY + window.location.pathname;
    var raw = storageGet(key);
    if (!raw) {
      return;
    }
    storageSet(key, null);
    var saved;
    try {
      saved = JSON.parse(raw);
    } catch (err) {
      return;
    }
    var states = {};
    for (var i = 0; i < (saved.open || []).length; i++) {
      states[saved.open[i][0]] = saved.open[i][1];
    }
    var nodes = document.querySelectorAll("details[data-key]");
    for (var j = 0; j < nodes.length; j++) {
      var name = nodes[j].getAttribute("data-key");
      if (states[name] === true) {
        nodes[j].open = true;
      } else if (states[name] === false && !nodes[j].querySelector(".dot")) {
        nodes[j].open = false;
      }
    }
    window.scrollTo(0, saved.y || 0);
  }

  function relevant(feed, watch) {
    if (feed.reset) {
      return true;
    }
    var paths = feed.paths || [];
    for (var i = 0; i < paths.length; i++) {
      if (watch === "*" || paths[i] === watch) {
        return true;
      }
      if (watch === "*.rs" && /\.rs$/.test(paths[i])) {
        return true;
      }
    }
    return false;
  }

  function setupLive() {
    var url = document.body.getAttribute("data-events");
    var watch = document.body.getAttribute("data-watch");
    if (!url || !watch || typeof window.EventSource === "undefined") {
      return;
    }
    var source = null;
    function connect() {
      if (source) {
        return;
      }
      source = new window.EventSource(url);
      source.addEventListener("change", function (event) {
        var feed;
        try {
          feed = JSON.parse(event.data);
        } catch (err) {
          return;
        }
        if (relevant(feed, watch)) {
          disconnect();
          liveReload();
        }
      });
    }
    function disconnect() {
      if (source) {
        source.close();
        source = null;
      }
    }
    document.addEventListener("visibilitychange", function () {
      if (document.hidden) {
        disconnect();
      } else {
        connect();
      }
    });
    if (!document.hidden) {
      connect();
    }
  }

  function setupCodeBlocks() {
    var graph = document.querySelector(".cb-graph");
    if (!graph) {
      return;
    }
    var pills = graph.querySelectorAll(".cb-pill[data-node]");
    function clearHover() {
      var svg = graph.querySelector("svg.cb-edges");
      if (svg) {
        var tmp = svg.querySelectorAll("path.ext-line");
        for (var t = tmp.length - 1; t >= 0; t--) {
          tmp[t].remove();
        }
        var paths = svg.querySelectorAll("path.int-line");
        for (var c = 0; c < paths.length; c++) {
          paths[c].classList.remove("highlight");
          paths[c].classList.remove("dim");
        }
      }
      var chips = document.querySelectorAll(".externals .chip.ext-active");
      for (var h = 0; h < chips.length; h++) {
        chips[h].classList.remove("ext-active");
      }
    }
    function showHover(pill) {
      var svg = graph.querySelector("svg.cb-edges");
      if (!svg) {
        return;
      }
      clearHover();
      var id = pill.getAttribute("data-node");
      var paths = svg.querySelectorAll("path.int-line");
      for (var k = 0; k < paths.length; k++) {
        var f = paths[k].getAttribute("data-from");
        var tt = paths[k].getAttribute("data-to");
        if (f === id || tt === id) {
          paths[k].classList.add("highlight");
        } else {
          paths[k].classList.add("dim");
        }
      }
      var wrap = pill.closest(".cb-wrap");
      var exts = wrap ? (wrap.getAttribute("data-exts") || "") : "";
      if (!exts) {
        return;
      }
      var names = exts.split(",");
      var ns = "http://www.w3.org/2000/svg";
      var box = graph.getBoundingClientRect();
      var pillBox = pill.getBoundingClientRect();
      for (var e = 0; e < names.length; e++) {
        var name = names[e];
        if (!name) {
          continue;
        }
        var chip = document.querySelector('.externals .chip[data-ext="' + name + '"]');
        if (!chip) {
          continue;
        }
        chip.classList.add("ext-active");
        var chipBox = chip.getBoundingClientRect();
        var x1 = chipBox.left + chipBox.width / 2 - box.left;
        var y1 = chipBox.top - box.top;
        var x2 = pillBox.left + pillBox.width / 2 - box.left;
        var y2 = pillBox.top + pillBox.height - box.top;
        var mid = (y1 + y2) / 2;
        var line = document.createElementNS(ns, "path");
        line.setAttribute("class", "ext-line");
        line.setAttribute(
          "d",
          "M " + x1 + " " + y1 + " C " + x1 + " " + mid + ", " + x2 + " " + mid + ", " + x2 + " " + y2
        );
        svg.appendChild(line);
      }
    }
    for (var p = 0; p < pills.length; p++) {
      (function (pill) {
        pill.addEventListener("mouseenter", function () {
          showHover(pill);
        });
        pill.addEventListener("focus", function () {
          showHover(pill);
        });
        pill.addEventListener("mouseleave", function () {
          if (!pill.classList.contains("pinned")) {
            clearHover();
          }
        });
        pill.addEventListener("blur", function () {
          if (!pill.classList.contains("pinned")) {
            clearHover();
          }
        });
        pill.addEventListener("click", function (event) {
          var pinned = pill.classList.contains("pinned");
          var all = graph.querySelectorAll(".cb-pill.pinned");
          for (var q = 0; q < all.length; q++) {
            all[q].classList.remove("pinned");
          }
          clearHover();
          if (!pinned) {
            pill.classList.add("pinned");
            showHover(pill);
          }
          event.stopPropagation();
        });
      })(pills[p]);
    }
    document.addEventListener("click", function () {
      var all = graph.querySelectorAll(".cb-pill.pinned");
      for (var q = 0; q < all.length; q++) {
        all[q].classList.remove("pinned");
      }
      clearHover();
    });
    drawCodeEdges(graph);
    var timer = null;
    window.addEventListener("resize", function () {
      if (timer) {
        clearTimeout(timer);
      }
      timer = setTimeout(function () {
        drawCodeEdges(graph);
      }, 150);
    });
  }

  function drawCodeEdges(graph) {
    var svg = graph.querySelector("svg.cb-edges");
    var raw = document.getElementById("cb-edges");
    if (!svg || !raw) {
      return;
    }
    var edges = [];
    try {
      edges = JSON.parse(raw.textContent || "[]");
    } catch (err) {
      return;
    }
    while (svg.firstChild) {
      svg.removeChild(svg.firstChild);
    }
    if (edges.length > 600) {
      return;
    }
    var ns = "http://www.w3.org/2000/svg";
    var defs = document.createElementNS(ns, "defs");
    defs.innerHTML =
      '<marker id="cb-arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="4" markerHeight="4" orient="auto"><path d="M 0 1 L 9 5 L 0 9 z" fill="#58a6ff"></path></marker>';
    svg.appendChild(defs);
    var box = graph.getBoundingClientRect();
    for (var i = 0; i < edges.length; i++) {
      var dep = graph.querySelector('[data-node="' + edges[i].from + '"]');
      var user = graph.querySelector('[data-node="' + edges[i].to + '"]');
      if (!dep || !user) {
        continue;
      }
      var a = dep.getBoundingClientRect();
      var b = user.getBoundingClientRect();
      var x1 = a.left + a.width / 2 - box.left;
      var y1 = a.top - box.top;
      var x2 = b.left + b.width / 2 - box.left;
      var y2 = b.top + b.height - box.top;
      var mid = (y1 + y2) / 2;
      var path = document.createElementNS(ns, "path");
      path.setAttribute("class", "int-line");
      path.setAttribute("data-from", edges[i].from);
      path.setAttribute("data-to", edges[i].to);
      path.setAttribute(
        "d",
        "M " + x1 + " " + y1 + " C " + x1 + " " + mid + ", " + x2 + " " + mid + ", " + x2 + " " + y2
      );
      svg.appendChild(path);
    }
  }

  window.addEventListener("hashchange", highlightHash);
  window.addEventListener("load", function () {
    setupDrawer();
    setupProjectFilter();
    setupCodeBlocks();
    highlightHash();
    restoreLiveState();
    setupLive();
  });
})();
