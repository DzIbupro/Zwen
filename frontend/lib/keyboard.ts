export function shouldSendQuestion(event:KeyboardEvent){
 return event.target instanceof HTMLTextAreaElement&&(event.ctrlKey||event.metaKey)&&event.key==='Enter'&&!event.isComposing&&event.keyCode!==229;
}
