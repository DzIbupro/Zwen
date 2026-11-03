import {it,expect} from 'vitest';
import {shouldSendQuestion} from '../lib/keyboard';
function dispatch(tag:string,options:KeyboardEventInit,id='question'){const el=document.createElement(tag);el.id=id;let answer:boolean|undefined;el.addEventListener('keydown',e=>answer=shouldSendQuestion(e as KeyboardEvent));el.dispatchEvent(new KeyboardEvent('keydown',options));return answer;}
it('sends with plain Enter in either question composer',()=>{for(const id of ['question','capsule-question'])expect(dispatch('textarea',{key:'Enter'},id)).toBe(true);});
it('keeps Shift Enter for newlines and rejects old or modified shortcuts',()=>{for(const option of ['shiftKey','ctrlKey','metaKey','altKey'])expect(dispatch('textarea',{key:'Enter',[option]:true})).toBe(false);});
it('never sends while accepting an IME candidate',()=>{expect(dispatch('textarea',{key:'Enter',isComposing:true})).toBe(false);expect(dispatch('textarea',{key:'Enter',keyCode:229})).toBe(false);});
it('does not send from settings or knowledge editors',()=>{expect(dispatch('input',{key:'Enter'})).toBe(false);expect(dispatch('textarea',{key:'Enter'},'note-body')).toBe(false);});
