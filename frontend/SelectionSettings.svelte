<script lang="ts">
 import type {Snippet} from 'svelte';
 let {desktop,enabled,onToggle,children}:{desktop:boolean;enabled:boolean;onToggle:(on:boolean)=>void;children?:Snippet}=$props();
</script>
<section aria-label="划词与快捷动作" class="selection-settings">
 <h1>划词与快捷动作</h1>
 <div class="group-card">
  <h2>触发方式</h2>
  {#if desktop}
   <div class="setting-row">
    <div class="setting-label">
     <label for="candidate-toggle">自动划词菜单</label>
     <details class="hint-disclosure inline-hint"><summary>说明</summary><div class="hint-disclosure-body"><p>选择文字后，在支持选区的应用中弹出操作菜单；点击翻译、解释或自定义动作后才发送。</p></div></details>
    </div>
    <input id="candidate-toggle" type="checkbox" class="toggle-check" checked={enabled} onchange={e=>onToggle(e.currentTarget.checked)}/>
   </div>
  {:else}
   <p class="preview-note">请在桌面应用中设置自动划词与快捷动作。</p>
  {/if}
 </div>
 {#if children}{@render children()}{/if}
</section>
<style>
 .selection-settings{max-width:580px;margin:20px auto;padding:0 18px 28px;color:var(--text);font-size:14px}
 h1{font-size:20px;font-weight:600;margin:0 0 18px}
 h2{margin:0 0 6px;font-size:12.5px;font-weight:600;color:var(--muted);letter-spacing:.02em}
 .group-card{background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:14px 16px;margin-bottom:14px}
 .setting-row{display:flex;align-items:center;justify-content:space-between;gap:14px;padding:6px 0 2px}
 .setting-label{display:flex;align-items:center;gap:10px;min-width:0;flex-wrap:wrap}
 .setting-label label{font-size:13.5px;font-weight:500;color:var(--text);cursor:pointer}
 .inline-hint{margin-top:0}
 .toggle-check{width:16px;height:16px;margin:0;accent-color:var(--accent);cursor:pointer;flex-shrink:0}
 .preview-note{margin:6px 0 0;font-size:13px;color:var(--muted)}
 @media(max-width:520px){.selection-settings{padding-inline:0}}
</style>
