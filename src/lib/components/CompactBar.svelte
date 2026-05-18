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

    // Total disk I/O across drives (sum of read+write bps)
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
        if (d > 0) return `${d}d${h}h`;
        if (h > 0) return `${h}h${m}m`;
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

    // ============ window auto-fit ============
    /** @type {HTMLDivElement|undefined} */
    let barEl;
    const MIN_W = 200;
    const MIN_H = 24;

    async function fitWindow() {
        if (!barEl) return;
        await tick();
        const w = Math.ceil(barEl.scrollWidth) + 6;
        const h = Math.ceil(barEl.scrollHeight);
        if (!Number.isFinite(w) || !Number.isFinite(h) || w < MIN_W || h < MIN_H) return;
        try {
            const win = getCurrentWebviewWindow();
            await win.setSize(new LogicalSize(w, h));
        } catch (e) {}
    }

    onMount(() => {
        requestAnimationFrame(() => fitWindow());
        setTimeout(() => fitWindow(), 400);
    });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="bar" data-tauri-drag-region onmouseup={scheduleSave} bind:this={barEl}>
    <span class="chip {tone(cpu, 80, 95)}" data-tauri-drag-region>
        <span class="icon">▮</span>
        <span class="v">{cpu.toFixed(0)}<span class="unit">%</span></span>
        {#if cpuTemp != null}
            <span class="sub {tone(cpuTemp, 80, 90)}">{cpuTemp.toFixed(0)}°</span>
        {/if}
    </span>

    <span class="chip {tone(ramPct, 85, 95)}" data-tauri-drag-region>
        <span class="icon">▤</span>
        <span class="v">{ramPct.toFixed(0)}<span class="unit">%</span></span>
        <span class="sub">{ramUsedGb}G</span>
    </span>

    <span class="chip" data-tauri-drag-region>
        <span class="icon">◈</span>
        <span class="v">{gpuLoad != null ? gpuLoad.toFixed(0) : '—'}<span class="unit">%</span></span>
        {#if gpuTemp != null}
            <span class="sub {tone(gpuTemp, 80, 90)}">{gpuTemp.toFixed(0)}°</span>
        {/if}
    </span>

    <span class="chip" data-tauri-drag-region>
        <span class="icon">≡</span>
        <span class="v small">{formatBytes(diskBps)}<span class="unit">/s</span></span>
    </span>

    <span class="chip" data-tauri-drag-region>
        <span class="icon down">▼</span>
        <span class="v small">{formatSpeed(s.speed.download_bps)}</span>
        <span class="icon up">▲</span>
        <span class="v small">{formatSpeed(s.speed.upload_bps)}</span>
    </span>

    <span class="chip" data-tauri-drag-region>
        <span class="icon">⌨</span>
        <span class="v small">{procCount}<span class="unit">p</span></span>
    </span>

    <span class="chip" data-tauri-drag-region>
        <span class="icon">⏱</span>
        <span class="v small">{uptimeStr}</span>
    </span>

    <span class="chip ip" data-tauri-drag-region>
        <span class="icon">◉</span>
        <span class="v small ip-text">{ip}</span>
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
        display: inline-flex;
        align-items: center;
        gap: 2px;
        height: 26px;
        min-width: 320px;
        padding: 0 4px;
        font-family: var(--font-mono, 'JetBrains Mono', 'Consolas', monospace);
        font-size: 11px;
        line-height: 1;
        color: var(--text-green, #00ff41);
        background: var(--bg-primary, #0d1117);
        border: 1px solid rgba(0, 255, 65, 0.45);
        border-radius: 6px;
        user-select: none;
        white-space: nowrap;
        width: fit-content;
        box-shadow:
            0 0 10px rgba(0, 255, 65, 0.15),
            inset 0 0 8px rgba(0, 255, 65, 0.04);
    }

    .chip {
        display: inline-flex;
        align-items: center;
        gap: 4px;
        padding: 3px 7px;
        border-radius: 4px;
        background: rgba(0, 255, 65, 0.035);
        white-space: nowrap;
        letter-spacing: 0.2px;
    }

    .chip + .chip {
        margin-left: 2px;
    }

    .icon {
        color: var(--text-cyan, #00d4ff);
        opacity: 0.75;
        font-size: 10px;
        line-height: 1;
    }

    .icon.down {
        color: var(--text-green, #00ff41);
        opacity: 0.9;
    }

    .icon.up {
        color: var(--text-orange, #ff6600);
        opacity: 0.85;
        margin-left: 4px;
    }

    .v {
        color: var(--text-green, #00ff41);
        font-weight: 600;
    }

    .v.small {
        font-size: 10px;
        font-weight: 500;
    }

    .unit {
        color: var(--text-green, #00ff41);
        opacity: 0.55;
        font-size: 9px;
        font-weight: 400;
        margin-left: 1px;
    }

    .sub {
        color: var(--text-cyan, #00d4ff);
        opacity: 0.75;
        font-size: 10px;
        margin-left: 3px;
    }

    /* Warn / crit color overrides */
    .chip.warn .v, .chip.warn .icon { color: var(--text-orange, #ff6600); }
    .chip.warn .unit                { color: var(--text-orange, #ff6600); opacity: 0.7; }
    .chip.crit .v, .chip.crit .icon { color: var(--text-red, #ff0040); }
    .chip.crit .unit                { color: var(--text-red, #ff0040); opacity: 0.7; }
    .sub.warn { color: var(--text-orange, #ff6600); opacity: 0.95; }
    .sub.crit { color: var(--text-red, #ff0040); opacity: 0.95; }

    .ip { background: rgba(0, 212, 255, 0.06); }
    .ip-text { font-size: 10px; color: var(--text-cyan, #00d4ff); }
    .ip .icon { color: var(--text-cyan, #00d4ff); opacity: 0.85; }

    .expand {
        margin-left: 4px;
        background: rgba(0, 255, 65, 0.05);
        border: 1px solid rgba(0, 255, 65, 0.5);
        color: var(--text-green, #00ff41);
        cursor: pointer;
        padding: 0 6px;
        font: inherit;
        font-size: 11px;
        height: 18px;
        line-height: 16px;
        border-radius: 3px;
        transition: background-color 0.15s, color 0.15s;
    }

    .expand:hover {
        background: var(--text-green, #00ff41);
        color: var(--bg-primary, #0d1117);
    }
</style>
