export interface Draft { text: string; revision: number }
export interface DraftPort { load(): Promise<Draft>; save(revision: number, text: string): Promise<Draft> }
export interface Snapshot extends Draft { status: 'loading'|'saved'|'dirty'|'saving'|'error'; error: string|null; loaded: boolean }

export class DraftController {
  snapshot: Snapshot = { text: '', revision: 0, status: 'loading', error: null, loaded: false };
  private savedText = '';
  private timer: ReturnType<typeof setTimeout> | undefined;
  private inFlight: Promise<void> | undefined;
  constructor(private port: DraftPort, private changed: (s: Snapshot)=>void) {}
  private publish(patch: Partial<Snapshot>): void {
    this.snapshot = { ...this.snapshot, ...patch };
    this.changed(this.snapshot);
  }
  async start(): Promise<void> {
    try {
      const draft = await this.port.load();
      this.savedText = draft.text;
      this.publish({ ...draft, loaded: true, status: 'saved', error: null });
    } catch (error) {
      this.publish({ status: 'error', error: this.message(error) });
      throw error;
    }
  }
  edit(text: string): void {
    if (!this.snapshot.loaded) return;
    this.publish({ text, status: text === this.savedText && !this.inFlight ? 'saved' : 'dirty' });
    clearTimeout(this.timer);
    this.timer = setTimeout(()=>{ void this.flush().catch(()=>{}); }, 500);
  }
  insertPrompt(text: string): void { this.edit(this.snapshot.text ? this.snapshot.text + "\n\n" + text : text); }
  async flush(): Promise<void> {
    clearTimeout(this.timer);
    if (!this.snapshot.loaded) throw Error('本地草稿尚未读取成功，原有文件已保留');
    if (this.inFlight) {
      await this.inFlight;
      if (this.snapshot.text !== this.savedText) await this.flush();
      return;
    }
    if (this.snapshot.text === this.savedText) return;
    const text = this.snapshot.text;
    const revision = this.snapshot.revision;
    this.publish({ status: 'saving' });
    this.inFlight = Promise.resolve().then(()=>this.port.save(revision, text)).then(draft=>{
      this.savedText = draft.text;
      this.publish({ revision: draft.revision, status: this.snapshot.text === draft.text ? 'saved' : 'dirty', error: null });
    }).catch(error=>{
      this.publish({ status: 'error', error: this.message(error) });
      throw error;
    }).finally(()=>{ this.inFlight = undefined; });
    await this.inFlight;
    if (this.snapshot.text !== this.savedText) await this.flush();
  }
  async leave(action: ()=>Promise<void>): Promise<void> { await this.flush(); await action(); }
  dispose(): void { clearTimeout(this.timer); }
  private message(error: unknown): string { return error instanceof Error ? error.message : '无法保存；当前输入已保留，请重试'; }
}
