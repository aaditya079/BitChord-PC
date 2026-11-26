/**
 * BitChord PC — Interactive Experience Script
 * Author: BitChord Project
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

  const WIN_URL = "https://github.com/aaditya079/BitChord-PC/releases/download/v2.2.0/bitchord-setup-2.2.0.exe";
  const MAC_URL = "https://github.com/aaditya079/BitChord-PC/releases/download/v2.2.0/bitchord-setup-2.2.0-macos-aarch64.dmg";
  const LINUX_URL = "https://github.com/aaditya079/BitChord-PC/releases/download/v2.2.0/bitchord-2.2.0-linux-x86_64.tar.gz";

  if (isMac) {
    if (heroLabel) heroLabel.textContent = "Download for macOS";
    if (heroBtn) heroBtn.href = MAC_URL;
    document.getElementById("dl-mac")?.classList.add("featured");
  } else if (isLinux) {
    if (heroLabel) heroLabel.textContent = "Download for Linux";
    if (heroBtn) heroBtn.href = LINUX_URL;
    document.getElementById("dl-linux")?.classList.add("featured");
  } else {
    // Windows default
    if (heroLabel) heroLabel.textContent = "Download for Windows";
    if (heroBtn) heroBtn.href = WIN_URL;
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
        updateEqFilterGain(idx, val);
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
      updateEqFilterGain(idx, currentGains[idx]);
    });
    drawCurve();
  }

  // --- Web Audio API Interactive Sound Preview ---
  let audioCtx = null;
  let eqFilters = [];
  let masterGain = null;
  let audioAnalyser = null;
  let isAudioPlaying = false;
  let activeSources = [];

  function initAudioContext() {
    if (audioCtx) return true;
    const AudioCtx = window.AudioContext || window.webkitAudioContext;
    if (!AudioCtx) return false;
    try {
      audioCtx = new AudioCtx();

      // Build 10 peaking biquad filters matching the 10 bands
      eqFilters = BANDS.map((band, idx) => {
        const f = audioCtx.createBiquadFilter();
        f.type = "peaking";
        f.frequency.setValueAtTime(band.freq, audioCtx.currentTime);
        f.Q.setValueAtTime(1.4, audioCtx.currentTime);
        f.gain.setValueAtTime(currentGains[idx], audioCtx.currentTime);
        return f;
      });

      // Chain filters in series
      for (let i = 0; i < eqFilters.length - 1; i++) {
        eqFilters[i].connect(eqFilters[i + 1]);
      }

      // Analyser for real-time FFT / spectrum
      audioAnalyser = audioCtx.createAnalyser();
      audioAnalyser.fftSize = 64;
      audioAnalyser.smoothingTimeConstant = 0.82;
      eqFilters[eqFilters.length - 1].connect(audioAnalyser);

      // Master Gain
      masterGain = audioCtx.createGain();
      masterGain.gain.setValueAtTime(0, audioCtx.currentTime);
      audioAnalyser.connect(masterGain);
      masterGain.connect(audioCtx.destination);
      return true;
    } catch (err) {
      console.warn("Web Audio API initialization failed:", err);
      return false;
    }
  }

  function updateEqFilterGain(idx, val) {
    if (eqFilters[idx] && audioCtx) {
      eqFilters[idx].gain.setTargetAtTime(val, audioCtx.currentTime, 0.03);
    }
  }

  function startTone() {
    if (!initAudioContext()) return;
    if (audioCtx.state === "suspended") {
      audioCtx.resume();
    }

    stopTone(false);

    // Rich harmonic chord (A2, E3, A3, C4, E4, A5)
    // Spans 100 Hz to 10 kHz so all 10 EQ bands have audible material to shape
    const freqs = [
      { f: 110.0, type: "sawtooth", gain: 0.07 },
      { f: 164.81, type: "triangle", gain: 0.06 },
      { f: 220.0, type: "sawtooth", gain: 0.06 },
      { f: 329.63, type: "triangle", gain: 0.05 },
      { f: 523.25, type: "sawtooth", gain: 0.04 },
      { f: 880.0, type: "sine", gain: 0.03 }
    ];

    const chordGain = audioCtx.createGain();
    chordGain.gain.value = 1.0;

    freqs.forEach((spec) => {
      const osc = audioCtx.createOscillator();
      const g = audioCtx.createGain();
      osc.type = spec.type;
      osc.frequency.setValueAtTime(spec.f, audioCtx.currentTime);
      g.gain.setValueAtTime(spec.gain, audioCtx.currentTime);
      osc.connect(g);
      g.connect(chordGain);
      osc.start();
      activeSources.push(osc);
    });

    // Gentle shaped noise buffer for 4 kHz - 16 kHz brilliance band testing
    const bufferSize = audioCtx.sampleRate * 2;
    const noiseBuffer = audioCtx.createBuffer(1, bufferSize, audioCtx.sampleRate);
    const output = noiseBuffer.getChannelData(0);
    for (let i = 0; i < bufferSize; i++) {
      output[i] = (Math.random() * 2 - 1) * 0.012;
    }
    const noiseSource = audioCtx.createBufferSource();
    noiseSource.buffer = noiseBuffer;
    noiseSource.loop = true;

    const noiseFilter = audioCtx.createBiquadFilter();
    noiseFilter.type = "highpass";
    noiseFilter.frequency.setValueAtTime(3200, audioCtx.currentTime);

    noiseSource.connect(noiseFilter);
    noiseFilter.connect(chordGain);
    noiseSource.start();
    activeSources.push(noiseSource);

    // Connect sound source into first EQ band
    chordGain.connect(eqFilters[0]);

    // Fade in master volume comfortably
    masterGain.gain.cancelScheduledValues(audioCtx.currentTime);
    masterGain.gain.setValueAtTime(masterGain.gain.value, audioCtx.currentTime);
    masterGain.gain.setTargetAtTime(0.12, audioCtx.currentTime, 0.05);

    isAudioPlaying = true;
    const eqBtn = document.getElementById("eq-audio-btn");
    const eqLabel = document.getElementById("eq-audio-label");
    if (eqBtn) eqBtn.classList.add("playing");
    if (eqLabel) eqLabel.textContent = "Stop Tone";
  }

  function stopTone(updateUI = true) {
    if (masterGain && audioCtx) {
      masterGain.gain.cancelScheduledValues(audioCtx.currentTime);
      masterGain.gain.setValueAtTime(masterGain.gain.value, audioCtx.currentTime);
      masterGain.gain.setTargetAtTime(0, audioCtx.currentTime, 0.03);
    }
    setTimeout(() => {
      activeSources.forEach((src) => {
        try {
          src.stop();
          src.disconnect();
        } catch (_) {}
      });
      activeSources = [];
    }, 60);

    isAudioPlaying = false;
    if (updateUI) {
      const eqBtn = document.getElementById("eq-audio-btn");
      const eqLabel = document.getElementById("eq-audio-label");
      if (eqBtn) eqBtn.classList.remove("playing");
      if (eqLabel) eqLabel.textContent = "Preview Tone";
    }
  }

  const eqAudioBtn = document.getElementById("eq-audio-btn");
  if (eqAudioBtn) {
    eqAudioBtn.addEventListener("click", () => {
      if (isAudioPlaying) {
        stopTone(true);
      } else {
        startTone();
      }
    });
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

  // --- Hero Titlebar Spectrum Visualizer ---
  const spectrumCanvas = document.getElementById("hero-spectrum");
  const specCtx = spectrumCanvas ? spectrumCanvas.getContext("2d") : null;
  const BARS_COUNT = 16;
  const BAR_WIDTH = 4;
  const BAR_GAP = 2;
  const barsData = new Float32Array(BARS_COUNT).fill(2);
  let spectrumAnimationId = null;
  let isSpectrumVisible = true;

  function renderSpectrum(timestamp) {
    if (!spectrumCanvas || !specCtx) return;
    const w = spectrumCanvas.width;
    const h = spectrumCanvas.height;
    specCtx.clearRect(0, 0, w, h);

    if (isAudioPlaying && audioAnalyser) {
      const freqData = new Uint8Array(audioAnalyser.frequencyBinCount);
      audioAnalyser.getByteFrequencyData(freqData);
      for (let i = 0; i < BARS_COUNT; i++) {
        const binIdx = Math.floor(Math.pow(i / BARS_COUNT, 1.4) * (freqData.length - 1));
        const targetHeight = Math.max(2, (freqData[binIdx] / 255) * h);
        barsData[i] += (targetHeight - barsData[i]) * 0.35;
      }
    } else {
      // Ambient live music simulation
      const t = timestamp * 0.003;
      for (let i = 0; i < BARS_COUNT; i++) {
        const beat = Math.sin(t * 2.2 + i * 0.4) * 0.5 + 0.5;
        const noise = Math.sin(t * 5.7 - i * 0.9) * 0.3 + 0.5;
        const rollOff = 1 - (i / BARS_COUNT) * 0.4;
        const target = Math.max(2, (beat * 0.6 + noise * 0.4) * h * rollOff);
        barsData[i] += (target - barsData[i]) * 0.18;
      }
    }

    for (let i = 0; i < BARS_COUNT; i++) {
      const x = i * (BAR_WIDTH + BAR_GAP);
      const barH = Math.min(h, Math.max(2, barsData[i]));
      const y = h - barH;

      specCtx.fillStyle = i % 3 === 0 ? "#fa586a" : "rgba(250, 88, 106, 0.75)";
      specCtx.fillRect(x, y, BAR_WIDTH, barH);
    }

    if (isSpectrumVisible) {
      spectrumAnimationId = requestAnimationFrame(renderSpectrum);
    }
  }

  // Pause animation when hero preview is offscreen (zero CPU waste)
  const heroWrapper = document.getElementById("hero-preview-wrapper");
  if (spectrumCanvas && "IntersectionObserver" in window) {
    const specObserver = new IntersectionObserver(
      ([entry]) => {
        isSpectrumVisible = entry.isIntersecting;
        if (isSpectrumVisible) {
          cancelAnimationFrame(spectrumAnimationId);
          spectrumAnimationId = requestAnimationFrame(renderSpectrum);
        }
      },
      { threshold: 0.05 }
    );
    specObserver.observe(heroWrapper || spectrumCanvas);
  } else if (spectrumCanvas) {
    spectrumAnimationId = requestAnimationFrame(renderSpectrum);
  }

  // --- Hero Window 3D Perspective Tilt & Specular Shine ---
  const previewFrame = document.getElementById("hero-preview-frame");
  const previewWrapper = document.getElementById("hero-preview-wrapper");

  if (previewWrapper && previewFrame) {
    let curRotX = 0;
    let curRotY = 0;
    let targetRotX = 0;
    let targetRotY = 0;
    let tiltRafId = null;

    const updateTilt = () => {
      curRotX += (targetRotX - curRotX) * 0.12;
      curRotY += (targetRotY - curRotY) * 0.12;
      previewFrame.style.transform = `perspective(1200px) rotateX(${curRotX.toFixed(2)}deg) rotateY(${curRotY.toFixed(2)}deg) translateY(-2px)`;

      if (Math.abs(curRotX - targetRotX) > 0.02 || Math.abs(curRotY - targetRotY) > 0.02) {
        tiltRafId = requestAnimationFrame(updateTilt);
      } else {
        tiltRafId = null;
      }
    };

    previewWrapper.addEventListener("mousemove", (e) => {
      const rect = previewWrapper.getBoundingClientRect();
      const normX = (e.clientX - rect.left) / rect.width;
      const normY = (e.clientY - rect.top) / rect.height;

      targetRotX = (normY - 0.5) * -6.0;
      targetRotY = (normX - 0.5) * 6.0;

      previewWrapper.style.setProperty("--shine-x", `${(normX * 100).toFixed(1)}%`);
      previewWrapper.style.setProperty("--shine-y", `${(normY * 100).toFixed(1)}%`);

      if (!tiltRafId) tiltRafId = requestAnimationFrame(updateTilt);
    });

    previewWrapper.addEventListener("mouseleave", () => {
      targetRotX = 0;
      targetRotY = 0;
      if (!tiltRafId) tiltRafId = requestAnimationFrame(updateTilt);
    });
  }

  // --- Card Cursor Spotlight Effect ---
  const spotlightCards = document.querySelectorAll(".bento-card, .platform-btn");
  spotlightCards.forEach((card) => {
    card.addEventListener("mousemove", (e) => {
      const rect = card.getBoundingClientRect();
      card.style.setProperty("--mouse-x", `${(e.clientX - rect.left).toFixed(1)}px`);
      card.style.setProperty("--mouse-y", `${(e.clientY - rect.top).toFixed(1)}px`);
    });
    card.addEventListener("mouseleave", () => {
      card.style.setProperty("--mouse-x", "-500px");
      card.style.setProperty("--mouse-y", "-500px");
    });
  });

  // --- Scroll Reveal Transitions ---
  const revealElements = document.querySelectorAll(".reveal");
  if (revealElements.length > 0 && "IntersectionObserver" in window) {
    const revealObserver = new IntersectionObserver(
      (entries, observer) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add("revealed");
            observer.unobserve(entry.target);
          }
        });
      },
      { threshold: 0.1, rootMargin: "0px 0px -40px 0px" }
    );
    revealElements.forEach((el) => revealObserver.observe(el));
  } else {
    revealElements.forEach((el) => el.classList.add("revealed"));
  }
})();
