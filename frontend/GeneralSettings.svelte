<script lang="ts">
  import {DEFAULT_GENERAL_PREFERENCES, type GeneralPreferences} from './lib/preferences';
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

  function toggleAutostart(event: Event & {currentTarget: HTMLInputElement}) {
    const requested = event.currentTarget.checked;
    event.currentTarget.checked = enabled === true;
    onToggle(requested);
  }
</script>

<section aria-label="常规" class="general-settings">
  <h1>常规</h1>

  <div class="group-card">
    <h2>启动与窗口</h2>
    {#if desktop}
      <div class="setting-row">
        <div class="setting-label">
          <label for="autostart-toggle">开机自启动</label>
          <details class="hint-disclosure inline-hint"><summary>说明</summary><div class="hint-disclosure-body"><p>登录桌面后在后台运行并显示托盘图标，主窗口保持隐藏。</p></div></details>
        </div>
        <input id="autostart-toggle" type="checkbox" class="toggle-check" checked={enabled === true} disabled={busy || enabled === null} onchange={toggleAutostart} />
      </div>
      {#if busy}
        <p class="row-status" role="status">{enabled === null ? '正在读取设置…' : '正在保存设置…'}</p>
      {:else if error}
        <div class="row-alert" role="alert"><span>{error}</span>{#if enabled === null}<button type="button" onclick={onReload}>重新读取</button>{/if}</div>
      {/if}
    {/if}

    <div class="setting-row">
      <label class="row-text" for="esc-hide-toggle">按 Esc 键收起窗口</label>
      <input id="esc-hide-toggle" type="checkbox" class="toggle-check" checked={prefs.escToHide} onchange={e => onPrefsChange({escToHide: e.currentTarget.checked})} />
    </div>

    <div class="setting-row">
      <label class="row-text" for="welcome-mark-toggle">显示欢迎页背景水印</label>
      <input id="welcome-mark-toggle" type="checkbox" class="toggle-check" checked={prefs.showWelcomeMark} onchange={e => onPrefsChange({showWelcomeMark: e.currentTarget.checked})} />
    </div>
  </div>

  <div class="group-card">
    <h2>输入与动效</h2>
    <div class="setting-row stacked">
      <span class="row-text">发送快捷键</span>
      <div class="segmented" role="radiogroup" aria-label="发送快捷键">
        <label class:active={prefs.sendKey === 'enter'}><input type="radio" name="send-key" value="enter" checked={prefs.sendKey === 'enter'} onchange={() => onPrefsChange({sendKey: 'enter'})} /><span>Enter 发送</span></label>
        <label class:active={prefs.sendKey === 'ctrl-enter'}><input type="radio" name="send-key" value="ctrl-enter" checked={prefs.sendKey === 'ctrl-enter'} onchange={() => onPrefsChange({sendKey: 'ctrl-enter'})} /><span>Ctrl + Enter 发送</span></label>
      </div>
    </div>

    <div class="setting-row stacked">
      <span class="row-text">界面切换动效</span>
      <div class="segmented" role="radiogroup" aria-label="界面切换动效">
        <label class:active={prefs.motionSpeed === 'expressive'}><input type="radio" name="motion-speed" value="expressive" checked={prefs.motionSpeed === 'expressive'} onchange={() => onPrefsChange({motionSpeed: 'expressive'})} /><span>舒展</span></label>
        <label class:active={prefs.motionSpeed === 'snappy'}><input type="radio" name="motion-speed" value="snappy" checked={prefs.motionSpeed === 'snappy'} onchange={() => onPrefsChange({motionSpeed: 'snappy'})} /><span>轻快</span></label>
        <label class:active={prefs.motionSpeed === 'off'}><input type="radio" name="motion-speed" value="off" checked={prefs.motionSpeed === 'off'} onchange={() => onPrefsChange({motionSpeed: 'off'})} /><span>关闭</span></label>
      </div>
    </div>
  </div>

  {#if onResetAll}
    <div class="reset-bar">
      <button type="button" onclick={onResetAll}>恢复默认界面与常规偏好</button>
    </div>
  {/if}
</section>

<style>
  .general-settings{max-width:580px;margin:20px auto;padding:0 18px 28px;color:var(--text);font-size:14px}
  h1{margin:0 0 18px;font-size:20px;font-weight:600}
  h2{margin:0 0 8px;font-size:12.5px;font-weight:600;color:var(--muted);letter-spacing:.02em}
  .group-card{background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:14px 16px;margin-bottom:14px}
  .setting-row{display:flex;align-items:center;justify-content:space-between;gap:14px;padding:10px 0;border-top:1px solid var(--edge)}
  .group-card h2+.setting-row{border-top:0;padding-top:4px}
  .setting-row:last-child{padding-bottom:2px}
  .setting-row.stacked{flex-wrap:wrap}
  .setting-label{display:flex;align-items:center;gap:10px;min-width:0;flex-wrap:wrap}
  .setting-label label,.row-text{font-size:13.5px;font-weight:500;color:var(--text);cursor:pointer}
  .inline-hint{margin-top:0}
  .toggle-check{width:16px;height:16px;margin:0;accent-color:var(--accent);cursor:pointer;flex-shrink:0}
  .toggle-check:disabled{cursor:default}
  .segmented{display:inline-flex;gap:4px;padding:3px;border-radius:var(--radius-control);background:var(--surface-soft);border:1px solid var(--edge)}
  .segmented label{position:relative;display:inline-flex;align-items:center;padding:5px 11px;border-radius:calc(var(--radius-control) - 2px);font-size:12.5px;font-weight:500;color:var(--muted);cursor:pointer;transition:background-color 140ms ease,color 140ms ease}
  .segmented input{position:absolute;opacity:0;pointer-events:none}
  .segmented label.active{background:var(--surface);color:var(--text);box-shadow:var(--raised-shadow);font-weight:600}
  .segmented label:focus-within{outline:2px solid var(--focus-ring);outline-offset:1px}
  .row-status{margin:6px 0 2px;font-size:12.5px;color:var(--muted)}
  .row-alert{display:flex;align-items:center;gap:10px;margin:6px 0 2px;font-size:12.5px;color:var(--danger)}
  .reset-bar{margin-top:16px;display:flex;justify-content:flex-start}
  @media(max-width:520px){.general-settings{padding-inline:0}}
</style>
