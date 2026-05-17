<script>
    import { onMount } from 'svelte';
    import '$lib/../styles/global.css';
    import '$lib/../styles/terminal.css';
    import TitleBar from '$lib/components/TitleBar.svelte';
    import TabBar from '$lib/components/TabBar.svelte';
    import OverviewTab from '$lib/components/OverviewTab.svelte';
    import CpuTab from '$lib/components/CpuTab.svelte';
    import MemTab from '$lib/components/MemTab.svelte';
    import GpuTab from '$lib/components/GpuTab.svelte';
    import NetTab from '$lib/components/NetTab.svelte';
    import ProcTab from '$lib/components/ProcTab.svelte';
    import SysTab from '$lib/components/SysTab.svelte';
    import ContextMenu from '$lib/components/ContextMenu.svelte';
    import { initSystemListener } from '$lib/stores/system.js';

    let activeTab = $state('overview');
    let menuVisible = $state(false);
    let menuX = $state(0);
    let menuY = $state(0);

    /** @param {MouseEvent} e */
    function handleContextMenu(e) {
        e.preventDefault();
        menuX = e.clientX;
        menuY = e.clientY;
        menuVisible = true;
    }

    /** @type {(() => void) | null} */
    let unlistenFn = null;

    onMount(() => {
        initSystemListener().then(fn => { unlistenFn = fn; });
        return () => { if (unlistenFn) unlistenFn(); };
    });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="app scanlines" oncontextmenu={handleContextMenu}>
    <TitleBar />
    <TabBar bind:activeTab />

    {#if activeTab === 'overview'}
        <OverviewTab />
    {:else if activeTab === 'cpu'}
        <CpuTab />
    {:else if activeTab === 'mem'}
        <MemTab />
    {:else if activeTab === 'gpu'}
        <GpuTab />
    {:else if activeTab === 'net'}
        <NetTab />
    {:else if activeTab === 'proc'}
        <ProcTab />
    {:else if activeTab === 'sys'}
        <SysTab />
    {/if}

    <ContextMenu bind:visible={menuVisible} x={menuX} y={menuY} />
</div>

<style>
    .app {
        width: 420px;
        height: 440px;
        border: 1px solid var(--border-green);
        box-shadow: var(--glow-green);
        display: flex;
        flex-direction: column;
        overflow: hidden;
        background: var(--bg-primary);
    }
</style>
