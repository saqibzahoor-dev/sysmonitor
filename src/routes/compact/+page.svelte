<script>
    import { onMount } from 'svelte';
    import '$lib/../styles/global.css';
    import '$lib/../styles/terminal.css';
    import CompactBar from '$lib/components/CompactBar.svelte';
    import { initSystemListener } from '$lib/stores/system.js';

    /** @type {(() => void) | null} */
    let unlistenFn = null;

    onMount(() => {
        initSystemListener().then(fn => { unlistenFn = fn; });
        return () => { if (unlistenFn) unlistenFn(); };
    });
</script>

<CompactBar />

<style>
    :global(html), :global(body) {
        background: var(--bg-primary, #0d1117);
    }
</style>
