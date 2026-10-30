export interface ProviderConfig {baseUrl:string;model:string;keyEnv:string;imageModel?:string}
export interface ProviderView {config:ProviderConfig|null;revision:number;keyAvailable:boolean}
export interface ProviderPort {load():Promise<ProviderView>;save(revision:number,config:ProviderConfig):Promise<ProviderView>}
export interface SettingsState extends ProviderView {fields:ProviderConfig;status:'loading'|'saved'|'dirty'|'saving'|'error';loaded:boolean;error:string|null}
export function imageModelOf(config:ProviderConfig|null|undefined):string{return typeof config?.imageModel==='string'?config.imageModel:'';}
export class ProviderController {
 state:SettingsState={config:null,revision:0,keyAvailable:false,fields:{baseUrl:'',model:'',keyEnv:'ZWEN_API_KEY',imageModel:''},status:'loading',loaded:false,error:null};
 constructor(private port:ProviderPort,private changed:(s:SettingsState)=>void){}
 private publish(change:Partial<SettingsState>){this.state={...this.state,...change};this.changed(this.state);}
 async load(){
  if(this.state.status==='saving')return;
  this.publish({status:'loading',error:null});
  try {const next=await this.port.load();this.publish({...next,fields:next.config?{...next.config,imageModel:imageModelOf(next.config)}:{baseUrl:'',model:'',keyEnv:'ZWEN_API_KEY',imageModel:''},loaded:true,status:'saved'});}
  catch(error){this.publish({status:'error',error:error instanceof Error?error.message:'模型设置读取失败'});}
 }
 edit(name:keyof ProviderConfig,value:string){this.publish({fields:{...this.state.fields,[name]:value},status:'dirty',error:null});}
 async save(){
  if(!this.state.loaded||this.state.status==='saving')return;
  const sent={...this.state.fields};const revision=this.state.revision;this.publish({status:'saving',error:null});
  try {const next=await this.port.save(revision,sent);const unchanged=(Object.keys(sent) as (keyof ProviderConfig)[]).every(k=>sent[k]===this.state.fields[k]);this.publish({...next,...(unchanged?{fields:next.config?{...next.config,imageModel:imageModelOf(next.config)}:sent}:{}),status:unchanged?'saved':'dirty'});}
  catch(error){this.publish({status:'error',error:error instanceof Error?error.message:'模型设置保存失败'});}
 }
}
