<script lang="ts">
  import {onMount} from 'svelte';
  import {chooseExportDirectory, loadBackgroundNotifications, loadKeepAwake, loadQuickSurface, openLocalDirectory, saveBackgroundNotifications, saveKeepAwake, saveQuickSurface} from './lib/bridge';
  import {t} from './lib/i18n.svelte';
  import type {QuickSurface} from './lib/preferences';
  import type {QuickActionConfig} from './lib/quick-actions';

  let {config, onDirectoryChange}: {
    config: QuickActionConfig | null;
    onDirectoryChange: (config: QuickActionConfig) => void;
  } = $props();
  let directoryBusy = $state(false);
  let directoryError = $state('');
  const toggles = [
    {id: 'background-notification-toggle', label: 'general.backgroundNotifications', hint: 'general.backgroundNotificationsHint',
      load: loadBackgroundNotifications, save: saveBackgroundNotifications},
    {id: 'keep-awake-toggle', label: 'general.keepAwake', hint: 'general.keepAwakeHint',
      load: loadKeepAwake, save: saveKeepAwake},
  ] as const;
  type Setting = typeof toggles[number];
  type ToggleState = {value: boolean | null; busy: boolean; error: string};
  let toggleState = $state<Record<Setting['id'], ToggleState>>({
    'background-notification-toggle': {value: null, busy: false, error: ''},
    'keep-awake-toggle': {value: null, busy: false, error: ''},
  });
  let quickSurface = $state<QuickSurface | null>(null);
  let quickBusy = $state(false);
  let quickError = $state('');

  async function directoryAction(action: 'choose' | 'export' | 'data') {
    if (directoryBusy) return;
    directoryBusy = true;
    directoryError = '';
    try {
      if (action === 'choose' && config) {
        const saved = await chooseExportDirectory(config.revision);
        if (saved) onDirectoryChange(saved);
      } else if (action !== 'choose') {
        await openLocalDirectory(action === 'export');
      }
    } catch (error) {
      directoryError = String(error instanceof Error ? error.message : error);
    } finally {
      directoryBusy = false;
    }
  }

  async function updateToggle(setting: Setting, value?: boolean) {
    const state = toggleState[setting.id];
    if (state.busy) return;
    state.busy = true;
    state.error = '';
    try {
      state.value = await (value === undefined ? setting.load() : setting.save(value));
    } catch (error) {
      state.error = error instanceof Error ? error.message : String(error);
    } finally {
      state.busy = false;
    }
  }

  async function updateQuickSurface(surface?: QuickSurface, control?: HTMLSelectElement) {
    if (quickBusy) return;
    quickBusy = true;
    quickError = '';
    try {
      quickSurface = await (surface === undefined ? loadQuickSurface() : saveQuickSurface(surface));
    } catch (error) {
      if (control && quickSurface) control.value = quickSurface;
      quickError = error instanceof Error ? error.message : String(error);
    } finally {
      quickBusy = false;
    }
  }

  onMount(() => {
    for (const setting of toggles) void updateToggle(setting);
    void updateQuickSurface();
  });
</script>

<div class="group-card">
  <h2>{t('general.storageGroup')}</h2>
  <div class="row">
    <div class="directory">
      <div id="export-directory-label">{t('general.exportDirectory')}</div>
      {#if config}<span title={config.outputDirectory}>{config.outputDirectory}</span>{/if}
    </div>
    <button id="change-export-directory" disabled={!config || directoryBusy} aria-describedby="export-directory-label" onclick={() => void directoryAction('choose')}>{t('general.changeDirectory')}</button>
  </div>
  <div class="row actions">
    <button disabled={!config || directoryBusy} onclick={() => void directoryAction('export')}>{t('general.openExportDirectory')}</button>
    <button disabled={directoryBusy} onclick={() => void directoryAction('data')}>{t('general.openDataDirectory')}</button>
  </div>
  {#if directoryError}<p role="alert">{directoryError}</p>{/if}
</div>

<div class="group-card">
  <div class="row">
    <label for="quick-surface">{t('general.quickSurface')}</label>
    <select id="quick-surface" value={quickSurface ?? 'capsule'} disabled={!quickSurface || quickBusy}
      onchange={event => void updateQuickSurface(event.currentTarget.value as QuickSurface, event.currentTarget)}>
      <option value="capsule">{t('general.quickCapsule')}</option>
      <option value="full">{t('general.quickFull')}</option>
    </select>
  </div>
  {#if quickError}
    <p role="alert">{quickError}</p>
    {#if !quickSurface}<button onclick={() => void updateQuickSurface()}>{t('general.reload')}</button>{/if}
  {/if}
</div>

<div class="group-card">
  {#each toggles as setting (setting.id)}
    {@const state = toggleState[setting.id]}
    <div class="toggle-setting">
      <div class="row">
        <div>
          <label for={setting.id}>{t(setting.label)}</label>
          <div id={setting.id + '-hint'} class="hint">{t(setting.hint)}</div>
        </div>
        <input id={setting.id} type="checkbox" aria-describedby={setting.id + '-hint'}
          checked={state.value === true} disabled={state.value === null || state.busy}
          onchange={event => {
            const next = event.currentTarget.checked;
            event.currentTarget.checked = state.value === true;
            void updateToggle(setting, next);
          }} />
      </div>
      {#if state.error}
        <p role="alert">{state.error}</p>
        {#if state.value === null}<button onclick={() => void updateToggle(setting)}>{t('general.reload')}</button>{/if}
      {/if}
    </div>
  {/each}
</div>

<style>
  .group-card{background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:14px 16px;margin-bottom:14px}
  h2{margin:0 0 8px;font-size:.86rem;font-weight:600;color:var(--muted)}
  .row{display:flex;align-items:center;justify-content:space-between;gap:12px;font-size:.96rem}
  .row+.row,.toggle-setting+.toggle-setting{border-top:1px solid var(--edge);margin-top:10px;padding-top:10px}
  .actions{justify-content:flex-start;flex-wrap:wrap}
  .directory{min-width:0}
  .directory span{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .directory span,.hint{margin-top:3px;font-size:.85rem;color:var(--muted)}
  button{flex-shrink:0;font-size:.85rem}
  select{font:inherit;font-size:.92rem;color:var(--text);background:var(--surface-soft);border:1px solid var(--edge);border-radius:var(--radius-control);padding:6px 10px}
  select:focus-visible{outline:2px solid var(--focus-ring);outline-offset:1px}
  input{flex-shrink:0;width:16px;height:16px;accent-color:var(--accent);margin:0}
  p{margin:8px 0 0;font-size:.85rem;color:var(--danger)}
</style>
