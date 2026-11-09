import type {ReadingOptions} from './papers';
export interface RetryState {attempt:number;nextRetryAt:number;reason:'timeout'|'rate-limit'}
export interface ChatReply {retry?:RetryState;requestId:string;conversationId:string;contextTurns:number;revision:number;answer:string;prompt:string;model:string;state:'streaming'|'complete'|'stopped'|'failed'|'interrupted'|'limited'|'filtered';error:string|null;saved:boolean;createdAt:number}
export interface Overview {reply:ChatReply|null;activeRequestId:string|null}
export interface ChatPort {startPaper?(id:string,paperId:string,options:ReadingOptions,question?:string):Promise<ChatReply>;startSelection?(id:string,selection:string,actionId:string,expectedRevision:number):Promise<ChatReply>;listen(cb:(r:ChatReply)=>void):Promise<()=>void>;load():Promise<Overview>;start(id:string,prompt:string,conversationId?:string|null):Promise<ChatReply>;stop(id:string):Promise<ChatReply|null>;settle():Promise<ChatReply|null>}
export interface ChatState {conversationId?:string|null;answer:string;running:boolean;stopping:boolean;loaded:boolean;error:string|null;reply:ChatReply|null}
export class ChatController {
 state:ChatState={answer:'',running:false,stopping:false,loaded:false,error:null,reply:null};
 private active:string|null=null;private conversation:string|null=null;private revision=-1;private operation=0;private disposed=false;private unlisten:(()=>void)|null=null;private loading:Promise<void>|null=null;private stopping:Promise<void>|null=null;private freshStart:boolean;
 constructor(private port:ChatPort,private changed:(s:ChatState)=>void,options:{freshStart?:boolean}={}){this.freshStart=options.freshStart===true;}
 private publish(p:Partial<ChatState>){if(!this.disposed){this.state={...this.state,...p,conversationId:this.conversation};this.changed(this.state);}}
 private receive(reply:ChatReply){if(reply.requestId!==this.active||reply.revision<this.revision)return;this.revision=reply.revision;this.conversation=reply.conversationId;this.publish({reply,answer:reply.answer,running:reply.state==='streaming',error:reply.saved?reply.error:reply.state!=='streaming'?'回答尚未保存，请重试保存后再退出':reply.error});}
 async load(){if(this.loading)return this.loading;this.loading=this.loadOnce().finally(()=>{this.loading=null;});return this.loading;}
 private async loadOnce(){
  this.publish({error:null});
  let pending:ChatReply|null=null;
  try {const off=await this.port.listen(reply=>{if(!this.state.loaded)pending=reply;else this.receive(reply);});if(this.disposed){off();return;}this.unlisten?.();this.unlisten=off;
   const view=await this.port.load();if(this.disposed)return;
   const queued=pending as ChatReply|null;
   const recoverable=(r:ChatReply|null)=>!!r&&(!r.saved||r.state==='streaming');
   if(this.freshStart&&!view.activeRequestId&&!recoverable(view.reply)&&!recoverable(queued)){
    this.freshStart=false;this.operation++;this.active=null;this.conversation=null;this.revision=-1;
    this.publish({loaded:true,answer:'',reply:null,running:false,error:null});return;
   }
   this.freshStart=false;this.active=view.activeRequestId??view.reply?.requestId??null;this.revision=-1;this.publish({loaded:true});if(view.reply)this.receive(view.reply);if(pending)this.receive(pending);
  }catch(e){this.publish({error:e instanceof Error?e.message:'对话读取失败，原有记录保留'});}
 }
 async send(prompt:string,save:()=>Promise<void>){return this.sendUsing(crypto.randomUUID(),prompt,save,id=>this.port.start(id,prompt,this.conversation));}
 async sendPaper(paperId:string,options:ReadingOptions,question='',save:()=>Promise<void>=async()=>{}){
  if(!this.port.startPaper){this.publish({error:'论文阅读需要新版桌面程序'});return false;}
  return this.sendUsing('paper-'+crypto.randomUUID(),question.trim()||'论文阅读',save,id=>this.port.startPaper!(id,paperId,options,question));
 }
 async sendSelection(requestId:string,selection:string,actionId:string,expectedRevision=0){
  if(!this.port.startSelection){this.publish({error:'划词操作不可用，请更新桌面程序'});return false;}
  return this.sendUsing(requestId,selection,async()=>{},id=>this.port.startSelection!(id,selection,actionId,expectedRevision));
 }
 private async sendUsing(id:string,prompt:string,save:()=>Promise<void>,start:(id:string)=>Promise<ChatReply>){
  if(this.state.reply&&!this.state.reply.saved&&!this.state.running){this.publish({error:'上一条回答尚未保存，请先重试保存'});return false;}
  if(!this.state.loaded||this.state.running||this.stopping||!prompt.trim())return;
  const previous=this.state.reply,previousAnswer=this.state.answer,previousActive=this.active,previousRevision=this.revision;
  this.active=id;this.revision=-1;const operation=++this.operation;this.publish({answer:'',reply:null,running:true,error:null});
  try {await save();if(this.disposed||operation!==this.operation)return;const reply=await start(id);if(operation!==this.operation)return false;this.receive(reply);return true;}
  catch(e){if(this.disposed||operation!==this.operation)return;try{const stopped=await this.port.stop(id);if(stopped)this.receive(stopped);}catch{}if(this.disposed||operation!==this.operation)return false;if(!this.state.reply){this.active=previousActive;this.revision=previousRevision;this.publish({reply:previous,answer:previousAnswer});}this.publish({running:false,error:e instanceof Error?e.message:'请求未开始，问题草稿保留'});return false;}
 }
 async stop(){
  if(this.stopping)return this.stopping;
  const id=this.active;if(!id)return;
  this.active=null;const operation=++this.operation;this.publish({running:false,stopping:true});
  this.stopping=(async()=>{try{const reply=await this.port.stop(id);if(operation===this.operation&&reply){this.active=id;this.receive(reply);}}catch(e){this.publish({error:e instanceof Error?e.message:'停止或保存未完成，请保留窗口'});throw e;}finally{this.stopping=null;this.publish({stopping:false});}})();
  return this.stopping;
 }
 newConversation(){if(!this.state.loaded||this.state.running||this.stopping||this.state.reply&&!this.state.reply.saved)return false;this.operation++;this.active=null;this.conversation=null;this.revision=-1;this.publish({answer:'',reply:null,error:null});return true;}
 adopt(reply:ChatReply){if(reply.state==='streaming'||!this.newConversation())return false;this.active=reply.requestId;this.receive(reply);return true;}
 async settle(){await this.stop();const reply=await this.port.settle();if(reply){this.active=reply.requestId;this.receive(reply);}}
 dispose(){this.disposed=true;this.operation++;this.unlisten?.();}
}
