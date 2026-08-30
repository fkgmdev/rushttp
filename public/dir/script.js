"use strict";

const $ = (sel) => document.querySelector(sel);

function markLoaded(sel, label) {
  const el = $(sel);
  if (el) el.textContent = label + ": loaded";
}

markLoaded("#js-check", "script.js");

window.addEventListener("load", () => {
  markLoaded("#css-check", "style.css (linked in head)");
  $("#timestamp").textContent = new Date().toISOString();
});

const cssColor = getComputedStyle(document.documentElement)
  .getPropertyValue("--primary").trim();
const supportsGradient = CSS.supports("background", "linear-gradient(0deg, red, blue)");
$("#feature-detection").textContent =
  "CSS var retrieved: " + (cssColor || "none") +
  " | gradients: " + (supportsGradient ? "yes" : "no") +
  " | fetch: " + (typeof fetch === "function" ? "yes" : "no");

let clickCount = 0;
$("#click-counter-btn").addEventListener("click", () => {
  clickCount++;
  $("#click-count").textContent = String(clickCount);
});

const boxColors = ["var(--primary)", "var(--success)", "var(--danger)", "var(--warning)", "var(--secondary)"];
$("#add-box-btn").addEventListener("click", () => {
  const box = document.createElement("div");
  box.className = "demo-box";
  box.style.margin = "4px";
  box.style.background = boxColors[Math.floor(Math.random() * boxColors.length)];
  box.dataset.box = "true";
  $("#click-counter-btn").closest(".card").appendChild(box);
});

$("#clear-boxes-btn").addEventListener("click", () => {
  document.querySelectorAll('[data-box="true"]').forEach((b) => b.remove());
});

const hue = () => `hsl(${Math.floor(Math.random() * 360)} 60% 40%)`;
$("#random-bg-btn").addEventListener("click", () => {
  document.body.style.background = `linear-gradient(135deg, ${hue()}, ${hue()})`;
});

$("#random-bg-btn").closest(".card").querySelector(".muted[data-demo]").textContent =
  "JS event listeners attached successfully.";

$("#fetch-btn").addEventListener("click", async () => {
  const out = $("#fetch-result");
  out.textContent = "fetching /dir/info.json...";
  try {
    const res = await fetch("info.json");
    const body = await res.text();
    out.textContent = "HTTP " + res.status + " " + res.statusText + "\nContent-Type: " +
      (res.headers.get("content-type") || "missing") + "\n\n" + body;
  } catch (err) {
    out.textContent = "fetch failed: " + err;
  }
});