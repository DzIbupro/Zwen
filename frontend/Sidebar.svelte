<script lang="ts">
 import BrandLogo from './BrandLogo.svelte';import Icon from './Icon.svelte';
 export type Destination='chat'|'papers'|'history'|'knowledge'|'graph'|'settings';
 let {collapsed,active,disabled=false,canNew=true,onToggle,onNavigate,onNew}:{collapsed:boolean;active:Destination;disabled?:boolean;canNew?:boolean;onToggle:()=>void;onNavigate:(d:Destination)=>void;onNew:()=>void}=$props();
 const items=[{id:'chat',label:'对话',icon:'message'},{id:'papers',label:'论文阅读',icon:'book'},{id:'history',label:'历史',icon:'history'},{id:'knowledge',label:'知识库',icon:'book'},{id:'graph',label:'知识网络',icon:'network'}] as const;
</script>
<aside class="sidebar" class:collapsed data-sidebar data-collapsed={collapsed}>
 <div class="sidebar-brand"><BrandLogo/>{#if !collapsed}<strong>Zwen</strong>{/if}</div>
 <button class="sidebar-toggle" aria-label={collapsed?'展开侧栏':'收起侧栏'} aria-expanded={!collapsed} title={collapsed?'展开侧栏':'收起侧栏'} onclick={onToggle} {disabled}><Icon name={collapsed?'expand':'collapse'}/>{#if !collapsed}<span>收起侧栏</span>{/if}</button>
 <nav aria-label="主要导航">
  <button class="new-chat" data-new-conversation aria-label="新对话" title={collapsed?'新对话':undefined} disabled={!canNew} onclick={onNew}><Icon name="plus"/><span class:sr-only={collapsed}>新对话</span></button>
  {#each items as item}<button class:active={active===item.id} aria-current={active===item.id?'page':undefined} aria-label={item.label} title={collapsed?item.label:undefined} data-nav={item.id} onclick={()=>onNavigate(item.id)}><Icon name={item.icon}/><span class:sr-only={collapsed}>{item.label}</span></button>{/each}
  <button class="settings-link" class:active={active==='settings'} aria-current={active==='settings'?'page':undefined} aria-label="设置" title={collapsed?'设置':undefined} data-nav="settings" onclick={()=>onNavigate('settings')}><Icon name="settings"/><span class:sr-only={collapsed}>设置</span></button>
 </nav>
</aside>
