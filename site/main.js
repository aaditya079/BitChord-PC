/**
 * BitChord PC — Interactive Experience Script
 * Author: Aaditya Srinivasan
 */
(() => {
  "use strict";

  // --- Sticky Header Scroll ---
  const header = document.getElementById("header");
  const onScroll = () => {
    if (header) {
      header.classList.toggle("scrolled", window.scrollY > 24);
    }
  };
  window.addEventListener("scroll", onScroll, { passive: true });
  onScroll();

  // --- Live GitHub Stars & Release Assets ---
  const REPO = "aaditya079/BitChord-PC";
  const starCountEl = document.getElementById("star-count");

  fetch(`https://api.github.com/repos/${REPO}`)
    .then((r) => (r.ok ? r.json() : null))
    .then((data) => {
      if (data && typeof data.stargazers_count === "number" && starCountEl) {
        starCountEl.textContent = `★ ${data.stargazers_count}`;
      }
    })
    .catch(() => {});

  // --- OS Detection & Dynamic Download Linking ---
  const ua = navigator.userAgentData?.platform || navigator.platform || navigator.userAgent;
  const isMac = /mac/i.test(ua);
  const isLinux = /linux/i.test(ua) && !/android/i.test(ua);
  const isWindows = /win/i.test(ua);

  const heroBtn = document.getElementById("hero-download-btn");
  const heroLabel = document.getElementById("hero-download-label");
  const primaryBtn = document.getElementById("primary-dl-btn");
  const primaryLabel = document.getElementById("primary-dl-label");

  const WIN_URL = "https://github.com/aaditya079/BitChord-PC/releases/download/v2.2.0/bitchord-setup-2.2.0.exe";
  const MAC_URL = "https://github.com/aaditya079/BitChord-PC/releases/download/v2.2.0/bitchord-setup-2.2.0-macos-aarch64.dmg";
  const LINUX_URL = "https://github.com/aaditya079/BitChord-PC/releases/download/v2.2.0/bitchord-2.2.0-linux-x86_64.tar.gz";

  if (isMac) {
    if (heroLabel) heroLabel.textContent = "Download for macOS";
    if (heroBtn) heroBtn.href = MAC_URL;
    if (primaryLabel) primaryLabel.textContent = "Download v2.2.0 for macOS (.dmg)";
    if (primaryBtn) primaryBtn.href = MAC_URL;
    document.getElementById("dl-mac")?.classList.add("featured");
  } else if (isLinux) {
    if (heroLabel) heroLabel.textContent = "Download for Linux";
    if (heroBtn) heroBtn.href = LINUX_URL;
    if (primaryLabel) primaryLabel.textContent = "Download v2.2.0 for Linux (.tar.gz)";
    if (primaryBtn) primaryBtn.href = LINUX_URL;
    document.getElementById("dl-linux")?.classList.add("featured");
  } else {
    // Windows default
    if (heroLabel) heroLabel.textContent = "Download for Windows";
    if (heroBtn) heroBtn.href = WIN_URL;
    if (primaryLabel) primaryLabel.textContent = "Download v2.2.0 for Windows (.exe)";
    if (primaryBtn) primaryBtn.href = WIN_URL;
    document.getElementById("dl-win")?.classList.add("featured");
  }

  // --- Interactive 10-Band Graphic Equalizer Simulator ---
  const BANDS = [
    { label: "31 Hz", freq: 31.25 },
    { label: "62 Hz", freq: 62.5 },
    { label: "125 Hz", freq: 125.0 },
    { label: "250 Hz", freq: 250.0 },
    { label: "500 Hz", freq: 500.0 },
    { label: "1 kHz", freq: 1000.0 },
    { label: "2 kHz", freq: 2000.0 },
    { label: "4 kHz", freq: 4000.0 },
    { label: "8 kHz", freq: 8000.0 },
    { label: "16 kHz", freq: 16000.0 }
  ];

  const PRESETS = {
    flat: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    bass: [6, 5, 4, 2, 0.5, 0, 0, 0, 0, 0],
    vocal: [-2, -2, -1, 0, 2, 3, 3, 2, 0, -1],
    rock: [4, 3, 2, 0, -1, -1, 1, 2, 3, 4],
    pop: [-1, 1, 2, 3, 2, 0, -1, -1, 1, 1],
    electronic: [5, 4, 1, 0, -2, 0, 1, 2, 4, 5],
    latenight: [5, 4, 2, 0, 0, 0, 0, 1, 3, 4]
  };

  let currentGains = [...PRESETS.flat];

  const slidersContainer = document.getElementById("eq-sliders");
  const canvas = document.getElementById("eq-canvas");
  const ctx = canvas ? canvas.getContext("2d") : null;

  function renderSliders() {
    if (!slidersContainer) return;
    slidersContainer.innerHTML = "";
    BANDS.forEach((band, idx) => {
      const col = document.createElement("div");
      col.className = "eq-col";

      const gainText = document.createElement("span");
      gainText.className = "eq-gain";
      gainText.id = `gain-${idx}`;
      gainText.textContent = `${currentGains[idx] > 0 ? "+" : ""}${currentGains[idx]}dB`;

      const input = document.createElement("input");
      input.type = "range";
      input.className = "eq-slider";
      input.min = "-12";
      input.max = "12";
      input.step = "0.5";
      input.value = currentGains[idx];

      input.addEventListener("input", (e) => {
        const val = parseFloat(e.target.value);
        currentGains[idx] = val;
        gainText.textContent = `${val > 0 ? "+" : ""}${val}dB`;
        document.querySelectorAll(".preset-chip").forEach((c) => c.classList.remove("active"));
        drawCurve();
      });

      const label = document.createElement("span");
      label.className = "eq-label";
      label.textContent = band.label;

      col.appendChild(gainText);
      col.appendChild(input);
      col.appendChild(label);
      slidersContainer.appendChild(col);
    });
  }

  function drawCurve() {
    if (!canvas || !ctx) return;
    const w = canvas.width;
    const h = canvas.height;
    ctx.clearRect(0, 0, w, h);

    const midY = h / 2;
    const maxDb = 12;

    // Grid lines
    ctx.strokeStyle = "rgba(255, 255, 255, 0.08)";
    ctx.lineWidth = 1;
    [maxDb, 0, -maxDb].forEach((db) => {
      const y = midY - (db / maxDb) * (midY - 20);
      ctx.beginPath();
      ctx.moveTo(30, y);
      ctx.lineTo(w - 30, y);
      ctx.stroke();

      ctx.fillStyle = "rgba(255, 255, 255, 0.35)";
      ctx.font = "10px JetBrains Mono, monospace";
      ctx.fillText(`${db > 0 ? "+" : ""}${db} dB`, 5, y + 3);
    });

    // Peaking Biquad response interpolation curve
    ctx.beginPath();
    const points = [];
    const count = 120;
    for (let k = 0; k <= count; k++) {
      const normX = k / count;
      const x = 30 + normX * (w - 60);

      // Estimate total gain using a smooth gaussian biquad approximation across bands
      let totalDb = 0;
      BANDS.forEach((_, bIdx) => {
        const centerNorm = (bIdx + 0.5) / BANDS.length;
        const dist = Math.abs(normX - centerNorm);
        const weight = Math.exp(-Math.pow(dist / 0.12, 2));
        totalDb += currentGains[bIdx] * weight;
      });

      totalDb = Math.max(-12, Math.min(12, totalDb));
      const y = midY - (totalDb / maxDb) * (midY - 20);
      points.push({ x, y });
    }

    // Fill curve
    ctx.lineTo(points[0].x, midY);
    points.forEach((p) => ctx.lineTo(p.x, p.y));
    ctx.lineTo(points[points.length - 1].x, midY);
    ctx.closePath();

    // Flat fill area under curve
    ctx.fillStyle = "rgba(250, 88, 106, 0.12)";
    ctx.fill();

    // Clean solid BitChord Red stroke line (no blurry glow/gradient)
    ctx.beginPath();
    points.forEach((p, idx) => {
      if (idx === 0) ctx.moveTo(p.x, p.y);
      else ctx.lineTo(p.x, p.y);
    });
    ctx.strokeStyle = "#fa586a";
    ctx.lineWidth = 2.0;
    ctx.stroke();
  }

  function applyPreset(name) {
    if (!PRESETS[name]) return;
    currentGains = [...PRESETS[name]];
    document.querySelectorAll(".eq-slider").forEach((s, idx) => {
      s.value = currentGains[idx];
      const g = document.getElementById(`gain-${idx}`);
      if (g) g.textContent = `${currentGains[idx] > 0 ? "+" : ""}${currentGains[idx]}dB`;
    });
    drawCurve();
  }

  // Preset button listeners
  document.querySelectorAll(".preset-chip").forEach((btn) => {
    btn.addEventListener("click", () => {
      document.querySelectorAll(".preset-chip").forEach((c) => c.classList.remove("active"));
      btn.classList.add("active");
      applyPreset(btn.dataset.preset);
    });
  });

  // Init EQ
  renderSliders();
  drawCurve();
})();
