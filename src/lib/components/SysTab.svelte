<script>
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    /** @type {any} */
    let info = $state(null);
    let loading = $state(true);
    /** @type {string|null} */
    let err = $state(null);

    async function load() {
        loading = true;
        err = null;
        try {
            info = await invoke('get_system_info');
        } catch (e) {
            err = String(e);
        } finally {
            loading = false;
        }
    }

    onMount(load);

    /** @param {number|null|undefined} bytes */
    function gb(bytes) {
        return bytes ? (bytes / (1024 ** 3)).toFixed(1) + ' GB' : '—';
    }

    /** @param {string|null|undefined} s */
    function parseWmiDate(s) {
        // WMI dates look like "20260516081402.000000+300" — extract the date portion
        if (!s || typeof s !== 'string') return s || '—';
        const m = s.match(/^(\d{4})(\d{2})(\d{2})/);
        if (!m) return s;
        return `${m[1]}-${m[2]}-${m[3]}`;
    }
</script>

<div class="sys-tab tab-content">
    <div class="prompt-line">
        <span class="prompt">root@sysmon:~$</span>
        <span class="prompt-cmd"> hwinfo --full</span>
        <button class="refresh" onclick={load} title="Refresh inventory">↻</button>
    </div>
    <hr class="separator" />

    <div class="scroll">
        {#if loading}
            <div class="dim">loading inventory...</div>
        {/if}
        {#if err}
            <div class="error">error: {err}</div>
        {/if}
        {#if info && !loading}
            <section>
                <div class="hdr">OS</div>
                <div class="line">{info.os.name ?? '—'} <span class="dim">(build {info.os.build ?? '—'})</span></div>
                <div class="line dim">install {parseWmiDate(info.os.install_date)} · boot {parseWmiDate(info.os.last_boot)}</div>
            </section>

            <section>
                <div class="hdr">MOTHERBOARD</div>
                <div class="line">{info.motherboard.manufacturer ?? '—'} {info.motherboard.product ?? ''}</div>
                <div class="line dim">BIOS {info.motherboard.bios_vendor ?? '—'} {info.motherboard.bios_version ?? ''} <span>({parseWmiDate(info.motherboard.bios_date)})</span></div>
            </section>

            <section>
                <div class="hdr">CPU</div>
                <div class="line">{info.cpu.name ?? '—'}</div>
                <div class="line dim">{info.cpu.cores ?? '?'}c / {info.cpu.threads ?? '?'}t · {info.cpu.max_clock_mhz ?? '?'} MHz · L2 {info.cpu.l2_cache_kb ?? '?'} KB · L3 {info.cpu.l3_cache_kb ?? '?'} KB</div>
            </section>

            <section>
                <div class="hdr">RAM ({info.ram.length} {info.ram.length === 1 ? 'stick' : 'sticks'})</div>
                {#each info.ram as r}
                    <div class="line">
                        <span class="slot">{r.slot ?? '?'}</span>
                        {r.manufacturer ?? '—'} {r.part_number ?? ''}
                        <span class="dim">— {gb(r.capacity_bytes)} {r.memory_type ?? ''} @ {r.speed_mhz ?? '?'} MHz</span>
                    </div>
                {/each}
            </section>

            <section>
                <div class="hdr">GPU</div>
                {#each info.gpus as g}
                    <div class="line">{g.name ?? '—'} <span class="dim">({gb(g.vram_bytes)})</span></div>
                    <div class="line dim">drv {g.driver_version ?? '—'} · {parseWmiDate(g.driver_date)}</div>
                {/each}
            </section>

            <section>
                <div class="hdr">DRIVES</div>
                {#each info.drives as d}
                    <div class="line">{d.model ?? '—'} <span class="dim">— {gb(d.capacity_bytes)} [{d.interface_type ?? '?'}] · {d.health ?? '?'}</span></div>
                {/each}
            </section>
        {/if}
    </div>
</div>

<style>
    .sys-tab {
        padding: 8px 12px;
        font-size: 10px;
    }

    .refresh {
        background: none;
        border: 1px solid var(--border-green);
        color: var(--text-green);
        padding: 0 4px;
        margin-left: 6px;
        cursor: pointer;
        font: inherit;
        font-size: 10px;
        line-height: 14px;
    }

    .scroll {
        max-height: 320px;
        overflow-y: auto;
        padding-right: 4px;
    }

    section {
        margin-bottom: 6px;
    }

    .hdr {
        color: var(--text-cyan);
        font-size: 10px;
        border-bottom: 1px dashed var(--border-green);
        margin-bottom: 2px;
        letter-spacing: 1px;
    }

    .line {
        color: var(--text-green);
        word-break: break-word;
        line-height: 1.4;
    }

    .dim {
        color: var(--text-dim);
    }

    .error {
        color: var(--text-red);
        padding: 6px;
    }

    .slot {
        color: var(--text-cyan);
        margin-right: 4px;
    }
</style>
