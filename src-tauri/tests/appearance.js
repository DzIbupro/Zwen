(async()=>{
 const {core:{invoke},event}=window.__TAURI__;
 const wait=async f=>{for(let i=0;i<200;i++){if(await f())return;await new Promise(r=>setTimeout(r,50));}throw Error('appearance timeout');};
 try{
  await wait(()=>document.querySelector('[data-sidebar]')&&!document.querySelector('.sidebar-toggle').disabled);
  const prior=await invoke('load_appearance'),restored=prior.theme==='dark';
  if(document.documentElement.dataset.theme!==prior.theme||document.querySelector('[data-sidebar]').dataset.collapsed!==String(prior.sidebarCollapsed))throw Error('startup preference injection mismatch');
  if(restored){if(!prior.sidebarCollapsed||document.querySelector('#question').value!=='appearance fixture')throw Error('restart lost draft or collapse');}
  else{
   const q=document.querySelector('#question');q.value='appearance fixture';q.dispatchEvent(new Event('input',{bubbles:true}));
   document.querySelector('.sidebar-toggle').click();document.querySelector('[data-nav=settings]').click();await wait(()=>document.querySelector('[name=interface-theme][value=dark]'));
   document.querySelector('[name=interface-theme][value=dark]').click();
   await wait(async()=>{const p=await invoke('load_appearance');return p.theme==='dark'&&p.sidebarCollapsed;});
   document.querySelector('[data-nav=chat]').click();await wait(()=>document.querySelector('#question'));
   if(document.querySelector('#question').value!=='appearance fixture')throw Error('navigation discarded draft');
   document.querySelector('[aria-label=保存草稿]').click();await wait(()=>document.body.innerText.includes('已保存到本机'));
  }
  if((await invoke('get_chat')).reply)throw Error('appearance generated a request');
  await event.emit('foundation-smoke',{ok:true,mode:'appearance',restored,preferences:await invoke('load_appearance'),draftPreserved:true});
 }catch(e){await event.emit('foundation-smoke',{ok:false,mode:'appearance',error:String(e)});}
})();
