import {it,expect} from 'vitest';
import {proseLinkClick} from '../lib/links';
function clickEvent(href:string|null){const anchor=href===null?null:Object.assign(document.createElement('a'),href?{setAttribute:()=>{},getAttribute:()=>href}:{}) ;const div=document.createElement('div');if(anchor)div.appendChild(anchor);const target=anchor??div;const event=new MouseEvent('click',{bubbles:true,cancelable:true});Object.defineProperty(event,'target',{value:target});return event;}
it('http(s) links are intercepted and opened externally',()=>{
 const opened:string[]=[];const event=clickEvent('https://example.com/paper');
 proseLinkClick(event,url=>opened.push(url));
 expect(opened).toEqual(['https://example.com/paper']);
 expect(event.defaultPrevented).toBe(true);
});
it('non-http schemes and plain text are ignored',()=>{
 const opened:string[]=[];
 const event=clickEvent('javascript:alert(1)');
 proseLinkClick(event,url=>opened.push(url));
 const plain=clickEvent(null);
 proseLinkClick(plain,url=>opened.push(url));
 expect(opened).toEqual([]);
 expect(plain.defaultPrevented).toBe(false);
});
