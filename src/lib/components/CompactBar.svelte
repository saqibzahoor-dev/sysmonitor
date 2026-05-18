<script>
    import { onMount, tick } from 'svelte';
    import { system } from '../stores/system.js';
    import { formatSpeed, formatBytes } from '../utils/formatting.js';
    import { invoke } from '@tauri-apps/api/core';
    import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
    import { LogicalSize } from '@tauri-apps/api/dpi';

    let s = $derived($system);

    // ============ derived metrics ============
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

    // ============ position persistence ============
    /** @type {ReturnType<typeof setTimeout>|null} */
    let saveTimer = null;
    function scheduleSave() {
        if (saveTimer) clearTimeout(saveTimer);
        saveTimer = setTimeout(() => {
            invoke('save_compact_position').catch(() => {});
        }, 250);
    }

    // ============ window auto-fit with strict guards ============
    /** @type {HTMLDivElement|undefined} */
    let barEl;
    const MIN_W = 200;
    const MIN_H = 24;
    const MAX_H = 40;  // hard cap so window can NEVER become huge
    let lastW = 0;
    let lastH = 0;

    async function fitWindow() {
        if (!barEl) return;
        await tick();
        const rect = barEl.getBoundingClientRect();
        const w = Math.ceil(rect.width);
        let h = Math.ceil(rect.height);
        // Clamp height — bar is always single-line, ~28px tall.
        if (h < MIN_H) return;
        if (h > MAX_H) h = MAX_H;
        if (w < MIN_W) return;
        // Skip if no meaningful change (avoids jitter)
        if (Math.abs(w - lastW) < 2 && Math.abs(h - lastH) < 2) return;
        lastW = w;
        lastH = h;
        try {
            const win = getCurrentWebviewWindow();
            await win.setSize(new LogicalSize(w, h));
        } catch (e) {}
    }

    onMount(() => {
        // Three fit attempts to handle font loading + layout + Tauri WebView quirks
        requestAnimationFrame(() => fitWindow());
        setTimeout(() => fitWindow(), 200);
        setTimeout(() => fitWindow(), 800);
    });

    // Re-fit when content changes meaningfully (chip text widens/narrows)
    $effect(() => {
        void cpu; void cpuTemp; void ramPct; void gpuLoad; void gpuTemp;
        void procCount; void uptimeStr; void ip;
        requestAnimationFrame(() => fitWindow());
    });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="bar" data-tauri-drag-region onmouseup={scheduleSave} bind:this={barEl}>
    <span class="m {tone(cpu, 80, 95)}" data-tauri-drag-region>
        <span class="ico">▮</span>
        <span class="lbl">CPU</span>
        <span class="val">{cpu.toFixed(0)}<span class="u">%</span></span>
        {#if cpuTemp != null}
            <span class="t {tone(cpuTemp, 80, 90)}">{cpuTemp.toFixed(0)}°</span>
        {/if}
    </span>

    <span class="m {tone(ramPct, 85, 95)}" data-tauri-drag-region>
        <span class="ico">▤</span>
        <span class="lbl">RAM</span>
        <span class="val">{ramPct.toFixed(0)}<span class="u">%</span></span>
        <span class="t">{ramUsedGb}<span class="u">G</span></span>
    </span>

    <span class="m" data-tauri-drag-region>
        <span class="ico">◈</span>
        <span class="lbl">GPU</span>
        <span class="val">{gpuLoad != null ? gpuLoad.toFixed(0) : '—'}<span class="u">%</span></span>
        {#if gpuTemp != null}
            <span class="t {tone(gpuTemp, 80, 90)}">{gpuTemp.toFixed(0)}°</span>
        {/if}
    </span>

    <span class="m" data-tauri-drag-region>
        <span class="ico">≡</span>
        <span class="lbl">DISK</span>
        <span class="val sm">{formatBytes(diskBps)}<span class="u">/s</span></span>
    </span>

    <span class="m" data-tauri-drag-region>
        <span class="ico down">▼</span>
        <span class="lbl">NET</span>
        <span class="val sm">{formatSpeed(s.speed.download_bps)}</span>
        <span class="ico up">▲</span>
        <span class="val sm">{formatSpeed(s.speed.upload_bps)}</span>
    </span>

    <span class="m" data-tauri-drag-region>
        <span class="ico">⌨</span>
        <span class="lbl">PROC</span>
        <span class="val sm">{procCount}</span>
    </span>

    <span class="m" data-tauri-drag-region>
        <span class="ico">⏱</span>
        <span class="lbl">UP</span>
        <span class="val sm">{uptimeStr}</span>
    </span>

    <span class="m" data-tauri-drag-region>
        <span class="ico">◉</span>
        <span class="lbl">IP</span>
        <span class="val sm ip-text">{ip}</span>
    </span>

    <button class="expand" onclick={expand} title="Open full window">▣</button>
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
        display: flex;
        align-items: center;
        justify-content: center;     /* CENTER content — important for AppBar mode */
        gap: 14px;
        height: 100%;                /* fill window height (we control window height) */
        min-height: 26px;
        padding: 0 10px;
        font-family: var(--font-mono, 'JetBrains Mono', 'Consolas', monospace);
        font-size: 11px;
        line-height: 1;
        color: var(--text-green, #00ff41);
        background: var(--bg-primary, #0d1117);
        border-top: 1px solid rgba(0, 255, 65, 0.35);
        border-bottom: 1px solid rgba(0, 255, 65, 0.35);
        user-select: none;
        white-space: nowrap;
        box-sizing: border-box;
    }

    .m {
        display: inline-flex;
        align-items: baseline;
        gap: 4px;
        white-space: nowrap;
        letter-spacing: 0.2px;
    }

    .ico {
        color: var(--text-cyan, #00d4ff);
        opacity: 0.85;
        font-size: 10px;
    }
    .ico.down { color: var(--text-green, #00ff41); opacity: 1; margin-right: 1px; }
    .ico.up   { color: var(--text-orange, #ff6600); opacity: 0.95; margin-left: 5px; margin-right: 1px; }

    .lbl {
        color: var(--text-cyan, #00d4ff);
        font-size: 10px;
        opacity: 0.8;
        letter-spacing: 0.5px;
        margin-right: 1px;
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
        opacity: 0.5;
        font-size: 9px;
        font-weight: 400;
        margin-left: 1px;
    }

    .t {
        color: var(--text-cyan, #00d4ff);
        opacity: 0.85;
        font-size: 10px;
        margin-left: 2px;
    }

    /* Threshold colors — applied to the WHOLE metric */
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

    .expand {
        background: transparent;
        border: 1px solid rgba(0, 255, 65, 0.4);
        color: var(--text-green, #00ff41);
        cursor: pointer;
        padding: 0 6px;
        font: inherit;
        font-size: 11px;
        height: 18px;
        line-height: 16px;
        border-radius: 3px;
        margin-left: 4px;
        transition: background-color 0.15s, color 0.15s;
    }
    .expand:hover {
        background: var(--text-green, #00ff41);
        color: var(--bg-primary, #0d1117);
    }
</style>
