<script>
    import { onMount } from 'svelte';
    import '$lib/../styles/global.css';
    import '$lib/../styles/terminal.css';
    import CompactBar from '$lib/components/CompactBar.svelte';
    import ContextMenu from '$lib/components/ContextMenu.svelte';
    import { initSystemListener } from '$lib/stores/system.js';

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
<div class="compact-root" oncontextmenu={handleContextMenu}>
    <CompactBar />
    <ContextMenu bind:visible={menuVisible} x={menuX} y={menuY} />
</div>

<style>
    .compact-root {
        width: 100vw;
        height: 100vh;
        overflow: hidden;
    }
</style>
