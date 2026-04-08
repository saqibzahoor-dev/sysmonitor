<script>
    import { getCurrentWindow } from '@tauri-apps/api/window';

    const appWindow = getCurrentWindow();

    function startDrag() {
        appWindow.startDragging();
    }

    function minimize() {
        appWindow.hide();
    }

    function closeWindow() {
        appWindow.hide();
    }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="title-bar" onmousedown={startDrag}>
    <span class="title glow">
        <span class="prompt">root@netmon</span>:<span class="prompt-cmd">~</span>$
    </span>
    <div class="controls">
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <span class="ctrl-btn minimize" onclick={(e) => { e.stopPropagation(); minimize(); }} title="Minimize to tray">&ndash;</span>
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <span class="ctrl-btn close" onclick={(e) => { e.stopPropagation(); closeWindow(); }} title="Hide to tray">&times;</span>
    </div>
</div>

<style>
    .title-bar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        height: 32px;
        padding: 0 8px 0 12px;
        background: var(--bg-secondary);
        border-bottom: 1px solid var(--border-green);
        cursor: grab;
        flex-shrink: 0;
    }

    .title {
        font-size: 13px;
        font-weight: 700;
    }

    .controls {
        display: flex;
        gap: 4px;
    }

    .ctrl-btn {
        width: 24px;
        height: 24px;
        border: 1px solid var(--border-green);
        background: transparent;
        color: var(--text-green);
        font-family: var(--font-mono);
        font-size: 14px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.15s;
        line-height: 1;
    }

    .ctrl-btn:hover {
        background: var(--text-green);
        color: var(--bg-primary);
    }

    .ctrl-btn.close:hover {
        background: var(--text-red);
        border-color: var(--text-red);
    }
</style>
