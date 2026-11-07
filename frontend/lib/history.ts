import type {ChatReply} from './chat';
export interface Summary {requestId:string;conversationId:string;title:string;model:string;state:ChatReply['state'];createdAt:number}
export interface HistoryPort {list(page:number):Promise<Summary[]>;get(id:string):Promise<ChatReply|null>;remove?(id:string):Promise<void>;removeMany?(ids:string[]):Promise<number>;clear?():Promise<number>}
export interface HistoryState {items:Summary[];selected:ChatReply|null;page:number;hasMore:boolean;loading:boolean;deleting:boolean;checked:Record<string,boolean>;notice:string|null;error:string|null}
export class HistoryController {
 state:HistoryState={items:[],selected:null,page:0,hasMore:false,loading:false,deleting:false,checked:{},notice:null,error:null};private operation=0;private disposed=false;
 constructor(private port:HistoryPort,private changed:(s:HistoryState)=>void,private removed:(id:string)=>void=()=>{},private bulkRemoved:(ids:string[]|null)=>void=()=>{}){}
 private publish(p:Partial<HistoryState>){if(!this.disposed){this.state={...this.state,...p};this.changed(this.state);}}
 async open(id:string){if(this.disposed||this.state.deleting)return;const operation=++this.operation;this.publish({loading:true,error:null});try{const reply=await this.port.get(id);if(operation===this.operation)this.publish({selected:reply,loading:false,error:reply?null:'此记录已删除或暂不可读取'});}catch(e){if(operation===this.operation)this.publish({loading:false,error:e instanceof Error?e.message:'历史读取失败'});}}
 async load(page:number){if(this.disposed||this.state.deleting)return;const operation=++this.operation;this.publish({loading:true,error:null});try{const rows=await this.port.list(page);if(operation===this.operation)this.publish({items:rows.slice(0,20),hasMore:rows.length>20,page,loading:false});}catch(e){if(operation===this.operation)this.publish({loading:false,error:e instanceof Error?e.message:'历史读取失败'});}}
 toggleChecked(id:string){
  if(this.disposed||this.state.loading||this.state.deleting||!this.state.items.some(r=>r.requestId===id&&r.state!=='streaming'))return;
  const checked={...this.state.checked};if(checked[id])delete checked[id];else checked[id]=true;this.publish({checked});
 }
 togglePage(){
  if(this.disposed||this.state.loading||this.state.deleting)return;
  const ids=this.state.items.filter(r=>r.state!=='streaming').map(r=>r.requestId);const checked={...this.state.checked};const all=ids.every(id=>checked[id]);for(const id of ids){if(all)delete checked[id];else checked[id]=true;}this.publish({checked});
 }
 clearChecked(){if(!this.state.loading&&!this.state.deleting)this.publish({checked:{}});}
 async removeSelected(){
  const target=this.state.selected;if(!target||!target.saved||target.state==='streaming'||!this.port.remove)return;
  await this.deleteTargets([target.requestId],true);
 }
 async removeMany(ids:string[]){if(ids.length&&this.port.removeMany)await this.deleteTargets([...new Set(ids)],false);}
 async clearAll(){if(this.port.clear)await this.deleteTargets(null,false);}
 private async deleteTargets(ids:string[]|null,single:boolean){
  if(this.disposed||this.state.loading||this.state.deleting)return;
  ++this.operation;this.publish({deleting:true,error:null,notice:null});
  try{
   const count=single?(await this.port.remove!(ids![0]),1):ids===null?await this.port.clear!():await this.port.removeMany!(ids);
   if(single)this.removed(ids![0]);else this.bulkRemoved(ids);
   const checked={...this.state.checked};if(ids)for(const id of ids)delete checked[id];
   const selected=ids===null||ids.includes(this.state.selected?.requestId??'')?null:this.state.selected;
   this.publish({selected,checked:ids===null?{}:checked,deleting:false,notice:single?'已删除这条问答记录':`已删除 ${count} 条问答记录`});
   if(this.disposed)return;await this.load(single?this.state.page:0);
   if(this.state.page>0&&!this.state.items.length&&!this.state.error)await this.load(this.state.page-1);
  }catch(e){this.publish({error:e instanceof Error?e.message:'删除失败，记录仍保留'});}
  finally{this.publish({deleting:false});}
 }
 dispose(){this.disposed=true;this.operation++;}
}
