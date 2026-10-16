"use client";
import { useState } from "react";
import { Button } from "../components/Button";
import { PlanCard } from "../components/PlanCard";
import { UsageMeter } from "../components/UsageMeter";
export default function Home() {
 const [invited, setInvited] = useState(false);
 return <div className="workspace"><aside><div className="brand">◉ orbit<span>WORKSPACE</span></div><div className="team">Y2O Studio <span>TEAM</span></div><nav><a>◫ 概要</a><a>▦ プロジェクト</a><a>◎ メンバー</a><a className="active">◇ プランと利用状況</a></nav><div className="sidebar-foot">Next.js × React<br/>Storymesh sample application</div></aside>
 <main><header><span>ワークスペース / 設定</span><span className="avatar">YO</span></header>
 <div className="heading"><div><span className="eyebrow">BILLING & USAGE</span><h1>チームの成長に、ちょうどいいプラン。</h1><p>プランの変更とワークスペースの利用状況を確認できます。</p></div><span className="pill light">2026 / SAMPLE</span></div>
 <section className="grid"><PlanCard/><div><UsageMeter used={invited ? 8 : 7}/><div className="invite-card"><h3>新しいメンバーを招待</h3><p>{invited ? "招待リンクを作成しました。利用人数が更新されました。" : "チームに仲間を迎えて、一緒に進めましょう。"}</p><Button onClick={() => setInvited(true)} disabled={invited}>{invited ? "招待リンク作成済み" : "招待リンクを作成"}</Button></div></div></section>
 <footer><span className="dot"/> 実際の UI を Storybook で分離し、Storymesh で追加漏れを確認。</footer></main></div>;
}
