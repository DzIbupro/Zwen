export interface RoundView {round:{id:string;profile:string;noteIds:string[];completed:number;batchIds:string[];undone:boolean};status:string;model:string;baseUrl:string;error:string}
export interface RoundPort {prepare(ids:string[]|null):Promise<RoundView>;status():Promise<RoundView|null>;start(id:string):Promise<RoundView>;stop(id:string):Promise<void>;undo(id:string):Promise<RoundView>}
export interface RoundState {view:RoundView|null;busy:boolean;confirming:boolean;error:string}
export class RoundController {
 state:RoundState={view:null,busy:false,confirming:false,error:''};private disposed=false;private epoch=0;private timer:ReturnType<typeof setTimeout>|null=null;
 constructor(private port:RoundPort,private changed:(state:RoundState)=>void){}
 private publish(patch:Partial<RoundState>){if(this.disposed)return;this.state={...this.state,...patch};this.changed(this.state);}
 private schedule(){if(this.timer)clearTimeout(this.timer);if(!this.disposed&&['running','stopping'].includes(this.state.view?.status??''))this.timer=setTimeout(()=>void this.restore(),500);}
 async restore(){if(this.disposed||this.state.busy)return;const epoch=this.epoch;try{const view=await this.port.status();if(epoch!==this.epoch||this.disposed)return;this.publish({view,error:''});}catch(e){if(epoch===this.epoch)this.publish({error:String(e instanceof Error?e.message:e)});}finally{if(epoch===this.epoch)this.schedule();}}
 async prepare(ids:string[]|null){if(this.disposed||this.state.busy||this.active())return;this.epoch++;this.publish({busy:true,error:''});try{const view=await this.port.prepare(ids);this.publish({view,confirming:true});}catch(e){this.publish({error:e instanceof Error?e.message:String(e)});}finally{this.publish({busy:false});}}
 active(){return ['running','stopping'].includes(this.state.view?.status??'');}
 confirmRetry(){if(this.state.view&&!this.active())this.publish({confirming:true,error:''});}
 cancelConfirm(){this.publish({confirming:false});}
 async start(){if(this.disposed||this.state.busy||!this.state.confirming||!this.state.view)return;this.epoch++;this.publish({busy:true,error:''});try{const view=await this.port.start(this.state.view.round.id);this.publish({view,confirming:false});this.schedule();}catch(e){this.publish({error:e instanceof Error?e.message:String(e)});}finally{this.publish({busy:false});}}
 async stop(){if(this.disposed||this.state.busy||!this.state.view)return;this.publish({busy:true});try{await this.port.stop(this.state.view.round.id);}catch(e){this.publish({error:e instanceof Error?e.message:String(e)});}finally{this.publish({busy:false});}await this.restore();}
 async undo(){if(this.disposed||this.state.busy||this.active()||!this.state.view)return;this.epoch++;this.publish({busy:true,error:''});try{const view=await this.port.undo(this.state.view.round.id);this.publish({view,confirming:false});}catch(e){this.publish({error:e instanceof Error?e.message:String(e)});}finally{this.publish({busy:false});}}
 dispose(){this.disposed=true;this.epoch++;if(this.timer)clearTimeout(this.timer);}
}
