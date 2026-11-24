export type ColorPalette='monochrome'|'slate'|'paper';
export type FontPreset='default'|'noto-sans'|'open-sans'|'humanist'|'noto-serif'|'mono';
export type FontSizePreset='compact'|'normal'|'comfortable'|'large';
export type RadiusPreset='sharp'|'normal'|'soft';
export type WidthPreset='focused'|'normal'|'wide';
export interface CustomAppearance{
 accent:string|null;
 font:FontPreset;
 fontSize:FontSizePreset;
 radius:RadiusPreset;
 readingWidth:WidthPreset;
 customBg:string|null;
 customSidebar:string|null;
}
export const DEFAULT_CUSTOM_APPEARANCE:CustomAppearance={
 accent:null,
 font:'default',
 fontSize:'normal',
 radius:'normal',
 readingWidth:'normal',
 customBg:null,
 customSidebar:null,
};
export const DEFAULT_PALETTE:ColorPalette='monochrome';
export const PALETTE_KEY='zwen.palette.v1';
export const CUSTOM_APPEARANCE_KEY='zwen.custom-appearance.v1';
const HEX_COLOR=/^#[0-9a-fA-F]{6}$/;
export function parsePalette(value:unknown):ColorPalette{return value==='slate'||value==='paper'||value==='monochrome'?value:DEFAULT_PALETTE;}
export function loadPalette(storage:Pick<Storage,'getItem'>=localStorage):ColorPalette{try{return parsePalette(storage.getItem(PALETTE_KEY));}catch{return DEFAULT_PALETTE;}}
export function savePalette(palette:ColorPalette,storage:Pick<Storage,'setItem'>=localStorage){try{storage.setItem(PALETTE_KEY,palette);}catch{/* ignore storage errors */}}
export function applyPalette(palette:ColorPalette){document.documentElement.dataset.palette=parsePalette(palette);}
export function parseCustomAppearance(value:unknown):CustomAppearance{
 if(!value||typeof value!=='object')return{...DEFAULT_CUSTOM_APPEARANCE};
 const v=value as Record<string,unknown>;
 return{
  accent:typeof v.accent==='string'&&HEX_COLOR.test(v.accent)?v.accent:null,
  font:v.font==='noto-sans'||v.font==='noto-serif'||v.font==='open-sans'||v.font==='humanist'||v.font==='mono'?v.font:'default',
  fontSize:v.fontSize==='compact'||v.fontSize==='comfortable'||v.fontSize==='large'?v.fontSize:'normal',
  radius:v.radius==='sharp'||v.radius==='soft'?v.radius:'normal',
  readingWidth:v.readingWidth==='focused'||v.readingWidth==='wide'?v.readingWidth:'normal',
  customBg:typeof v.customBg==='string'&&HEX_COLOR.test(v.customBg)?v.customBg:null,
  customSidebar:typeof v.customSidebar==='string'&&HEX_COLOR.test(v.customSidebar)?v.customSidebar:null,
 };
}
export function loadCustomAppearance(storage:Pick<Storage,'getItem'>=localStorage):CustomAppearance{try{return parseCustomAppearance(JSON.parse(storage.getItem(CUSTOM_APPEARANCE_KEY)??'null'));}catch{return{...DEFAULT_CUSTOM_APPEARANCE};}}
export function saveCustomAppearance(custom:CustomAppearance,storage:Pick<Storage,'setItem'>=localStorage){try{storage.setItem(CUSTOM_APPEARANCE_KEY,JSON.stringify(custom));}catch{/* ignore storage errors */}}
function contrastInk(hex:string):string{
 const r=parseInt(hex.slice(1,3),16),g=parseInt(hex.slice(3,5),16),b=parseInt(hex.slice(5,7),16);
 return (0.299*r+0.587*g+0.114*b)>160?'#141416':'#ffffff';
}
export function applyCustomAppearance(custom:CustomAppearance){
 const root=document.documentElement;
 const s=root.style;
 const fonts:Record<FontPreset,string>={
  default:'"Red Hat Text","Noto Sans","Noto Sans CJK SC","Microsoft YaHei UI",system-ui,sans-serif',
  'noto-sans':'"Noto Sans","Noto Sans CJK SC","Microsoft YaHei UI",system-ui,sans-serif',
  'open-sans':'"Open Sans","Noto Sans CJK SC","Microsoft YaHei UI",system-ui,sans-serif',
  humanist:'"Cantarell","Inter","Ubuntu","Noto Sans CJK SC","Microsoft YaHei UI",system-ui,sans-serif',
  'noto-serif':'"Noto Serif","Noto Serif CJK SC","Source Han Serif SC",Georgia,serif',
  mono:'"Source Code Pro","JetBrains Mono","Noto Sans Mono CJK SC","Consolas",ui-monospace,monospace',
 };
 const sizes:Record<FontSizePreset,{scale:string;ui:string;prose:string}>={
  compact:{scale:'0.94',ui:'14.5px',prose:'15.5px'},
  normal:{scale:'1',ui:'16px',prose:'17.5px'},
  comfortable:{scale:'1.07',ui:'17px',prose:'19px'},
  large:{scale:'1.15',ui:'18.5px',prose:'20.5px'},
 };
 const radii:Record<RadiusPreset,[string,string]>={sharp:['5px','10px'],normal:['8px','16px'],soft:['12px','22px']};
 const widths:Record<WidthPreset,string>={focused:'680px',normal:'780px',wide:'940px'};
 root.dataset.fontSize=custom.fontSize;
 s.setProperty('--font-sans',fonts[custom.font]);
 s.setProperty('--font-scale',sizes[custom.fontSize].scale);
 s.setProperty('--ui-size',sizes[custom.fontSize].ui);
 s.setProperty('--prose-size',sizes[custom.fontSize].prose);
 s.setProperty('--radius-control',radii[custom.radius][0]);
 s.setProperty('--radius-card',radii[custom.radius][1]);
 s.setProperty('--reading-width',widths[custom.readingWidth]);
 if(custom.accent){
  s.setProperty('--accent',custom.accent);
  s.setProperty('--on-accent',contrastInk(custom.accent));
  s.setProperty('--focus-ring',custom.accent);
  s.setProperty('--accent-soft',`color-mix(in srgb,${custom.accent} 13%,transparent)`);
 }else{
  s.removeProperty('--accent');s.removeProperty('--on-accent');s.removeProperty('--focus-ring');s.removeProperty('--accent-soft');
 }
 if(custom.customBg){s.setProperty('--bg',custom.customBg);}else{s.removeProperty('--bg');}
 if(custom.customSidebar){s.setProperty('--sidebar-bg',custom.customSidebar);}else{s.removeProperty('--sidebar-bg');}
}
export interface Appearance{theme:'light'|'dark'|'system';sidebarCollapsed:boolean}
export interface AppearanceState extends Appearance{loaded:boolean;status:'loading'|'saving'|'saved'|'error';error:string|null}
export interface AppearancePort{load():Promise<Appearance>;save(preferences:Appearance):Promise<Appearance>}
export const DEFAULT_APPEARANCE:Appearance={theme:'light',sidebarCollapsed:false};
export const APPEARANCE_KEY='zwen.appearance.v1';
export function parseAppearance(value:unknown):Appearance|null{if(!value||typeof value!=='object')return null;const v=value as Record<string,unknown>;return(v.theme==='light'||v.theme==='dark'||v.theme==='system')&&typeof v.sidebarCollapsed==='boolean'?{theme:v.theme,sidebarCollapsed:v.sidebarCollapsed}:null;}
export function initialAppearance(native:unknown,storage:Pick<Storage,'getItem'>):Appearance{const n=parseAppearance(native);if(n)return n;try{return parseAppearance(JSON.parse(storage.getItem(APPEARANCE_KEY)??'null'))??{...DEFAULT_APPEARANCE};}catch{return{...DEFAULT_APPEARANCE};}}
export type ResolvedTheme='light'|'dark';
export function systemTheme():ResolvedTheme{const native=(window as Window & {__ZWEN_SYSTEM_THEME__?:ResolvedTheme}).__ZWEN_SYSTEM_THEME__;return native??(window.matchMedia?.('(prefers-color-scheme: dark)').matches?'dark':'light');}
export function applyAppearance(preferences:Appearance,system:ResolvedTheme=systemTheme()){const theme=preferences.theme==='system'?system:preferences.theme;document.documentElement.dataset.theme=theme;document.documentElement.style.colorScheme=theme;}
export function watchSystemTheme(current:()=>Appearance):()=>void{
 const native=(event:Event)=>{const theme=(event as CustomEvent).detail;if(theme!=='light'&&theme!=='dark')return;(window as Window & {__ZWEN_SYSTEM_THEME__?:ResolvedTheme}).__ZWEN_SYSTEM_THEME__=theme;applyAppearance(current());};
 const media=window.matchMedia?.('(prefers-color-scheme: dark)');const changed=()=>applyAppearance(current());
 window.addEventListener('zwen-system-theme',native);media?.addEventListener('change',changed);
 return()=>{window.removeEventListener('zwen-system-theme',native);media?.removeEventListener('change',changed);};
}
export class AppearanceController{
 state:AppearanceState;private stable:Appearance;private pending:Appearance|null=null;private worker:Promise<void>|null=null;private disposed=false;private loading=false;
 constructor(private port:AppearancePort,private changed:(s:AppearanceState)=>void,preset:Appearance=DEFAULT_APPEARANCE){this.stable={...preset};this.state={...preset,loaded:false,status:'loading',error:null};}
 private publish(p:Partial<AppearanceState>){if(this.disposed)return;this.state={...this.state,...p};this.changed(this.state);}
 async start(){if(this.worker||this.loading||this.disposed)return;this.loading=true;this.publish({loaded:false,status:'loading',error:null});try{const value=parseAppearance(await this.port.load());if(!value)throw Error('外观偏好格式无效，请重新读取');if(this.disposed)return;this.stable=value;this.publish({...value,loaded:true,status:'saved',error:null});}catch(e){this.publish({loaded:false,status:'error',error:e instanceof Error?e.message:'外观偏好读取失败'});}finally{this.loading=false;}}
 update(patch:Partial<Appearance>):Promise<void>{if(!this.state.loaded||this.disposed)return Promise.resolve();const desired=parseAppearance({...this.state,...patch});if(!desired)return Promise.resolve();this.pending=desired;this.publish({...desired,status:'saving',error:null});if(!this.worker)this.worker=this.drain().finally(()=>{this.worker=null;});return this.worker;}
 private async drain(){while(this.pending&&!this.disposed){const target=this.pending;this.pending=null;try{const saved=parseAppearance(await this.port.save(target));if(!saved)throw Error('保存的外观偏好无效');this.stable=saved;if(!this.pending)this.publish({...saved,status:'saved',error:null});}catch(e){const error=e instanceof Error?e.message:'外观偏好保存失败';if(!this.pending)this.publish({...this.stable,status:'error',error});else this.publish({error});}}}
 async flush(){await this.worker;}
 dispose(){this.disposed=true;this.pending=null;}
}
