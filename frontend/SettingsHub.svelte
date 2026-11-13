<script lang="ts">
 import type {Snippet} from 'svelte';
 export type SettingsCategory='provider'|'appearance'|'selection'|'quick';
 let {category=$bindable<SettingsCategory>('provider'),appearancePanel,providerPanel,selectionPanel,quickPanel}:{category?:SettingsCategory;appearancePanel:Snippet;providerPanel:Snippet;selectionPanel:Snippet;quickPanel:Snippet}=$props();
 const categories=[{id:'provider',label:'模型与凭据',detail:'服务地址、模型与密钥'},{id:'appearance',label:'外观',detail:'界面明暗与主题'},{id:'selection',label:'划词行为',detail:'自动划词菜单'},{id:'quick',label:'快捷操作',detail:'动作与输出目录'}] as const;
 function move(event:KeyboardEvent,index:number){
  let next=index;
  if(event.key==='ArrowDown'||event.key==='ArrowRight')next=(index+1)%categories.length;
  else if(event.key==='ArrowUp'||event.key==='ArrowLeft')next=(index+categories.length-1)%categories.length;
  else if(event.key==='Home')next=0;
  else if(event.key==='End')next=categories.length-1;
  else return;
  event.preventDefault();category=categories[next].id;document.getElementById(`settings-tab-${category}`)?.focus();
 }
</script>
<div class="settings-hub">
 <nav aria-label="设置分类"><div class="category-list" role="tablist" aria-label="设置分类">
  {#each categories as item,index}<button id="settings-tab-{item.id}" type="button" role="tab" aria-selected={category===item.id} aria-controls="settings-panel-{item.id}" tabindex={category===item.id?0:-1} onclick={()=>category=item.id} onkeydown={event=>move(event,index)}><strong>{item.label}</strong><small>{item.detail}</small></button>{/each}
 </div></nav>
 <div class="settings-panels">
  {#each categories as item}<div class="settings-panel" id="settings-panel-{item.id}" role="tabpanel" aria-labelledby="settings-tab-{item.id}" tabindex="0" hidden={category!==item.id}>
   {#if item.id==='appearance'}{@render appearancePanel()}{:else if item.id==='provider'}{@render providerPanel()}{:else if item.id==='selection'}{@render selectionPanel()}{:else}{@render quickPanel()}{/if}
  </div>{/each}
 </div>
</div>
<style>
 .settings-hub{display:grid;grid-template-columns:180px minmax(0,1fr);gap:24px;max-width:1040px;margin:0 auto;height:100%;min-height:0;color:var(--text)}nav{padding-top:24px}.category-list{display:flex;flex-direction:column;gap:8px}button{border:1px solid transparent;border-radius:14px;background:transparent;color:var(--muted);text-align:left;padding:14px;font:inherit;cursor:pointer;transition:transform 140ms var(--ease-out),background-color 160ms ease,color 160ms ease,box-shadow 160ms var(--ease-out)}strong{display:block;font-size:15px;font-weight:550}small{display:block;font-size:12px;margin-top:5px;line-height:1.5}@media(hover:hover) and (pointer:fine){button:hover{background:var(--surface)}}button:active:not(:disabled){transform:scale(0.97)}button[aria-selected=true]{background:var(--surface);color:var(--text);border-color:var(--edge);box-shadow:var(--inset-shadow);transform:none}button:focus-visible,.settings-panel:focus-visible{outline:2px solid var(--focus-ring);outline-offset:2px}.settings-panels{min-width:0;min-height:0}.settings-panel{height:100%;overflow:auto;overscroll-behavior:contain;scrollbar-gutter:stable;padding:0 8px;box-sizing:border-box}.settings-panel[hidden]{display:none}
 @media(max-width:760px){.settings-hub{grid-template-columns:minmax(0,1fr);grid-template-rows:auto minmax(0,1fr);gap:12px}nav{padding-top:10px;min-width:0}.category-list{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:6px;padding:3px}button{flex-shrink:0;padding:10px 14px}small{display:none}.settings-panel{padding:0 3px}}@media(prefers-reduced-motion:reduce){button{transition-property:background-color,color,opacity;transition-duration:120ms;transform:none}}
</style>
