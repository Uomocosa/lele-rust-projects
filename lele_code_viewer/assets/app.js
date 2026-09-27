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

  window.addEventListener("hashchange", highlightHash);
  window.addEventListener("load", function () {
    setupDrawer();
    setupProjectFilter();
    highlightHash();
    restoreLiveState();
    setupLive();
  });
})();
