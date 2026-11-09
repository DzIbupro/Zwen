export interface AttachmentItem {id:string;width:number;height:number;bytes:number;createdAt:number;thumbnail:string|null;error:string|null}
export interface CaptureInfo {state:'idle'|'pending';requestId:string|null;replaceId:string|null}
export interface AttachmentView {items:AttachmentItem[];capture:CaptureInfo;notice:string|null;max:number}
export interface AttachmentPort {
 list():Promise<AttachmentView>;
 capture(replaceId?:string|null):Promise<void>;
 cancel():Promise<void>;
 remove(id:string):Promise<void>;
 listen(cb:()=>void):Promise<()=>void>;
}
export interface AttachmentState {items:AttachmentItem[];pending:boolean;notice:string;loaded:boolean;max:number}

export class AttachmentController {
 state:AttachmentState={items:[],pending:false,notice:'',loaded:false,max:3};
 private disposed=false;
 private loading:Promise<void>|undefined;
 private again=false;
 private unlisten:(()=>void)|null=null;
 constructor(private port:AttachmentPort,private changed:(s:AttachmentState)=>void){}
 private publish(patch:Partial<AttachmentState>):void{
  if(this.disposed)return;
  this.state={...this.state,...patch};
  this.changed(this.state);
 }
 async start():Promise<void>{
  if(this.disposed)return;
  // Subscribe before the first read so an import that lands during startup is
  // still observed; the read itself also returns the latest persisted state.
  try{
   const off=await this.port.listen(()=>void this.load());
   if(this.disposed)off();else this.unlisten=off;
  }catch(error){
   this.publish({notice:this.message(error,'截图附件事件暂不可用，可重新读取状态')});
  }
  await this.load();
 }
 load():Promise<void>{
  if(this.disposed)return Promise.resolve();
  this.again=true;
  if(this.loading)return this.loading;
  this.loading=this.drain().finally(()=>{
   this.loading=undefined;
   if(this.again&&!this.disposed)void this.load();
  });
  return this.loading;
 }
 private async drain(){
  do{
   this.again=false;
   try{
    const view=await this.port.list();
    if(this.disposed)return;
    this.publish({items:view.items,pending:view.capture.state==='pending',notice:view.notice??'',loaded:true,max:view.max});
   }catch(error){
    if(this.disposed)return;
    this.publish({loaded:true,notice:this.message(error,'截图附件读取失败，请重试')});
   }
  }while(this.again&&!this.disposed);
 }
 async capture(replaceId?:string|null):Promise<void>{
  if(this.disposed||!this.state.loaded||this.state.pending)return;
  this.publish({pending:true,notice:'正在等待截图…'});
  try{
   await this.port.capture(replaceId??null);
  }catch(error){
   this.publish({pending:false,notice:this.message(error,'截图未能开始')});
  }finally{
   await this.load();
  }
 }
 async cancel():Promise<void>{
  if(this.disposed)return;
  try{await this.port.cancel();}catch(error){this.publish({notice:this.message(error,'取消截图未完成')});}
  await this.load();
 }
 async remove(id:string):Promise<void>{
  if(this.disposed||this.state.pending)return;
  try{await this.port.remove(id);}catch(error){this.publish({notice:this.message(error,'删除截图未完成')});}
  await this.load();
 }
 /// Consume the exact attachments that were accepted into a sent request.
 /// Missing ids are tolerated: a concurrent manual delete already removed them.
 async removeSent(ids:string[]):Promise<void>{
  if(this.disposed)return;
  for(const id of ids){try{await this.port.remove(id);}catch{/* reflected by the reload below */}}
  await this.load();
 }
 note(text:string):void{this.publish({notice:text});}
 dispose():void{this.disposed=true;this.unlisten?.();this.unlisten=null;}
 private message(error:unknown,fallback:string):string{return error instanceof Error?error.message:fallback;}
}
