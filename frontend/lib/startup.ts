const KEY='zwen.native-run.v1';
/** Session-only launch state: remember whether this run has an attached reply. */
export class StartupSession{
 readonly fresh:boolean;private runId:string|null;private last:string|null=null;private unavailable=false;
 constructor(runId:unknown,private store:Pick<Storage,'getItem'|'setItem'>){
  this.runId=typeof runId==='string'&&/^[a-zA-Z0-9-]{1,100}$/.test(runId)?runId:null;
  try{const raw=store.getItem(KEY);const prior=JSON.parse(raw??'null');this.fresh=!this.runId||prior?.runId!==this.runId||prior?.attached!==true;this.last=raw;}catch{this.fresh=true;}
 }
 markLoaded(attached=false){
  if(!this.runId||this.unavailable)return;const value=JSON.stringify({runId:this.runId,attached});if(value===this.last)return;
  try{this.store.setItem(KEY,value);this.last=value;}catch{this.unavailable=true;}
 }
}
