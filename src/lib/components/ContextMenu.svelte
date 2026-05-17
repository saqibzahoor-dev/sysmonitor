<script>
    import { getCurrentWindow } from '@tauri-apps/api/window';
    import { LogicalPosition } from '@tauri-apps/api/dpi';
    import { invoke } from '@tauri-apps/api/core';

    let { visible = $bindable(false), x = 0, y = 0 } = $props();

    const appWindow = getCurrentWindow();
    const isCompact = appWindow.label === 'compact';

    /** @param {'top-left'|'top-right'|'bottom-left'|'bottom-right'} corner */
    async function setPosition(corner) {
        if (isCompact) {
            // Compact bar: defer to Rust which knows the real window size
            // and computes a proper offset (and persists the choice).
            try {
                await invoke('set_compact_position', { corner });
            } catch (e) {
                console.error('set_compact_position failed', e);
            }
            visible = false;
            return;
        }
        // Full window: compute corner using the actual current window size
        try {
            const monitor = await appWindow.currentMonitor();
            const outer = await appWindow.outerSize();
            if (!monitor) return;
            const scale = monitor.scaleFactor;
            const sw = monitor.size.width / scale;
            const sh = monitor.size.height / scale;
            const ww = outer.width / scale;
            const wh = outer.height / scale;
            const margin = 8;
            const taskbarOffset = 48;
            const positions = {
                'top-left':     { x: margin,           y: margin },
                'top-right':    { x: sw - ww - margin, y: margin },
                'bottom-left':  { x: margin,           y: sh - wh - taskbarOffset - margin },
                'bottom-right': { x: sw - ww - margin, y: sh - wh - taskbarOffset - margin },
            };
            const p = positions[corner];
            if (p) await appWindow.setPosition(new LogicalPosition(p.x, p.y));
        } finally {
            visible = false;
        }
    }

    async function toggleAlwaysOnTop() {
        const current = await appWindow.isAlwaysOnTop();
        await appWindow.setAlwaysOnTop(!current);
        visible = false;
    }

    async function showFull() {
        try { await invoke('set_display_mode', { mode: 'full' }); } catch (e) {}
        visible = false;
    }

    function hideToTray() {
        appWindow.hide();
        visible = false;
    }

    function quit() {
        appWindow.close();
    }
</script>

{#if visible}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="overlay" onclick={() => visible = false} oncontextmenu={(e) => { e.preventDefault(); visible = false; }}>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="context-menu" style="left: {x}px; top: {y}px" onclick={(e) => e.stopPropagation()}>
            <button onclick={() => setPosition('top-left')}>{'↖'} Top-Left</button>
            <button onclick={() => setPosition('top-right')}>{'↗'} Top-Right</button>
            <button onclick={() => setPosition('bottom-left')}>{'↙'} Bottom-Left</button>
            <button onclick={() => setPosition('bottom-right')}>{'↘'} Bottom-Right</button>
            <div class="menu-sep"></div>
            <button onclick={toggleAlwaysOnTop}>{'⊞'} Always on Top</button>
            {#if isCompact}
                <button onclick={showFull}>{'▢'} Open Full Window</button>
            {:else}
                <button onclick={hideToTray}>{'─'} Hide to Tray</button>
            {/if}
            <div class="menu-sep"></div>
            <button class="quit" onclick={quit}>{'✕'} Quit</button>
        </div>
    </div>
{/if}

<style>
    .overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        z-index: 10000;
    }

    .context-menu {
        position: absolute;
        background: var(--bg-secondary);
        border: 1px solid var(--text-green);
        box-shadow: 0 0 15px #00ff4130;
        min-width: 180px;
        z-index: 10001;
    }

    button {
        display: block;
        width: 100%;
        padding: 6px 12px;
        background: transparent;
        border: none;
        color: var(--text-green);
        font-family: var(--font-mono);
        font-size: 11px;
        text-align: left;
        cursor: pointer;
    }

    button:hover {
        background: var(--text-green);
        color: var(--bg-primary);
    }

    button.quit:hover {
        background: var(--text-red);
    }

    .menu-sep {
        border-top: 1px solid var(--border-green);
        margin: 2px 0;
    }
</style>
