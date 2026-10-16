"use client";
import type { ButtonHTMLAttributes } from "react";
type Props = ButtonHTMLAttributes<HTMLButtonElement> & { loading?: boolean };
export function Button({ loading = false, children = "プランを選択", ...props }: Props) {
  return <button {...props} disabled={loading || props.disabled} className="button" aria-busy={loading}>
    {loading ? "処理中…" : children}<span aria-hidden="true">↗</span>
  </button>;
}
