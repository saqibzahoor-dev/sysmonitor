<script>
    import { system } from '../stores/system.js';
    import { formatSpeed } from '../utils/formatting.js';
    import { invoke } from '@tauri-apps/api/core';

    let s = $derived($system);

    let cpu = $derived(s.cpu.usage);
    let ramPct = $derived(s.mem.total ? (s.mem.used / s.mem.total * 100) : 0);
    let gpu = $derived(s.gpu.gpus[0]);
    let cpuTemp = $derived(null); // placeholder until CPU temp wired into store
    let gpuTemp = $derived(gpu?.temp_c ?? null);

    /** @param {number|null|undefined} value @param {number} warn @param {number} crit */
    function cls(value, warn, crit) {
        if (value == null) return '';
        if (value >= crit) return 'crit';
        if (value >= warn) return 'warn';
        return '';
    }

    async function expand() {
        try {
            await invoke('set_display_mode', { mode: 'full' });
        } catch (e) {
            console.error('set_display_mode failed', e);
        }
    }
</script>

<div class="bar">
    <span class="seg {cls(cpu, 80, 95)}">CPU {cpu.toFixed(0)}%
        {#if cpuTemp != null}<span class="temp {cls(cpuTemp, 80, 90)}">{cpuTemp.toFixed(0)}°C</span>{/if}
    </span>
    <span class="sep">│</span>
    <span class="seg {cls(ramPct, 85, 95)}">RAM {ramPct.toFixed(0)}%</span>
    <span class="sep">│</span>
    <span class="seg">GPU {gpu?.load_pct != null ? gpu.load_pct.toFixed(0) + '%' : '--'}
        {#if gpuTemp != null}<span class="temp {cls(gpuTemp, 80, 90)}">{gpuTemp.toFixed(0)}°C</span>{/if}
    </span>
    <span class="sep">│</span>
    <span class="seg">↓ {formatSpeed(s.speed.download_bps)} ↑ {formatSpeed(s.speed.upload_bps)}</span>
    <button class="expand" onclick={expand} title="Open full window">▣</button>
</div>

<style>
    :global(html), :global(body) {
        margin: 0;
        padding: 0;
        height: 100%;
        background: var(--bg-primary, #0d1117);
        overflow: hidden;
    }

    .bar {
        display: flex;
        align-items: center;
        gap: 6px;
        height: 30px;
        padding: 0 8px;
        font-family: var(--font-mono, 'JetBrains Mono', monospace);
        font-size: 11px;
        color: var(--text-green, #00ff41);
        background: var(--bg-primary, #0d1117);
        border-top: 1px solid var(--border-green, #00ff41);
        border-bottom: 1px solid var(--border-green, #00ff41);
        user-select: none;
    }

    .sep {
        color: var(--text-green, #00ff41);
        opacity: 0.35;
    }

    .seg {
        white-space: nowrap;
    }

    .seg.warn, .temp.warn {
        color: var(--text-orange, #ff6600);
    }

    .seg.crit, .temp.crit {
        color: var(--text-red, #ff0040);
    }

    .temp {
        margin-left: 4px;
        opacity: 0.85;
    }

    .expand {
        margin-left: auto;
        background: none;
        border: 1px solid var(--border-green, #00ff41);
        color: var(--text-green, #00ff41);
        cursor: pointer;
        padding: 0 6px;
        font: inherit;
        line-height: 18px;
    }

    .expand:hover {
        background: rgba(0, 255, 65, 0.1);
    }
</style>
