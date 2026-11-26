export type ReasoningEffort = 'none'|'minimal'|'low'|'medium'|'high'|'xhigh'|'max';
export interface ProviderConfig {baseUrl:string;model:string;keyEnv:string;imageModel?:string;reasoningEffort?:ReasoningEffort}
export interface ProviderView {config:ProviderConfig|null;revision:number;keyAvailable:boolean;keySource?:'environment'|'keyring'|null;reasoningLevels?:readonly ReasoningEffort[]}
export interface ProviderPort {load():Promise<ProviderView>;save(revision:number,config:ProviderConfig):Promise<ProviderView>;saveSecret(value:string,expectedRevision:number):Promise<void>;clearSecret():Promise<void>}
type SettingsFields = Omit<ProviderConfig,'reasoningEffort'> & {reasoningEffort?:string};
export interface SettingsState extends ProviderView {fields:SettingsFields;status:'loading'|'saved'|'dirty'|'saving'|'error';loaded:boolean;error:string|null;secretSaving:boolean;secretNotice:string|null}
export function imageModelOf(config:ProviderConfig|null|undefined):string{return typeof config?.imageModel==='string'?config.imageModel:'';}
export function reasoningLevelsOf({config,fields,reasoningLevels}:SettingsState):readonly ReasoningEffort[]{
 const endpoint=fields.baseUrl.trim().replace(/\/+$/,'');
 return config?.baseUrl===endpoint&&config.model===fields.model.trim()?reasoningLevels??[]:[];
}
export class ProviderController {
 state:SettingsState={config:null,revision:0,keyAvailable:false,keySource:null,fields:{baseUrl:'',model:'',keyEnv:'ZWEN_API_KEY',imageModel:''},status:'loading',loaded:false,error:null,secretSaving:false,secretNotice:null};
 constructor(private port:ProviderPort,private changed:(s:SettingsState)=>void){}
 private publish(change:Partial<SettingsState>){this.state={...this.state,...change};this.changed(this.state);}
 async load(){
  if(this.state.status==='saving')return;
  this.publish({status:'loading',error:null});
  try {const next=await this.port.load();this.publish({...next,fields:next.config?{...next.config,imageModel:imageModelOf(next.config)}:{baseUrl:'',model:'',keyEnv:'ZWEN_API_KEY',imageModel:''},loaded:true,status:'saved'});}
  catch(error){this.publish({status:'error',error:error instanceof Error?error.message:'模型设置读取失败'});}
 }
 edit(name:keyof ProviderConfig,value:string){
  const fieldValue=name==='reasoningEffort'?reasoningLevelsOf(this.state).find(level=>level===value):value;
  this.publish({fields:{...this.state.fields,[name]:fieldValue},status:'dirty',error:null});
 }
 async save(){
  if(!this.state.loaded||this.state.status==='saving')return;
  const fields=this.state.fields;
  const {reasoningEffort,...sent}=fields;
  const selected=reasoningLevelsOf(this.state).find(level=>level===reasoningEffort);
  const config:ProviderConfig=selected?{...sent,reasoningEffort:selected}:sent;
  const revision=this.state.revision;this.publish({status:'saving',error:null});
  try {
   const next=await this.port.save(revision,config);
   const current=this.state.fields;
   const unchanged=(Object.keys({...fields,...current}) as (keyof ProviderConfig)[]).every(k=>fields[k]===current[k]);
   this.publish({...next,...(unchanged?{fields:next.config?{...next.config,imageModel:imageModelOf(next.config)}:config}:{}),status:unchanged?'saved':'dirty'});
  }
  catch(error){this.publish({status:'error',error:error instanceof Error?error.message:'模型设置保存失败'});}
 }
 async saveSecret(value:string){
  if(!this.state.loaded||this.state.secretSaving||!value.trim())return;
  if(this.state.status!=='saved'||!this.state.config){this.publish({secretNotice:'请先保存模型服务设置，再保存对应密钥'});return;}
  const revision=this.state.revision;
  this.publish({secretSaving:true,secretNotice:null});
  try{
   await this.port.saveSecret(value,revision);
   await this.load();
   this.publish({secretNotice:null});
  }
  catch(error){this.publish({secretNotice:error instanceof Error?error.message:'密钥保存未完成'});}
  finally{this.publish({secretSaving:false});}
 }
 async clearSecret(){
  if(!this.state.loaded||this.state.secretSaving)return;
  this.publish({secretSaving:true,secretNotice:null});
  try{
   // A reload would discard unsaved field edits; keep them visible.
   const fields=this.state.fields;const dirty=this.state.status==='dirty';
   await this.port.clearSecret();
   await this.load();
   if(dirty)this.publish({fields,status:'dirty'});
   this.publish({secretNotice:null});
  }
  catch(error){this.publish({secretNotice:error instanceof Error?error.message:'清除已保存密钥未完成'});}
  finally{this.publish({secretSaving:false});}
 }
}
