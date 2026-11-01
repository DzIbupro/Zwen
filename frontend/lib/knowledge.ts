export interface NoteMeta {id:string;title:string;createdAt:number;source:string;model:string;attachments:string[];request?:string|null;sourceTitle?:string|null;sourceUrl?:string|null;sourceLoc?:string|null}
export interface Note {meta:NoteMeta;body:string}
export interface SourceMetaInput {sourceTitle?:string|null;sourceUrl?:string|null;sourceLoc?:string|null}
export interface SaveNoteInput {title:string;body:string;source:string;model:string;attachmentIds:string[];requestId?:string|null}
export interface TopicInfo {id:string;name:string;column:string;createdBy:string;batchId:string|null;createdAt:number}
export interface TopicView {topic:TopicInfo;noteIds:string[];count:number}
export interface OrganizeSummaryView {batchId:string;topics:number;assignments:number}
export interface KnowledgePort {
 list(query:string):Promise<NoteMeta[]>;
 read(id:string):Promise<Note>;
 save(input:SaveNoteInput):Promise<NoteMeta>;
 remove(id:string):Promise<void>;
 update(id:string,title:string,body:string,source:SourceMetaInput):Promise<NoteMeta>;
 updateSource(id:string,source:SourceMetaInput):Promise<void>;
 exportNote(id:string):Promise<string>;
 exportAll():Promise<string>;
 rebuild():Promise<number>;
 topics():Promise<TopicView[]>;
 topicNotes(topicId:string):Promise<Note[]>;
 run(noteIds:string[]):Promise<OrganizeSummaryView>;
 undo(batchId:string):Promise<void>;
}
export interface KnowledgeState {notes:NoteMeta[];query:string;current:Note|null;busy:boolean;loaded:boolean;notice:string;
 topics:TopicView[];organizeMode:boolean;selected:Record<string,boolean>;confirming:boolean;organizeBusy:boolean;
 lastBatch:string|null;lastSummary:OrganizeSummaryView|null;topicView:{title:string;notes:Note[]}|null}

export class KnowledgeController {
 state:KnowledgeState={notes:[],query:'',current:null,busy:false,loaded:false,notice:'',topics:[],organizeMode:false,selected:{},confirming:false,organizeBusy:false,lastBatch:null,lastSummary:null,topicView:null};
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
 async saveEdit(id:string,title:string,body:string,source:SourceMetaInput):Promise<NoteMeta|null>{
  if(this.disposed||this.state.busy)return null;
  this.publish({busy:true});
  try{
   const meta=await this.port.update(id,title,body,source);
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
 async loadTopics(){
  if(this.disposed)return;
  try{const topics=await this.port.topics();this.publish({topics});}
  catch(error){this.publish({notice:error instanceof Error?error.message:'主题读取失败'});}
 }
 enterOrganize(){
  if(this.disposed||this.state.busy)return;
  this.publish({organizeMode:true,selected:{},confirming:false,lastSummary:null,notice:''});
 }
 exitOrganize():void{this.publish({organizeMode:false,selected:{},confirming:false});}
 toggleSelect(id:string){
  if(this.disposed)return;
  const selected={...this.state.selected};
  if(selected[id])delete selected[id];
  else{
   if(Object.keys(selected).length>=8)return;
   selected[id]=true;
  }
  this.publish({selected});
 }
 async runOrganize(modelLabel:string){
  if(this.disposed||this.state.organizeBusy)return;
  const ids=Object.keys(this.state.selected);
  if(!this.state.confirming||ids.length===0)return;
  this.publish({organizeBusy:true,notice:'',lastSummary:null});
  try{
   const summary=await this.port.run(ids);
   this.publish({lastBatch:summary.batchId,lastSummary:summary,selected:{},confirming:false});
   await this.loadTopics();
   await this.refresh();
   return summary;
  }
  catch(error){
   this.publish({notice:error instanceof Error?error.message:'整理未完成'});
   return null;
  }
  finally{this.publish({organizeBusy:false});}
 }
 startConfirm():void{if(Object.keys(this.state.selected).length>0)this.publish({confirming:true});}
 cancelConfirm():void{this.publish({confirming:false});}
 async undoLast(){
  if(this.disposed||this.state.organizeBusy||!this.state.lastBatch)return;
  this.publish({organizeBusy:true});
  try{
   await this.port.undo(this.state.lastBatch);
   this.publish({lastBatch:null,lastSummary:null,notice:'已撤销上次整理'});
   await this.loadTopics();
   await this.refresh();
  }
  catch(error){this.publish({notice:error instanceof Error?error.message:'撤销未完成'});}
  finally{this.publish({organizeBusy:false});}
 }
 async openTopic(topicId:string){
  if(this.disposed||this.state.busy)return;
  this.publish({busy:true});
  try{
   const notes=await this.port.topicNotes(topicId);
   const topic=this.state.topics.find(t=>t.topic.id===topicId);
   this.publish({topicView:{title:topic?.topic.name??'主题',notes}});
  }
  catch(error){this.publish({notice:error instanceof Error?error.message:'主题笔记读取失败'});}
  finally{this.publish({busy:false});}
 }
 closeTopic():void{this.publish({topicView:null});}
 async saveSource(id:string,source:SourceMetaInput):Promise<boolean>{
  if(this.disposed||this.state.busy)return false;
  this.publish({busy:true});
  try{
   await this.port.updateSource(id,source);
   const note=await this.port.read(id);
   this.publish({current:note,notice:''});
   return true;
  }
  catch(error){
   this.publish({notice:error instanceof Error?error.message:'来源信息保存未完成'});
   return false;
  }
  finally{this.publish({busy:false});}
 }
 async exportNote(id:string):Promise<string|null>{
  if(this.disposed||this.state.busy)return null;
  this.publish({busy:true});
  try{
   const path=await this.port.exportNote(id);
   this.publish({notice:`已导出到 ${path}`});
   return path;
  }
  catch(error){
   this.publish({notice:error instanceof Error?error.message:'导出未完成'});
   return null;
  }
  finally{this.publish({busy:false});}
 }
 async exportAll():Promise<string|null>{
  if(this.disposed||this.state.busy)return null;
  this.publish({busy:true});
  try{
   const path=await this.port.exportAll();
   this.publish({notice:`已导出全部笔记到 ${path}`});
   return path;
  }
  catch(error){
   this.publish({notice:error instanceof Error?error.message:'导出未完成'});
   return null;
  }
  finally{this.publish({busy:false});}
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
