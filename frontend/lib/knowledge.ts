export interface NoteMeta {id:string;title:string;createdAt:number;source:string;model:string;attachments:string[];request?:string|null}
export interface Note {meta:NoteMeta;body:string}
export interface SaveNoteInput {title:string;body:string;source:string;model:string;attachmentIds:string[];requestId?:string|null}
export interface KnowledgePort {
 list(query:string):Promise<NoteMeta[]>;
 read(id:string):Promise<Note>;
 save(input:SaveNoteInput):Promise<NoteMeta>;
 remove(id:string):Promise<void>;
 update(id:string,title:string,body:string):Promise<NoteMeta>;
 rebuild():Promise<number>;
}
export interface KnowledgeState {notes:NoteMeta[];query:string;current:Note|null;busy:boolean;loaded:boolean;notice:string}

export class KnowledgeController {
 state:KnowledgeState={notes:[],query:'',current:null,busy:false,loaded:false,notice:''};
 private disposed=false;
 constructor(private port:KnowledgePort,private changed:(s:KnowledgeState)=>void){}
 private publish(patch:Partial<KnowledgeState>):void{
  if(this.disposed)return;
  this.state={...this.state,...patch};
  this.changed(this.state);
 }
 async refresh(query?:string){
  if(this.disposed||this.state.busy)return;
  const next=typeof query==='string'?query:this.state.query;
  this.publish({busy:true,query:next});
  try{
   const notes=await this.port.list(next);
   // Success keeps any action notice; only failures report here.
   this.publish({notes,loaded:true});
  }
  catch(error){
   this.publish({loaded:true,notice:error instanceof Error?error.message:'知识库读取失败，请重试'});
  }
  finally{this.publish({busy:false});}
 }
 async open(id:string){
  if(this.disposed||this.state.busy)return;
  this.publish({busy:true});
  try{
   const note=await this.port.read(id);
   this.publish({current:note,notice:''});
  }
  catch(error){
   this.publish({notice:error instanceof Error?error.message:'笔记读取失败'});
   await this.refresh();
  }
  finally{this.publish({busy:false});}
 }
 close():void{this.publish({current:null});}
 async remove(id:string){
  if(this.disposed||this.state.busy)return;
  this.publish({busy:true});
  try{
   await this.port.remove(id);
   this.publish({current:null,notice:''});
  }
  catch(error){
   this.publish({notice:error instanceof Error?error.message:'删除未完成'});
  }
  this.publish({busy:false});
  await this.refresh();
 }
 async save(input:SaveNoteInput):Promise<NoteMeta|null>{
  if(this.disposed||this.state.busy)return null;
  this.publish({busy:true});
  try{
   const meta=await this.port.save(input);
   this.publish({notice:''});
  }
  catch(error){
   this.publish({notice:error instanceof Error?error.message:'收藏未完成'});
   return null;
  }
  this.publish({busy:false});
  await this.refresh();
  return this.state.notes.find(note=>note.title===input.title)??null;
 }
 async saveEdit(id:string,title:string,body:string):Promise<NoteMeta|null>{
  if(this.disposed||this.state.busy)return null;
  this.publish({busy:true});
  try{
   const meta=await this.port.update(id,title,body);
   const note=await this.port.read(id);
   this.publish({current:note,notice:''});
   this.publish({busy:false});
   await this.refresh();
   return meta;
  }
  catch(error){
   this.publish({notice:error instanceof Error?error.message:'保存编辑未完成'});
   this.publish({busy:false});
   return null;
  }
 }
 async rebuild(){
  if(this.disposed||this.state.busy)return;
  this.publish({busy:true});
  try{
   const count=await this.port.rebuild();
   this.publish({notice:`索引已重建，共 ${count} 篇笔记`});
  }
  catch(error){
   this.publish({notice:error instanceof Error?error.message:'索引重建未完成'});
  }
  this.publish({busy:false});
  await this.refresh();
 }
 dispose():void{this.disposed=true;}
}
