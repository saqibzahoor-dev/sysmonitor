<script>
    import { system } from '../stores/system.js';

    let s = $derived($system);
    let gpu = $derived(s.gpu);

    /** @param {number|null|undefined} pct @param {number} [width] */
    function bar(pct, width = 22) {
        if (!pct || pct < 0) pct = 0;
        if (pct > 100) pct = 100;
        const filled = Math.round(pct / 100 * width);
        return '█'.repeat(filled) + '░'.repeat(width - filled);
    }

    /** @param {number|null|undefined} t @param {number} [warn] @param {number} [crit] */
    function tempTone(t, warn = 80, crit = 90) {
        if (t == null) return '';
        if (t >= crit) return 'error';
        if (t >= warn) return 'warning';
        return '';
    }

    /** @param {number|null|undefined} v */
    function mb(v) {
        if (v == null) return '—';
        return v >= 1024 ? `${(v / 1024).toFixed(1)} GB` : `${v} MB`;
    }
</script>

<div class="gpu-tab tab-content">
    <div class="prompt-line">
        <span class="prompt">root@sysmon:~$</span>
        <span class="prompt-cmd"> gpu --sensors</span>
        <span class="cursor"></span>
    </div>
    <hr class="separator" />

    {#if !gpu.sensor_available}
        <div class="alert">
            sensor sidecar unavailable<br>
            right-click tray → Retry sensors
        </div>
    {:else if gpu.gpus.length === 0}
        <div class="empty">no GPU detected by sidecar</div>
    {:else}
        {#each gpu.gpus as g}
            <div class="card">
                <div class="gname">{g.name}</div>
                <div class="row">
                    <span class="label">LOAD</span>
                    <span class="value glow">{g.load_pct != null ? g.load_pct.toFixed(0) + '%' : '—'}</span>
                </div>
                <div class="bar">{bar(g.load_pct ?? 0)}</div>
                <div class="row">
                    <span class="label">TEMP</span>
                    <span class="value glow {tempTone(g.temp_c)}">{g.temp_c != null ? g.temp_c.toFixed(0) + '°C' : '—'}</span>
                </div>
                <div class="row">
                    <span class="label">VRAM</span>
                    <span class="value">{mb(g.vram_used_mb)} / {mb(g.vram_total_mb)}</span>
                </div>
                <div class="bar">{bar(g.vram_total_mb ? (g.vram_used_mb / g.vram_total_mb * 100) : 0)}</div>
            </div>
        {/each}
    {/if}
</div>

<style>
    .gpu-tab {
        padding: 8px 12px;
        font-size: 12px;
    }

    .alert {
        color: var(--text-orange);
        text-align: center;
        padding: 20px 8px;
        line-height: 1.4;
        font-size: 11px;
    }

    .empty {
        color: var(--text-dim);
        text-align: center;
        padding: 20px 8px;
        font-style: italic;
    }

    .card {
        margin-bottom: 6px;
    }

    .gname {
        color: var(--text-cyan);
        font-size: 11px;
        margin-bottom: 2px;
        letter-spacing: 1px;
    }

    .row {
        display: flex;
        justify-content: space-between;
        font-size: 11px;
        padding: 1px 0;
    }

    .bar {
        color: var(--text-green);
        letter-spacing: -1px;
        font-size: 13px;
        line-height: 1;
        margin: 1px 0 3px;
    }

    .value.warning {
        color: var(--text-orange);
    }

    .value.error {
        color: var(--text-red);
    }
</style>
