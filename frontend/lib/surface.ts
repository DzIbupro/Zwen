export interface SurfaceInfo { mode:'full'|'capsule'; scaleFactor:number; generation:number; bounds:{width:number;initialHeight:number;maxHeight:number} }
export function boundedHeight(h:number,min:number,max:number):number {return Number.isFinite(h)&&h>=0 ? Math.max(min,Math.min(max,Math.ceil(h))) : min;}
export function canApplySurface(current:number,incoming:number):boolean {return incoming>=current;}

export function cssToLogicalRatio(dpr:number,nativeScale:number):number {return Number.isFinite(dpr)&&dpr>0&&Number.isFinite(nativeScale)&&nativeScale>0?dpr/nativeScale:1;}
export class ResizeQueue {
 private next:{height:number;generation:number}|undefined;
 private running=false;
 constructor(private send:(height:number,generation:number)=>Promise<void>,private error:(e:unknown)=>void){}
 request(height:number,generation:number):void {this.next={height,generation};if(!this.running)void this.drain();}
 clear():void {this.next=undefined;}
 private async drain():Promise<void>{
  this.running=true;
  try {while(this.next){const next=this.next;this.next=undefined;try{await this.send(next.height,next.generation);}catch(e){this.error(e);}}}
  finally{this.running=false;}
 }
}

/** A native selection command already presents the window before delivery. */
export async function ensureSurface(mode:SurfaceInfo['mode'],port:{get():Promise<SurfaceInfo>;set(mode:SurfaceInfo['mode']):Promise<SurfaceInfo>}):Promise<SurfaceInfo>{
 const current=await port.get();return current.mode===mode?current:port.set(mode);
}
