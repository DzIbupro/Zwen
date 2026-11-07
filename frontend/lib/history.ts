import type {ChatReply} from './chat';
export interface Summary {requestId:string;conversationId:string;title:string;model:string;state:ChatReply['state'];createdAt:number}
export interface HistoryPort {list(page:number):Promise<Summary[]>;get(id:string):Promise<ChatReply|null>;remove?(id:string):Promise<void>}
export interface HistoryState {items:Summary[];selected:ChatReply|null;page:number;hasMore:boolean;loading:boolean;deleting:boolean;notice:string|null;error:string|null}
export class HistoryController {
 state:HistoryState={items:[],selected:null,page:0,hasMore:false,loading:false,deleting:false,notice:null,error:null};private operation=0;private disposed=false;
 constructor(private port:HistoryPort,private changed:(s:HistoryState)=>void,private removed:(id:string)=>void=()=>{}){}
 private publish(p:Partial<HistoryState>){if(!this.disposed){this.state={...this.state,...p};this.changed(this.state);}}
 async open(id:string){if(this.disposed||this.state.deleting)return;const operation=++this.operation;this.publish({loading:true,error:null});try{const reply=await this.port.get(id);if(operation===this.operation)this.publish({selected:reply,loading:false,error:reply?null:'此记录已删除或暂不可读取'});}catch(e){if(operation===this.operation)this.publish({loading:false,error:e instanceof Error?e.message:'历史读取失败'});}}
 async load(page:number){if(this.disposed||this.state.deleting)return;const operation=++this.operation;this.publish({loading:true,error:null});try{const rows=await this.port.list(page);if(operation===this.operation)this.publish({items:rows.slice(0,20),hasMore:rows.length>20,page,loading:false});}catch(e){if(operation===this.operation)this.publish({loading:false,error:e instanceof Error?e.message:'历史读取失败'});}}
 async removeSelected(){
  const target=this.state.selected;
  if(this.disposed||this.state.loading||this.state.deleting||!target||!target.saved||target.state==='streaming'||!this.port.remove)return;
  ++this.operation;this.publish({deleting:true,error:null,notice:null});
  try{
   await this.port.remove(target.requestId);this.removed(target.requestId);
   this.publish({selected:null,deleting:false,notice:'已删除这条问答记录'});
   if(this.disposed)return;await this.load(this.state.page);
   if(this.state.page>0&&!this.state.items.length&&!this.state.error)await this.load(this.state.page-1);
  }catch(e){this.publish({error:e instanceof Error?e.message:'删除失败，记录仍保留'});}
  finally{this.publish({deleting:false});}
 }
 dispose(){this.disposed=true;this.operation++;}
}
