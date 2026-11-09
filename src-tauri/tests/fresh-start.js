(async()=>{
 const {core:{invoke},event}=window.__TAURI__;
 const wait=async f=>{for(let i=0;i<200;i++){if(await f())return;await new Promise(r=>setTimeout(r,50));}throw Error('fresh startup timed out');};
 try{
  await wait(()=>document.querySelector('.welcome-mark')&&!document.querySelector('#question')?.disabled);
  if(document.querySelector('.answer'))throw Error('previous reply opened on startup');
  const rows=await invoke('get_chat_history',{page:0});
  if(rows.length===1){
   if((await invoke('load_provider')).revision===0)await invoke('save_provider',{expectedRevision:0,config:{baseUrl:window.__ZWEN_FIXTURE_BASE__,model:'fixture-model',keyEnv:'ZWEN_PROVIDER_TEST_KEY'}});
   document.querySelector('[data-nav=settings]').click();await wait(()=>document.querySelector('.settings button'));
   [...document.querySelectorAll('.settings button')].find(b=>b.textContent==='重新读取').click();await wait(()=>document.querySelector('#model-name')?.value==='fixture-model');
   document.querySelector('[data-nav=chat]').click();await wait(()=>document.querySelector('#question'));
   const input=document.querySelector('#question');input.value='fresh-fixture';input.dispatchEvent(new Event('input',{bubbles:true}));
   await wait(()=>!document.querySelector('[data-send]').disabled);
   const key='fixture-blank-reload-'+window.__ZWEN_RUN_ID__;
   if(!sessionStorage.getItem(key)){sessionStorage.setItem(key,'1');document.querySelector('[aria-label="保存草稿"]').click();await wait(async()=>(await invoke('load_draft')).text==='fresh-fixture');location.reload();return;}
   document.querySelector('[data-send]').click();
   await wait(()=>document.querySelector('.answer .prose')?.textContent.includes('new synthetic answer'));
   await wait(async()=>{const r=(await invoke('get_chat')).reply;return r?.state==='complete'&&r.saved;});
   input.value='保留的合成草稿';input.dispatchEvent(new Event('input',{bubbles:true}));
   document.querySelector('[data-quit-app]').click();
  }else if(rows.length===2){
   if(document.querySelector('#question').value!=='保留的合成草稿')throw Error('unsent draft lost on restart');
   document.querySelector('[data-nav=history]').click();await wait(()=>document.querySelectorAll('.records button').length===2);
   const old=[...document.querySelectorAll('.records button')].find(b=>b.textContent.includes('以前的问题'));if(!old)throw Error('history disappeared');old.click();
   await wait(()=>document.querySelector('.record-detail .prose')?.textContent.includes('以前的回答'));
   await event.emit('foundation-smoke',{ok:true,mode:'fresh-start',freshOnRestart:true,draftPreserved:true,historyAccessible:true,independentRequest:true});
  }else throw Error('unexpected history count '+rows.length);
 }catch(e){await event.emit('foundation-smoke',{ok:false,mode:'fresh-start',error:String(e),ui:document.body.innerText.slice(0,1200)});}
})();
