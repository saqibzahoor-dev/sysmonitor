<script>
    import { networkData } from '../stores/network.js';
    import { formatSignal } from '../utils/formatting.js';

    let info = $derived($networkData.info);
    let signal = $derived(formatSignal(info.signal_percent));
    let isDisconnected = $derived(!info.local_ip);
</script>

<div class="network-tab tab-content">
    <div class="prompt-line">
        <span class="prompt">root@netmon:~$</span>
        <span class="prompt-cmd"> ifconfig --detail</span>
        <span class="cursor"></span>
    </div>
    <hr class="separator" />

    {#if isDisconnected}
        <p class="value error glow disconnected">[DISCONNECTED]</p>
    {:else}
        <div class="metrics">
            <div class="data-row">
                <span class="label">SSID</span>
                <span class="value">{info.ssid || 'N/A'}</span>
            </div>
            <div class="data-row">
                <span class="label">TYPE</span>
                <span class="value">{info.connection_type || 'N/A'}</span>
            </div>
            <div class="data-row">
                <span class="label">LOCAL IP</span>
                <span class="value glow">{info.local_ip}</span>
            </div>
            <div class="data-row">
                <span class="label">PUBLIC IP</span>
                <span class="value glow">{info.public_ip || 'fetching...'}</span>
            </div>
            <div class="data-row">
                <span class="label">GATEWAY</span>
                <span class="value">{info.gateway || 'N/A'}</span>
            </div>
            <div class="data-row">
                <span class="label">DNS</span>
                <span class="value">{info.dns.length ? info.dns.join(', ') : 'N/A'}</span>
            </div>
            <div class="data-row">
                <span class="label">MAC</span>
                <span class="value">{info.mac || 'N/A'}</span>
            </div>
            <div class="data-row">
                <span class="label">SIGNAL</span>
                <span class="value">
                    <span class="signal-filled">{signal.filled}</span><span class="signal-empty">{signal.empty}</span>
                    {signal.text}
                </span>
            </div>
        </div>
    {/if}
</div>

<style>
    .network-tab {
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

    .disconnected {
        margin-top: 20px;
        text-align: center;
        font-size: 16px;
    }
</style>
