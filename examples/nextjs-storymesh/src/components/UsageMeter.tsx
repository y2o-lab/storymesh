export function UsageMeter({ used = 7, limit = 10 }: { used?: number; limit?: number }) {
 const percent = Math.min(100, Math.max(0, (used / Math.max(1, limit)) * 100));
 return <article className="usage-card"><span className="eyebrow">WORKSPACE USAGE</span>
  <h3>メンバーの利用状況</h3><div className="usage-number">{used}<small> / {limit} 名</small></div>
  <div className="meter" role="progressbar" aria-label="メンバー利用率" aria-valuenow={percent} aria-valuemin={0} aria-valuemax={100}><div style={{ width: `${percent}%` }} /></div>
  <p>あと {Math.max(0, limit - used)} 名を招待できます。</p>
 </article>;
}
