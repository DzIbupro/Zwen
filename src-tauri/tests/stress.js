(async()=>{
 const api=window.__TAURI__,invoke=api.core.invoke;
 const wait=async(check)=>{for(let i=0;i<200;i++){if(await check())return;await new Promise(r=>setTimeout(r,40));}throw Error('Stress fixture timeout');};
 try{
  await invoke('save_provider',{expectedRevision:0,config:{baseUrl:window.__ZWEN_FIXTURE_BASE__,model:'fixture-model',keyEnv:'ZWEN_PROVIDER_TEST_KEY'}});
  await invoke('set_surface',{mode:'full'});await wait(()=>[...document.querySelectorAll('nav button')].some(b=>b.textContent==='模型设置'));
  [...document.querySelectorAll('nav button')].find(b=>b.textContent==='模型设置').click();await wait(()=>[...document.querySelectorAll('.settings button')].some(b=>b.textContent==='重新读取'));
  [...document.querySelectorAll('.settings button')].find(b=>b.textContent==='重新读取').click();await wait(()=>document.querySelector('#model-name')?.value==='fixture-model');
  await invoke('set_surface',{mode:'capsule'});await wait(()=>document.querySelector('[data-capsule]'));
  for(let i=0;i<20;i++){
   const input=document.querySelector('textarea');input.value=`stress-${i}`;input.dispatchEvent(new Event('input',{bubbles:true}));await wait(()=>!document.querySelector('[data-send]').disabled);document.querySelector('[data-send]').click();
   await wait(async()=>{const s=await invoke('get_chat');return s.reply?.prompt===`stress-${i}`&&s.reply.state==='complete'&&s.reply.saved;});
   await wait(()=>document.querySelector('[data-capsule-answer]')?.textContent.includes(`stress-finished-${i}`));
  }
  const surface=await invoke('get_surface');const ratio=devicePixelRatio/surface.scaleFactor;const width=document.querySelector('[data-capsule]').getBoundingClientRect().width*ratio;
  if(Math.abs(width-surface.bounds.width)>2)throw Error('Capsule width changed');
  await api.event.emit('foundation-smoke',{mode:'stress',ok:true,turns:20,formulae:true,bodyNodes:document.querySelectorAll('*').length,width});
 }catch(e){await api.event.emit('foundation-smoke',{mode:'stress',ok:false,error:String(e)});}
})();
