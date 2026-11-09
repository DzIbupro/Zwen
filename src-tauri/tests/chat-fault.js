(async()=>{
 const api=window.__TAURI__,invoke=api.core.invoke,mode=window.__ZWEN_SMOKE_MODE__;
 const wait=async(check)=>{for(let i=0;i<300;i++){if(await check())return;await new Promise(r=>setTimeout(r,50));}throw Error('Fault fixture timeout');};
 try{
  if(mode==='chat-fault-reload'){
   await wait(()=>document.querySelector('.prose')?.textContent.includes('fault-partial'));
   await wait(()=>!document.querySelector('textarea')?.disabled);
   if(!document.querySelector('[data-send]').disabled)throw Error('Send enabled over unsaved answer');
   const box=document.querySelector('textarea');box.value='must-not-send';box.dispatchEvent(new Event('input',{bubbles:true}));box.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',ctrlKey:true,bubbles:true}));
   await new Promise(r=>setTimeout(r,100));if(!document.querySelector('.prose').textContent.includes('fault-partial'))throw Error('Unsaved answer erased');
   let rejected=false;try{await invoke('settle_chat');}catch{rejected=true;}if(!rejected)throw Error('Quit settlement accepted unsaved answer');
   await api.event.emit('foundation-smoke',{mode,ok:true,ramRecovered:true,unsavedSendBlocked:true,unsafeQuitBlocked:true});return;
  }
  let unsaved=null;await api.event.listen('chat-update',e=>{if(e.payload.requestId==='fault-fixture'&&!e.payload.saved&&e.payload.state==='failed')unsaved=e.payload;});
  await invoke('save_provider',{expectedRevision:0,config:{baseUrl:window.__ZWEN_FIXTURE_BASE__,model:'fixture-model',keyEnv:'ZWEN_PROVIDER_TEST_KEY'}});
  await invoke('start_chat',{requestId:'fault-fixture',prompt:'fault-fixture',conversationId:null});await wait(()=>unsaved);
  let ready=false;await api.event.listen('fault-ready',()=>ready=true);await api.event.emit('fault-stage','unsaved');await wait(()=>ready);
  const snapshot=await invoke('get_chat');if(snapshot.reply?.answer!=='fault-partial'||snapshot.reply.saved)throw Error('Retained answer unavailable');
  location.reload();
 }catch(e){await api.event.emit('foundation-smoke',{mode,ok:false,error:String(e)});}
})();
