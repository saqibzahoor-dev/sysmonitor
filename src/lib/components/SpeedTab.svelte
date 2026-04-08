<script>
    import { networkData, speedHistory } from '../stores/network.js';
    import { formatSpeed, formatPing } from '../utils/formatting.js';
    import AsciiChart from './AsciiChart.svelte';

    let speed = $derived($networkData.speed);
    let ping = $derived(formatPing($networkData.ping));
    let history = $derived($speedHistory);
</script>

<div class="speed-tab tab-content">
    <div class="prompt-line">
        <span class="prompt">root@netmon:~$</span>
        <span class="prompt-cmd"> speed --monitor</span>
        <span class="cursor"></span>
    </div>
    <hr class="separator" />

    <div class="metrics">
        <div class="data-row">
            <span class="label">{'\u25BC'} DOWN</span>
            <span class="value glow speed-value">{formatSpeed(speed.download_bps)}</span>
        </div>
        <div class="data-row">
            <span class="label">{'\u25B2'} UP</span>
            <span class="value glow speed-value">{formatSpeed(speed.upload_bps)}</span>
        </div>
        <div class="data-row">
            <span class="label">{'\u25C6'} PING</span>
            <span class="value {ping.cls}">{ping.text}</span>
        </div>
    </div>

    <div class="chart-section">
        <AsciiChart data={history} width={38} height={5} />
    </div>
</div>

<style>
    .speed-tab {
        padding: 8px 12px;
    }

    .metrics {
        margin: 6px 0;
    }

    .data-row {
        display: flex;
        justify-content: space-between;
        padding: 3px 0;
        font-size: 14px;
    }

    .speed-value {
        font-size: 14px;
        font-weight: 700;
    }

    .chart-section {
        margin-top: 6px;
    }
</style>
