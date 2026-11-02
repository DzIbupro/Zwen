import type {DraftController} from './draft';
export interface SelectionItem{id:number;mode:'raw'|'explain'|'translate'|'candidate';text:string|null;error:string|null;claimed:boolean}
export interface SelectionPort{claim():Promise<{reading:boolean;item:SelectionItem|null}>;acknowledge(id:number):Promise<void>}
export interface SelectionState{busy:boolean;pending:SelectionItem|null;inserted:boolean;notice:string}
export class SelectionController{
 state:SelectionState={busy:false,pending:null,inserted:false,notice:''};
 private loading:Promise<void>|undefined;private again=false;private disposed=false;private mutating=false;
 constructor(private port:SelectionPort,private draft:DraftController,private changed:(s:SelectionState)=>void){}
 private publish(p:Partial<SelectionState>){if(this.disposed)return;this.state={...this.state,...p};this.changed(this.state);}
 refresh():Promise<void>{
  if(this.disposed||!this.draft.snapshot.loaded)return Promise.resolve();
  this.again=true;if(this.mutating)return Promise.resolve();if(this.loading)return this.loading;
  this.loading=this.drain().finally(()=>{this.loading=undefined;if(this.again&&!this.mutating&&!this.disposed)void this.refresh();});return this.loading;
 }
 private async drain(){
  do{
   this.again=false;
   try{
    const {reading,item}=await this.port.claim();if(this.disposed)return;
    if(this.mutating){this.again=true;return;}
    this.publish({busy:reading});
    if(!item)continue;
    if(this.state.pending?.id===item.id)continue;
    this.publish({pending:item,inserted:false});
    if(item.error){this.publish({notice:item.error});await this.discard(false);continue;}
    if(item.claimed){this.publish({notice:'上次导入未确认，请先检查草稿，再决定重新添加或丢弃待处理选区'});continue;}
    if(item.mode==='candidate'){this.publish({notice:'检测到选区，仅在本机候选区显示；点击添加后才写入草稿'});continue;}
    await this.applyPending();
   }catch(e){if(this.mutating){this.again=true;return;}this.publish({busy:false,notice:e instanceof Error?e.message:'选区状态读取失败，请重试'});}
  }while(this.again&&!this.disposed);
 }
 async applyPending(){
  const item=this.state.pending;if(this.disposed||this.state.busy||!item?.text)return;
  this.mutating=true;this.publish({busy:true});
  try{
   await this.draft.flush();if(this.disposed)return;
   if(!this.state.inserted){
    const label={candidate:'选区：',raw:'选区：',explain:'请解释以下选区：',translate:'请将以下选区翻译成中文：'}[item.mode];
    const snippet=label+'\n\n'+item.text;const original=this.draft.snapshot.text;
    const merged=original?original+'\n\n'+snippet:snippet;
    if(new TextEncoder().encode(item.text).length>64*1024||new TextEncoder().encode(merged).length>256*1024)throw Error('添加后将超过文字上限；原草稿已保留，可缩短草稿后重试');
    this.draft.edit(merged);this.publish({inserted:true});
   }
   await this.draft.flush();if(this.disposed)return;
   await this.port.acknowledge(item.id);if(this.disposed)return;
   this.publish({pending:null,inserted:false,notice:'选区已添加并保存到草稿 · 未发送'});
  }catch(e){this.publish({notice:(e instanceof Error?e.message:'选区导入未完成')+'；材料保留，请检查草稿'});}
  finally{this.mutating=false;this.publish({busy:false});if(this.again&&!this.loading&&!this.disposed)void this.refresh();}
 }
 async discard(announce=true){
  const item=this.state.pending;if(!item||this.state.busy||this.disposed)return;
  this.mutating=true;this.publish({busy:true});
  try{await this.port.acknowledge(item.id);if(!this.disposed)this.publish({pending:null,inserted:false,...(announce?{notice:'待处理选区已丢弃，现有草稿保留'}:{})});}
  catch(e){this.publish({notice:e instanceof Error?e.message:'选区确认未完成'});}
  finally{this.mutating=false;this.publish({busy:false});if(this.again&&!this.loading&&!this.disposed)void this.refresh();}
 }
 dispose(){this.disposed=true;}
}
