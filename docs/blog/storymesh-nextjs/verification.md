# 検証記録

## 実行した確認

| 対象 | 確認 | 結果 / 証拠 |
| --- | --- | --- |
| 依存の再現 | `npm ci --no-audit --no-fund` | lockfile からクリーンインストール成功 |
| Next.js / React | `npm run typecheck`、`npm run build` | 成功、`/` と `/_not-found` をビルド |
| Storybook | `npm run build-storybook -- --disable-telemetry` | 成功 |
| ファイル検査 | `npm run storymesh:check` | 完成状態の 3/3 が成功 |
| Spec 検査 | `npm run storybook:index`、`npm run storymesh:spec` | 新しい index で PASS、警告 0 |
| ソース版 CLI | `mise exec -- cargo run --quiet -- report examples/nextjs-storymesh/src/components --framework react` と `spec check` | 3/3、missing 0、Spec PASS |
| CLI の機能実演 | `npm run demo:exercise` | 27 コマンドの期待終了コードを確認 |
| 実画面 / 操作 | `npm run demo:capture` | Next.js の招待操作、Storybook の描画と Loading の disabled を検証 |
| 掲載画像 | PNG 15 枚 | 実画面 6 枚、保存した CLI 出力のログビューアー 9 枚 |

ビルドと完成状態の検査は [validation.json](evidence/validation.json)、全コマンドの stdout / stderr / 終了コードは [commands.json](evidence/commands.json)、ブラウザの検証結果と URL は [screenshots.json](evidence/screenshots.json) にあります。validation の出力内のリポジトリ絶対パスは `<workspace>` に置換しています。CLI のコマンド表示は `npx --no-install` 形式ですが、実演スクリプトは同じ導入済みパッケージの `node_modules/.bin` を直接呼びます。実際の引数も JSON に記録しています。

## 記事の主要な主張の根拠

| 主張 | 保存した実行結果 |
| --- | --- |
| Story が不足すると `check` が 1 | `02-missing`、UsageMeter.tsx |
| coverage / report は不足があっても 0 | `02-missing`、66.7% / 2 of 3 |
| ignore / ignore-file で対象を除外できる | `03-ignore`、2 of 2 |
| `!` で再包含できる | `03-ignore`、PreviewCard が加わり 2 of 4 |
| PATH と framework を省略できる | `03-ignore`、src/components を cwd として check |
| 生成後は 100%、再実行で上書きしない | `04-generate`、生成 1 件 → 対象なし → 3 of 3 |
| ファイル coverage 100% でも UI 状態は不足し得る | `05-spec-fail`、Button/loading、FAIL / 1 |
| index 再生成後に Loading を検出できる | `06-spec-pass`、PASS / 0 |
| init は空の下書き、with-stories は CSF も生成 | `07-spec-init`、生成 YAML / CSF |
| import は既存を skip | `08-spec-import`、Skipped 3 |
| merge の形式制約と整形後の成功 | `08-spec-import`、2 → 0、Updated 1 |
| 宣言外 Story は警告だけなら PASS | `09-warning-error`、Button/disabled、WARNING / 0 |
| index がない場合は入力エラー | `09-warning-error`、終了 2、PASS なし |

## 確認した制約と警告

- 公開 npm 版 0.1.0 は import したインデントなしリストをそのまま merge できません。記事に形式エラーと 2 スペース下げる回避手順を掲載し、整形前後の YAML を保存しました。Storymesh 本体の挙動はこのタスクで変更していません。
- 通常の本編 Spec は Button の状態要求を管理します。別の import 実演では 3 コンポーネント・6 Story 全体を検査して PASS を確認しました。
- `--with-stories` の FreshCard は作業用ディレクトリで生成を確認し、本編の Storybook glob に登録していません。記事では index を再生成して登録を検査する追加手順を説明しています。
- Next.js のビルドには、このマシンのリポジトリ外にある pnpm-lock.yaml を無視したという警告が出ました。サンプルのビルド自体は成功しています。
- Storybook の Vite ビルドには Next.js の `use client` directive に関する警告が出ました。ビルドは成功し、ブラウザで今回の UI を実際に描画・操作できました。
- npm は tsconfck の deprecated と、一部依存の install-script policy の警告を表示しました。今回の環境では追加承認なしで型検査、両ビルド、CLI 実行に成功しています。
- リモート CI の実行、公開、デプロイはしていません。GitHub Actions は記事用の設定例です。Vue / Angular の実画面も今回の対象外です。

## リポジトリの最終ゲート

最終ゲートのコマンドは `mise run handoff` です。サンプルのコードとスクリプトを含むため、handoff スキルは `verify` を選択します。Rust formatting / Clippy / tests、harness、npm 配布検査を実行し、成功時に `.codex-runs/handoff.json` へ現在の worktree の検証記録を保存します。このファイルはリポジトリ方針により Git 管理対象外です。

記事の CI 設定例は上のサンプル用 npm 検査を実行するもので、この Rust リポジトリの handoff ゲートとは用途が異なります。
