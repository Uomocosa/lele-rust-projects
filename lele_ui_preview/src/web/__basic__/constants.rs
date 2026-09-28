pub const DEFAULT_MAX_STATES: usize = 40;
pub const DEFAULT_MAX_DEPTH: usize = 3;
pub const DEFAULT_SETTLE_MS: u64 = 400;
pub const DEFAULT_START: &str = "/";
pub const DEFAULT_FILL_TEXT: &str = "src";
pub const DEFAULT_CHROME: &str = "google-chrome";
pub const DEFAULT_STARTUP_TIMEOUT_SECS: u64 = 900;

pub const INTERACTIVE_SELECTOR: &str =
    "a[href], button, input, textarea, select, summary, [tabindex], [onclick], [role=\"button\"]";

pub const PROBE_JS: &str = r##"(() => {
  const SEL = __SELECTOR__;
  const STATE = new Set(["active", "open", "show", "current", "selected", "target", "hl", "recent", "live"]);
  const part = (el) => {
    const classes = Array.from(el.classList).filter((c) => !STATE.has(c)).sort().join(".");
    const id = el.id && /^[A-Za-z_-]+$/.test(el.id) ? "#" + el.id : "";
    return el.tagName.toLowerCase() + (classes ? "." + classes : "") + id;
  };
  const key = (el) => {
    const parts = [part(el)];
    let node = el.parentElement;
    let depth = 0;
    while (node && node !== document.body && depth < 3) {
      parts.unshift(part(node));
      node = node.parentElement;
      depth += 1;
    }
    return parts.join(">");
  };
  const elements = [];
  document.querySelectorAll(SEL).forEach((el, index) => {
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    if (r.width <= 0 || r.height <= 0) return;
    if (cs.visibility === "hidden" || cs.display === "none" || parseFloat(cs.opacity) === 0) return;
    if (r.right <= 0 || r.left >= innerWidth) return;
    if (el.disabled || el.closest("[inert]")) return;
    const label = (el.getAttribute("aria-label") || el.innerText || el.value || el.getAttribute("placeholder") || el.id || el.tagName)
      .trim().replace(/\s+/g, " ").slice(0, 40);
    elements.push({
      index,
      group: key(el),
      tag: el.tagName.toLowerCase(),
      href: el.tagName === "A" ? el.getAttribute("href") : null,
      absolute: el.tagName === "A" ? el.href : null,
      input_type: el.tagName === "INPUT" ? (el.getAttribute("type") || "text") : (el.tagName === "TEXTAREA" ? "text" : null),
      label,
    });
  });
  return JSON.stringify({
    url: location.pathname + location.search,
    hash: location.hash,
    origin: location.origin,
    body_class: document.body ? document.body.className : "",
    scrolled: window.scrollY > 4,
    scrollable: document.documentElement.scrollHeight > window.innerHeight + 40,
    typed: Array.from(document.querySelectorAll("input, textarea")).some((e) => e.value),
    elements,
  });
})()"##;
