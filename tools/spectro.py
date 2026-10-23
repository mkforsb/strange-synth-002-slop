"""Render spectrogram + waveform PNGs for WAV files (dev aid; needs numpy/matplotlib).

usage: python tools/spectro.py out.png a.wav [b.wav ...]
"""
import sys, wave
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

out, files = sys.argv[1], sys.argv[2:]
fig, axes = plt.subplots(len(files), 2, figsize=(14, 2.3 * len(files)), squeeze=False,
                         gridspec_kw={"width_ratios": [3, 1]})
for row, path in zip(axes, files):
    with wave.open(path) as w:
        sr = w.getframerate()
        x = np.frombuffer(w.readframes(w.getnframes()), dtype=np.int16).reshape(-1, 2).mean(axis=1) / 32768
    n, hop = 2048, 256
    win = np.hanning(n)
    frames = np.array([x[i:i + n] * win for i in range(0, len(x) - n, hop)])
    S = 20 * np.log10(np.abs(np.fft.rfft(frames, axis=1)).T + 1e-7)
    t = np.arange(S.shape[1]) * hop / sr
    f = np.fft.rfftfreq(n, 1 / sr)
    ax = row[0]
    ax.pcolormesh(t, f, S, vmin=S.max() - 90, vmax=S.max(), shading="auto", cmap="magma")
    ax.set_yscale("symlog", linthresh=200); ax.set_ylim(30, 20000)
    ax.set_title(path.split("/")[-1], fontsize=8, loc="left")
    ax.tick_params(labelsize=6)
    # zoomed waveform at the loudest point
    i = int(np.argmax(np.abs(x)))
    seg = x[max(0, i - 1200):i + 1200]
    row[1].plot(seg, lw=0.5, color="orange"); row[1].set_ylim(-1, 1); row[1].tick_params(labelsize=6)
plt.tight_layout(); plt.savefig(out, dpi=70)
