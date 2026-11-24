<script lang="ts">
  import {DEFAULT_GENERAL_PREFERENCES, type GeneralPreferences} from './lib/preferences';
  import {APP_VERSION, GITHUB_REPO_URL, checkGithubUpdate, openExternal, setAppLocale, type ReleaseCheck} from './lib/bridge';
  import {LOCALES, getLocale, setLocale, t, type Locale} from './lib/i18n.svelte';

  let {desktop, enabled, busy, error, prefs=DEFAULT_GENERAL_PREFERENCES, onToggle, onReload, onPrefsChange=()=>{}, onResetAll}: {
    desktop: boolean;
    enabled: boolean | null;
    busy: boolean;
    error: string | null;
    prefs?: GeneralPreferences;
    onToggle: (enabled: boolean) => void;
    onReload: () => void;
    onPrefsChange?: (patch: Partial<GeneralPreferences>) => void;
    onResetAll?: () => void;
  } = $props();

  let checkingUpdate = $state(false);
  let updateResult = $state<ReleaseCheck | null>(null);
  let updateError = $state<string | null>(null);

  function toggleAutostart(event: Event & {currentTarget: HTMLInputElement}) {
    const requested = event.currentTarget.checked;
    event.currentTarget.checked = enabled === true;
    onToggle(requested);
  }

  async function runUpdateCheck() {
    if (checkingUpdate) return;
    checkingUpdate = true;
    updateError = null;
    try {
      updateResult = await checkGithubUpdate();
    } catch (e) {
      updateResult = null;
      updateError = e instanceof Error ? e.message : String(e);
    } finally {
      checkingUpdate = false;
    }
  }

  function openLink(url: string) {
    if (desktop) {
      void openExternal(url);
    } else {
      window.open(url, '_blank', 'noopener,noreferrer');
    }
  }
</script>

<section aria-label={t('general.title')} class="general-settings">
  <h1>{t('general.title')}</h1>

  <div class="group-card">
    <h2>{t('general.languageGroup')}</h2>
    <div class="setting-row">
      <label class="row-text" for="interface-locale">{t('general.language')}</label>
      <select id="interface-locale" value={getLocale()} onchange={e => { const next = e.currentTarget.value as Locale; setLocale(next); void setAppLocale(next); }}>
        {#each LOCALES as item}
          <option value={item.id}>{item.label}</option>
        {/each}
      </select>
    </div>
  </div>

  <div class="group-card">
    <h2>{t('general.startupGroup')}</h2>
    {#if desktop}
      <div class="setting-row">
        <div class="setting-label">
          <label for="autostart-toggle">{t('general.autostart')}</label>
          <details class="hint-disclosure inline-hint"><summary>{t('general.helpSummary')}</summary><div class="hint-disclosure-body"><p>{t('general.autostartHint')}</p></div></details>
        </div>
        <input id="autostart-toggle" type="checkbox" class="toggle-check" checked={enabled === true} disabled={busy || enabled === null} onchange={toggleAutostart} />
      </div>
      {#if busy}
        <p class="row-status" role="status">{enabled === null ? t('general.loading') : t('general.saving')}</p>
      {:else if error}
        <div class="row-alert" role="alert"><span>{error}</span>{#if enabled === null}<button type="button" onclick={onReload}>{t('general.reload')}</button>{/if}</div>
      {/if}
    {/if}

    <div class="setting-row">
      <label class="row-text" for="esc-hide-toggle">{t('general.escHide')}</label>
      <input id="esc-hide-toggle" type="checkbox" class="toggle-check" checked={prefs.escToHide} onchange={e => onPrefsChange({escToHide: e.currentTarget.checked})} />
    </div>

    <div class="setting-row">
      <label class="row-text" for="welcome-mark-toggle">{t('general.welcomeMark')}</label>
      <input id="welcome-mark-toggle" type="checkbox" class="toggle-check" checked={prefs.showWelcomeMark} onchange={e => onPrefsChange({showWelcomeMark: e.currentTarget.checked})} />
    </div>
  </div>

  <div class="group-card">
    <h2>{t('general.inputGroup')}</h2>
    <div class="setting-row stacked">
      <span class="row-text">{t('general.sendKey')}</span>
      <div class="segmented" role="radiogroup" aria-label={t('general.sendKey')}>
        <label class:active={prefs.sendKey === 'enter'}><input type="radio" name="send-key" value="enter" checked={prefs.sendKey === 'enter'} onchange={() => onPrefsChange({sendKey: 'enter'})} /><span>{t('general.sendEnter')}</span></label>
        <label class:active={prefs.sendKey === 'ctrl-enter'}><input type="radio" name="send-key" value="ctrl-enter" checked={prefs.sendKey === 'ctrl-enter'} onchange={() => onPrefsChange({sendKey: 'ctrl-enter'})} /><span>{t('general.sendCtrlEnter')}</span></label>
      </div>
    </div>

    <div class="setting-row stacked">
      <span class="row-text">{t('general.motionSpeed')}</span>
      <div class="segmented" role="radiogroup" aria-label={t('general.motionSpeed')}>
        <label class:active={prefs.motionSpeed === 'expressive'}><input type="radio" name="motion-speed" value="expressive" checked={prefs.motionSpeed === 'expressive'} onchange={() => onPrefsChange({motionSpeed: 'expressive'})} /><span>{t('general.motionExpressive')}</span></label>
        <label class:active={prefs.motionSpeed === 'snappy'}><input type="radio" name="motion-speed" value="snappy" checked={prefs.motionSpeed === 'snappy'} onchange={() => onPrefsChange({motionSpeed: 'snappy'})} /><span>{t('general.motionSnappy')}</span></label>
        <label class:active={prefs.motionSpeed === 'off'}><input type="radio" name="motion-speed" value="off" checked={prefs.motionSpeed === 'off'} onchange={() => onPrefsChange({motionSpeed: 'off'})} /><span>{t('general.motionOff')}</span></label>
      </div>
    </div>
  </div>

  <div class="group-card">
    <h2>{t('general.aboutGroup')}</h2>
    <div class="setting-row stacked">
      <div class="version-meta">
        <span class="row-text">{t('general.currentVersion')}</span>
        <span class="version-badge" data-app-version>v{updateResult?.currentVersion ?? APP_VERSION}</span>
        <span class="stage-pill">{t('general.devStage')}</span>
      </div>
      <div class="update-actions">
        <button type="button" class="check-btn" data-check-update disabled={checkingUpdate} onclick={() => void runUpdateCheck()}>
          {checkingUpdate ? t('general.checkingUpdate') : t('general.checkUpdate')}
        </button>
        <button type="button" class="repo-btn" onclick={() => openLink(GITHUB_REPO_URL)}>
          {t('general.openRepo')}
        </button>
      </div>
    </div>

    {#if updateResult}
      <div class="update-box" class:has-update={updateResult.hasUpdate} role="status">
        <div class="update-head">
          {#if updateResult.status === 'update_available' && updateResult.latestVersion}
            <strong>{t('general.updateAvailable', {version: updateResult.latestVersion})}</strong>
          {:else if updateResult.status === 'up_to_date'}
            <span>{t('general.upToDate', {version: updateResult.currentVersion})}</span>
          {:else}
            <span>{t('general.noRelease', {version: updateResult.currentVersion})}</span>
          {/if}
          <button type="button" class="link-btn" onclick={() => openLink(updateResult!.releaseUrl)}>
            {t('general.openRelease')}
          </button>
        </div>
        {#if updateResult.status === 'update_available' && updateResult.notes}
          <p class="release-notes">{updateResult.notes}</p>
        {/if}
      </div>
    {:else if updateError}
      <div class="row-alert" role="alert"><span>{updateError}</span></div>
    {/if}
  </div>

  {#if onResetAll}
    <div class="reset-bar">
      <button type="button" onclick={onResetAll}>{t('general.resetAll')}</button>
    </div>
  {/if}
</section>

<style>
  .general-settings{max-width:600px;margin:20px auto;padding:0 18px 28px;color:var(--text);font-size:1rem}
  h1{margin:0 0 18px;font-size:1.35rem;font-weight:600}
  h2{margin:0 0 8px;font-size:.86rem;font-weight:600;color:var(--muted);letter-spacing:.02em}
  .group-card{background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:14px 16px;margin-bottom:14px}
  .setting-row{display:flex;align-items:center;justify-content:space-between;gap:14px;padding:10px 0;border-top:1px solid var(--edge)}
  .group-card h2+.setting-row{border-top:0;padding-top:4px}
  .setting-row:last-child{padding-bottom:2px}
  .setting-row.stacked{flex-wrap:wrap}
  .setting-label{display:flex;align-items:center;gap:10px;min-width:0;flex-wrap:wrap}
  .setting-label label,.row-text{font-size:.96rem;font-weight:500;color:var(--text);cursor:pointer}
  .inline-hint{margin-top:0}
  select{border:1px solid var(--edge);border-radius:var(--radius-control);background:var(--surface-soft);color:var(--text);padding:6px 10px;font:inherit;font-size:.92rem;min-width:140px}
  select:focus-visible{outline:2px solid var(--focus-ring);outline-offset:1px}
  .toggle-check{width:16px;height:16px;margin:0;accent-color:var(--accent);cursor:pointer;flex-shrink:0}
  .toggle-check:disabled{cursor:default}
  .segmented{display:inline-flex;gap:4px;padding:3px;border-radius:var(--radius-control);background:var(--surface-soft);border:1px solid var(--edge)}
  .segmented label{position:relative;display:inline-flex;align-items:center;padding:5px 11px;border-radius:calc(var(--radius-control) - 2px);font-size:.88rem;font-weight:500;color:var(--muted);cursor:pointer;transition:background-color 140ms ease,color 140ms ease}
  .segmented input{position:absolute;opacity:0;pointer-events:none}
  .segmented label.active{background:var(--surface);color:var(--text);box-shadow:var(--raised-shadow);font-weight:600}
  .segmented label:focus-within{outline:2px solid var(--focus-ring);outline-offset:1px}
  .version-meta{display:flex;align-items:center;gap:8px;flex-wrap:wrap}
  .version-badge{font-family:var(--font-mono);font-size:12.5px;font-weight:600;padding:2px 8px;border-radius:999px;background:var(--surface-soft);border:1px solid var(--edge);color:var(--text);font-variant-numeric:tabular-nums}
  .stage-pill{font-size:11.5px;font-weight:500;padding:2px 7px;border-radius:999px;background:var(--accent-soft);color:var(--text)}
  .update-actions{display:flex;align-items:center;gap:8px;flex-wrap:wrap}
  .check-btn{background:var(--accent);color:var(--on-accent);border-color:var(--accent);font-size:12.5px;font-weight:500;padding:6px 12px}
  @media(hover:hover) and (pointer:fine){.check-btn:hover:not(:disabled){background:var(--accent);opacity:.9;color:var(--on-accent)}}
  .repo-btn{font-size:12.5px;padding:6px 10px;color:var(--muted)}
  .update-box{margin-top:8px;padding:10px 12px;border-radius:var(--radius-control);background:var(--surface-soft);border:1px solid var(--edge);font-size:12.5px;color:var(--muted)}
  .update-box.has-update{border-color:var(--accent);background:var(--accent-soft);color:var(--text)}
  .update-head{display:flex;align-items:center;justify-content:space-between;gap:10px;flex-wrap:wrap}
  .link-btn{font-size:12px;padding:4px 9px}
  .release-notes{margin:8px 0 0;white-space:pre-wrap;line-height:1.55;color:var(--text);font-size:12.5px}
  .row-status{margin:6px 0 2px;font-size:12.5px;color:var(--muted)}
  .row-alert{display:flex;align-items:center;gap:10px;margin:6px 0 2px;font-size:12.5px;color:var(--danger)}
  .reset-bar{margin-top:16px;display:flex;justify-content:flex-start}
  @media(max-width:520px){.general-settings{padding-inline:0}}
</style>
