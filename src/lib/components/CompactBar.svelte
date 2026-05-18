<script>
    import { onMount, tick } from 'svelte';
    import { system } from '../stores/system.js';
    import { formatSpeed, formatBytes } from '../utils/formatting.js';
    import { invoke } from '@tauri-apps/api/core';
    import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
    import { LogicalSize } from '@tauri-apps/api/dpi';

    let s = $derived($system);

    let cpu = $derived(s.cpu.usage);
    let cpuTemp = $derived(s.cpu.temp_c);
    let ramPct = $derived(s.mem.total ? (s.mem.used / s.mem.total * 100) : 0);
    let ramUsedGb = $derived((s.mem.used / 1024 ** 3).toFixed(1));
    /** @type {any} */
    let gpu = $derived(s.gpu.gpus[0]);
    let gpuTemp = $derived(gpu?.temp_c ?? null);
    let gpuLoad = $derived(gpu?.load_pct ?? null);
    let diskBps = $derived(
        (s.disk?.disks ?? []).reduce((a, d) => a + d.read_bps + d.write_bps, 0)
    );
    let procCount = $derived(s.proc?.count ?? 0);
    let uptimeStr = $derived(formatUptime(s.proc?.uptime_secs ?? 0));
    let ip = $derived(s.net_info.local_ip || '—');

    /** @param {number} secs */
    function formatUptime(secs) {
        if (!secs) return '—';
        const d = Math.floor(secs / 86400);
        const h = Math.floor((secs % 86400) / 3600);
        const m = Math.floor((secs % 3600) / 60);
        if (d > 0) return `${d}d ${h}h`;
        if (h > 0) return `${h}h ${m}m`;
        return `${m}m`;
    }

    /** @param {number|null|undefined} v @param {number} warn @param {number} crit */
    function tone(v, warn, crit) {
        if (v == null) return '';
        if (v >= crit) return 'crit';
        if (v >= warn) return 'warn';
        return '';
    }

    async function expand() {
        try { await invoke('set_display_mode', { mode: 'full' }); } catch (e) {}
    }

    /** @type {ReturnType<typeof setTimeout>|null} */
    let saveTimer = null;
    function scheduleSave() {
        if (saveTimer) clearTimeout(saveTimer);
        saveTimer = setTimeout(() => {
            invoke('save_compact_position').catch(() => {});
        }, 250);
    }

    /** @type {HTMLDivElement|undefined} */
    let barEl;
    const MIN_W = 200;
    const MIN_H = 24;
    const MAX_H = 40;
    let lastW = 0;
    let lastH = 0;

    async function fitWindow() {
        if (!barEl) return;
        await tick();
        // Use offsetWidth + the bar's actual outer rect for accurate measurement
        const rect = barEl.getBoundingClientRect();
        const w = Math.ceil(rect.width);
        let h = Math.ceil(rect.height);
        if (h < MIN_H) return;
        if (h > MAX_H) h = MAX_H;
        if (w < MIN_W) return;
        if (Math.abs(w - lastW) < 2 && Math.abs(h - lastH) < 2) return;
        lastW = w;
        lastH = h;
        try {
            const win = getCurrentWebviewWindow();
            await win.setSize(new LogicalSize(w, h));
        } catch (e) {}
    }

    onMount(() => {
        requestAnimationFrame(() => fitWindow());
        setTimeout(() => fitWindow(), 200);
        setTimeout(() => fitWindow(), 800);
    });

    $effect(() => {
        void cpu; void cpuTemp; void ramPct; void gpuLoad; void gpuTemp;
        void procCount; void uptimeStr; void ip;
        requestAnimationFrame(() => fitWindow());
    });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="bar" data-tauri-drag-region onmouseup={scheduleSave} bind:this={barEl}>
    <span class="m {tone(cpu, 80, 95)}" data-tauri-drag-region>
        <span class="ico">▮</span><span class="lbl">CPU</span>
        <span class="val">{cpu.toFixed(0)}<span class="u">%</span></span>
        {#if cpuTemp != null}
            <span class="t {tone(cpuTemp, 80, 90)}">{cpuTemp.toFixed(0)}°</span>
        {/if}
    </span>

    <span class="m {tone(ramPct, 85, 95)}" data-tauri-drag-region>
        <span class="ico">▤</span><span class="lbl">RAM</span>
        <span class="val">{ramPct.toFixed(0)}<span class="u">%</span></span>
        <span class="t">{ramUsedGb}<span class="u">G</span></span>
    </span>

    <span class="m" data-tauri-drag-region>
        <span class="ico">◈</span><span class="lbl">GPU</span>
        <span class="val">{gpuLoad != null ? gpuLoad.toFixed(0) : '—'}<span class="u">%</span></span>
        {#if gpuTemp != null}
            <span class="t {tone(gpuTemp, 80, 90)}">{gpuTemp.toFixed(0)}°</span>
        {/if}
    </span>

    <span class="m" data-tauri-drag-region>
        <span class="ico">≡</span><span class="lbl">DISK</span>
        <span class="val sm">{formatBytes(diskBps)}<span class="u">/s</span></span>
    </span>

    <span class="m" data-tauri-drag-region>
        <span class="ico down">▼</span><span class="lbl">NET</span>
        <span class="val sm">{formatSpeed(s.speed.download_bps)}</span>
        <span class="ico up">▲</span>
        <span class="val sm">{formatSpeed(s.speed.upload_bps)}</span>
    </span>

    <span class="m" data-tauri-drag-region>
        <span class="ico">⌨</span><span class="lbl">PROC</span>
        <span class="val sm">{procCount}</span>
    </span>

    <span class="m" data-tauri-drag-region>
        <span class="ico">⏱</span><span class="lbl">UP</span>
        <span class="val sm">{uptimeStr}</span>
    </span>

    <span class="m" data-tauri-drag-region>
        <span class="ico">◉</span><span class="lbl">IP</span>
        <span class="val sm ip-text">{ip}</span>
    </span>

    <button class="expand" onclick={expand} title="Open full window" aria-label="Open full window">
        <svg viewBox="0 0 12 12" width="11" height="11" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="square">
            <rect x="1.5" y="1.5" width="9" height="9"></rect>
        </svg>
    </button>
</div>

<style>
    :global(html), :global(body) {
        margin: 0;
        padding: 0;
        height: 100%;
        width: 100%;
        background: var(--bg-primary, #0d1117);
        overflow: hidden;
    }

    .bar {
        /* inline-flex so width shrinks to content; window setSize matches */
        display: inline-flex;
        align-items: center;       /* vertical centering for ALL children incl. button */
        gap: 0;
        height: 100%;
        min-height: 26px;
        padding: 0;
        font-family: var(--font-mono, 'JetBrains Mono', 'Cascadia Code', 'Consolas', monospace);
        font-size: 11px;
        line-height: 1;
        color: var(--text-green, #00ff41);
        background: var(--bg-primary, #0d1117);
        border-top: 1px solid rgba(0, 255, 65, 0.35);
        border-bottom: 1px solid rgba(0, 255, 65, 0.35);
        border-radius: 0;          /* no corners, sharp edges */
        user-select: none;
        white-space: nowrap;
        box-sizing: border-box;
        /* tabular numbers so percentages don't shift width when digits change */
        font-variant-numeric: tabular-nums;
    }

    .m {
        display: inline-flex;
        align-items: center;
        gap: 4px;
        padding: 0 12px;
        white-space: nowrap;
        letter-spacing: 0.2px;
        height: 100%;
        position: relative;
    }

    /* Thin vertical divider between metrics — 1px hairline in dim green */
    .m + .m::before {
        content: "";
        position: absolute;
        left: 0;
        top: 18%;
        bottom: 18%;
        width: 1px;
        background: rgba(0, 255, 65, 0.18);
    }

    .ico {
        color: var(--text-cyan, #00d4ff);
        opacity: 0.7;
        font-size: 10px;
        display: inline-block;
        width: 10px;
        text-align: center;
        line-height: 1;
    }
    .ico.down { color: var(--text-green, #00ff41); opacity: 0.95; margin-right: 1px; }
    .ico.up   { color: var(--text-orange, #ff6600); opacity: 0.95; margin-left: 6px; margin-right: 1px; }

    .lbl {
        color: var(--text-cyan, #00d4ff);
        font-size: 9.5px;
        opacity: 0.78;
        letter-spacing: 0.7px;
        text-transform: uppercase;
        font-weight: 500;
        margin-right: 2px;
    }

    .val {
        color: var(--text-green, #00ff41);
        font-weight: 600;
        font-size: 11.5px;
    }
    .val.sm {
        font-size: 10.5px;
        font-weight: 500;
    }

    .u {
        color: var(--text-green, #00ff41);
        opacity: 0.45;
        font-size: 8.5px;
        font-weight: 400;
        margin-left: 1px;
    }

    .t {
        color: var(--text-cyan, #00d4ff);
        opacity: 0.85;
        font-size: 10px;
        margin-left: 3px;
    }

    /* Threshold colors */
    .m.warn .val, .m.warn .ico, .m.warn .lbl, .m.warn .u { color: var(--text-orange, #ff6600); }
    .m.warn .lbl { opacity: 1; }
    .m.crit .val, .m.crit .ico, .m.crit .lbl, .m.crit .u { color: var(--text-red, #ff0040); }
    .m.crit .lbl { opacity: 1; }
    .t.warn { color: var(--text-orange, #ff6600); opacity: 1; }
    .t.crit { color: var(--text-red, #ff0040); opacity: 1; }

    .ip-text {
        color: var(--text-cyan, #00d4ff);
        opacity: 0.95;
    }

    /* Expand button — square, vertically centered, hairline border */
    .expand {
        align-self: center;          /* centers vertically in bar */
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 18px;
        height: 18px;
        margin: 0 8px 0 4px;
        padding: 0;
        background: transparent;
        border: 1px solid rgba(0, 255, 65, 0.45);
        color: var(--text-green, #00ff41);
        cursor: pointer;
        border-radius: 0;
        transition: background-color 0.12s ease, color 0.12s ease, border-color 0.12s ease;
    }
    .expand:hover {
        background: var(--text-green, #00ff41);
        color: var(--bg-primary, #0d1117);
        border-color: var(--text-green, #00ff41);
    }
    .expand svg {
        display: block;
    }
</style>
