(async()=>{for(let i=0;i<200&&!document.querySelector('.shell');i++)await new Promise(r=>setTimeout(r,25));await window.__TAURI__.core.invoke('capture_screenshot',{replaceId:null});})();
