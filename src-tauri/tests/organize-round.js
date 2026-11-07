(async()=>{
 const invoke=window.__TAURI__.core.invoke;
 const wait=async check=>{for(let i=0;i<400;i++){const value=await check();if(value)return value;await new Promise(r=>setTimeout(r,50));}throw Error('round fixture timeout');};
 try{
  if(localStorage.getItem('round-reload')==='yes'){
   const v=await wait(async()=>{const v=await invoke('organize_round_status');return v?.status==='stopped'?v:null;});
   if(v.round.completed!==0)throw Error('late result committed after stop');
   localStorage.removeItem('round-reload');await window.__TAURI__.event.emit('foundation-smoke',{ok:true,mode:'organize-round',all:205,batches:[8,8,1],stoppedAfterReload:true});return;
  }
  await wait(()=>document.querySelector('.shell'));
  const restored=await invoke('organize_round_status');if(restored){if(restored.status!=='interrupted'||restored.round.completed!==0||restored.model!=='changed-model')throw Error('restart restored incorrectly');await window.__TAURI__.event.emit('foundation-smoke',{ok:true,mode:'organize-round',processRestart:true,noAutoResume:true});return;}
  await invoke('save_provider',{expectedRevision:0,config:{baseUrl:window.__ZWEN_FIXTURE_BASE__,model:'fixed-model',keyEnv:'ZWEN_PROVIDER_TEST_KEY'}});
  for(let i=0;i<205;i++)await invoke('notes_save',{input:{title:'合成 '+i,body:'合成正文 '+i,source:'fixture',model:'fixture',attachmentIds:[]}});
  document.querySelector('[data-nav=knowledge]').click();await wait(()=>document.querySelector('[data-organize-start]'));document.querySelector('[data-organize-start]').click();await wait(()=>document.querySelector('[data-organize-select-all]'));document.querySelector('[data-organize-select-all]').click();await wait(()=>document.querySelector('[data-organize-confirm-view]')?.textContent.includes('205'));
  const full=await invoke('organize_round_status');if(full.round.noteIds.length!==205)throw Error('all truncated');
  const r=await invoke('organize_prepare',{noteIds:full.round.noteIds.slice(0,17)});
  await invoke('organize_round_start',{roundId:r.round.id});
  const failed=await wait(async()=>{const v=await invoke('organize_round_status');return v?.status==='failed'?v:null;});
  if(failed.round.completed!==8||failed.round.batchIds.length!==1)throw Error('failed progress lost '+JSON.stringify(failed));
  await invoke('save_provider',{expectedRevision:1,config:{baseUrl:window.__ZWEN_FIXTURE_BASE__,model:'changed-model',keyEnv:'ZWEN_PROVIDER_TEST_KEY'}});
  await invoke('organize_round_start',{roundId:r.round.id});
  const done=await wait(async()=>{const v=await invoke('organize_round_status');return v?.status==='complete'?v:null;});
  if(done.round.completed!==17||done.round.batchIds.length!==3||done.model!=='fixed-model')throw Error('retry changed round/model');
  await invoke('organize_round_undo',{roundId:r.round.id});if((await invoke('organize_topics')).length)throw Error('undo left topics');
  const next=await invoke('organize_prepare',{noteIds:full.round.noteIds.slice(0,9)});
  await invoke('organize_round_start',{roundId:next.round.id});
  await new Promise(r=>setTimeout(r,300));await invoke('organize_round_stop',{roundId:next.round.id});
  localStorage.setItem('round-reload','yes');location.reload();
 }catch(e){await window.__TAURI__.event.emit('foundation-smoke',{ok:false,mode:'organize-round',error:String(e)});}
})();
