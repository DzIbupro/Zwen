import type {SendKeyMode} from './preferences';
export function shouldSendQuestion(event:KeyboardEvent,mode:SendKeyMode='enter'){
 const target=event.target;
 if(!(target instanceof HTMLTextAreaElement&&(target.id==='question'||target.id==='capsule-question'))||event.key!=='Enter'||event.isComposing||event.keyCode===229)return false;
 if(mode==='ctrl-enter')return (event.ctrlKey||event.metaKey)&&!event.shiftKey&&!event.altKey;
 return !event.shiftKey&&!event.ctrlKey&&!event.metaKey&&!event.altKey;
}
