(async()=>{
 const api=window.__TAURI__;
 const wait=async(check)=>{for(let i=0;i<100;i++){if(await check())return;await new Promise(r=>setTimeout(r,100));}throw Error('Timed out waiting for UI or saved draft');};
 const base='桌面测试：中文 🌿\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n$E=mc^2$';
 const text=base+'\n\n关闭前追加';
 try {
  await wait(()=>document.querySelector('textarea') && !document.querySelector('textarea').disabled);
  await wait(()=>{const logo=document.querySelector('img.app-logo');return logo?.complete&&logo.naturalWidth===96;});
  if(['edit','quit'].includes(window.__ZWEN_SMOKE_MODE__)) {
   const input=document.querySelector('textarea');input.value=base;input.dispatchEvent(new Event('input',{bubbles:true}));
   await wait(async()=>{const d=await api.core.invoke('load_draft');return d.text===base&&d.revision>0;});
  } else if(document.querySelector('textarea').value!==text) throw Error('Saved draft was not restored into textarea');
  if(document.querySelector('[data-demo],[data-preview]'))throw Error('removed controls still present');
  const d=await api.core.invoke('load_draft');
  if(['edit','quit'].includes(window.__ZWEN_SMOKE_MODE__)){const input=document.querySelector('textarea');input.value=text;input.dispatchEvent(new Event('input',{bubbles:true}));}
  if(window.__ZWEN_SMOKE_MODE__==='quit'){const button=document.querySelector('[data-quit-app]');if(!button||button.disabled)throw Error('direct quit unavailable');if(document.querySelector('[aria-label="更多操作"]'))throw Error('obsolete menu still present');button.click();return;}
  await api.event.emit('foundation-smoke',{ok:true,mode:window.__ZWEN_SMOKE_MODE__,revision:d.revision,draftLifecycle:true,width:innerWidth,height:innerHeight,dpr:devicePixelRatio});
 } catch(e) { await api.event.emit('foundation-smoke',{ok:false,error:String(e)}); }
})();
