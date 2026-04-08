/**
 * Format bytes per second to human-readable speed.
 * @param {number} bps - bytes per second
 * @returns {string}
 */
export function formatSpeed(bps) {
    if (bps === 0 || bps == null) return '0 b/s';
    const bits = bps * 8;
    if (bits >= 1_000_000_000) return (bits / 1_000_000_000).toFixed(1) + ' Gb/s';
    if (bits >= 1_000_000) return (bits / 1_000_000).toFixed(1) + ' Mb/s';
    if (bits >= 1_000) return (bits / 1_000).toFixed(1) + ' Kb/s';
    return bits.toFixed(0) + ' b/s';
}

/**
 * Format bytes to human-readable size.
 * @param {number} bytes
 * @returns {string}
 */
export function formatBytes(bytes) {
    if (bytes === 0 || bytes == null) return '0 B';
    if (bytes >= 1_073_741_824) return (bytes / 1_073_741_824).toFixed(1) + ' GB';
    if (bytes >= 1_048_576) return (bytes / 1_048_576).toFixed(1) + ' MB';
    if (bytes >= 1_024) return (bytes / 1_024).toFixed(1) + ' KB';
    return bytes + ' B';
}

/**
 * Format seconds to HH:MM:SS.
 * @param {number} secs
 * @returns {string}
 */
export function formatDuration(secs) {
    if (secs == null) return '00:00:00';
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = Math.floor(secs % 60);
    return [h, m, s].map(v => String(v).padStart(2, '0')).join(':');
}

/**
 * Format ping latency.
 * @param {object} ping - { latency_ms, status }
 * @returns {{ text: string, cls: string }}
 */
export function formatPing(ping) {
    if (!ping || ping.status === 'Error') return { text: '--- ms', cls: 'error' };
    if (ping.status === 'Timeout') return { text: 'timeout', cls: 'warning' };
    if (ping.latency_ms == null) return { text: '--- ms', cls: '' };
    const ms = ping.latency_ms;
    let cls = '';
    if (ms > 100) cls = 'warning';
    if (ms > 300) cls = 'error';
    return { text: ms + ' ms', cls };
}

/**
 * Generate signal bar string.
 * @param {number|null} percent
 * @returns {{ filled: string, empty: string, text: string }}
 */
export function formatSignal(percent) {
    if (percent == null) return { filled: '', empty: '\u2588'.repeat(10), text: 'N/A' };
    const filled = Math.round(percent / 10);
    const empty = 10 - filled;
    return {
        filled: '\u2588'.repeat(filled),
        empty: '\u2592'.repeat(empty),
        text: percent + '%',
    };
}
