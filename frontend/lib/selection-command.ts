import type {QuickActionConfig} from './quick-actions';
import type {SelectionItem} from './selection';
export interface SelectionCommand{selectionId:number;actionId:string;configRevision:number;commandId:string}
interface CommandPort{
 refresh():Promise<void>;
 pending():SelectionItem|null;
 freeze(expected:SelectionItem):Promise<SelectionItem|null>;
 loadConfig():Promise<QuickActionConfig>;
 configChanged?(config:QuickActionConfig):void;
 execute(command:SelectionCommand,item:SelectionItem,config:QuickActionConfig):Promise<void>;
}
export class SelectionCommandCoordinator{
 private disposed=false;
 private seen=new Set<string>();
 private inFlight=new Set<string>();
 constructor(private port:CommandPort){}
 async handle(command:SelectionCommand):Promise<void>{
  if(this.disposed||!command||!Number.isSafeInteger(command.selectionId)||command.selectionId<=0||!Number.isSafeInteger(command.configRevision)||command.configRevision<0||typeof command.commandId!=='string'||!command.commandId.length||command.commandId.length>256||typeof command.actionId!=='string'||!command.actionId.length||command.actionId.length>64||this.seen.has(command.commandId)||this.inFlight.has(command.commandId)||this.inFlight.size>=256)return;
  // Record before the first await, including commands refused by preflight.
  // Remember the last 256 commands; refresh/reload never replays one.
  this.seen.add(command.commandId);
  if(this.seen.size>256)this.seen.delete(this.seen.values().next().value!);
  this.inFlight.add(command.commandId);
  try{
  await this.port.refresh();if(this.disposed)return;
  const pending=this.port.pending();
  if(!pending?.text||pending.error||pending.id!==command.selectionId)return;
  const expected={...pending};
  const frozen=await this.port.freeze(expected);if(this.disposed)return;
  if(!frozen?.text||frozen.error||frozen.id!==expected.id||frozen.text!==expected.text)return;
  const config=await this.port.loadConfig();if(this.disposed)return;
  const current=this.port.pending();
  if(current?.id!==frozen.id||current.text!==frozen.text)return;
  if(config.revision!==command.configRevision){
   this.port.configChanged?.(config);
   throw Error('快捷提示词已变化；选区保留，请重新选择动作');
  }
  if(command.actionId!=='ask'&&!config.actions.some(a=>a.id===command.actionId&&a.enabled))return;
  await this.port.execute(command,frozen,config);
  }finally{this.inFlight.delete(command.commandId);}
 }
 dispose(){this.disposed=true;}
}

export function selectionActionBlock(state:{text:string;attachmentCount:number;attachmentPending:boolean;loaded:boolean;unsaved:boolean;running:boolean}):string|null{
 if(state.running)return '请先完成或停止当前回答，再执行划词操作';
 if(!state.loaded)return '正在读取本机草稿、截图和回答，请稍后重试';
 if(state.unsaved)return '请先保存上一次回答，再处理新的选区；材料已保留';
 if(state.text.trim()||state.attachmentPending||state.attachmentCount)return '请先处理现有草稿和截图，再执行划词操作；原材料和选区已保留';
 return null;
}
