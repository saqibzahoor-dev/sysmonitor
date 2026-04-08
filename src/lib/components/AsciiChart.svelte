<script>
    let { data = [], width = 38, height = 5 } = $props();

    function render(data, width, height) {
        if (!data || data.length === 0) {
            return Array(height).fill(' '.repeat(width));
        }

        const max = Math.max(...data, 1);
        const points = data.slice(-width);
        const lines = [];

        for (let row = height - 1; row >= 0; row--) {
            let line = '';
            for (let col = 0; col < width; col++) {
                const idx = col - (width - points.length);
                if (idx < 0 || idx >= points.length) {
                    line += ' ';
                    continue;
                }
                const val = points[idx];
                const normalized = (val / max) * height;
                if (normalized >= row + 0.5) {
                    line += '\u2588';
                } else if (normalized >= row) {
                    line += '\u2584';
                } else {
                    line += ' ';
                }
            }
            lines.push(line);
        }

        return lines;
    }

    let chartLines = $derived(render(data, width, height));
    let topBorder = $derived('\u256D' + '\u2500'.repeat(width) + '\u256E');
    let bottomBorder = $derived('\u2570' + '\u2500'.repeat(width) + '\u256F');
</script>

<div class="ascii-chart">
    <div class="chart-border">{topBorder}</div>
    {#each chartLines as line}
        <div class="chart-row"><span class="chart-border">\u2502</span><span class="chart-line">{line}</span><span class="chart-border">\u2502</span></div>
    {/each}
    <div class="chart-border">{bottomBorder}</div>
</div>

<style>
    .ascii-chart {
        font-size: 10px;
        line-height: 1.1;
        color: var(--text-green);
        text-shadow: var(--glow-text);
        white-space: pre;
        font-family: var(--font-mono);
    }

    .chart-border {
        color: var(--text-dim);
    }

    .chart-row {
        display: block;
    }

    .chart-line {
        color: var(--text-green);
    }
</style>
