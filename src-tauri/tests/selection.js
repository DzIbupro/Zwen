(async()=>{
 const {core:{invoke},event:{emit,listen}}=window.__TAURI__;
 const wait=async(check)=>{for(let i=0;i<140;i++){if(await check())return;await new Promise(r=>setTimeout(r,50));}throw Error('Selection fixture timeout');};
 const draft=()=>invoke('load_draft');
 const stage=async s=>emit('selection-progress',{stage:s});
 try{
  await wait(()=>document.querySelector('[data-capsule] textarea'));
  if(window.__ZWEN_SMOKE_MODE__==='selection-lifecycle'){
   await wait(async()=>(await draft()).text.includes('合成选区'));
   await wait(async()=>!(await invoke('claim_selection')).item);
   await stage('hide-during');await wait(async()=>(await invoke('claim_selection')).reading);
   await invoke('hide_window');await new Promise(r=>setTimeout(r,850));await stage('assert-hidden');
   await wait(async()=>!(await invoke('claim_selection')).item);
   await stage('expand-during');await wait(async()=>(await invoke('claim_selection')).reading);
   await invoke('set_surface',{mode:'full'});await new Promise(r=>setTimeout(r,1000));
   if((await invoke('get_surface')).mode!=='full')throw Error('Late selection overrode explicit full window');
   await emit('foundation-smoke',{ok:true,mode:'selection-lifecycle',hidePreserved:true,expandPreserved:true});return;
  }
  if(window.__ZWEN_SMOKE_MODE__.startsWith('selection-recovery')){
   await wait(()=>document.querySelector('.selection-notice details'));
   if((await draft()).text!=='合成旧草稿')throw Error('Claimed selection duplicated automatically');
   let exitBlocked=false;try{await invoke('quit_app');}catch{exitBlocked=true;}if(!exitBlocked)throw Error('Quit discarded pending selection');
   if(window.__ZWEN_SMOKE_MODE__==='selection-recovery'){
    await stage('recover');return;
   }
   document.querySelector('.selection-notice details').open=true;
   [...document.querySelectorAll('.selection-notice button')].find(b=>b.textContent.includes('已检查草稿')).click();
   await wait(async()=>!(await invoke('claim_selection')).item);
   if((await draft()).text!=='合成旧草稿\n\n请解释以下选区：\n\n合成选区')throw Error('Explicit recovery did not save once');
   await emit('foundation-smoke',{ok:true,mode:'selection-recovery',stagedAfterReload:true});return;
  }
  const first='合成旧草稿\n\n请解释以下选区：\n\n合成选区';
  await wait(async()=>(await draft()).text===first);
  await wait(async()=>!(await invoke('claim_selection')).item);
  await stage('warm');
  const second=first+'\n\n请将以下选区翻译成中文：\n\n第二段';
  await wait(async()=>(await draft()).text===second);
  await wait(async()=>!(await invoke('claim_selection')).item);
  await stage('failure');
  await wait(()=>document.querySelector('.selection-notice')?.textContent.includes('未能读取PRIMARY'));
  if((await draft()).text!==second)throw Error('Failed capture overwrote the draft');
  await invoke('hide_window');await stage('burst');
  const third=second+'\n\n请解释以下选区：\n\n并发选区';
  await wait(async()=>(await draft()).text===third);
  await wait(async()=>!(await invoke('claim_selection')).item);
  if((await invoke('get_surface')).mode!=='capsule')throw Error('Selection opened full window');
  if((await invoke('get_chat')).activeRequestId)throw Error('Selection started a model request');
  await emit('foundation-smoke',{ok:true,mode:'selection',cold:true,warm:true,failedCapturePreserved:true,burst:true,neverSent:true});
 }catch(e){await emit('foundation-smoke',{ok:false,mode:'selection',error:String(e)});}
})();
