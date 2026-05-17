<script>
    import { system } from '../stores/system.js';
    import { formatDuration } from '../utils/formatting.js';

    let s = $derived($system);
    let sortKey = $state('cpu');

    let sorted = $derived(
        [...s.proc.top].sort((a, b) =>
            sortKey === 'cpu' ? b.cpu_pct - a.cpu_pct : b.mem_bytes - a.mem_bytes
        ).slice(0, 10)
    );

    function mb(bytes) {
        return (bytes / (1024 ** 2)).toFixed(0);
    }
</script>

<div class="proc-tab tab-content">
    <div class="prompt-line">
        <span class="prompt">root@sysmon:~$</span>
        <span class="prompt-cmd"> ps --top10</span>
        <span class="cursor"></span>
    </div>
    <hr class="separator" />

    <div class="meta">
        <span class="meta-item">UPTIME <span class="value">{formatDuration(s.proc.uptime_secs)}</span></span>
        <span class="meta-item">PROCS <span class="value">{s.proc.count}</span></span>
    </div>

    <div class="table-hdr">
        <span class="col-pid">PID</span>
        <span class="col-name">NAME</span>
        <button class="col-cpu sort-btn" class:active={sortKey === 'cpu'} onclick={() => sortKey = 'cpu'}>CPU%</button>
        <button class="col-mem sort-btn" class:active={sortKey === 'mem'} onclick={() => sortKey = 'mem'}>MEM MB</button>
    </div>

    {#if sorted.length === 0}
        <div class="empty">no process data</div>
    {/if}
    {#each sorted as p}
        <div class="proc-row">
            <span class="col-pid">{p.pid}</span>
            <span class="col-name">{p.name}</span>
            <span class="col-cpu">{p.cpu_pct.toFixed(1)}</span>
            <span class="col-mem">{mb(p.mem_bytes)}</span>
        </div>
    {/each}
</div>

<style>
    .proc-tab {
        padding: 8px 12px;
        font-size: 11px;
    }

    .meta {
        display: flex;
        gap: 14px;
        margin-bottom: 4px;
        color: var(--text-dim);
        font-size: 10px;
    }

    .meta-item .value {
        color: var(--text-green);
        margin-left: 4px;
    }

    .table-hdr, .proc-row {
        display: grid;
        grid-template-columns: 50px 1fr 50px 64px;
        gap: 4px;
        font-size: 10px;
    }

    .table-hdr {
        color: var(--text-cyan);
        border-bottom: 1px dashed var(--border-green);
        padding-bottom: 2px;
        margin-bottom: 2px;
    }

    .sort-btn {
        background: none;
        border: none;
        color: var(--text-cyan);
        padding: 0;
        cursor: pointer;
        font: inherit;
        text-align: right;
        opacity: 0.7;
    }

    .sort-btn.active {
        color: var(--text-green);
        text-decoration: underline;
        opacity: 1;
    }

    .proc-row {
        color: var(--text-green);
        padding: 1px 0;
    }

    .col-name {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .col-cpu, .col-mem {
        text-align: right;
    }

    .empty {
        color: var(--text-dim);
        font-style: italic;
        padding: 8px;
        text-align: center;
    }
</style>
