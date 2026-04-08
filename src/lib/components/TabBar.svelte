<script>
    let { activeTab = $bindable('speed') } = $props();

    const tabs = [
        { id: 'speed', label: 'SPEED' },
        { id: 'network', label: 'NETWORK' },
        { id: 'stats', label: 'STATS' },
    ];
</script>

<div class="tab-bar">
    {#each tabs as tab}
        <button
            class="tab"
            class:active={activeTab === tab.id}
            onclick={() => activeTab = tab.id}
        >
            [{tab.label}]
        </button>
    {/each}
</div>

<style>
    .tab-bar {
        display: flex;
        height: 28px;
        background: var(--bg-primary);
        border-bottom: 1px solid var(--border-green);
        flex-shrink: 0;
    }

    .tab {
        flex: 1;
        border: none;
        background: transparent;
        color: var(--text-dim);
        font-family: var(--font-mono);
        font-size: 12px;
        font-weight: 700;
        cursor: pointer;
        transition: all 0.15s;
        letter-spacing: 1px;
    }

    .tab:hover {
        color: var(--text-green);
    }

    .tab.active {
        color: var(--text-green);
        text-shadow: var(--glow-text);
        background: var(--bg-secondary);
        border-bottom: 2px solid var(--text-green);
    }
</style>
