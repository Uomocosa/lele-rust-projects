(function () {
  "use strict";

  var cbNodeCenter = {};
  var cbChipCenter = {};

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
    var HOLD_MS = 700;
    var adj = {};
    var edgePaths = [];
    var lastTouched = [];
    var lastChips = [];
    var chipByExt = {};
    var pills = graph.querySelectorAll(".cb-pill[data-node]");
    function clearHover() {
      var svg = graph.querySelector("svg.cb-edges");
      if (svg) {
        svg.classList.remove("has-active");
        var tmp = svg.querySelectorAll("path.ext-line");
        for (var t = tmp.length - 1; t >= 0; t--) {
          tmp[t].remove();
        }
        for (var c = 0; c < lastTouched.length; c++) {
          lastTouched[c].classList.remove("highlight");
        }
        lastTouched = [];
      }
      for (var h = 0; h < lastChips.length; h++) {
        lastChips[h].classList.remove("ext-active");
      }
      lastChips = [];
    }
    function rebuildAdjacency() {
      adj = {};
      edgePaths = [];
      var svg = graph.querySelector("svg.cb-edges");
      if (!svg) {
        return;
      }
      var raw = document.getElementById("cb-adj");
      var table = null;
      if (raw) {
        try {
          table = JSON.parse(raw.textContent || "{}");
        } catch (err) {
          table = null;
        }
      }
      var paths = svg.querySelectorAll("path.int-line");
      for (var i = 0; i < paths.length; i++) {
        var idx = paths[i].getAttribute("data-edge");
        if (idx === null || idx === "") {
          continue;
        }
        edgePaths[parseInt(idx, 10)] = paths[i];
      }
      if (table && typeof table === "object") {
        for (var id in table) {
          if (!Object.prototype.hasOwnProperty.call(table, id)) {
            continue;
          }
          var list = [];
          var edges = table[id] || [];
          for (var k = 0; k < edges.length; k++) {
            var path = edgePaths[edges[k]];
            if (path) {
              list.push(path);
            }
          }
          adj[id] = list;
        }
        return;
      }
      for (var j = 0; j < paths.length; j++) {
        var f = paths[j].getAttribute("data-from");
        var tt = paths[j].getAttribute("data-to");
        if (f) {
          (adj[f] = adj[f] || []).push(paths[j]);
        }
        if (tt && tt !== f) {
          (adj[tt] = adj[tt] || []).push(paths[j]);
        }
      }
    }
    function showHover(pill) {
      var svg = graph.querySelector("svg.cb-edges");
      if (!svg) {
        return;
      }
      clearHover();
      var id = pill.getAttribute("data-node");
      var list = adj[id] || [];
      if (list.length > 0) {
        svg.classList.add("has-active");
      }
      for (var k = 0; k < list.length; k++) {
        list[k].classList.add("highlight");
      }
      lastTouched = list;
      var wrap = pill.closest(".cb-wrap");
      var exts = wrap ? (wrap.getAttribute("data-exts") || "") : "";
      if (!exts) {
        return;
      }
      var pc = cbNodeCenter[id];
      if (!pc) {
        return;
      }
      var names = exts.split(",");
      var ns = "http://www.w3.org/2000/svg";
      for (var e = 0; e < names.length; e++) {
        var name = names[e];
        if (!name) {
          continue;
        }
        var cc = cbChipCenter[name];
        var chip = chipByExt[name];
        if (!cc || !chip) {
          continue;
        }
        chip.classList.add("ext-active");
        lastChips.push(chip);
        var mid = (cc[1] + pc[1]) / 2;
        var line = document.createElementNS(ns, "path");
        line.setAttribute("class", "ext-line");
        line.setAttribute(
          "d",
          "M " + cc[0] + " " + cc[1] + " C " + cc[0] + " " + mid + ", " + pc[0] + " " + mid + ", " + pc[0] + " " + pc[1]
        );
        svg.appendChild(line);
      }
    }
    function cacheChips() {
      chipByExt = {};
      var chips = document.querySelectorAll(".externals .chip[data-ext]");
      for (var m = 0; m < chips.length; m++) {
        var ext = chips[m].getAttribute("data-ext");
        if (ext && !chipByExt[ext]) {
          chipByExt[ext] = chips[m];
        }
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
          cancelHold(pill);
          if (!pill.classList.contains("pinned")) {
            clearHover();
          }
        });
        pill.addEventListener("blur", function () {
          cancelHold(pill);
          if (!pill.classList.contains("pinned")) {
            clearHover();
          }
        });
        pill.addEventListener("pointerdown", function (event) {
          if (event.button !== undefined && event.button !== 0) {
            return;
          }
          startHold(pill);
        });
        pill.addEventListener("pointerup", function () {
          cancelHold(pill);
        });
        pill.addEventListener("pointercancel", function () {
          cancelHold(pill);
        });
        pill.addEventListener("click", function (event) {
          if (pill.__lcvHoldFired) {
            pill.__lcvHoldFired = false;
            cancelHold(pill);
            return;
          }
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
    function cancelHold(pill) {
      var wrap = pill.closest(".cb-wrap");
      if (wrap) {
        wrap.classList.remove("holding");
        wrap.style.setProperty("--hold-p", "0");
      }
      if (pill.__lcvHoldRaf) {
        window.cancelAnimationFrame(pill.__lcvHoldRaf);
        pill.__lcvHoldRaf = 0;
      }
    }
    function startHold(pill) {
      var wrap = pill.closest(".cb-wrap");
      var href = wrap ? wrap.getAttribute("data-href") : null;
      if (!wrap || !href) {
        return;
      }
      cancelHold(pill);
      pill.__lcvHoldFired = false;
      var start = window.performance.now();
      wrap.classList.add("holding");
      var step = function (now) {
        var p = (now - start) / HOLD_MS;
        if (p >= 1) {
          wrap.style.setProperty("--hold-p", "1");
          pill.__lcvHoldRaf = 0;
          pill.__lcvHoldFired = true;
          window.location.href = href;
          return;
        }
        wrap.style.setProperty("--hold-p", String(p));
        pill.__lcvHoldRaf = window.requestAnimationFrame(step);
      };
      pill.__lcvHoldRaf = window.requestAnimationFrame(step);
    }
    document.addEventListener("click", function () {
      var all = graph.querySelectorAll(".cb-pill.pinned");
      for (var q = 0; q < all.length; q++) {
        all[q].classList.remove("pinned");
      }
      clearHover();
    });
    drawCodeEdges(graph);
    rebuildAdjacency();
    cacheChips();
    var timer = null;
    window.addEventListener("resize", function () {
      if (timer) {
        clearTimeout(timer);
      }
      timer = setTimeout(function () {
        drawCodeEdges(graph);
        rebuildAdjacency();
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
    var nodeEls = {};
    var found = graph.querySelectorAll("[data-node]");
    for (var n = 0; n < found.length; n++) {
      var key = found[n].getAttribute("data-node");
      if (key && !nodeEls[key]) {
        nodeEls[key] = found[n];
      }
    }
    var rects = [];
    for (var i = 0; i < edges.length; i++) {
      var dep = nodeEls[edges[i].from];
      var user = nodeEls[edges[i].to];
      if (!dep || !user) {
        rects.push(null);
        continue;
      }
      rects.push([dep.getBoundingClientRect(), user.getBoundingClientRect()]);
    }
    var frag = document.createDocumentFragment();
    for (var j = 0; j < edges.length; j++) {
      var pair = rects[j];
      if (!pair) {
        continue;
      }
      var a = pair[0];
      var b = pair[1];
      var x1 = a.left + a.width / 2 - box.left;
      var y1 = a.top - box.top;
      var x2 = b.left + b.width / 2 - box.left;
      var y2 = b.top + b.height - box.top;
      var mid = (y1 + y2) / 2;
      var path = document.createElementNS(ns, "path");
      path.setAttribute("class", "int-line");
      path.setAttribute("data-edge", j);
      path.setAttribute("data-from", edges[j].from);
      path.setAttribute("data-to", edges[j].to);
      path.setAttribute(
        "d",
        "M " + x1 + " " + y1 + " C " + x1 + " " + mid + ", " + x2 + " " + mid + ", " + x2 + " " + y2
      );
      frag.appendChild(path);
    }
    svg.appendChild(frag);
    cacheCenters(graph, nodeEls, box);
  }

  function cacheCenters(graph, nodeEls, box) {
    cbNodeCenter = {};
    for (var id in nodeEls) {
      if (!Object.prototype.hasOwnProperty.call(nodeEls, id)) {
        continue;
      }
      var r = nodeEls[id].getBoundingClientRect();
      cbNodeCenter[id] = [r.left + r.width / 2 - box.left, r.top + r.height - box.top];
    }
    cbChipCenter = {};
    var chips = document.querySelectorAll(".externals .chip[data-ext]");
    for (var c = 0; c < chips.length; c++) {
      var ext = chips[c].getAttribute("data-ext");
      if (ext && !cbChipCenter[ext]) {
        var b = chips[c].getBoundingClientRect();
        cbChipCenter[ext] = [b.left + b.width / 2 - box.left, b.top - box.top];
      }
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
