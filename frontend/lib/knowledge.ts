export interface NoteMeta {id:string;title:string;createdAt:number;source:string;model:string;attachments:string[];request?:string|null;sourceTitle?:string|null;sourceUrl?:string|null;sourceLoc?:string|null}
export interface Note {meta:NoteMeta;body:string}
export interface SourceMetaInput {sourceTitle?:string|null;sourceUrl?:string|null;sourceLoc?:string|null}
export interface SaveNoteInput {title:string;body:string;source:string;model:string;attachmentIds:string[];requestId?:string|null}
export interface TopicInfo {id:string;name:string;column:string;createdBy:string;batchId:string|null;createdAt:number;locked:boolean}
export interface TopicView {topic:TopicInfo;noteIds:string[];count:number}
export interface OrganizeSummaryView {batchId:string;topics:number;assignments:number}
export interface RelationView {id:string;fromTopic:string;toTopic:string;reason:string;evidence:string;addedBy:string}
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
 run(noteIds:string[],automatic?:boolean):Promise<OrganizeSummaryView>;
 undo(batchId:string):Promise<void>;
 renameTopic(topicId:string,name:string):Promise<void>;
 setTopicColumn(topicId:string,column:string):Promise<void>;
 mergeTopics(source:string,target:string):Promise<void>;
 splitNote(noteId:string,from:string,toName:string,column:string):Promise<string>;
 setLocked(topicId:string,locked:boolean):Promise<void>;
 deleteTopic(topicId:string):Promise<void>;
 listRelations():Promise<RelationView[]>;
 addRelation(fromTopic:string,toTopic:string,reason:string,evidence:string):Promise<string>;
 removeRelation(relationId:string):Promise<void>;
 setAuto(on:boolean):Promise<void>;
 getAuto():Promise<boolean>;
}
export interface KnowledgeState {notes:NoteMeta[];query:string;current:Note|null;busy:boolean;loaded:boolean;notice:string;
 topics:TopicView[];organizeMode:boolean;selected:Record<string,boolean>;confirming:boolean;organizeBusy:boolean;
 lastBatch:string|null;lastSummary:OrganizeSummaryView|null;topicView:{id:string;title:string;notes:Note[]}|null;
 relations:RelationView[];auto:boolean}

export interface NoteEditDraft {title:string;body:string;sourceTitle:string;sourceUrl:string;sourceLoc:string}

const ORGANIZE_LIMIT_NOTICE='每次最多整理 8 篇笔记；请先整理当前所选，再选择其他笔记。';
export class KnowledgeController {
 private navigation=0;
 private editorDrafts=new Map<string,NoteEditDraft>();
 pendingEditorId():string|undefined {return this.editorDrafts.keys().next().value;}
 getEditorDraft(id:string):NoteEditDraft|undefined {const draft=this.editorDrafts.get(id);return draft?{...draft}:undefined;}
 rememberEditorDraft(id:string,draft:NoteEditDraft):void {this.editorDrafts.set(id,{...draft});}
 discardEditorDraft(id:string):void {this.editorDrafts.delete(id);}
 topicsSnapshot():TopicView[]{return this.state.topics;}
 state:KnowledgeState={notes:[],query:'',current:null,busy:false,loaded:false,notice:'',topics:[],organizeMode:false,selected:{},confirming:false,organizeBusy:false,lastBatch:null,lastSummary:null,topicView:null,relations:[],auto:false};
 private disposed=false;
 private refreshing:Promise<void>|null=null;
 private refreshAgain=false;
 constructor(private port:KnowledgePort,private changed:(s:KnowledgeState)=>void){}
 private publish(patch:Partial<KnowledgeState>):void{
  if(this.disposed)return;
  this.state={...this.state,...patch};
  this.changed(this.state);
 }
 refresh(query?:string):Promise<void>{
  if(this.disposed)return Promise.resolve();
  if(typeof query==='string')this.publish({query});
  if(this.refreshing){this.refreshAgain=true;return this.refreshing;}
  if(this.state.busy)return Promise.resolve();
  this.refreshing=this.refreshLatest().finally(()=>{this.refreshing=null;});return this.refreshing;
 }
 private async refreshLatest(){
  this.publish({busy:true});
  try{
   do{
    this.refreshAgain=false;const query=this.state.query;
    try{
     const notes=await this.port.list(query);if(this.disposed)return;
     if(this.refreshAgain)continue;
     try{const auto=await this.port.getAuto();if(!this.refreshAgain)this.publish({auto});}catch(error){if(!this.refreshAgain)this.publish({auto:false,notice:error instanceof Error?error.message:'自动整理设置读取失败'});}
     if(!this.refreshAgain)this.publish({notes,loaded:true});
    }catch(error){if(!this.refreshAgain)this.publish({loaded:true,notice:error instanceof Error?error.message:'知识库读取失败，请重试'});}
   }while(this.refreshAgain&&!this.disposed);
  }finally{this.publish({busy:false});}
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
 close():void{this.navigation++;this.publish({current:null});}
 async remove(id:string){
  if(this.disposed||this.state.busy)return;
  this.publish({busy:true});
  try{
   await this.port.remove(id);
   this.discardEditorDraft(id);
   const selected={...this.state.selected};delete selected[id];
   const topicView=this.state.topicView;
   const topics=this.state.topics.map(topic=>({...topic,noteIds:topic.noteIds.filter(note=>note!==id),count:topic.count-(topic.noteIds.includes(id)?1:0)})).filter(topic=>topic.count>0);
   this.publish({current:null,selected,topics,relations:this.state.relations.filter(relation=>relation.evidence!==id&&topics.some(t=>t.topic.id===relation.fromTopic)&&topics.some(t=>t.topic.id===relation.toTopic)),topicView:topicView?{...topicView,notes:topicView.notes.filter(note=>note.meta.id!==id)}:null,notice:''});
   await this.loadTopics();await this.loadRelations();
   if(this.state.topicView&&!this.state.topics.some(topic=>topic.topic.id===this.state.topicView!.id))this.publish({topicView:null});
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
  let saved:NoteMeta;
  try{
   saved=await this.port.save(input);
   this.publish({notice:''});
  }
  catch(error){
   this.publish({notice:error instanceof Error?error.message:'收藏未完成'});
   return null;
  }
  finally{this.publish({busy:false});}
  await this.refresh();
  return saved;
 }
 async saveEdit(id:string,title:string,body:string,source:SourceMetaInput):Promise<NoteMeta|null>{
  if(this.disposed||this.state.busy)return null;
  const navigation=this.navigation;
  const draft=this.getEditorDraft(id);
  this.publish({busy:true});
  try{
   const meta=await this.port.update(id,title,body,source);
   const note=await this.port.read(id);
   if(navigation===this.navigation&&this.state.current?.meta.id===id)this.publish({current:note,notice:''});
   if(draft&&JSON.stringify(this.getEditorDraft(id))===JSON.stringify(draft))this.discardEditorDraft(id);
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
   if(Object.keys(selected).length>=8){this.publish({notice:ORGANIZE_LIMIT_NOTICE});return;}
   selected[id]=true;
  }
  this.publish({selected,...(this.state.notice===ORGANIZE_LIMIT_NOTICE?{notice:''}:{})});
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
   this.publish({topicView:{id:topicId,title:topic?.topic.name??'主题',notes}});
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
 async loadRelations(){
  if(this.disposed)return;
  try{const relations=await this.port.listRelations();this.publish({relations});}
  catch{ /* relation list is auxiliary */ }
 }
 async renameTopic(topicId:string,name:string){if(this.disposed||this.state.busy)return;this.publish({busy:true});try{await this.port.renameTopic(topicId,name);const tv=this.state.topicView;if(tv&&tv.id===topicId)this.publish({topicView:{...tv,title:name}});await this.loadTopics();}catch(error){this.publish({notice:error instanceof Error?error.message:'重命名未完成'});}finally{this.publish({busy:false});}}
 async setTopicColumn(topicId:string,column:string){if(this.disposed||this.state.busy)return;this.publish({busy:true});try{await this.port.setTopicColumn(topicId,column);await this.loadTopics();}catch(error){this.publish({notice:error instanceof Error?error.message:'栏目修改未完成'});}finally{this.publish({busy:false});}}
 async mergeTopics(source:string,target:string){if(this.disposed||this.state.busy)return;this.publish({busy:true});try{await this.port.mergeTopics(source,target);if(this.state.topicView?.id===source)this.publish({topicView:null});await this.loadTopics();}catch(error){this.publish({notice:error instanceof Error?error.message:'合并未完成'});}finally{this.publish({busy:false});}}
 async lockTopic(topicId:string,locked:boolean){if(this.disposed||this.state.busy)return;this.publish({busy:true});try{await this.port.setLocked(topicId,locked);await this.loadTopics();}catch(error){this.publish({notice:error instanceof Error?error.message:'锁定修改未完成'});}finally{this.publish({busy:false});}}
 async deleteTopic(topicId:string){if(this.disposed||this.state.busy)return;this.publish({busy:true});try{await this.port.deleteTopic(topicId);if(this.state.topicView?.id===topicId)this.publish({topicView:null});await this.loadTopics();}catch(error){this.publish({notice:error instanceof Error?error.message:'删除主题未完成'});}finally{this.publish({busy:false});}}
 async addRelation(fromTopic:string,toTopic:string,reason:string,evidence:string){if(this.disposed||this.state.busy)return;this.publish({busy:true});try{await this.port.addRelation(fromTopic,toTopic,reason,evidence);await this.loadRelations();}catch(error){this.publish({notice:error instanceof Error?error.message:'关系添加未完成'});}finally{this.publish({busy:false});}}
 async removeRelation(relationId:string){if(this.disposed||this.state.busy)return;try{await this.port.removeRelation(relationId);await this.loadRelations();}catch(error){this.publish({notice:error instanceof Error?error.message:'关系删除未完成'});}}
 async setAuto(on:boolean){if(this.disposed)return;try{await this.port.setAuto(on);this.publish({auto:on});}catch(error){this.publish({notice:error instanceof Error?error.message:'自动整理开关未完成'});}}
 /// Auto incremental organizing for a freshly collected note. Silent on
 /// failure: the manual flow remains the source of truth.
 async runAutoFor(noteId:string):Promise<void>{
  if(this.disposed||this.state.organizeBusy||this.state.busy)return;
  try{if(await this.port.getAuto())await this.port.run([noteId],true);}catch{/* silent by design */}
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
