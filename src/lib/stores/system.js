import { writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';

/**
 * @typedef {{ download_bps: number, upload_bps: number }} SpeedData
 * @typedef {{ latency_ms: number|null, status: string }} PingResult
 * @typedef {{ ssid: string, connection_type: string, local_ip: string,
 *   public_ip: string, gateway: string, dns: string[]|string, mac: string,
 *   signal_percent: number|null }} NetworkInfo
 * @typedef {{ usage: number, per_core: number[], freq_mhz: number }} CpuStats
 * @typedef {{ used: number, total: number, swap_used: number, swap_total: number }} MemStats
 * @typedef {{ name: string, mount: string, read_bps: number, write_bps: number,
 *   free: number, total: number }} DiskEntry
 * @typedef {{ disks: DiskEntry[] }} DiskStats
 * @typedef {{ pid: number, name: string, cpu_pct: number, mem_bytes: number }} ProcInfo
 * @typedef {{ top: ProcInfo[], count: number, uptime_secs: number }} ProcStats
 * @typedef {{ name: string, load_pct: number|null, temp_c: number|null,
 *   vram_used_mb: number, vram_total_mb: number }} GpuReading
 * @typedef {{ gpus: GpuReading[], sensor_available: boolean }} GpuStats
 */

const defaultState = {
    speed: { download_bps: 0, upload_bps: 0 },
    ping: { latency_ms: null, status: 'Ok' },
    net_info: {
        ssid: '', connection_type: '', local_ip: '',
        public_ip: '', gateway: '', dns: [], mac: '', signal_percent: null,
    },
    session: {
        duration_secs: 0, total_downloaded: 0, total_uploaded: 0,
        peak_download_bps: 0, peak_upload_bps: 0, active_connections: 0,
    },
    events: [],
    cpu: { usage: 0, per_core: [], freq_mhz: 0, temp_c: null },
    mem: { used: 0, total: 0, swap_used: 0, swap_total: 0 },
    disk: { disks: [] },
    proc: { top: [], count: 0, uptime_secs: 0 },
    disk: { disks: [] },
    gpu: { gpus: [], sensor_available: true },
};

export const system = writable(defaultState);

// Speed history for ASCII chart (last 60 data points)
export const speedHistory = writable([]);

let listenerInitialized = false;

export async function initSystemListener() {
    if (listenerInitialized) return () => {};
    listenerInitialized = true;

    const unlisten = await listen('system-update', (event) => {
        const data = event.payload;
        system.set(data);

        speedHistory.update(history => {
            const newHistory = [...history, data.speed.download_bps];
            if (newHistory.length > 60) newHistory.shift();
            return newHistory;
        });
    });

    return unlisten;
}
