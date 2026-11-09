/** Only an explicitly chosen selection command owns a disposable quick task. */
export interface QuickSessionClosePort {
 settle():Promise<void>;
 revokeSend():void;
 discardSelection(id:number):Promise<void>;
 clearMaterials():Promise<void>;
 clearDraft():Promise<void>;
 restoreDraft():Promise<void>;
 resetView():void;
}
export class QuickSession {
 private selectionId:number|null=null;
 closing=false;
 release(){if(!this.closing)this.selectionId=null;}
 get active(){return this.selectionId!==null;}
 begin(selectionId:number):boolean{if(this.closing)return false;this.selectionId=selectionId;return true;}
 async close(port:QuickSessionClosePort):Promise<boolean>{
  if(this.closing||this.selectionId===null)return false;
  const id=this.selectionId;this.closing=true;
  try{
   await port.settle();
   port.revokeSend();
   await port.discardSelection(id);
   await port.clearDraft();
   try{await port.clearMaterials();port.resetView();}
   catch(error){await port.restoreDraft();throw error;}
   this.selectionId=null;
   return true;
  }finally{this.closing=false;}
 }
}
