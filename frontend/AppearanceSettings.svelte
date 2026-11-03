<script lang="ts">
 import Icon from './Icon.svelte';import type {AppearanceState} from './lib/appearance';
 let {state,onTheme,onReload}:{state:AppearanceState;onTheme:(theme:'light'|'dark')=>void;onReload:()=>void}=$props();
</script>
<section class="appearance-settings" aria-label="外观设置" data-appearance-settings>
 <h2>外观</h2><p>选择适合当前环境的界面明暗。</p>
 <fieldset disabled={!state.loaded}><legend class="sr-only">界面主题</legend><div class="theme-options">
  <label class:selected={state.theme==='light'}><input type="radio" name="interface-theme" value="light" checked={state.theme==='light'} onchange={()=>onTheme('light')}/><Icon name="sun"/><span>浅色</span></label>
  <label class:selected={state.theme==='dark'}><input type="radio" name="interface-theme" value="dark" checked={state.theme==='dark'} onchange={()=>onTheme('dark')}/><Icon name="moon"/><span>深色</span></label>
 </div></fieldset>
 {#if state.status==='saving'}<small role="status">正在保存外观偏好…</small>{/if}
 {#if state.error}<div class="appearance-error" role="alert">{state.error}<button onclick={onReload}>重新读取</button></div>{/if}
</section>
<style>
 .appearance-settings{max-width:620px;margin:24px auto 16px;padding:0 18px 24px;border-bottom:1px solid var(--edge)}.appearance-settings h2{font-size:20px;margin:0 0 8px}.appearance-settings p{font-size:14px;color:var(--muted);margin:0 0 20px}fieldset{border:0;padding:0;margin:0}.theme-options{display:flex;gap:16px;flex-wrap:wrap}.theme-options label{display:flex;align-items:center;gap:10px;background:var(--surface);border-radius:16px;padding:14px 24px;box-shadow:var(--raised-shadow);cursor:pointer;font-size:16px;transition:box-shadow .3s ease-in-out}.theme-options label:hover{box-shadow:var(--hover-shadow)}.theme-options label.selected{box-shadow:var(--inset-shadow);font-weight:600}.theme-options input{accent-color:var(--accent);width:16px;height:16px;margin:0}.theme-options label:focus-within{outline:2px solid var(--focus-ring);outline-offset:4px}fieldset:disabled label{opacity:.5;cursor:default}small{display:block;font-size:13px;color:var(--muted);margin-top:12px}.appearance-error{color:var(--danger);font-size:14px;margin-top:14px}.appearance-error button{margin-left:12px}
@media(max-width:520px){.appearance-settings{padding-inline:0}}</style>
