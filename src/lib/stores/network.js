import { writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';

export const networkData = writable({
    speed: { download_bps: 0, upload_bps: 0 },
    ping: { latency_ms: null, status: 'Ok' },
    info: {
        ssid: '', connection_type: '', local_ip: '',
        public_ip: '', gateway: '', dns: [], mac: '', signal_percent: null,
    },
    session: {
        duration_secs: 0, total_downloaded: 0, total_uploaded: 0,
        peak_download_bps: 0, peak_upload_bps: 0, active_connections: 0,
    },
    events: [],
});

// Speed history for ASCII chart (last 60 data points)
export const speedHistory = writable([]);

let listenerInitialized = false;

export function initNetworkListener() {
    if (listenerInitialized) return;
    listenerInitialized = true;

    listen('network-update', (event) => {
        const data = event.payload;

        networkData.set({
            speed: data.speed,
            ping: data.ping,
            info: data.info,
            session: data.session,
            events: data.events,
        });

        // Update speed history
        speedHistory.update(history => {
            const newHistory = [...history, data.speed.download_bps];
            if (newHistory.length > 60) newHistory.shift();
            return newHistory;
        });
    });
}
