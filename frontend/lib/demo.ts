const TEXT=`## 演示回复\n\n这是**界面演示，不是 AI 回答**。回复逐步出现，胶囊只向下增长，宽度保持不变。\n\n| 场景 | 窗口 |\n|---|---|\n| 阅读时快速提问 | 小胶囊 |\n| 完整问答与管理 | 大窗口 |\n\n行内公式 $E=mc^2$。\n\n$$\nL(w)=\\frac{1}{n}\\sum_{i=1}^{n}(y_i-w^Tx_i)^2\n$$\n\n`+Array.from({length:18},(_,i)=>`### ${i+1}. 把一个概念讲清楚\n\n先理解材料本身，再用一个具体例子说明它。达到高度上限后，可在回答区域滚动查看前文，输入框仍然保留。切换到完整窗口会保留已经显示的内容。\n\n`).join('');
export class DemoStream {
  private timer:ReturnType<typeof setInterval>|undefined;
  private received='';
  constructor(private changed:(text:string,running:boolean)=>void){}
  start():void {
    clearInterval(this.timer);this.received='';this.changed('',true);
    this.timer=setInterval(()=>{
      this.received=TEXT.slice(0,this.received.length+28);
      const running=this.received.length<TEXT.length;
      this.changed(this.received,running);
      if(!running)clearInterval(this.timer);
    },45);
  }
  stop():void {clearInterval(this.timer);this.changed(this.received,false);}
}
