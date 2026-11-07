(async()=>{
 const api=window.__TAURI__,sleep=ms=>new Promise(r=>setTimeout(r,ms));
 const wait=async(check)=>{for(let i=0;i<150;i++){if(await check())return;await sleep(60);}throw Error('Bulk history fixture timeout');};
 const records=()=>document.querySelectorAll('.records button');
 const clickText=(selector,text)=>{const b=[...document.querySelectorAll(selector)].find(e=>e.textContent.includes(text));if(!b)throw Error('Missing '+text);b.click();};
 const total=async()=>{let count=0;for(let page=0;page<10;page++){const rows=await api.core.invoke('get_chat_history',{page});count+=Math.min(20,rows.length);if(rows.length<=20)return count;}throw Error('Unexpected fixture size');};
 let phase='startup';
 try{
  await wait(()=>document.querySelector('textarea')?.value==='合成批量草稿');
  const note=await api.core.invoke('notes_save',{input:{title:'批量保留收藏',body:'整批删除后保留正文',source:'chat',model:'fixture',attachmentIds:[],requestId:'history-24'}});
  document.querySelector('[data-nav=history]').click();await wait(()=>records().length===20);records()[0].click();await wait(()=>document.querySelector('.original-question')?.textContent==='批量问题24');clickText('.record-meta button','继续此对话');await wait(()=>document.querySelector('.user-question')?.textContent.includes('批量问题24'));
  document.querySelector('[data-nav=history]').click();await wait(()=>records().length===20);
  document.querySelector('[data-history-check="history-24"]').click();clickText('.pagination button','下一页');await wait(()=>records().length===5);document.querySelector('[data-history-check="history-0"]').click();
  phase='batch-cancel';document.querySelector('[data-history-bulk]').click();await wait(()=>document.querySelector('.delete-confirm')?.textContent.includes('2 条'));clickText('.delete-confirm button','取消');if(await total()!==25)throw Error('Cancel changed records');
  phase='batch-confirm';document.querySelector('[data-history-bulk]').click();await wait(()=>document.querySelector('[data-history-confirm-delete]'));document.querySelector('[data-history-confirm-delete]').click();await wait(()=>records().length===20&&document.querySelector('.pagination span')?.textContent==='1');if(await total()!==23)throw Error('Batch scope wrong');
  for(const requestId of ['history-24','history-0'])if(await api.core.invoke('get_chat_turn',{requestId}))throw Error('Batch target survived');
  document.querySelector('[data-nav=chat]').click();await wait(()=>document.querySelector('[data-welcome-background]'));if(document.querySelector('textarea').value!=='合成批量草稿')throw Error('Draft changed');
  phase='all-cancel';document.querySelector('[data-nav=history]').click();await wait(()=>records().length===20);clickText('.pagination button','下一页');await wait(()=>records().length===3);document.querySelector('[data-history-check-page]').click();document.querySelector('[data-history-clear]').click();await wait(()=>document.querySelector('.delete-confirm')?.textContent.includes('所有分页'));clickText('.delete-confirm button','取消');if(await total()!==23)throw Error('Clear cancel changed history');
  phase='all-confirm';document.querySelector('[data-history-clear]').click();await wait(()=>document.querySelector('[data-history-confirm-delete]'));document.querySelector('[data-history-confirm-delete]').click();await wait(()=>records().length===0&&document.querySelector('.pagination span')?.textContent==='1');if(await total()!==0||(await api.core.invoke('get_chat')).reply)throw Error('All did not clear other pages/cache');
  if((await api.core.invoke('notes_read',{id:note.id})).body!=='整批删除后保留正文'||(await api.core.invoke('load_draft')).text!=='合成批量草稿')throw Error('Saved material removed');
  await api.event.emit('foundation-smoke',{ok:true,mode:'history-bulk-delete',initial:25,afterBatch:23,afterAll:0,crossPageChecked:true,cancelPreserved:true,allPagesCleared:true,currentDetached:true,draftPreserved:true,notePreserved:true});
 }catch(e){await api.event.emit('foundation-smoke',{ok:false,mode:'history-bulk-delete',phase,error:String(e)});}
})();
