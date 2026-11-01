/// Keeps the webview from navigating: http(s) links inside rendered prose go to
/// the system browser via the passed opener; anything else is ignored.
export function proseLinkClick(e:MouseEvent,open:(url:string)=>void):void{
 const target=e.target as HTMLElement|null;
 const anchor=target?.closest?.('a');
 if(!anchor)return;
 const href=anchor.getAttribute('href')??'';
 if(/^https?:\/\//i.test(href)){
  e.preventDefault();
  open(href);
 }
}
