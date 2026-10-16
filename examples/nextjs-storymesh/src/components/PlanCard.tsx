import { Button } from "./Button";
export function PlanCard({ name = "Team", price = "2,400", current = false }: { name?: string; price?: string; current?: boolean }) {
 return <article className="plan-card">
   <div className="card-top"><span className="eyebrow">{name.toUpperCase()}</span><span className="pill">{current ? "利用中" : "おすすめ"}</span></div>
   <h3>チームのアイデアを、ひとつに。</h3><p>プロジェクトとメンバーをまとめて管理。</p>
   <div className="price">¥{price}<small> / 月</small></div>
   <ul><li>✓ 無制限のプロジェクト</li><li>✓ メンバー 10 名まで</li><li>✓ 共有ワークスペース</li></ul>
   <Button disabled={current}>{current ? "現在のプラン" : "Team にアップグレード"}</Button>
 </article>;
}
