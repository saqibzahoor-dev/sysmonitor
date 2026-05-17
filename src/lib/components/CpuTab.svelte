<script>
    import { system } from '../stores/system.js';

    let s = $derived($system);
    let cpu = $derived(s.cpu);
    let cores = $derived(cpu.per_core);
    let freqGhz = $derived(cpu.freq_mhz ? (cpu.freq_mhz / 1000).toFixed(2) : '--');
    let cpuTemp = $derived(s.gpu.sensor_available ? null : null); // CPU temp not yet wired; shows —

    /** @param {number} pct @param {number} [width] */
    function bar(pct, width = 16) {
        if (!pct || pct < 0) pct = 0;
        if (pct > 100) pct = 100;
        const filled = Math.round(pct / 100 * width);
        return '█'.repeat(filled) + '░'.repeat(width - filled);
    }

    /** @param {number|null|undefined} pct @param {number} [warn] @param {number} [crit] */
    function tone(pct, warn = 80, crit = 95) {
        if (pct == null) return '';
        if (pct >= crit) return 'error';
        if (pct >= warn) return 'warning';
        return '';
    }
</script>

<div class="cpu-tab tab-content">
    <div class="prompt-line">
        <span class="prompt">root@sysmon:~$</span>
        <span class="prompt-cmd"> top --cpu</span>
        <span class="cursor"></span>
    </div>
    <hr class="separator" />

    <div class="row">
        <span class="label">USAGE</span>
        <span class="value glow {tone(cpu.usage)}">{cpu.usage.toFixed(1)}%</span>
    </div>
    <div class="row">
        <span class="label">FREQ</span>
        <span class="value glow">{freqGhz} GHz</span>
    </div>
    <div class="row">
        <span class="label">TEMP</span>
        <span class="value {cpuTemp != null ? tone(cpuTemp, 80, 90) : ''}">{cpuTemp != null ? cpuTemp.toFixed(0) + '°C' : '—'}</span>
    </div>

    <div class="cores-section">
        <div class="cores-label">CORES ({cores.length})</div>
        <div class="cores-grid">
            {#each cores as c, i}
                <div class="core">
                    <span class="cidx">c{String(i).padStart(2, '0')}</span>
                    <span class="cbar {tone(c)}">{bar(c)}</span>
                    <span class="cpct {tone(c)}">{c.toFixed(0)}%</span>
                </div>
            {/each}
        </div>
    </div>
</div>

<style>
    .cpu-tab {
        padding: 8px 12px;
        font-size: 12px;
    }

    .row {
        display: flex;
        justify-content: space-between;
        padding: 2px 0;
    }

    .cores-section {
        margin-top: 6px;
    }

    .cores-label {
        color: var(--text-cyan);
        font-size: 10px;
        margin-bottom: 2px;
        letter-spacing: 1px;
    }

    .cores-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        column-gap: 8px;
        row-gap: 1px;
    }

    .core {
        display: grid;
        grid-template-columns: 26px 1fr 32px;
        gap: 4px;
        font-size: 11px;
        align-items: baseline;
    }

    .cidx {
        color: var(--text-cyan);
        opacity: 0.75;
    }

    .cbar {
        color: var(--text-green);
        letter-spacing: -1px;
        overflow: hidden;
    }

    .cpct {
        color: var(--text-green);
        text-align: right;
    }

    .cbar.warning, .cpct.warning {
        color: var(--text-orange);
    }

    .cbar.error, .cpct.error {
        color: var(--text-red);
    }
</style>
