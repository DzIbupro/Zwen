import {defaultReadingOptions,type ReadingOptions,type PaperMetadata} from './papers';
interface Port {importPdf?(path:string):Promise<PaperMetadata>;get?(id:string):Promise<PaperMetadata>;defaults():Promise<ReadingOptions>;choose():Promise<PaperMetadata|null>;list():Promise<PaperMetadata[]>}
interface Storage {getItem(key:string):string|null;setItem(key:string,value:string):void;removeItem(key:string):void}
export interface PaperDraftState {enabled:boolean;restoring:boolean;recoveryAvailable:boolean;loaded:boolean;importing:boolean;paper:PaperMetadata|null;options:ReadingOptions;notice:string}
const KEY='zwen.paper-draft.v1';
export class PaperDraftController {
 state:PaperDraftState={enabled:false,restoring:false,recoveryAvailable:false,loaded:false,importing:false,paper:null,options:defaultReadingOptions(),notice:''};
 private pendingId:string|null=null;
 private defaults:ReadingOptions|null=null;private loading:Promise<void>|null=null;private generation=0;private disposed=false;
 constructor(private port:Port,private changed:(s:PaperDraftState)=>void,private storage?:Storage){}
 private publish(p:Partial<PaperDraftState>){if(!this.disposed){this.state={...this.state,...p};this.changed(this.state);}}
 private persist(){try{if(this.state.enabled)this.storage?.setItem(KEY,JSON.stringify({id:this.state.paper?.id??this.pendingId,options:this.state.options}));else this.storage?.removeItem(KEY);}catch{this.publish({notice:'论文仍保存在本机，当前附件选择暂未缓存，重载后可从已导入论文重新选择'});}}
 private async load(){
  if(this.state.loaded)return;if(this.loading)return this.loading;
  this.loading=(async()=>{const defaults=await this.port.defaults();if(this.disposed)return;this.defaults=defaults;this.publish({loaded:true,options:{...defaults}});})().finally(()=>{this.loading=null;});return this.loading;
 }
 async prepare(){try{await this.load();}catch(e){this.publish({notice:e instanceof Error?e.message:'阅读要求加载失败'});}}
 async enable(){this.publish({enabled:true,notice:''});try{await this.load();this.persist();}catch(e){this.publish({notice:e instanceof Error?e.message:'阅读要求加载失败，请重新启用'});}}
 async restore(){
  if(this.disposed||this.state.importing||this.state.restoring)return;
  const generation=this.generation;
  try{const raw=this.storage?.getItem(KEY);if(!raw||raw.length>65536)return;const value=JSON.parse(raw);if(!value||typeof value!=='object')return;
   this.pendingId=typeof value.id==='string'&&value.id.length<=100?value.id:null;
   this.publish({enabled:true,restoring:true,recoveryAvailable:false});
   await this.load();if(this.disposed||generation!==this.generation)return;
   const options={...this.state.options};const saved=value.options;
   if(saved&&typeof saved==='object'){
    if(typeof saved.readImages==='boolean')options.readImages=saved.readImages;
    for(const key of ['mode','focus','outputRequirements','background'] as const)if(typeof saved[key]==='string'&&saved[key].length<=8192){if(key==='mode'){if(['deep','quick','reproduce','interactive'].includes(saved[key]))options.mode=saved[key] as ReadingOptions['mode'];}else options[key]=saved[key];}
    for(const key of ['startPage','endPage'] as const)if(Number.isInteger(saved[key])&&saved[key]>=0&&saved[key]<=300)options[key]=saved[key];
   }
   this.publish({options});
   const items=value.id?(this.port.get?[await this.port.get(value.id)]:await this.port.list()):[];if(this.disposed||generation!==this.generation)return;
   const paper=items.find(p=>p.id===value.id)??null;this.publish({enabled:true,paper,options,recoveryAvailable:!!this.pendingId&&!paper,notice:value.id&&!paper?'原附件不可用，请重新选择PDF':''});
  }catch{if(generation===this.generation)this.publish({enabled:true,recoveryAvailable:!!this.pendingId,notice:'附件暂时无法恢复，可重试或重新选择PDF；已保存论文和阅读要求保留'});}
  finally{if(generation===this.generation)this.publish({restoring:false});}
 }
 async choose(){await this.importFile(()=>this.port.choose());}
 async drop(paths:string[]){
  if(paths.length!==1||!paths[0]||!paths[0].toLowerCase().endsWith('.pdf')){this.publish({notice:'请拖入单篇 PDF；当前附件保留'});return;}
  if(!this.port.importPdf){this.publish({notice:'请在桌面应用中选择或拖入PDF'});return;}
  await this.importFile(()=>this.port.importPdf!(paths[0]));
 }
 private async importFile(load:()=>Promise<PaperMetadata|null>){
  if(this.disposed||this.state.importing||this.state.restoring)return;
  const ticket=++this.generation;this.publish({enabled:true,importing:true,notice:'正在本机导入PDF…'});
  try{await this.load();if(this.disposed||ticket!==this.generation)return;
   const paper=await load();if(this.disposed||ticket!==this.generation)return;if(paper)this.attach(paper);else {this.publish({notice:''});this.persist();}}
  catch(e){if(ticket===this.generation)this.publish({notice:e instanceof Error?e.message:'PDF导入失败'});}
  finally{if(ticket===this.generation)this.publish({importing:false});}
 }
 attach(paper:PaperMetadata){this.generation++;this.pendingId=paper.id;this.publish({enabled:true,paper,recoveryAvailable:false,loaded:true,importing:false,restoring:false,options:{...(this.defaults??this.state.options),endPage:paper.pageCount},notice:''});this.persist();}
 edit(change:Partial<ReadingOptions>){this.publish({options:{...this.state.options,...change}});this.persist();}
 reset(){if(this.defaults)this.edit({...this.defaults,endPage:this.state.paper?.pageCount??0});}
 remove(){this.generation++;this.pendingId=null;this.publish({paper:null,recoveryAvailable:false,importing:false,restoring:false,notice:''});this.persist();}
 close(){this.generation++;this.pendingId=null;this.publish({enabled:false,restoring:false,paper:null,recoveryAvailable:false,importing:false,notice:''});this.persist();}
 accepted(){this.close();}
 dispose(){this.disposed=true;this.generation++;}
}
