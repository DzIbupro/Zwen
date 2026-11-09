<script lang="ts">
 let {desktop,enabled,onToggle}:{desktop:boolean;enabled:boolean;onToggle:(on:boolean)=>void}=$props();
</script>
<section aria-label="划词行为" class="selection-settings">
 <h1>划词行为</h1><p>选择文字后，随手翻译、解释或使用自定义动作。</p>
 {#if desktop}<label><input id="candidate-toggle" type="checkbox" checked={enabled} onchange={e=>onToggle(e.currentTarget.checked)}/><span>自动划词菜单<small>在支持选区的应用中弹出操作菜单；点击翻译、解释或自定义动作后才发送。</small></span></label>{:else}<p>请在桌面应用中设置自动划词菜单。</p>{/if}
</section>
<style>
 .selection-settings{max-width:620px;margin:28px auto;padding:0 18px 28px;color:var(--text)}h1{font-size:26px;font-weight:550;margin:0}p{color:var(--muted);font-size:16px;line-height:1.65}label{display:flex;gap:12px;align-items:flex-start;padding:20px 0;font-size:16px;cursor:pointer}input{accent-color:var(--accent);margin-top:5px;flex-shrink:0}small{display:block;color:var(--muted);font-size:14px;line-height:1.7;margin-top:8px}input:focus-visible{outline:2px solid var(--focus-ring);outline-offset:3px}@media(max-width:520px){.selection-settings{padding-inline:0}}
</style>
