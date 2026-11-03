(async()=>{
 const api=window.__TAURI__;const invoke=api.core.invoke;
 const wait=async(check)=>{for(let i=0;i<100;i++){if(await check())return;await new Promise(r=>setTimeout(r,80));}throw Error('Provider fixture timeout');};
 try{
  await wait(()=>document.querySelector('textarea')&&!document.querySelector('textarea').disabled);
  const before=await invoke('load_draft');
  const initial=await invoke('load_provider');if(initial.revision!==0)throw Error('Unexpected fixture database');
  const config={baseUrl:'https://models.example/v1',model:'fixture-a',keyEnv:'ZWEN_PROVIDER_TEST_KEY'};
  const saved=await invoke('save_provider',{expectedRevision:0,config});
  if(saved.revision!==1||!saved.keyAvailable||JSON.stringify(saved).includes('synthetic-test-token'))throw Error('Provider snapshot leaked key or was not saved');
  let rejected=false;try{await invoke('save_provider',{expectedRevision:0,config});}catch{rejected=true;}if(!rejected)throw Error('Stale save accepted');
  rejected=false;try{await invoke('save_provider',{expectedRevision:1,config:{...config,baseUrl:'http://remote.example'}});}catch{rejected=true;}if(!rejected)throw Error('Unsafe endpoint accepted');
  [...document.querySelectorAll('nav button')].find(b=>b.textContent==='设置').click();
  await wait(()=>[...document.querySelectorAll('.settings button')].some(b=>b.textContent==='重新读取'));
  [...document.querySelectorAll('.settings button')].find(b=>b.textContent==='重新读取').click();
  await wait(()=>document.querySelector('#model-name')?.value==='fixture-a');
  const input=document.querySelector('#model-name');input.value='fixture-b';input.dispatchEvent(new Event('input',{bubbles:true}));
  document.querySelector('.settings button[type=submit]').click();
  await wait(async()=>{const s=await invoke('load_provider');return s.revision===2&&s.config.model==='fixture-b';});
  await invoke('save_provider',{expectedRevision:2,config:{...config,model:'external-c'}});
  input.value='keep-unsaved';input.dispatchEvent(new Event('input',{bubbles:true}));document.querySelector('.settings button[type=submit]').click();
  await wait(()=>document.querySelector('.settings [role=alert]'));
  if(input.value!=='keep-unsaved')throw Error('Unsaved settings lost after conflict');
  const after=await invoke('load_draft');if(JSON.stringify(before)!==JSON.stringify(after))throw Error('Settings modified draft');
  await api.event.emit('foundation-smoke',{ok:true,mode:'provider',cas:true,keyRedacted:true,unsafeUrlRejected:true,draftPreserved:true,failedSavePreserved:true});
 }catch(e){await api.event.emit('foundation-smoke',{ok:false,mode:'provider',error:String(e)});}
})();
