<script lang="ts">
 import {tick, type Snippet} from 'svelte';
 import {t, type I18nKey} from './lib/i18n.svelte';

 export type SettingsCategory = 'general' | 'provider' | 'appearance' | 'selection' | 'shortcuts' | 'archive';
 let {category=$bindable<SettingsCategory>('provider'), desktop=false, hasReasoning=false, generalPanel, appearancePanel, providerPanel, selectionPanel, shortcutsPanel, archivePanel}:{
   category?: SettingsCategory;
   desktop?: boolean;
   hasReasoning?: boolean;
   generalPanel: Snippet;
   appearancePanel: Snippet;
   providerPanel: Snippet;
   selectionPanel: Snippet;
   shortcutsPanel?: Snippet;
   archivePanel?: Snippet;
 } = $props();
 const categories=$derived([
  {id:'general' as const,label:t('settings.general'),detail:t('settings.generalDetail')},
  {id:'provider' as const,label:t('settings.provider'),detail:t('settings.providerDetail')},
  {id:'appearance' as const,label:t('settings.appearance'),detail:t('settings.appearanceDetail')},
  {id:'selection' as const,label:t('settings.selection'),detail:t('settings.selectionDetail')},
  ...(shortcutsPanel ? [{id:'shortcuts' as const,label:t('settings.shortcuts'),detail:t('settings.shortcutsDetail')}] : []),
  {id:'archive' as const,label:t('settings.archive'),detail:t('settings.archiveDetail')},
 ]);
 type SettingEntry = {category: SettingsCategory; target: string; key: I18nKey; aliases?: string; desktop?: boolean};
 const settingIndex: SettingEntry[] = [
   {category:'general',target:'interface-locale',key:'general.language',aliases:'语种 locale system 跟随系统'},
   {category:'general',target:'autostart-toggle',key:'general.autostart',desktop:true},
   {category:'general',target:'esc-hide-toggle',key:'general.escHide'},
   {category:'general',target:'confirm-close-toggle',key:'general.confirmRiskyClose',aliases:'关闭前确认 quit exit close',desktop:true},
   {category:'general',target:'home-suggestions-toggle',key:'general.homeSuggestions',aliases:'首页建议 home suggestions'},
   {category:'general',target:'welcome-mark-toggle',key:'general.welcomeMark'},
   {category:'general',target:'send-key-setting',key:'general.sendKey',aliases:'enter ctrl'},
   {category:'general',target:'motion-setting',key:'general.motionSpeed'},
   {category:'general',target:'change-export-directory',key:'general.exportDirectory',aliases:'保存位置 export folder',desktop:true},
   {category:'general',target:'quick-surface',key:'general.quickSurface',aliases:'胶囊 capsule tray',desktop:true},
   {category:'general',target:'background-notification-toggle',key:'general.backgroundNotifications',desktop:true},
   {category:'general',target:'keep-awake-toggle',key:'general.keepAwake',aliases:'睡眠 suspend sleep inhibitor',desktop:true},
   {category:'general',target:'check-update-setting',key:'general.checkUpdate'},
   {category:'general',target:'licenses-setting',key:'settings.notices',aliases:'开源 license notices'},
   {category:'provider',target:'api-root',key:'provider.apiRoot',aliases:'api endpoint'},
   {category:'provider',target:'model-name',key:'provider.modelName'},
   {category:'provider',target:'image-model',key:'provider.imageModel'},
   {category:'provider',target:'reasoning-effort',key:'settings.reasoning',aliases:'推理档位 推理强度 reasoning'},
   {category:'provider',target:'api-secret',key:'provider.apiSecret',aliases:'api key token',desktop:true},
   {category:'appearance',target:'theme-setting',key:'appearance.themeLegend',aliases:'主题 深色 light dark theme'},
   {category:'appearance',target:'custom-font',key:'appearance.font'},
   {category:'appearance',target:'font-size-setting',key:'appearance.fontSize'},
   {category:'appearance',target:'reading-width-setting',key:'appearance.readingWidth'},
   {category:'selection',target:'candidate-toggle',key:'selection.autoMenu',desktop:true},
   {category:'selection',target:'quick-actions-setting',key:'quick.heading',desktop:true},
   {category:'shortcuts',target:'niri-shortcuts',key:'shortcuts.global',aliases:'截图 screenshot niri 全局快捷键'},
   {category:'archive',target:'auto-archive-days',key:'archive.inactive',aliases:'自动归档 archive'},
 ];
 type SearchResult = {category: SettingsCategory; target: string; label: string; text: string};
 let query = $state('');
 const results = $derived.by(() => {
   const terms = query.normalize('NFKC').toLocaleLowerCase().trim().split(/\s+/).filter(Boolean);
   if (!terms.length) return [];
   const categoryResults: SearchResult[] = categories.map(item => ({
     category:item.id, target:`settings-tab-${item.id}`, label:item.label, text:`${item.label} ${item.detail}`,
   }));
   const controlResults: SearchResult[] = settingIndex
     .filter(item => (!item.desktop || desktop) && (item.target !== 'reasoning-effort' || hasReasoning)
       && (item.category !== 'shortcuts' || !!shortcutsPanel))
     .map(item => ({category:item.category,target:item.target,label:t(item.key),text:`${t(item.key)} ${item.aliases ?? ''}`}));
   return [...controlResults, ...categoryResults].filter(item => {
     const haystack = item.text.normalize('NFKC').toLocaleLowerCase();
     return terms.every(term => haystack.includes(term));
   }).slice(0, 10);
 });

 async function selectResult(result: SearchResult) {
   category = result.category;
   query = '';
   await tick();
   const fallback = document.getElementById(`settings-tab-${result.category}`);
   const target = document.getElementById(result.target) ?? fallback;
   if (target?.id === 'licenses-setting') target.querySelector('details')?.setAttribute('open', '');
   target?.scrollIntoView?.({block: 'nearest'});
   target?.focus();
   if (target !== document.activeElement) fallback?.focus();
 }

 function searchKeydown(event: KeyboardEvent) {
   if (event.isComposing) return;
   if (event.key === 'Enter' && results[0]) {
     event.preventDefault();
     event.stopPropagation();
     void selectResult(results[0]);
   }
   if (event.key === 'Escape' && query) {
     event.preventDefault();
     event.stopPropagation();
     query = '';
   }
 }

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
 <nav aria-label={t('settings.categories')}>
  <input type="search" id="settings-search" aria-label={t('settings.search')} placeholder={t('settings.search')}
    bind:value={query} onkeydown={searchKeydown} />
  {#if query.trim()}
    {#if results.length}
      <ul class="search-results">
        {#each results as result (result.target)}
          <li><button type="button" data-setting-result={result.target} onclick={() => void selectResult(result)}>
            <strong>{result.label}</strong>
            <small>{categories.find(item => item.id === result.category)?.label}</small>
          </button></li>
        {/each}
      </ul>
    {:else}<p role="status" class="no-results">{t('settings.noMatches')}</p>{/if}
  {/if}
  <div class="category-list" role="tablist" aria-label={t('settings.categories')}>
  {#each categories as item,index}<button id="settings-tab-{item.id}" type="button" role="tab" title={item.detail} aria-selected={category===item.id} aria-controls="settings-panel-{item.id}" tabindex={category===item.id?0:-1} onclick={()=>category=item.id} onkeydown={event=>move(event,index)}><strong>{item.label}</strong></button>{/each}
 </div></nav>
 <div class="settings-panels">
  {#each categories as item}<div class="settings-panel" id="settings-panel-{item.id}" role="tabpanel" aria-labelledby="settings-tab-{item.id}" tabindex="0" hidden={category!==item.id}>
   {#if item.id==='general'}{@render generalPanel()}{:else if item.id==='appearance'}{@render appearancePanel()}{:else if item.id==='provider'}{@render providerPanel()}{:else if item.id==='selection'}{@render selectionPanel()}{:else if item.id==='shortcuts' && shortcutsPanel}{@render shortcutsPanel()}{:else if archivePanel}{@render archivePanel()}{/if}
  </div>{/each}
 </div>
</div>
<style>
 .settings-hub{display:grid;grid-template-columns:182px minmax(0,1fr);gap:24px;max-width:980px;margin:0 auto;height:100%;min-height:0;color:var(--text);font-size:1rem}nav{padding-top:20px}.category-list{display:flex;flex-direction:column;gap:4px}button{border:1px solid transparent;border-radius:var(--radius-control);background:transparent;color:var(--muted);text-align:left;padding:9px 12px;font:inherit;cursor:pointer;transition:transform 140ms var(--ease-out),background-color 140ms ease,color 140ms ease,border-color 140ms ease}strong{display:block;font-size:.96rem;font-weight:500}@media(hover:hover) and (pointer:fine){button:hover{background:var(--surface-hover);color:var(--text)}}button:active:not(:disabled){transform:scale(0.97)}button[aria-selected=true]{background:var(--accent-soft);color:var(--text);border-color:transparent;font-weight:600;transform:none}button[aria-selected=true] strong{font-weight:600}button:focus-visible,.settings-panel:focus-visible{outline:2px solid var(--focus-ring);outline-offset:2px}.settings-panels{min-width:0;min-height:0}.settings-panel{height:100%;overflow:auto;overscroll-behavior:contain;scrollbar-gutter:stable;padding:0 8px;box-sizing:border-box}.settings-panel[hidden]{display:none}
 nav{min-height:0;overflow:auto;scrollbar-width:thin}
 #settings-search{width:100%;box-sizing:border-box;margin:0 0 8px;padding:7px 9px;background:var(--surface);color:var(--text);border:1px solid var(--edge);border-radius:var(--radius-control);font:inherit;font-size:.9rem}
 #settings-search:focus-visible{outline:2px solid var(--focus-ring);outline-offset:-2px}
 .search-results{list-style:none;margin:0 0 10px;padding:3px;border:1px solid var(--edge);border-radius:var(--radius-control);max-height:236px;overflow:auto}
 .search-results button{display:block;width:100%;white-space:normal;overflow-wrap:anywhere}
 .search-results small,.no-results{font-size:.78rem;color:var(--muted)}
 .no-results{padding:0 6px}
 @media(max-width:760px){.settings-hub{grid-template-columns:minmax(0,1fr);grid-template-rows:auto minmax(0,1fr);gap:12px}nav{padding-top:10px;min-width:0;max-height:42vh}.category-list{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:6px;padding:2px}button{flex-shrink:0;padding:8px 12px}.settings-panel{padding:0 2px}}@media(prefers-reduced-motion:reduce){button{transition-property:background-color,color,opacity;transition-duration:120ms;transform:none}}
</style>
