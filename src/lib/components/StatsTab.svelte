<script>
    import { networkData } from '../stores/network.js';
    import { formatSpeed, formatBytes, formatDuration } from '../utils/formatting.js';

    let session = $derived($networkData.session);
    let events = $derived($networkData.events);
</script>

<div class="stats-tab tab-content">
    <div class="prompt-line">
        <span class="prompt">root@netmon:~$</span>
        <span class="prompt-cmd"> stats --session</span>
        <span class="cursor"></span>
    </div>
    <hr class="separator" />

    <div class="metrics">
        <div class="data-row">
            <span class="label">SESSION</span>
            <span class="value glow">{formatDuration(session.duration_secs)}</span>
        </div>
        <div class="data-row">
            <span class="label">{'\u2193'} TOTAL</span>
            <span class="value">{formatBytes(session.total_downloaded)}</span>
        </div>
        <div class="data-row">
            <span class="label">{'\u2191'} TOTAL</span>
            <span class="value">{formatBytes(session.total_uploaded)}</span>
        </div>
        <div class="data-row">
            <span class="label">{'\u2193'} PEAK</span>
            <span class="value">{formatSpeed(session.peak_download_bps)}</span>
        </div>
        <div class="data-row">
            <span class="label">{'\u2191'} PEAK</span>
            <span class="value">{formatSpeed(session.peak_upload_bps)}</span>
        </div>
        <div class="data-row">
            <span class="label">ACTIVE</span>
            <span class="value">{session.active_connections} connections</span>
        </div>
    </div>

    <hr class="separator" />
    <div class="log-header">[LOG]</div>
    <div class="event-log">
        {#if events.length === 0}
            <div class="event-entry">
                <span class="event-time">--:--:--</span>
                <span>monitoring started</span>
            </div>
        {/if}
        {#each [...events].reverse() as event}
            <div class="event-entry">
                <span class="event-time">{event.timestamp}</span>
                <span>{event.message}</span>
            </div>
        {/each}
    </div>
</div>

<style>
    .stats-tab {
        padding: 8px 12px;
    }

    .metrics {
        margin-top: 6px;
    }

    .data-row {
        display: flex;
        justify-content: space-between;
        padding: 2px 0;
        font-size: 12px;
    }

    .label {
        min-width: 90px;
    }

    .log-header {
        color: var(--text-cyan);
        font-weight: 700;
        font-size: 11px;
        margin-top: 2px;
    }
</style>
