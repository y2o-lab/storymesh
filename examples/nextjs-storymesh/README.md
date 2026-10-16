# Next.js + React + Storybook + Storymesh サンプル

公開 npm 版 Storymesh 0.1.0 を実際に導入したブログ用サンプルです。記事と PNG は [ブログ資料](../../docs/blog/storymesh-nextjs/article.md) を参照してください。

```sh
cd examples/nextjs-storymesh
npm ci
npm run dev         # http://127.0.0.1:3000
```

別のターミナルで `npm run storybook` を実行すると http://127.0.0.1:6006 で UI を単独表示できます。

```sh
npm run storymesh:check
npm run storymesh:coverage
npm run storymesh:report
npm run storybook:index
npm run storymesh:spec
```

完成状態ではコンポーネント coverage は 3/3、Button Spec は PASS です。`.storymesh/specs/Button.yaml` は Button の Default / Loading を required、Disabled を optional とします。PlanCard / UsageMeter の状態要求はこの本編 Spec の対象外です。`demo:exercise` は別の Spec ディレクトリへ 3 コンポーネントを import して全 6 Story も検証します。

`npm run demo:reset` は UsageMeter の Story を削除し Button の Loading を取り除きます。`npm run demo:complete` はその二つをデモ完成状態へ書き直します。Story を独自に編集した場合は上書きに注意してください。

```sh
npm run demo:exercise  # 27 コマンド、ログ保存、完成状態で終了
npm run typecheck
npm run build
npm run build-storybook -- --disable-telemetry
```

`demo:exercise` は `.demo/` を消して作り直し、リポジトリ内 `docs/blog/storymesh-nextjs/evidence` を更新します。ログを使った撮影には Playwright が必要です。

```sh
npx --no-install playwright install chromium
# 別ターミナルで npm run start と npm run storybook を起動してから:
npm run demo:capture
```

`capture.mjs` は固定ポート 3000 / 6006 の実画面を撮影し、ブラウザの pageerror、不正な操作結果、Story の描画不足を失敗として扱います。CLI 画像は保存した実行ログのブラウザ表示であり、実際のターミナルアプリの画面ではありません。

本サンプルは npm を使用します。Storymesh の Rust ビルドはサンプル実行には不要です。Next.js / Storybook の build に出る警告と公開版の merge 形式制約は [検証記録](../../docs/blog/storymesh-nextjs/verification.md) を参照してください。
