<script>
    import { onMount } from 'svelte';
    import '$lib/../styles/global.css';
    import '$lib/../styles/terminal.css';
    import TitleBar from '$lib/components/TitleBar.svelte';
    import TabBar from '$lib/components/TabBar.svelte';
    import SpeedTab from '$lib/components/SpeedTab.svelte';
    import NetworkTab from '$lib/components/NetworkTab.svelte';
    import StatsTab from '$lib/components/StatsTab.svelte';
    import ContextMenu from '$lib/components/ContextMenu.svelte';
    import { initNetworkListener } from '$lib/stores/network.js';

    let activeTab = $state('speed');
    let menuVisible = $state(false);
    let menuX = $state(0);
    let menuY = $state(0);

    function handleContextMenu(e) {
        e.preventDefault();
        menuX = e.clientX;
        menuY = e.clientY;
        menuVisible = true;
    }

    onMount(() => {
        initNetworkListener();
    });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="app scanlines" oncontextmenu={handleContextMenu}>
    <TitleBar />
    <TabBar bind:activeTab />

    {#if activeTab === 'speed'}
        <SpeedTab />
    {:else if activeTab === 'network'}
        <NetworkTab />
    {:else if activeTab === 'stats'}
        <StatsTab />
    {/if}

    <ContextMenu bind:visible={menuVisible} x={menuX} y={menuY} />
</div>

<style>
    .app {
        width: 420px;
        height: 380px;
        border: 1px solid var(--border-green);
        box-shadow: var(--glow-green);
        display: flex;
        flex-direction: column;
        overflow: hidden;
        background: var(--bg-primary);
    }
</style>
