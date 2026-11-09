import type {ChatReply} from './chat';
export interface ActionTicket {requestId:string;selection:string;actionId:string;actionName:string;outputDirectory:string;output:'display'|'document'}
export interface ActionOutputState extends ActionTicket {stage:'waiting'|'saving'|'saved'|'error';path:string|null;error:string|null}
export interface ActionOutputPort {save(input:Omit<ActionTicket,'output'>):Promise<{noteId:string;path:string}>}
/** Tracks one explicitly chosen action. Never starts or retries a model request. */
export class ActionOutputController {
 state:ActionOutputState|null=null;
 private ready=false;private disposed=false;
 constructor(private port:ActionOutputPort,private changed:(state:ActionOutputState|null)=>void){}
 private publish(state:ActionOutputState|null){if(!this.disposed){this.state=state;this.changed(state);}}
 track(ticket:ActionTicket):boolean {
  if(this.state&&this.state.stage!=='saved')return false;
  this.ready=false;this.publish(ticket.output==='document'?{...ticket,stage:'waiting',path:null,error:null}:null);return true;
 }
 observe(reply:ChatReply|null){
  const ticket=this.state;
  if(!ticket||!reply||ticket.requestId!==reply.requestId||ticket.stage!=='waiting')return;
  if(reply.state==='complete'&&reply.saved){this.ready=true;void this.retry();}
 }
 async retry(){
  const ticket=this.state;if(!ticket||!this.ready||ticket.stage==='saving'||ticket.stage==='saved'||this.disposed)return;
  this.publish({...ticket,stage:'saving',error:null});
  try{const result=await this.port.save(ticket);if(this.state?.requestId===ticket.requestId)this.publish({...ticket,stage:'saved',path:result.path,error:null});}
  catch(error){if(this.state?.requestId===ticket.requestId)this.publish({...ticket,stage:'error',error:error instanceof Error?error.message:'文档保存未完成，回答已保留'});}
 }
 dismiss(){if(this.state?.stage!=='saving'){this.ready=false;this.publish(null);}}
 dispose(){this.disposed=true;}
}
