export function shouldSendQuestion(event:KeyboardEvent){
 const target=event.target;
 return target instanceof HTMLTextAreaElement&&(target.id==='question'||target.id==='capsule-question')&&event.key==='Enter'&&!event.shiftKey&&!event.ctrlKey&&!event.metaKey&&!event.altKey&&!event.isComposing&&event.keyCode!==229;
}
