import type {ChatReply} from './chat';
export interface Summary {requestId:string;conversationId:string;title:string;model:string;state:ChatReply['state'];createdAt:number}
export interface HistoryPort {list(page:number):Promise<Summary[]>;get(id:string):Promise<ChatReply|null>}
export interface HistoryState {items:Summary[];selected:ChatReply|null;page:number;hasMore:boolean;loading:boolean;error:string|null}
export class HistoryController {
 state:HistoryState={items:[],selected:null,page:0,hasMore:false,loading:false,error:null};private operation=0;private disposed=false;
 constructor(private port:HistoryPort,private changed:(s:HistoryState)=>void){}
 private publish(p:Partial<HistoryState>){if(!this.disposed){this.state={...this.state,...p};this.changed(this.state);}}
 async open(id:string){const operation=++this.operation;this.publish({loading:true,error:null});try{const reply=await this.port.get(id);if(operation===this.operation)this.publish({selected:reply,loading:false,error:reply?null:'此记录暂不可读取，原文件保留'});}catch(e){if(operation===this.operation)this.publish({loading:false,error:e instanceof Error?e.message:'历史读取失败'});}}
 async load(page:number){const operation=++this.operation;this.publish({loading:true,error:null});try{const rows=await this.port.list(page);if(operation===this.operation)this.publish({items:rows.slice(0,20),hasMore:rows.length>20,page,selected:null,loading:false});}catch(e){if(operation===this.operation)this.publish({loading:false,error:e instanceof Error?e.message:'历史读取失败'});}}
 dispose(){this.disposed=true;this.operation++;}
}
