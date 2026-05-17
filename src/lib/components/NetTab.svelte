<script>
    import { system, speedHistory } from '../stores/system.js';
    import { formatSpeed, formatPing, formatSignal } from '../utils/formatting.js';
    import AsciiChart from './AsciiChart.svelte';

    let s = $derived($system);
    let speed = $derived(s.speed);
    let ping = $derived(formatPing(s.ping));
    let info = $derived(s.net_info);
    let signal = $derived(formatSignal(info.signal_percent));
    let history = $derived($speedHistory);
    let isDisconnected = $derived(!info.local_ip);
    let dnsText = $derived(Array.isArray(info.dns) ? info.dns.join(', ') : (info.dns || 'N/A'));
</script>

<div class="net-tab tab-content">
    <div class="prompt-line">
        <span class="prompt">root@sysmon:~$</span>
        <span class="prompt-cmd"> netstat --live</span>
        <span class="cursor"></span>
    </div>
    <hr class="separator" />

    {#if isDisconnected}
        <p class="value error glow disconnected">[DISCONNECTED]</p>
    {:else}
        <div class="speeds">
            <div class="row">
                <span class="label">{'▼'} DOWN</span>
                <span class="value glow">{formatSpeed(speed.download_bps)}</span>
            </div>
            <div class="row">
                <span class="label">{'▲'} UP</span>
                <span class="value glow">{formatSpeed(speed.upload_bps)}</span>
            </div>
            <div class="row">
                <span class="label">{'◆'} PING</span>
                <span class="value {ping.cls}">{ping.text}</span>
            </div>
        </div>

        <div class="chart">
            <AsciiChart data={history} width={36} height={4} />
        </div>

        <div class="info">
            <div class="row">
                <span class="label">SSID</span>
                <span class="value">{info.ssid || 'N/A'}</span>
            </div>
            <div class="row">
                <span class="label">LOCAL</span>
                <span class="value glow">{info.local_ip}</span>
            </div>
            <div class="row">
                <span class="label">PUBLIC</span>
                <span class="value glow">{info.public_ip || 'fetching...'}</span>
            </div>
            <div class="row">
                <span class="label">GW</span>
                <span class="value">{info.gateway || 'N/A'}</span>
            </div>
            <div class="row">
                <span class="label">DNS</span>
                <span class="value dns">{dnsText}</span>
            </div>
            <div class="row">
                <span class="label">MAC</span>
                <span class="value">{info.mac || 'N/A'}</span>
            </div>
            <div class="row">
                <span class="label">SIG</span>
                <span class="value">
                    <span class="signal-filled">{signal.filled}</span><span class="signal-empty">{signal.empty}</span>
                    {signal.text}
                </span>
            </div>
        </div>
    {/if}
</div>

<style>
    .net-tab {
        padding: 8px 12px;
        font-size: 12px;
    }

    .speeds {
        margin-bottom: 4px;
    }

    .row {
        display: flex;
        justify-content: space-between;
        padding: 1px 0;
        font-size: 11px;
    }

    .speeds .row {
        font-size: 12px;
    }

    .label {
        min-width: 70px;
    }

    .chart {
        margin: 4px 0;
        overflow: hidden;
        max-width: 100%;
    }

    .info {
        font-size: 11px;
    }

    .dns {
        text-align: right;
        max-width: 280px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .disconnected {
        margin-top: 20px;
        text-align: center;
        font-size: 16px;
    }
</style>
