<script>
    import { system } from '../stores/system.js';
    import { formatSpeed } from '../utils/formatting.js';

    let s = $derived($system);
    let cpu = $derived(s.cpu.usage);
    let ramPct = $derived(s.mem.total ? (s.mem.used / s.mem.total * 100) : 0);
    let gpu = $derived(s.gpu.gpus[0]);
    let gpuTemp = $derived(gpu?.temp_c ?? null);

    function cls(value, warn, crit) {
        if (value >= crit) return 'crit';
        if (value >= warn) return 'warn';
        return '';
    }
</script>

<div class="overview tab-content">
    <div class="prompt-line">
        <span class="prompt">root@sysmon:~$</span>
        <span class="prompt-cmd"> stat --all</span>
        <span class="cursor"></span>
    </div>
    <hr class="separator" />

    <div class="grid">
        <div class="cell">
            <div class="label">CPU</div>
            <div class="value glow {cls(cpu, 80, 95)}">{cpu.toFixed(0)}%</div>
        </div>
        <div class="cell">
            <div class="label">RAM</div>
            <div class="value glow {cls(ramPct, 85, 95)}">{ramPct.toFixed(0)}%</div>
        </div>
        <div class="cell">
            <div class="label">GPU</div>
            <div class="value glow">{gpu?.load_pct != null ? gpu.load_pct.toFixed(0) + '%' : '--'}</div>
            <div class="sub {gpuTemp != null ? cls(gpuTemp, 80, 90) : ''}">{gpuTemp != null ? gpuTemp.toFixed(0) + '°C' : '—'}</div>
        </div>
        <div class="cell">
            <div class="label">{'▼'} DOWN</div>
            <div class="value glow small">{formatSpeed(s.speed.download_bps)}</div>
        </div>
        <div class="cell">
            <div class="label">{'▲'} UP</div>
            <div class="value glow small">{formatSpeed(s.speed.upload_bps)}</div>
        </div>
        <div class="cell">
            <div class="label">IP</div>
            <div class="value tiny">{s.net_info.local_ip || '--'}</div>
        </div>
    </div>
</div>

<style>
    .overview {
        padding: 8px 12px;
    }

    .grid {
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        gap: 4px;
        margin-top: 4px;
    }

    .cell {
        background: var(--bg-secondary, rgba(0, 255, 65, 0.04));
        border: 1px solid var(--border-green);
        padding: 4px 4px 6px;
        text-align: center;
        min-height: 52px;
    }

    .label {
        font-size: 9px;
        color: var(--text-cyan, #00d4ff);
        opacity: 0.85;
        letter-spacing: 1px;
    }

    .value {
        font-size: 18px;
        color: var(--text-green, #00ff41);
        line-height: 1.1;
        margin-top: 2px;
    }

    .value.small {
        font-size: 12px;
    }

    .value.tiny {
        font-size: 11px;
        word-break: break-all;
    }

    .value.warn {
        color: var(--text-orange, #ff6600);
    }

    .value.crit {
        color: var(--text-red, #ff0040);
    }

    .sub {
        font-size: 10px;
        color: var(--text-cyan, #00d4ff);
        opacity: 0.75;
    }

    .sub.warn {
        color: var(--text-orange, #ff6600);
        opacity: 1;
    }

    .sub.crit {
        color: var(--text-red, #ff0040);
        opacity: 1;
    }
</style>
