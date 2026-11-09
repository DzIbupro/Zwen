// Tauri supplies local paths; no DOM FileReader, uploads or model calls here.
type DropEvent={type:'enter'|'drop';paths:string[]}|{type:'over'}|{type:'leave'};
interface Hooks {blocked():string|null;hover(value:boolean):void;note(message:string):void;import(paths:string[]):Promise<void>}
export function createPaperDropHandler(hooks:Hooks){return async(event:DropEvent)=>{
 if(event.type==='leave'){hooks.hover(false);return;}
 if(event.type==='over'){if(hooks.blocked())hooks.hover(false);return;}
 const valid=event.paths.length===1&&event.paths[0].toLowerCase().endsWith('.pdf');
 const reason=hooks.blocked();
 if(event.type==='enter'){hooks.hover(valid&&!reason);return;}
 hooks.hover(false);
 if(reason){hooks.note(reason);return;}
 if(!valid){hooks.note('请拖入单篇 PDF；当前附件保留');return;}
 hooks.note('');await hooks.import(event.paths);
};}
