<script>
    import { system } from '../stores/system.js';
    import { formatBytes } from '../utils/formatting.js';

    let s = $derived($system);
    let mem = $derived(s.mem);
    let ramPct = $derived(mem.total ? (mem.used / mem.total * 100) : 0);
    let swapPct = $derived(mem.swap_total ? (mem.swap_used / mem.swap_total * 100) : 0);

    let topRam = $derived(
        [...s.proc.top].sort((a, b) => b.mem_bytes - a.mem_bytes).slice(0, 5)
    );

    function bar(pct, width = 24) {
        if (!pct || pct < 0) pct = 0;
        if (pct > 100) pct = 100;
        const filled = Math.round(pct / 100 * width);
        return '█'.repeat(filled) + '░'.repeat(width - filled);
    }

    function tone(pct, warn = 85, crit = 95) {
        if (pct >= crit) return 'error';
        if (pct >= warn) return 'warning';
        return '';
    }

    function gb(bytes) {
        return (bytes / (1024 ** 3)).toFixed(2);
    }
</script>

<div class="mem-tab tab-content">
    <div class="prompt-line">
        <span class="prompt">root@sysmon:~$</span>
        <span class="prompt-cmd"> free --human</span>
        <span class="cursor"></span>
    </div>
    <hr class="separator" />

    <div class="block">
        <div class="block-hdr">RAM</div>
        <div class="bar {tone(ramPct)}">{bar(ramPct)}</div>
        <div class="block-line">
            <span class="left">{gb(mem.used)} / {gb(mem.total)} GB</span>
            <span class="right {tone(ramPct)}">{ramPct.toFixed(1)}%</span>
        </div>
    </div>

    <div class="block">
        <div class="block-hdr">SWAP</div>
        <div class="bar {tone(swapPct, 70, 90)}">{bar(swapPct)}</div>
        <div class="block-line">
            <span class="left">{gb(mem.swap_used)} / {gb(mem.swap_total)} GB</span>
            <span class="right {tone(swapPct, 70, 90)}">{mem.swap_total ? swapPct.toFixed(1) + '%' : '—'}</span>
        </div>
    </div>

    <div class="block">
        <div class="block-hdr">TOP RAM</div>
        {#if topRam.length === 0}
            <div class="empty">no process data</div>
        {/if}
        {#each topRam as p}
            <div class="proc-row">
                <span class="pname">{p.name}</span>
                <span class="pmem">{formatBytes(p.mem_bytes)}</span>
            </div>
        {/each}
    </div>
</div>

<style>
    .mem-tab {
        padding: 8px 12px;
        font-size: 12px;
    }

    .block {
        margin-bottom: 6px;
    }

    .block-hdr {
        color: var(--text-cyan);
        font-size: 10px;
        letter-spacing: 1px;
        margin-bottom: 2px;
    }

    .bar {
        color: var(--text-green);
        letter-spacing: -1px;
        font-size: 13px;
        line-height: 1;
    }

    .bar.warning {
        color: var(--text-orange);
    }

    .bar.error {
        color: var(--text-red);
    }

    .block-line {
        display: flex;
        justify-content: space-between;
        font-size: 11px;
        color: var(--text-green);
    }

    .block-line .right.warning {
        color: var(--text-orange);
    }

    .block-line .right.error {
        color: var(--text-red);
    }

    .empty {
        color: var(--text-dim);
        font-size: 11px;
        font-style: italic;
    }

    .proc-row {
        display: grid;
        grid-template-columns: 1fr auto;
        gap: 8px;
        font-size: 11px;
    }

    .pname {
        color: var(--text-green);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .pmem {
        color: var(--text-cyan);
    }
</style>
