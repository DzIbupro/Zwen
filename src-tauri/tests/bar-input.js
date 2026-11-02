(async()=>{
 const api=window.__TAURI__,invoke=api.core.invoke;const wait=async(check)=>{for(let i=0;i<100;i++){if(await check())return;await new Promise(r=>setTimeout(r,50));}throw Error('input fixture startup');};
 try{await wait(()=>document.querySelector('.shell'));await invoke('load_draft');await invoke('hide_window');await invoke('candidate_set_enabled',{enabled:true});
  const seen=new Set();await api.event.listen('selection-bar-state',e=>{const b=e.payload;if(b.visible&&!seen.has(b.selectionId)){seen.add(b.selectionId);void api.event.emit('bar-input-observed',b);}});
  setInterval(()=>void api.event.emit('selection-bar-test','inspect'),100);await api.event.emit('bar-input-ready',true);
 }catch(e){await api.event.emit('foundation-smoke',{ok:false,mode:'bar-input',error:String(e)});}
})();
