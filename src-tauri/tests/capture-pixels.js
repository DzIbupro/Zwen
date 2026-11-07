(async()=>{
 const api=window.__TAURI__,invoke=api.core.invoke,sleep=ms=>new Promise(r=>setTimeout(r,ms));
 const wait=async f=>{for(let i=0;i<240;i++){const v=await f();if(v)return v;await sleep(30);}throw Error('pixel fixture timeout');};
 const inspect=()=>new Promise(async resolve=>{const off=await api.event.listen('capture-frame-state',e=>{off();resolve(e.payload)});await api.event.emit('capture-frame-inspect',{});});
 const view=()=>invoke('get_attachments');let phase='load';
 try{
  await wait(()=>document.querySelector('.shell'));await wait(async()=>(await inspect()).visible);
  const style=document.createElement('style');style.textContent='*{background:#ff0000!important;color:#ff0000!important;box-shadow:none!important;border-color:#ff0000!important}';document.head.append(style);
  await sleep(2000);await api.event.emit('capture-pixel-stage',{phase:'baseline'});await sleep(500);
  const ids=[];
  for(const mode of ['full','capsule','hidden']){
   phase=mode;
   if(mode==='capsule'){await invoke('set_surface',{mode:'capsule'});await wait(async()=>(await inspect()).visible);await sleep(2000);}
   if(mode==='hidden'){await invoke('hide_window');await wait(async()=>!(await inspect()).visible);}
   await invoke('capture_screenshot',{replaceId:null});
   const count=ids.length+1;await wait(async()=>{const v=await view();if(v.capture.state==='idle'&&v.items.length!==count&&v.notice&&!v.notice.includes('已附加'))throw Error(v.notice);return v.capture.state==='idle'&&v.items.length===count;});
   const v=await view();ids.push(v.items.find(item=>!ids.includes(item.id)).id);await wait(async()=>(await inspect()).visible);await sleep(300);await api.event.emit('capture-pixel-stage',{phase:'restored-'+mode});await sleep(150);
  }
  await api.event.emit('foundation-smoke',{ok:true,mode:'capture-pixels',ids});
 }catch(e){await api.event.emit('foundation-smoke',{ok:false,mode:'capture-pixels',phase,error:String(e)});}
})();
