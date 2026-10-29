import {it,expect} from 'vitest';
import {shouldSendQuestion} from '../lib/keyboard';
function dispatch(tag:string,options:KeyboardEventInit){const el=document.createElement(tag);let answer:boolean|undefined;el.addEventListener('keydown',e=>answer=shouldSendQuestion(e as KeyboardEvent));el.dispatchEvent(new KeyboardEvent('keydown',options));return answer;}
it('does not send a draft while editing model settings',()=>{expect(dispatch('input',{key:'Enter',ctrlKey:true})).toBe(false);});
it('sends only an explicit shortcut from the question textarea',()=>{expect(dispatch('textarea',{key:'Enter',ctrlKey:true})).toBe(true);expect(dispatch('textarea',{key:'Enter'})).toBe(false);expect(dispatch('textarea',{key:'Enter',ctrlKey:true,isComposing:true})).toBe(false);});
