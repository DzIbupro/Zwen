export interface Appearance{theme:'light'|'dark';sidebarCollapsed:boolean}
export interface AppearanceState extends Appearance{loaded:boolean;status:'loading'|'saving'|'saved'|'error';error:string|null}
export interface AppearancePort{load():Promise<Appearance>;save(preferences:Appearance):Promise<Appearance>}
export const DEFAULT_APPEARANCE:Appearance={theme:'light',sidebarCollapsed:false};
export const APPEARANCE_KEY='zwen.appearance.v1';
export function parseAppearance(value:unknown):Appearance|null{if(!value||typeof value!=='object')return null;const v=value as Record<string,unknown>;return(v.theme==='light'||v.theme==='dark')&&typeof v.sidebarCollapsed==='boolean'?{theme:v.theme,sidebarCollapsed:v.sidebarCollapsed}:null;}
export function initialAppearance(native:unknown,storage:Pick<Storage,'getItem'>):Appearance{const n=parseAppearance(native);if(n)return n;try{return parseAppearance(JSON.parse(storage.getItem(APPEARANCE_KEY)??'null'))??{...DEFAULT_APPEARANCE};}catch{return{...DEFAULT_APPEARANCE};}}
export function applyAppearance(preferences:Appearance){document.documentElement.dataset.theme=preferences.theme;document.documentElement.style.colorScheme=preferences.theme;}
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
