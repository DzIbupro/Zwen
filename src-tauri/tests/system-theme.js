(async()=>{
 const {core:{invoke},event}=window.__TAURI__;
 const wait=async f=>{for(let i=0;i<160;i++){if(await f())return;await new Promise(r=>setTimeout(r,25));}throw Error('system theme timeout');};
 const assert=(v,m)=>{if(!v)throw Error(m);};
 try{
 await wait(()=>document.querySelector('[data-sidebar]')&&!document.querySelector('.sidebar-toggle').disabled);
 const marker='system-fixture-'+window.__ZWEN_RUN_ID__;const resumed=sessionStorage.getItem(marker);if(resumed){await wait(()=>document.documentElement.dataset.theme==='dark');await event.emit('foundation-smoke',JSON.parse(resumed));return;}
 const prior=await invoke('load_appearance');const restored=prior.theme==='system';
 document.querySelector('[data-nav=settings]').click();await wait(()=>document.querySelector('[value=system]'));
 document.querySelector('[value=system]').click();await wait(async()=> (await invoke('load_appearance')).theme==='system');
 for(const dark of [true,false,true]){await event.emit('system-theme-fixture',dark);await wait(()=>document.documentElement.dataset.theme===(dark?'dark':'light'));}
 document.querySelector('[value=light]').click();await wait(async()=> (await invoke('load_appearance')).theme==='light');await event.emit('system-theme-fixture',false);await wait(()=>window.__ZWEN_SYSTEM_THEME__==='light');await event.emit('system-theme-fixture',true);await wait(()=>window.__ZWEN_SYSTEM_THEME__==='dark');assert(document.documentElement.dataset.theme==='light','manual theme overridden');
 document.querySelector('[value=system]').click();await wait(()=>document.documentElement.dataset.theme==='dark');
 document.querySelector('[data-nav=chat]').click();
 const logo=()=>document.querySelector('.sidebar-brand img').getBoundingClientRect().x;
 const initial=logo();const samples=[initial];document.querySelector('.sidebar-toggle').click();for(let i=0;i<24;i++){await new Promise(requestAnimationFrame);samples.push(logo());}assert(samples.every(x=>Math.abs(x-initial)<1),'logo jumped '+samples.join(','));
 document.querySelector('.sidebar-toggle').click();await new Promise(r=>setTimeout(r,350));assert(Math.abs(logo()-initial)<1,'expanded logo moved');
 assert(!(await invoke('get_chat')).reply,'unexpected model request');
 sessionStorage.setItem(marker,JSON.stringify({ok:true,mode:'system-theme',restored,samples,rendererReload:true,preferences:await invoke('load_appearance')}));location.reload();
 }catch(e){await event.emit('foundation-smoke',{ok:false,mode:'system-theme',error:String(e)});}
})();