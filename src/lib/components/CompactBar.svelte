<script>
    import { onMount, tick } from 'svelte';
    import { system } from '../stores/system.js';
    import { formatSpeed } from '../utils/formatting.js';
    import { invoke } from '@tauri-apps/api/core';
    import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
    import { LogicalSize } from '@tauri-apps/api/dpi';

    let s = $derived($system);

    let cpu = $derived(s.cpu.usage);
    let cpuTemp = $derived(s.cpu.temp_c);
    let ramPct = $derived(s.mem.total ? (s.mem.used / s.mem.total * 100) : 0);
    /** @type {any} */
    let gpu = $derived(s.gpu.gpus[0]);
    let gpuTemp = $derived(gpu?.temp_c ?? null);
    let ip = $derived(s.net_info.local_ip || '—');

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

    // Auto-fit window width to content. Runs on mount and whenever the content
    // changes (CPU temp may pop in/out, IP may go from -- to value, etc).
    async function fitWindow() {
        if (!barEl) return;
        await tick();
        // measure content
        const w = Math.ceil(barEl.scrollWidth) + 4; // small breathing room
        const h = Math.ceil(barEl.scrollHeight);
        try {
            const win = getCurrentWebviewWindow();
            await win.setSize(new LogicalSize(w, h));
        } catch (e) {
            // setSize may fail if capability missing — just ignore
        }
    }

    onMount(() => {
        fitWindow();
    });

    // Re-fit when content visibly changes
    $effect(() => {
        void cpuTemp; void ip; void gpu?.name;
        fitWindow();
    });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="bar" data-tauri-drag-region onmouseup={scheduleSave} bind:this={barEl}>
    <span class="seg cpu {cls(cpu, 80, 95)}" data-tauri-drag-region>
        CPU {cpu.toFixed(0)}%{#if cpuTemp != null}<span class="temp {cls(cpuTemp, 80, 90)}">·{cpuTemp.toFixed(0)}°</span>{/if}
    </span>
    <span class="sep" data-tauri-drag-region>│</span>
    <span class="seg {cls(ramPct, 85, 95)}" data-tauri-drag-region>RAM {ramPct.toFixed(0)}%</span>
    <span class="sep" data-tauri-drag-region>│</span>
    <span class="seg" data-tauri-drag-region>
        GPU {gpu?.load_pct != null ? gpu.load_pct.toFixed(0) + '%' : '--'}{#if gpuTemp != null}<span class="temp {cls(gpuTemp, 80, 90)}">·{gpuTemp.toFixed(0)}°</span>{/if}
    </span>
    <span class="sep" data-tauri-drag-region>│</span>
    <span class="seg net" data-tauri-drag-region>↓{formatSpeed(s.speed.download_bps)}</span>
    <span class="seg net" data-tauri-drag-region>↑{formatSpeed(s.speed.upload_bps)}</span>
    <span class="sep" data-tauri-drag-region>│</span>
    <span class="seg ip" data-tauri-drag-region>{ip}</span>
    <button class="expand" onclick={expand} title="Open full window">▣</button>
</div>

<style>
    :global(html), :global(body) {
        margin: 0;
        padding: 0;
        height: 100%;
        background: transparent;
        overflow: hidden;
    }

    .bar {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        height: 26px;
        padding: 1px 6px 1px 8px;
        font-family: var(--font-mono, 'JetBrains Mono', 'Consolas', monospace);
        font-size: 11px;
        line-height: 1;
        color: var(--text-green, #00ff41);
        background: var(--bg-primary, #0d1117);
        border: 1px solid var(--border-green, #00ff41);
        border-radius: 4px;
        user-select: none;
        white-space: nowrap;
        width: fit-content;
        box-shadow: 0 0 8px rgba(0, 255, 65, 0.18);
    }

    .sep {
        color: var(--text-green, #00ff41);
        opacity: 0.35;
        padding: 0 1px;
    }

    .seg {
        white-space: nowrap;
        letter-spacing: 0.3px;
    }

    .seg.warn, .temp.warn {
        color: var(--text-orange, #ff6600);
    }

    .seg.crit, .temp.crit {
        color: var(--text-red, #ff0040);
    }

    .temp {
        margin-left: 2px;
        opacity: 0.9;
        font-size: 10px;
    }

    .ip {
        font-size: 10px;
        opacity: 0.85;
    }

    .net {
        min-width: 0;
    }

    .expand {
        margin-left: 4px;
        background: none;
        border: 1px solid var(--border-green, #00ff41);
        color: var(--text-green, #00ff41);
        cursor: pointer;
        padding: 0 4px;
        font: inherit;
        font-size: 11px;
        height: 18px;
        line-height: 16px;
        border-radius: 3px;
    }

    .expand:hover {
        background: rgba(0, 255, 65, 0.12);
    }
</style>
