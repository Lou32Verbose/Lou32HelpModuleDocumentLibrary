import init, { search_index } from "./__WASM_MODULE__.js";

const MIN_QUERY_LENGTH = __MIN_QUERY_LENGTH__;
const MAX_RESULTS = __MAX_RESULTS__;

const status = document.getElementById("search-status");
const resultsContainer = document.getElementById("search-results");
const queryInput = document.getElementById("query");
const topicInput = document.getElementById("topic-filter");
const typeInput = document.getElementById("type-filter");
const platformInput = document.getElementById("platform-filter");
const searchForm = document.getElementById("search-form");

let indexJson = "";

function updateStatus(message) {
  if (status) {
    status.textContent = message;
  }
}

function safeSlug(slug) {
  if (typeof slug !== "string") {
    return "/";
  }

  if (!slug.startsWith("/") || slug.startsWith("//") || slug.includes("\\") || slug.includes("\0")) {
    return "/";
  }

  return slug;
}

function slugToHref(slug) {
  const safe = safeSlug(slug);
  if (safe === "/") {
    return "../index.html";
  }

  return `..${safe}index.html`;
}

function appendTextElement(parent, tagName, className, text) {
  const element = document.createElement(tagName);
  if (className) {
    element.className = className;
  }
  element.textContent = text;
  parent.appendChild(element);
  return element;
}

function renderEmptyState() {
  const article = document.createElement("article");
  article.className = "doc-card";
  appendTextElement(article, "h3", "", "No matches");
  appendTextElement(
    article,
    "p",
    "summary",
    "Try a broader query, a different topic, or remove a filter."
  );
  resultsContainer.appendChild(article);
}

function renderResults(results) {
  if (!resultsContainer) {
    return;
  }

  resultsContainer.replaceChildren();
  if (!results.length) {
    renderEmptyState();
    return;
  }

  for (const result of results) {
    const article = document.createElement("article");
    article.className = "doc-card";

    const title = document.createElement("h3");
    const link = document.createElement("a");
    link.href = slugToHref(result.slug);
    link.textContent = result.title ?? "";
    title.appendChild(link);
    article.appendChild(title);

    appendTextElement(article, "p", "summary", result.summary ?? "");
    appendTextElement(
      article,
      "p",
      "meta",
      `${result.topic ?? ""} • ${result.type ?? ""} • score ${result.score ?? 0}`
    );

    resultsContainer.appendChild(article);
  }
}

function searchNow() {
  if (!indexJson) {
    updateStatus("Search index is still loading.");
    return;
  }

  const query = queryInput?.value?.trim() ?? "";
  if (query.length < MIN_QUERY_LENGTH) {
    updateStatus(`Type at least ${MIN_QUERY_LENGTH} character(s) to search the library.`);
    renderResults([]);
    return;
  }

  const payload = search_index(
    indexJson,
    query,
    topicInput?.value ?? "",
    typeInput?.value ?? "",
    platformInput?.value ?? "",
    MAX_RESULTS
  );

  const results = JSON.parse(payload);
  if (results.error) {
    console.error("Browser search failed", results.error);
    updateStatus("Failed to run browser search. Rebuild the WASM search bundle and try again.");
    renderResults([]);
    return;
  }

  updateStatus(`${results.length} result(s) for "${query}"`);
  renderResults(results);

  const params = new URLSearchParams(window.location.search);
  params.set("q", query);
  if (topicInput?.value) params.set("topic", topicInput.value); else params.delete("topic");
  if (typeInput?.value) params.set("type", typeInput.value); else params.delete("type");
  if (platformInput?.value) params.set("platform", platformInput.value); else params.delete("platform");
  history.replaceState(null, "", `${window.location.pathname}?${params.toString()}`);
}

const WASM_INTEGRITY = "__WASM_INTEGRITY__";

async function verifyWasmIntegrity(wasmUrl) {
  if (!WASM_INTEGRITY || WASM_INTEGRITY === "") {
    return;
  }

  if (typeof crypto === "undefined" || !crypto.subtle) {
    console.warn("crypto.subtle not available (non-secure context); skipping WASM integrity check");
    return;
  }

  const response = await fetch(wasmUrl);
  if (!response.ok) {
    throw new Error(`Failed to fetch WASM for integrity check: ${response.status}`);
  }
  const wasmBytes = await response.arrayBuffer();
  const hashBuffer = await crypto.subtle.digest("SHA-384", wasmBytes);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const hashBase64 = btoa(String.fromCharCode(...hashArray));
  const computed = `sha384-${hashBase64}`;

  if (computed !== WASM_INTEGRITY) {
    throw new Error(`WASM integrity check failed: expected ${WASM_INTEGRITY}, got ${computed}`);
  }
}

searchForm?.addEventListener("submit", (event) => {
  event.preventDefault();
  searchNow();
});

for (const element of [queryInput, topicInput, typeInput, platformInput]) {
  element?.addEventListener("input", searchNow);
  element?.addEventListener("change", searchNow);
}

async function main() {
  updateStatus("Loading search module...");
  const wasmUrl = new URL("./__WASM_MODULE___bg.wasm", import.meta.url);
  await verifyWasmIntegrity(wasmUrl);
  await init();
  const response = await fetch(new URL("./search-index.json", import.meta.url));
  indexJson = await response.text();

  const params = new URLSearchParams(window.location.search);
  if (params.has("q")) {
    queryInput.value = params.get("q") ?? "";
    topicInput.value = params.get("topic") ?? "";
    typeInput.value = params.get("type") ?? "";
    platformInput.value = params.get("platform") ?? "";
  }

  if (queryInput.value.length >= MIN_QUERY_LENGTH) {
    searchNow();
  } else {
    updateStatus(`Type at least ${MIN_QUERY_LENGTH} character(s) to search the library.`);
  }
}

main().catch((error) => {
  console.error("Failed to load browser search", error);
  updateStatus("Failed to load browser search. Rebuild the WASM search bundle and try again.");
});
