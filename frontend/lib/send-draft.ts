import type {DraftController} from './draft';
import type {ChatState} from './chat';
interface Ticket {text:string;recoveryText:string;source:'draft'|'selection'|'paper';ids:string[];version:number;requestId:string|null;clearedVersion:number|null;accepted:boolean}
/** Ownership spans synchronous terminal events, delayed start acknowledgements and edits. */
export class SendDraftCoordinator {
 private pending:Ticket|null=null;
 constructor(private draft:DraftController,private consume:(ids:string[])=>void){}
 begin(ids:string[],source:'draft'|'selection'|'paper'='draft'):Ticket|null{
  if(this.pending)return null;
  const ticket:Ticket={text:this.draft.snapshot.text,recoveryText:this.draft.snapshot.text,source,ids:source==='paper'?[]:[...ids],version:this.draft.editVersion,requestId:null,clearedVersion:null,accepted:false};
  this.pending=ticket;return ticket;
 }
 revoke(){this.pending=null;}
 cancel(ticket:Ticket){if(this.pending===ticket)this.pending=null;}
 accept(ticket:Ticket,state:ChatState){
  if(this.pending!==ticket)return;
  ticket.accepted=true;ticket.requestId=state.reply?.requestId??null;if(ticket.source==='selection'&&state.reply)ticket.recoveryText=state.reply.prompt;
  if(this.draft.snapshot.loaded&&this.draft.editVersion===ticket.version&&this.draft.snapshot.text===ticket.text){
   this.draft.edit('');ticket.clearedVersion=this.draft.editVersion;
  }
  this.reconcile(state);
 }
 reconcile(state:ChatState){
  const ticket=this.pending,reply=state.reply;
  if(!ticket?.accepted||!reply||reply.requestId!==ticket.requestId)return;
  if(reply.state==='complete'&&reply.saved){
   this.pending=null;if(ticket.ids.length)this.consume(ticket.ids);return;
  }
  if(!state.running&&!state.stopping&&reply.state!=='streaming'){
   this.pending=null;
   if(ticket.clearedVersion!==null&&this.draft.snapshot.loaded&&this.draft.editVersion===ticket.clearedVersion&&this.draft.snapshot.text==='')this.draft.edit(ticket.recoveryText);
  }
 }
}
