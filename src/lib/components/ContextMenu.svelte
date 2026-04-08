<script>
    import { getCurrentWindow } from '@tauri-apps/api/window';
    import { LogicalPosition } from '@tauri-apps/api/dpi';

    let { visible = $bindable(false), x = 0, y = 0 } = $props();

    const appWindow = getCurrentWindow();

    async function setPosition(pos) {
        const monitor = await appWindow.currentMonitor();
        if (!monitor) return;
        const { width, height } = monitor.size;
        const scale = monitor.scaleFactor;
        const sw = width / scale;
        const sh = height / scale;
        const positions = {
            'top-left': { x: 10, y: 10 },
            'top-right': { x: sw - 430, y: 10 },
            'bottom-left': { x: 10, y: sh - 390 - 48 },
            'bottom-right': { x: sw - 430, y: sh - 390 - 48 },
        };
        const p = positions[pos];
        if (p) await appWindow.setPosition(new LogicalPosition(p.x, p.y));
        visible = false;
    }

    async function toggleAlwaysOnTop() {
        const current = await appWindow.isAlwaysOnTop();
        await appWindow.setAlwaysOnTop(!current);
        visible = false;
    }

    function hide() {
        appWindow.hide();
        visible = false;
    }

    function quit() {
        // Actually exit the app
        appWindow.close();
    }
</script>

{#if visible}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="overlay" onclick={() => visible = false} oncontextmenu={(e) => { e.preventDefault(); visible = false; }}>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="context-menu" style="left: {x}px; top: {y}px" onclick={(e) => e.stopPropagation()}>
            <button onclick={() => setPosition('top-left')}>{'\u2196'} Top-Left</button>
            <button onclick={() => setPosition('top-right')}>{'\u2197'} Top-Right</button>
            <button onclick={() => setPosition('bottom-left')}>{'\u2199'} Bottom-Left</button>
            <button onclick={() => setPosition('bottom-right')}>{'\u2198'} Bottom-Right</button>
            <div class="menu-sep"></div>
            <button onclick={toggleAlwaysOnTop}>{'\u229E'} Always on Top</button>
            <button onclick={hide}>{'\u2500'} Hide to Tray</button>
            <div class="menu-sep"></div>
            <button class="quit" onclick={quit}>{'\u2715'} Quit</button>
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
        min-width: 170px;
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
