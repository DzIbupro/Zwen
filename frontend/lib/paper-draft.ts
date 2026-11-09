import {defaultReadingOptions,type ReadingOptions,type PaperMetadata} from './papers';
interface Port {defaults():Promise<ReadingOptions>;choose():Promise<PaperMetadata|null>;list():Promise<PaperMetadata[]>}
interface Storage {getItem(key:string):string|null;setItem(key:string,value:string):void;removeItem(key:string):void}
export interface PaperDraftState {enabled:boolean;restoring:boolean;loaded:boolean;importing:boolean;paper:PaperMetadata|null;options:ReadingOptions;notice:string}
const KEY='zwen.paper-draft.v1';
export class PaperDraftController {
 state:PaperDraftState={enabled:false,restoring:false,loaded:false,importing:false,paper:null,options:defaultReadingOptions(),notice:''};
 private defaults:ReadingOptions|null=null;private loading:Promise<void>|null=null;private generation=0;private disposed=false;
 constructor(private port:Port,private changed:(s:PaperDraftState)=>void,private storage?:Storage){}
 private publish(p:Partial<PaperDraftState>){if(!this.disposed){this.state={...this.state,...p};this.changed(this.state);}}
 private persist(){try{if(this.state.enabled)this.storage?.setItem(KEY,JSON.stringify({id:this.state.paper?.id??null,options:this.state.options}));else this.storage?.removeItem(KEY);}catch{this.publish({notice:'论文仍保存在本机，当前附件选择暂未缓存，重载后可从已导入论文重新选择'});}}
 private async load(){
  if(this.state.loaded)return;if(this.loading)return this.loading;
  this.loading=(async()=>{const defaults=await this.port.defaults();if(this.disposed)return;this.defaults=defaults;this.publish({loaded:true,options:{...defaults}});})().finally(()=>{this.loading=null;});return this.loading;
 }
 async prepare(){try{await this.load();}catch(e){this.publish({notice:e instanceof Error?e.message:'阅读要求加载失败'});}}
 async enable(){this.publish({enabled:true,notice:''});try{await this.load();this.persist();}catch(e){this.publish({notice:e instanceof Error?e.message:'阅读要求加载失败，请重新启用'});}}
 async restore(){
  const generation=this.generation;
  try{const raw=this.storage?.getItem(KEY);if(!raw||raw.length>65536)return;const value=JSON.parse(raw);if(!value||typeof value!=='object')return;
   this.publish({enabled:true,restoring:true});
   await this.load();if(this.disposed||generation!==this.generation)return;
   const items=value.id?await this.port.list():[];if(this.disposed||generation!==this.generation)return;
   const options={...this.state.options};const saved=value.options;
   if(saved&&typeof saved==='object'){
    for(const key of ['mode','focus','outputRequirements','background'] as const)if(typeof saved[key]==='string'&&saved[key].length<=8192){if(key==='mode'){if(['deep','quick','reproduce','interactive'].includes(saved[key]))options.mode=saved[key] as ReadingOptions['mode'];}else options[key]=saved[key];}
    for(const key of ['startPage','endPage'] as const)if(Number.isInteger(saved[key])&&saved[key]>=0&&saved[key]<=300)options[key]=saved[key];
   }
   const paper=items.find(p=>p.id===value.id)??null;this.publish({enabled:true,paper,options,notice:value.id&&!paper?'原附件不可用，请重新选择PDF':''});
  }catch{if(generation===this.generation)this.publish({enabled:true,notice:'论文附件缓存无法恢复，请重新选择PDF；已保存论文和会话保留'});}
  finally{if(generation===this.generation)this.publish({restoring:false});}
 }
 async choose(){
  await this.enable();if(!this.state.loaded||this.disposed||!this.state.enabled)return;
  const ticket=++this.generation;this.publish({importing:true,notice:'正在本机导入PDF…'});
  try{const paper=await this.port.choose();if(this.disposed||ticket!==this.generation)return;if(paper)this.attach(paper);else this.publish({notice:''});}
  catch(e){if(ticket===this.generation)this.publish({notice:e instanceof Error?e.message:'PDF导入失败'});}
  finally{if(ticket===this.generation)this.publish({importing:false});}
 }
 attach(paper:PaperMetadata){this.generation++;this.publish({enabled:true,paper,loaded:true,importing:false,options:{...(this.defaults??this.state.options),endPage:paper.pageCount},notice:''});this.persist();}
 edit(change:Partial<ReadingOptions>){this.publish({options:{...this.state.options,...change}});this.persist();}
 reset(){if(this.defaults)this.edit({...this.defaults,endPage:this.state.paper?.pageCount??0});}
 remove(){this.generation++;this.publish({paper:null,importing:false,notice:''});this.persist();}
 close(){this.generation++;this.publish({enabled:false,restoring:false,paper:null,importing:false,notice:''});this.persist();}
 accepted(){this.close();}
 dispose(){this.disposed=true;this.generation++;}
}
