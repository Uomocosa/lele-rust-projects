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
    function redraw() {
      drawCodeEdges(graph);
      drawGroupHulls(graph);
      rebuildAdjacency();
      cacheChips();
    }
    var toggle = document.getElementById("cb-show-layers");
    var layered = readPref("lcv-show-layers", toggle ? toggle.checked : true);
    function relayout() {
      graph.classList.add("animating");
      forceLayout(graph, toggle ? toggle.checked : true);
      window.setTimeout(function () {
        graph.classList.remove("animating");
        var fitButton = document.getElementById("cb-fit");
        if (fitButton) {
          fitButton.click();
        }
        redraw();
      }, 560);
    }
    if (toggle) {
      toggle.checked = layered;
      toggle.addEventListener("change", function () {
        writePref("lcv-show-layers", toggle.checked);
        relayout();
      });
    }
    forceLayout(graph, layered);
    redraw();
    setupPanZoom(graph, redraw);
    var timer = null;
    window.addEventListener("resize", function () {
      if (timer) {
        clearTimeout(timer);
      }
      timer = setTimeout(redraw, 150);
    });
  }

  function readPref(key, fallback) {
    try {
      var v = window.localStorage.getItem(key);
      return v === null ? fallback : v === "1";
    } catch (err) {
      return fallback;
    }
  }

  function writePref(key, on) {
    try {
      window.localStorage.setItem(key, on ? "1" : "0");
    } catch (err) {
      return;
    }
  }

  function readJson(id, fallback) {
    var raw = document.getElementById(id);
    if (!raw) {
      return fallback;
    }
    try {
      return JSON.parse(raw.textContent || "");
    } catch (err) {
      return fallback;
    }
  }

  function groupColor(g) {
    return "hsl(" + Math.round((g * 137.508 + 200) % 360) + ", 70%, 62%)";
  }

  var LAYER_GAP = 96;
  var LINK_LEN = 90;
  var PAD = 48;
  var ROW_H = 44;
  var BAND_WIDTH = 1500;

  // Force-directed layout: edges are springs, nodes repel (harder across groups),
  // every group pulls its members to its centroid (shared nodes settle in between).
  // With layers on, each node's y is pinned to its layer row.
  function forceLayout(graph, layered) {
    var nodes = graph.__lcvNodes;
    if (!nodes) {
      nodes = initNodes(graph);
      graph.__lcvNodes = nodes;
    }
    if (nodes.list.length === 0) {
      return;
    }
    assignGroups(nodes, readJson("cb-groups", []));
    graph.classList.toggle("layered", layered);
    assignPrimary(nodes);
    if (layered) {
      laneLayout(nodes);
    } else {
      nodes.bands = null;
      simulate(nodes, false);
    }
    var minX = Infinity;
    var minY = Infinity;
    var maxX = -Infinity;
    var maxY = -Infinity;
    for (var i = 0; i < nodes.list.length; i++) {
      var n = nodes.list[i];
      minX = Math.min(minX, n.x - n.w / 2);
      minY = Math.min(minY, n.y - n.h / 2);
      maxX = Math.max(maxX, n.x + n.w / 2);
      maxY = Math.max(maxY, n.y + n.h / 2);
    }
    var offX = PAD + (layered ? 40 : 0) - minX;
    var offY = PAD - minY;
    for (var j = 0; j < nodes.list.length; j++) {
      var m = nodes.list[j];
      m.x += offX;
      m.y += offY;
      m.el.style.left = Math.round(m.x - m.w / 2) + "px";
      m.el.style.top = Math.round(m.y - m.h / 2) + "px";
    }
    graph.style.width = Math.ceil(maxX + offX + PAD) + "px";
    graph.style.height = Math.ceil(maxY + offY + PAD) + "px";
    nodes.offY = offY;
    nodes.layered = layered;
  }

  function initNodes(graph) {
    var box = graph.getBoundingClientRect();
    var wraps = graph.querySelectorAll(".cb-wrap[data-node]");
    var starts = [];
    for (var s0 = 0; s0 < wraps.length; s0++) {
      starts.push(wraps[s0].getBoundingClientRect());
    }
    graph.classList.add("forced");
    var list = [];
    var byId = {};
    var top = 0;
    for (var i = 0; i < wraps.length; i++) {
      var el = wraps[i];
      var id = el.getAttribute("data-node");
      if (!id || byId[id]) {
        continue;
      }
      var r = starts[i];
      var size = el.getBoundingClientRect();
      var node = {
        id: id,
        el: el,
        w: size.width,
        h: size.height,
        x: r.left - box.left + r.width / 2,
        y: r.top - box.top + r.height / 2,
        vx: 0,
        vy: 0,
        layer: parseInt(el.getAttribute("data-layer") || "0", 10),
        groups: []
      };
      top = Math.max(top, node.layer);
      byId[id] = node;
      list.push(node);
    }
    var edges = readJson("cb-edges", []);
    var links = [];
    for (var e = 0; e < edges.length; e++) {
      var a = byId[edges[e].from];
      var b = byId[edges[e].to];
      if (a && b && a !== b) {
        links.push([a, b]);
      }
    }
    return { list: list, byId: byId, members: [], links: links, top: top };
  }

  function assignGroups(nodes, groups) {
    for (var i = 0; i < nodes.list.length; i++) {
      nodes.list[i].groups = [];
    }
    nodes.members = [];
    for (var g = 0; g < groups.length; g++) {
      var ms = [];
      for (var k = 0; k < groups[g].length; k++) {
        var n = nodes.byId[groups[g][k]];
        if (n) {
          n.groups.push(g);
          ms.push(n);
        }
      }
      nodes.members.push(ms);
    }
  }

  // Each node's main group: the one it has the most links into (ties: lower index).
  function assignPrimary(nodes) {
    for (var i = 0; i < nodes.list.length; i++) {
      nodes.list[i].primary = -1;
      nodes.list[i].linkCount = {};
    }
    for (var l = 0; l < nodes.links.length; l++) {
      var a = nodes.links[l][0];
      var b = nodes.links[l][1];
      for (var x = 0; x < b.groups.length; x++) {
        a.linkCount[b.groups[x]] = (a.linkCount[b.groups[x]] || 0) + 1;
      }
      for (var y = 0; y < a.groups.length; y++) {
        b.linkCount[a.groups[y]] = (b.linkCount[a.groups[y]] || 0) + 1;
      }
    }
    for (var n = 0; n < nodes.list.length; n++) {
      var node = nodes.list[n];
      var best = -1;
      for (var g = 0; g < node.groups.length; g++) {
        var cand = node.groups[g];
        if (best === -1 || (node.linkCount[cand] || 0) > (node.linkCount[best] || 0)) {
          best = cand;
        }
      }
      node.primary = best;
    }
  }

  // Layered mode: one column per group (ordered so linked groups are adjacent), pills of a
  // layer wrap into rows inside their column, and each layer band is as tall as its
  // fullest column. Overlap-free by construction.
  function laneLayout(nodes) {
    var anchors = groupAnchors(nodes, {});
    var order = anchors.order.slice();
    order.push(-1);
    var cells = {};
    for (var i = 0; i < nodes.list.length; i++) {
      var n = nodes.list[i];
      var key = n.primary + "|" + n.layer;
      (cells[key] = cells[key] || []).push(n);
    }
    var GAP_X = 16;
    var LANE_GAP = 44;
    var laneX = {};
    var laneW = {};
    var rowsOf = {};
    var cursor = 0;
    for (var o = 0; o < order.length; o++) {
      var g = order[o];
      var total = 0;
      var widest = 0;
      for (var layer = 0; layer <= nodes.top; layer++) {
        var cell = cells[g + "|" + layer] || [];
        var sum = 0;
        for (var c = 0; c < cell.length; c++) {
          sum += cell[c].w + GAP_X;
          widest = Math.max(widest, cell[c].w + GAP_X);
        }
        total = Math.max(total, sum);
      }
      if (total === 0) {
        continue;
      }
      var width = Math.max(widest, Math.min(total, Math.sqrt(total * 110)));
      var used = 0;
      for (var layer2 = 0; layer2 <= nodes.top; layer2++) {
        var list = cells[g + "|" + layer2] || [];
        list.sort(function (a, b) {
          return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
        });
        var rows = [];
        var row = [];
        var rowW = 0;
        for (var m = 0; m < list.length; m++) {
          var w = list[m].w + GAP_X;
          if (row.length > 0 && rowW + w > width) {
            rows.push(row);
            row = [];
            rowW = 0;
          }
          row.push(list[m]);
          rowW += w;
          used = Math.max(used, rowW);
        }
        if (row.length > 0) {
          rows.push(row);
        }
        rowsOf[g + "|" + layer2] = rows;
      }
      laneX[g] = cursor;
      laneW[g] = used;
      cursor += used + LANE_GAP;
    }
    var bands = {};
    var y = 0;
    for (var layer3 = nodes.top; layer3 >= 0; layer3--) {
      var maxRows = 1;
      for (var o2 = 0; o2 < order.length; o2++) {
        var r = rowsOf[order[o2] + "|" + layer3];
        if (r) {
          maxRows = Math.max(maxRows, r.length);
        }
      }
      bands[layer3] = [y, y + maxRows * ROW_H, maxRows];
      y += maxRows * ROW_H + LAYER_GAP - ROW_H;
    }
    for (var o3 = 0; o3 < order.length; o3++) {
      var lane = order[o3];
      if (laneX[lane] === undefined) {
        continue;
      }
      for (var layer4 = 0; layer4 <= nodes.top; layer4++) {
        var rows2 = rowsOf[lane + "|" + layer4] || [];
        var top = bands[layer4][0] + (bands[layer4][2] - rows2.length) * ROW_H / 2;
        for (var ri = 0; ri < rows2.length; ri++) {
          var rw = 0;
          for (var k = 0; k < rows2[ri].length; k++) {
            rw += rows2[ri][k].w + GAP_X;
          }
          var x = laneX[lane] + (laneW[lane] - rw) / 2;
          for (var k2 = 0; k2 < rows2[ri].length; k2++) {
            var node = rows2[ri][k2];
            node.x = x + node.w / 2;
            node.y = top + ri * ROW_H + ROW_H / 2;
            x += node.w + GAP_X;
          }
        }
      }
    }
    nodes.bands = bands;
  }

  function sharesGroup(a, b) {
    for (var i = 0; i < a.groups.length; i++) {
      if (b.groups.indexOf(a.groups[i]) !== -1) {
        return true;
      }
    }
    return false;
  }

  function simulate(nodes, layered) {
    var list = nodes.list;
    var count = list.length;
    var ticks = 320;
    var bands = layered ? layerBands(nodes) : null;
    var seen = {};
    for (var i = 0; i < count; i++) {
      list[i].vx = 0;
      list[i].vy = 0;
      if (bands) {
        var band = bands[list[i].layer];
        var slot = seen[list[i].layer] || 0;
        seen[list[i].layer] = slot + 1;
        list[i].y = band[0] + ROW_H / 2 + (slot % band[2]) * ROW_H;
      }
    }
    nodes.bands = bands;
    var anchors = groupAnchors(nodes, bands);
    for (var t = 0; t < ticks; t++) {
      var alpha = 1 - t / ticks;
      for (var p = 0; p < count; p++) {
        var a = list[p];
        for (var q = p + 1; q < count; q++) {
          var b = list[q];
          var dx = b.x - a.x;
          var dy = b.y - a.y;
          if (layered && a.layer !== b.layer && Math.abs(dy) > ROW_H * 1.5) {
            continue;
          }
          var ex = (a.w + b.w) / 2 + 14;
          var ey = (a.h + b.h) / 2 + 12;
          var nx = dx / ex;
          var ny = dy / ey;
          var d2 = nx * nx + ny * ny;
          if (d2 > 25) {
            continue;
          }
          var k = (sharesGroup(a, b) ? 1.2 : 3.2) * alpha / (d2 + 0.35);
          a.vx -= nx * ex * k * 0.06;
          a.vy -= ny * ey * k * 0.06;
          b.vx += nx * ex * k * 0.06;
          b.vy += ny * ey * k * 0.06;
        }
      }
      for (var l = 0; l < nodes.links.length; l++) {
        var s = nodes.links[l][0];
        var u = nodes.links[l][1];
        var lx = u.x - s.x;
        var ly = u.y - s.y;
        var d = Math.sqrt(lx * lx + ly * ly) + 0.01;
        var f = ((d - LINK_LEN) / d) * 0.04 * alpha;
        s.vx += lx * f;
        s.vy += ly * f;
        u.vx -= lx * f;
        u.vy -= ly * f;
      }
      for (var g = 0; g < nodes.members.length; g++) {
        var ms = nodes.members[g];
        if (ms.length === 0) {
          continue;
        }
        var cx = 0;
        var cy = 0;
        for (var m = 0; m < ms.length; m++) {
          cx += ms[m].x;
          cy += ms[m].y;
        }
        cx /= ms.length;
        cy /= ms.length;
        var anchor = anchors[g];
        for (var n = 0; n < ms.length; n++) {
          var share = ms[n].groups.length;
          var pull = 0.05 * alpha / share;
          ms[n].vx += (cx - ms[n].x) * pull + (anchor[0] - ms[n].x) * 0.12 / share;
          ms[n].vy += (cy - ms[n].y) * pull;
          if (!bands) {
            ms[n].vy += (anchor[1] - ms[n].y) * 0.12 / share;
          }
        }
      }
      var meanX = 0;
      var meanY = 0;
      for (var z = 0; z < count; z++) {
        meanX += list[z].x / count;
        meanY += list[z].y / count;
      }
      for (var c = 0; c < count; c++) {
        var node = list[c];
        if (node.groups.length === 0) {
          node.vx += (meanX - node.x) * 0.004 * alpha;
          node.vy += (meanY - node.y) * 0.004 * alpha;
        }
        node.vx = Math.max(-24, Math.min(24, node.vx));
        node.vy = Math.max(-24, Math.min(24, node.vy));
        node.x += node.vx;
        node.vx *= 0.55;
        node.y += node.vy;
        node.vy *= 0.55;
      }
      if (t > ticks / 3) {
        resolveOverlaps(list, bands);
      }
      clampToBands(list, bands);
    }
    for (var r = 0; r < 200; r++) {
      var moved = resolveOverlaps(list, bands);
      clampToBands(list, bands);
      if (!moved) {
        break;
      }
    }
  }

  // Order groups so strongly linked ones are neighbours (greedy chain), then give
  // each an anchor: a lane x when layered, a grid cell otherwise.
  function groupAnchors(nodes, bands) {
    var count = nodes.members.length;
    var link = [];
    for (var i = 0; i < count; i++) {
      link.push(new Array(count).fill(0));
    }
    for (var l = 0; l < nodes.links.length; l++) {
      var ga = nodes.links[l][0].groups;
      var gb = nodes.links[l][1].groups;
      for (var x = 0; x < ga.length; x++) {
        for (var y = 0; y < gb.length; y++) {
          if (ga[x] !== gb[y]) {
            link[ga[x]][gb[y]] += 1;
            link[gb[y]][ga[x]] += 1;
          }
        }
      }
    }
    var order = [];
    var placed = new Array(count).fill(false);
    for (var step = 0; step < count; step++) {
      var best = -1;
      var bestScore = -1;
      for (var c = 0; c < count; c++) {
        if (placed[c]) {
          continue;
        }
        var score = order.length === 0 ? nodes.members[c].length : link[order[order.length - 1]][c] * 1000 + nodes.members[c].length;
        if (score > bestScore) {
          best = c;
          bestScore = score;
        }
      }
      placed[best] = true;
      order.push(best);
    }
    var sizes = [];
    for (var g = 0; g < count; g++) {
      var perLayer = {};
      var area = 0;
      var ms = nodes.members[g];
      for (var m = 0; m < ms.length; m++) {
        perLayer[ms[m].layer] = (perLayer[ms[m].layer] || 0) + (ms[m].w + 24) / ms[m].groups.length;
        area += (ms[m].w + 24) * (ms[m].h + 20);
      }
      var lane = 160;
      for (var key in perLayer) {
        if (Object.prototype.hasOwnProperty.call(perLayer, key)) {
          var rows = bands && bands[key] ? bands[key][2] : 1;
          lane = Math.max(lane, perLayer[key] / Math.min(rows, 2));
        }
      }
      sizes.push([lane, Math.sqrt(area) * 1.3]);
    }
    var anchors = new Array(count);
    anchors.order = order;
    if (bands) {
      var cursor = 0;
      for (var o = 0; o < order.length; o++) {
        var w = sizes[order[o]][0];
        anchors[order[o]] = [cursor + w / 2, 0];
        cursor += w + 60;
      }
      return anchors;
    }
    var cols = Math.max(1, Math.ceil(Math.sqrt(count)));
    var rowY = 0;
    var rowH = 0;
    var colX = 0;
    for (var p = 0; p < order.length; p++) {
      if (p > 0 && p % cols === 0) {
        rowY += rowH + 80;
        rowH = 0;
        colX = 0;
      }
      var cell = sizes[order[p]][1];
      anchors[order[p]] = [colX + cell / 2, rowY + cell / 2];
      colX += cell + 80;
      rowH = Math.max(rowH, cell);
    }
    return anchors;
  }

  // Each layer is a horizontal band tall enough to hold its pills in a few rows.
  function layerBands(nodes) {
    var widths = {};
    for (var i = 0; i < nodes.list.length; i++) {
      var n = nodes.list[i];
      widths[n.layer] = (widths[n.layer] || 0) + n.w + 24;
    }
    var bands = {};
    var y = 0;
    for (var layer = nodes.top; layer >= 0; layer--) {
      var rows = Math.max(1, Math.ceil((widths[layer] || 0) / BAND_WIDTH));
      bands[layer] = [y, y + rows * ROW_H, rows];
      y += rows * ROW_H + LAYER_GAP - ROW_H;
    }
    return bands;
  }

  function clampToBands(list, bands) {
    if (!bands) {
      return;
    }
    for (var i = 0; i < list.length; i++) {
      var band = bands[list[i].layer];
      if (band) {
        list[i].y = Math.max(band[0] + ROW_H / 2, Math.min(band[1] - ROW_H / 2, list[i].y));
      }
    }
  }

  function resolveOverlaps(list, bands) {
    var moved = false;
    for (var p = 0; p < list.length; p++) {
      var a = list[p];
      for (var q = p + 1; q < list.length; q++) {
        var b = list[q];
        var dx = b.x - a.x;
        var dy = b.y - a.y;
        var ox = (a.w + b.w) / 2 + 14 - Math.abs(dx);
        var oy = (a.h + b.h) / 2 + 12 - Math.abs(dy);
        if (ox <= 0.5 || oy <= 0.5) {
          continue;
        }
        moved = true;
        var vertical = ox >= oy;
        if (bands && vertical) {
          var band = a.layer === b.layer ? bands[a.layer] : null;
          var lo = band ? band[0] + ROW_H / 2 : 0;
          var hi = band ? band[1] - ROW_H / 2 : 0;
          var up = Math.min(a.y, b.y) - oy / 2;
          var down = Math.max(a.y, b.y) + oy / 2;
          vertical = band !== null && band[2] > 1 && up >= lo - 0.5 && down <= hi + 0.5;
        }
        if (!vertical) {
          var sx = (dx < 0 || (dx === 0 && p % 2 === 1) ? -1 : 1) * ox / 2;
          a.x -= sx;
          b.x += sx;
        } else {
          var sy = (dy < 0 ? -1 : 1) * oy / 2;
          a.y -= sy;
          b.y += sy;
        }
      }
    }
    return moved;
  }

  // Dashed rounded convex hull per group, plus layer guides when layered.
  function drawGroupHulls(graph) {
    var svg = graph.querySelector("svg.cb-groups");
    var nodes = graph.__lcvNodes;
    if (!svg || !nodes) {
      return;
    }
    while (svg.firstChild) {
      svg.removeChild(svg.firstChild);
    }
    var ns = "http://www.w3.org/2000/svg";
    var width = graph.clientWidth;
    if (nodes.layered) {
      for (var layer in nodes.bands) {
        if (!Object.prototype.hasOwnProperty.call(nodes.bands, layer)) {
          continue;
        }
        var y = nodes.bands[layer][0] + nodes.offY - (LAYER_GAP - ROW_H) / 2;
        var line = document.createElementNS(ns, "line");
        line.setAttribute("class", "cb-layer-line");
        line.setAttribute("x1", "0");
        line.setAttribute("x2", String(width));
        line.setAttribute("y1", String(y));
        line.setAttribute("y2", String(y));
        svg.appendChild(line);
        var label = document.createElementNS(ns, "text");
        label.setAttribute("class", "cb-layer-label");
        label.setAttribute("x", "4");
        label.setAttribute("y", String(y + 14));
        label.textContent = "L" + layer;
        svg.appendChild(label);
      }
    }
    for (var g = 0; g < nodes.members.length; g++) {
      var split = nodes.layered ? splitByPrimary(nodes.members[g], g) : splitOutliers(nodes.members[g]);
      var ms = split[0];
      for (var o = 0; o < split[1].length; o++) {
        var out = split[1][o];
        var slot = out.groups.indexOf(g);
        var dot = document.createElementNS(ns, "circle");
        dot.setAttribute("class", "cb-member-dot");
        dot.setAttribute("cx", String(out.x - out.w / 2 + 10 + slot * 9));
        dot.setAttribute("cy", String(out.y - out.h / 2 - 3));
        dot.setAttribute("r", "4");
        dot.style.setProperty("--g", groupColor(g));
        svg.appendChild(dot);
      }
      var pts = [];
      for (var m = 0; m < ms.length; m++) {
        var n = ms[m];
        var hw = n.w / 2 + 12;
        var hh = n.h / 2 + 10;
        pts.push([n.x - hw, n.y - hh], [n.x + hw, n.y - hh], [n.x - hw, n.y + hh], [n.x + hw, n.y + hh]);
      }
      var hull = convexHull(pts);
      if (hull.length < 3) {
        continue;
      }
      var d = "M " + hull[0][0] + " " + hull[0][1];
      for (var h = 1; h < hull.length; h++) {
        d += " L " + hull[h][0] + " " + hull[h][1];
      }
      var path = document.createElementNS(ns, "path");
      path.setAttribute("class", "cb-hull");
      path.setAttribute("d", d + " Z");
      path.setAttribute("data-group", String(g));
      path.style.setProperty("--g", groupColor(g));
      svg.appendChild(path);
    }
  }

  function splitByPrimary(ms, g) {
    var inside = [];
    var outside = [];
    for (var i = 0; i < ms.length; i++) {
      (ms[i].primary === g ? inside : outside).push(ms[i]);
    }
    return [inside, outside];
  }

  // Members far from the group's median centre get a ring instead of stretching the hull.
  function splitOutliers(ms) {
    if (ms.length < 4) {
      return [ms, []];
    }
    var xs = ms.map(function (n) { return n.x; }).sort(function (a, b) { return a - b; });
    var ys = ms.map(function (n) { return n.y; }).sort(function (a, b) { return a - b; });
    var mid = Math.floor(ms.length / 2);
    var cx = xs[mid];
    var cy = ys[mid];
    var dist = ms.map(function (n) { return Math.hypot(n.x - cx, n.y - cy); });
    var sorted = dist.slice().sort(function (a, b) { return a - b; });
    var limit = Math.max(sorted[mid] * 2.2, 160);
    var inside = [];
    var outside = [];
    for (var i = 0; i < ms.length; i++) {
      (dist[i] <= limit ? inside : outside).push(ms[i]);
    }
    return [inside, outside];
  }

  function graphScale(graph) {
    return graph.__lcvView ? graph.__lcvView.k : 1;
  }

  // Pan (drag the background) and zoom (wheel / pinch) the graph viewport; "fit" frames it.
  function setupPanZoom(graph, redraw) {
    var port = graph.parentElement;
    if (!port) {
      return;
    }
    var view = { x: 0, y: 0, k: 1 };
    graph.__lcvView = view;
    function apply() {
      graph.style.transform = "translate(" + view.x + "px, " + view.y + "px) scale(" + view.k + ")";
    }
    function zoomAt(px, py, factor) {
      var k = Math.max(0.15, Math.min(2.5, view.k * factor));
      view.x = px - (px - view.x) * (k / view.k);
      view.y = py - (py - view.y) * (k / view.k);
      view.k = k;
      apply();
    }
    function fit() {
      var w = graph.offsetWidth;
      var h = graph.offsetHeight;
      var k = Math.min(1, port.clientWidth / w, port.clientHeight / h);
      view.k = Math.max(0.15, k);
      view.x = (port.clientWidth - w * view.k) / 2;
      view.y = Math.max(0, (port.clientHeight - h * view.k) / 2);
      apply();
    }
    port.addEventListener("wheel", function (event) {
      event.preventDefault();
      var r = port.getBoundingClientRect();
      zoomAt(event.clientX - r.left, event.clientY - r.top, Math.exp(-event.deltaY * 0.0015));
    }, { passive: false });
    var pointers = {};
    var last = null;
    var dragged = false;
    port.addEventListener("pointerdown", function (event) {
      if (event.target.closest && event.target.closest(".cb-wrap")) {
        return;
      }
      pointers[event.pointerId] = [event.clientX, event.clientY];
      port.setPointerCapture(event.pointerId);
      dragged = false;
      last = null;
    });
    port.addEventListener("pointermove", function (event) {
      if (!pointers[event.pointerId]) {
        return;
      }
      var prev = pointers[event.pointerId];
      pointers[event.pointerId] = [event.clientX, event.clientY];
      var ids = Object.keys(pointers);
      if (ids.length === 2) {
        var a = pointers[ids[0]];
        var b = pointers[ids[1]];
        var dist = Math.hypot(a[0] - b[0], a[1] - b[1]);
        if (last) {
          var r = port.getBoundingClientRect();
          zoomAt((a[0] + b[0]) / 2 - r.left, (a[1] + b[1]) / 2 - r.top, dist / last);
        }
        last = dist;
        dragged = true;
        return;
      }
      view.x += event.clientX - prev[0];
      view.y += event.clientY - prev[1];
      if (Math.abs(event.clientX - prev[0]) + Math.abs(event.clientY - prev[1]) > 0) {
        dragged = true;
      }
      apply();
    });
    function release(event) {
      delete pointers[event.pointerId];
      last = null;
    }
    port.addEventListener("pointerup", release);
    port.addEventListener("pointercancel", release);
    port.addEventListener("click", function (event) {
      if (dragged) {
        event.stopPropagation();
        dragged = false;
      }
    }, true);
    var button = document.getElementById("cb-fit");
    if (button) {
      button.addEventListener("click", function (event) {
        event.stopPropagation();
        fit();
      });
    }
    fit();
    redraw();
  }

  function convexHull(points) {
    var pts = points.slice().sort(function (a, b) {
      return a[0] - b[0] || a[1] - b[1];
    });
    if (pts.length < 3) {
      return pts;
    }
    function cross(o, a, b) {
      return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
    }
    var lower = [];
    for (var i = 0; i < pts.length; i++) {
      while (lower.length >= 2 && cross(lower[lower.length - 2], lower[lower.length - 1], pts[i]) <= 0) {
        lower.pop();
      }
      lower.push(pts[i]);
    }
    var upper = [];
    for (var j = pts.length - 1; j >= 0; j--) {
      while (upper.length >= 2 && cross(upper[upper.length - 2], upper[upper.length - 1], pts[j]) <= 0) {
        upper.pop();
      }
      upper.push(pts[j]);
    }
    lower.pop();
    upper.pop();
    return lower.concat(upper);
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
    if (edges.length > 4000) {
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
      var sc = graphScale(graph);
      var x1 = (a.left + a.width / 2 - box.left) / sc;
      var y1 = (a.top - box.top) / sc;
      var x2 = (b.left + b.width / 2 - box.left) / sc;
      var y2 = (b.top + b.height - box.top) / sc;
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
      var sc = graphScale(graph);
      cbNodeCenter[id] = [(r.left + r.width / 2 - box.left) / sc, (r.top + r.height - box.top) / sc];
    }
    cbChipCenter = {};
    var chips = document.querySelectorAll(".externals .chip[data-ext]");
    for (var c = 0; c < chips.length; c++) {
      var ext = chips[c].getAttribute("data-ext");
      if (ext && !cbChipCenter[ext]) {
        var b = chips[c].getBoundingClientRect();
        var sk = graphScale(graph);
        cbChipCenter[ext] = [(b.left + b.width / 2 - box.left) / sk, (b.top - box.top) / sk];
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
