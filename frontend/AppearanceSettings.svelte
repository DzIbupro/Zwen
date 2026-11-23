<script lang="ts">
 import Icon from './Icon.svelte';
 import {DEFAULT_CUSTOM_APPEARANCE, type AppearanceState, type ColorPalette, type CustomAppearance, type FontPreset, type FontSizePreset, type RadiusPreset, type WidthPreset} from './lib/appearance';
 let {
  state,
  palette='monochrome',
  custom=DEFAULT_CUSTOM_APPEARANCE,
  onTheme,
  onPalette=()=>{},
  onCustom=()=>{},
  onResetCustom=()=>{},
  onReload,
 }:{
  state:AppearanceState;
  palette?:ColorPalette;
  custom?:CustomAppearance;
  onTheme:(theme:'light'|'dark'|'system')=>void;
  onPalette?:(palette:ColorPalette)=>void;
  onCustom?:(patch:Partial<CustomAppearance>)=>void;
  onResetCustom?:()=>void;
  onReload:()=>void;
 }=$props();

 const palettes=[
  {id:'monochrome',label:'极简黑白',swatches:['#131313','#f7f7f8','#ffffff']},
  {id:'slate',label:'冷静石板',swatches:['#111318','#f3f5f8','#2563eb']},
  {id:'paper',label:'暖纸墨炭',swatches:['#171615','#faf9f6','#655f56']},
 ] as const;

 const accentSwatches=[
  {label:'方案默认',value:null},
  {label:'石板蓝',value:'#2563eb'},
  {label:'翠绿',value:'#16a34a'},
  {label:'琥珀橙',value:'#d97706'},
  {label:'玫瑰红',value:'#e11d48'},
  {label:'靛紫',value:'#7c3aed'},
 ] as const;

 const fonts:{id:FontPreset;label:string}[]=[
  {id:'default',label:'系统推荐（Red Hat + Noto）'},
  {id:'noto-sans',label:'Noto Sans（无衬线）'},
  {id:'noto-serif',label:'Noto Serif（衬线阅读）'},
  {id:'open-sans',label:'Open Sans（现代清晰）'},
  {id:'mono',label:'Source Code Pro（等宽）'},
 ];

 const fontSizes:{id:FontSizePreset;label:string}[]=[
  {id:'compact',label:'紧凑'},
  {id:'normal',label:'标准'},
  {id:'comfortable',label:'舒适'},
  {id:'large',label:'大号'},
 ];

 const radii:{id:RadiusPreset;label:string}[]=[
  {id:'sharp',label:'利落'},
  {id:'normal',label:'标准'},
  {id:'soft',label:'柔和'},
 ];

 const widths:{id:WidthPreset;label:string}[]=[
  {id:'focused',label:'专注'},
  {id:'normal',label:'标准'},
  {id:'wide',label:'宽幅'},
 ];

 const hasCustom=$derived(
  custom.accent!==null||custom.font!=='default'||custom.fontSize!=='normal'||custom.radius!=='normal'||custom.readingWidth!=='normal'||custom.customBg!==null||custom.customSidebar!==null
 );
</script>

<section class="appearance-settings" aria-label="外观设置" data-appearance-settings>
 <div class="head-row">
  <h2>外观</h2>
  {#if hasCustom}<button type="button" class="reset-btn" onclick={onResetCustom}>恢复默认外观</button>{/if}
 </div>

 <div class="group-card">
  <fieldset disabled={!state.loaded}><legend>界面明暗</legend><div class="theme-options">
   <label class:selected={state.theme==='light'}><input type="radio" name="interface-theme" value="light" checked={state.theme==='light'} onchange={()=>onTheme('light')}/><Icon name="sun" size={15}/><span>浅色</span></label>
   <label class:selected={state.theme==='dark'}><input type="radio" name="interface-theme" value="dark" checked={state.theme==='dark'} onchange={()=>onTheme('dark')}/><Icon name="moon" size={15}/><span>深色</span></label>
   <label class:selected={state.theme==='system'}><input type="radio" name="interface-theme" value="system" checked={state.theme==='system'} onchange={()=>onTheme('system')}/><Icon name="settings" size={15}/><span>跟随系统</span></label>
  </div></fieldset>

  <fieldset class="palette-group"><legend>预设配色</legend><div class="theme-options">
   {#each palettes as item}
    <label class:selected={palette===item.id}><input type="radio" name="interface-palette" value={item.id} checked={palette===item.id} onchange={()=>onPalette(item.id)}/><span class="swatches" aria-hidden="true">{#each item.swatches as color}<i style={`background:${color}`}></i>{/each}</span><span>{item.label}</span></label>
   {/each}
  </div></fieldset>
 </div>

 <div class="group-card">
  <h3>自定义外观</h3>

  <div class="custom-row">
   <span class="custom-label">强调色</span>
   <div class="accent-bar">
    {#each accentSwatches as item}
     <button type="button" class="swatch-btn" class:active={custom.accent===item.value} title={item.label} aria-label={item.label} aria-pressed={custom.accent===item.value} onclick={()=>onCustom({accent:item.value})}>
      {#if item.value}<i style={`background:${item.value}`}></i>{:else}<span class="auto-dot">默认</span>{/if}
     </button>
    {/each}
    <label class="color-picker-btn" title="自定义强调色">
     <input type="color" aria-label="自定义强调色" value={custom.accent??'#2563eb'} oninput={e=>onCustom({accent:e.currentTarget.value})}/>
     <span>自选</span>
    </label>
   </div>
  </div>

  <div class="custom-row">
   <label class="custom-label" for="custom-font">界面字体</label>
   <select id="custom-font" value={custom.font} onchange={e=>onCustom({font:e.currentTarget.value as FontPreset})}>
    {#each fonts as item}<option value={item.id}>{item.label}</option>{/each}
   </select>
  </div>

  <div class="custom-row">
   <span class="custom-label">阅读字号</span>
   <div class="segmented" role="radiogroup" aria-label="阅读字号">
    {#each fontSizes as item}
     <label class:active={custom.fontSize===item.id}><input type="radio" name="font-size" value={item.id} checked={custom.fontSize===item.id} onchange={()=>onCustom({fontSize:item.id})}/><span>{item.label}</span></label>
    {/each}
   </div>
  </div>

  <div class="custom-row">
   <span class="custom-label">界面圆角</span>
   <div class="segmented" role="radiogroup" aria-label="界面圆角">
    {#each radii as item}
     <label class:active={custom.radius===item.id}><input type="radio" name="ui-radius" value={item.id} checked={custom.radius===item.id} onchange={()=>onCustom({radius:item.id})}/><span>{item.label}</span></label>
    {/each}
   </div>
  </div>

  <div class="custom-row">
   <span class="custom-label">阅读宽度</span>
   <div class="segmented" role="radiogroup" aria-label="阅读宽度">
    {#each widths as item}
     <label class:active={custom.readingWidth===item.id}><input type="radio" name="reading-width" value={item.id} checked={custom.readingWidth===item.id} onchange={()=>onCustom({readingWidth:item.id})}/><span>{item.label}</span></label>
    {/each}
   </div>
  </div>

  <details class="advanced-colors">
   <summary>自定义画布与侧栏底色</summary>
   <div class="advanced-grid">
    <div class="color-field">
     <span>主画布背景</span>
     <div class="color-controls">
      <input type="color" aria-label="主画布背景色" value={custom.customBg??'#131313'} oninput={e=>onCustom({customBg:e.currentTarget.value})}/>
      {#if custom.customBg}<button type="button" class="mini-clear" onclick={()=>onCustom({customBg:null})}>重置</button>{/if}
     </div>
    </div>
    <div class="color-field">
     <span>侧栏背景</span>
     <div class="color-controls">
      <input type="color" aria-label="侧栏背景色" value={custom.customSidebar??'#1a1a1a'} oninput={e=>onCustom({customSidebar:e.currentTarget.value})}/>
      {#if custom.customSidebar}<button type="button" class="mini-clear" onclick={()=>onCustom({customSidebar:null})}>重置</button>{/if}
     </div>
    </div>
   </div>
  </details>
 </div>

 {#if state.status==='saving'}<small role="status">正在保存外观偏好…</small>{/if}
 {#if state.error}<div class="appearance-error" role="alert">{state.error}<button onclick={onReload}>重新读取</button></div>{/if}
</section>

<style>
 .appearance-settings{max-width:580px;margin:20px auto;padding:0 18px 28px}
 .head-row{display:flex;align-items:center;justify-content:space-between;gap:12px;margin-bottom:16px}
 .appearance-settings h2{font-size:20px;font-weight:600;margin:0;text-wrap:balance}
 .reset-btn{font-size:12.5px;padding:5px 10px;color:var(--muted)}
 .group-card{background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:14px 16px;margin-bottom:14px}
 h3{margin:0 0 8px;font-size:12.5px;font-weight:600;color:var(--muted);letter-spacing:.02em}
 fieldset{border:0;padding:0;margin:0}
 fieldset+fieldset{margin-top:16px;padding-top:14px;border-top:1px solid var(--edge)}
 legend{font-size:12.5px;font-weight:600;color:var(--muted);margin-bottom:10px;padding:0}
 .theme-options{display:flex;gap:8px;flex-wrap:wrap}
 .theme-options label{display:flex;align-items:center;gap:8px;background:var(--surface-soft);border:1px solid var(--edge);border-radius:var(--radius-control);padding:8px 13px;cursor:pointer;font-size:13px;font-weight:500;transition:transform 140ms var(--ease-out),background-color 140ms ease,border-color 140ms ease}
 @media(hover:hover) and (pointer:fine){.theme-options label:hover{background:var(--surface-hover);border-color:var(--edge-strong)}}
 .theme-options label:active{transform:scale(0.97)}
 .theme-options label.selected{background:var(--accent-soft);border-color:var(--accent);color:var(--text);font-weight:600;transform:none}
 .theme-options input{accent-color:var(--accent);width:14px;height:14px;margin:0}
 .theme-options label:focus-within{outline:2px solid var(--focus-ring);outline-offset:2px}
 fieldset:disabled label{opacity:.45;cursor:default;transform:none}
 .swatches{display:inline-flex;align-items:center;gap:3px}
 .swatches i{width:10px;height:10px;border-radius:50%;border:1px solid var(--edge-strong);display:inline-block}
 .custom-row{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:10px 0;border-top:1px solid var(--edge);flex-wrap:wrap}
 .group-card h3+.custom-row{border-top:0;padding-top:4px}
 .custom-label{font-size:13.5px;font-weight:500;color:var(--text)}
 .accent-bar{display:flex;align-items:center;gap:6px;flex-wrap:wrap}
 .swatch-btn{padding:4px;min-width:28px;height:28px;border-radius:999px;display:inline-grid;place-items:center;background:var(--surface-soft);border:1px solid var(--edge)}
 .swatch-btn i{width:16px;height:16px;border-radius:50%;display:block}
 .swatch-btn.active{border-color:var(--accent);box-shadow:0 0 0 1px var(--accent)}
 .auto-dot{font-size:11.5px;padding:0 5px;color:var(--text)}
 .color-picker-btn{display:inline-flex;align-items:center;gap:5px;padding:3px 9px 3px 5px;border-radius:999px;border:1px solid var(--edge);background:var(--surface-soft);font-size:12px;color:var(--muted);cursor:pointer}
 .color-picker-btn input{width:18px;height:18px;border:0;padding:0;background:transparent;cursor:pointer}
 select{border:1px solid var(--edge);border-radius:var(--radius-control);background:var(--surface-soft);color:var(--text);padding:6px 10px;font:inherit;font-size:13px;max-width:240px}
 .segmented{display:inline-flex;gap:3px;padding:3px;border-radius:var(--radius-control);background:var(--surface-soft);border:1px solid var(--edge)}
 .segmented label{position:relative;display:inline-flex;align-items:center;padding:4px 10px;border-radius:calc(var(--radius-control) - 2px);font-size:12.5px;font-weight:500;color:var(--muted);cursor:pointer;transition:background-color 140ms ease,color 140ms ease}
 .segmented input{position:absolute;opacity:0;pointer-events:none}
 .segmented label.active{background:var(--surface);color:var(--text);box-shadow:var(--raised-shadow);font-weight:600}
 .segmented label:focus-within{outline:2px solid var(--focus-ring);outline-offset:1px}
 .advanced-colors{margin-top:8px;padding-top:10px;border-top:1px solid var(--edge)}
 .advanced-colors>summary{font-size:12.5px;font-weight:500;color:var(--muted);cursor:pointer;user-select:none}
 .advanced-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px;margin-top:10px}
 .color-field{display:flex;align-items:center;justify-content:space-between;gap:8px;padding:8px 10px;border-radius:var(--radius-control);background:var(--surface-soft);border:1px solid var(--edge);font-size:12.5px}
 .color-controls{display:flex;align-items:center;gap:6px}
 .color-controls input{width:24px;height:22px;border:0;padding:0;background:transparent;cursor:pointer}
 .mini-clear{font-size:11.5px;padding:2px 7px}
 small{display:block;font-size:12.5px;color:var(--muted);margin-top:10px}
 .appearance-error{color:var(--danger);font-size:13px;margin-top:12px}
 .appearance-error button{margin-left:10px}
 @media(max-width:520px){.appearance-settings{padding-inline:0}.advanced-grid{grid-template-columns:1fr}}
</style>
