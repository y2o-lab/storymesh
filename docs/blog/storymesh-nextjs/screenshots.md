# 掲載画像一覧

実画面 6 枚、実際の CLI 出力を表示したログビューアー 9 枚、合計 15 枚です。PNG は加工せず保存しています。`cli-*` はターミナルアプリそのものの画像ではありません。

| ファイル | サイズ | キャプション / 掲載位置 |
| --- | --- | --- |
| [01-nextjs-app.png](images/01-nextjs-app.png) | 1440 × 960 | Next.js アプリのプラン管理画面 |
| [02-nextjs-interaction.png](images/02-nextjs-interaction.png) | 1440 × 960 | 招待操作後、利用人数が 7→8 名に更新 |
| [03-storybook-plancard.png](images/03-storybook-plancard.png) | 1440 × 960 | PlanCard の実描画と Controls |
| [04-storybook-loading.png](images/04-storybook-loading.png) | 1440 × 960 | required として追加した Loading Story |
| [05-storybook-generated.png](images/05-storybook-generated.png) | 1440 × 960 | Storymesh が生成した UsageMeter の Default を実描画 |
| [06-storybook-current.png](images/06-storybook-current.png) | 1440 × 960 | Current Story の disabled UI |
| [cli-01-install.png](images/cli-01-install.png) | 1280 × 900 | 保存した実際の CLI 実行結果のログビューアー |
| [cli-02-missing.png](images/cli-02-missing.png) | 1280 × 900 | 保存した実際の CLI 実行結果のログビューアー |
| [cli-03-ignore.png](images/cli-03-ignore.png) | 1280 × 900 | 保存した実際の CLI 実行結果のログビューアー |
| [cli-04-generate.png](images/cli-04-generate.png) | 1280 × 900 | 保存した実際の CLI 実行結果のログビューアー |
| [cli-05-spec-fail.png](images/cli-05-spec-fail.png) | 1280 × 900 | 保存した実際の CLI 実行結果のログビューアー |
| [cli-06-spec-pass.png](images/cli-06-spec-pass.png) | 1280 × 900 | 保存した実際の CLI 実行結果のログビューアー |
| [cli-07-spec-init.png](images/cli-07-spec-init.png) | 1280 × 900 | 保存した実際の CLI 実行結果のログビューアー |
| [cli-08-spec-import.png](images/cli-08-spec-import.png) | 1280 × 1236 | 保存した実際の CLI 実行結果のログビューアー |
| [cli-09-warning-error.png](images/cli-09-warning-error.png) | 1280 × 900 | 保存した実際の CLI 実行結果のログビューアー |

[記事本文](article.md) に掲載順で全画像を埋め込んでいます。UI の撮影元 URL と検証した操作は [screenshots.json](evidence/screenshots.json) に記録しています。

CLI の機能別原本は `evidence/01-install.txt` 〜 `09-warning-error.txt`、ブラウザ表示用 HTML は同じ名前の `.html` です。`08-spec-import` の二つの merge 実行の間では YAML のリストを 2 スペース下げています。手順は記事本文に説明しています。

再撮影はサンプルの `npm run demo:exercise` を完了後、3000 / 6006 のサーバーを起動し、`npm run demo:capture` を実行してください。CLI 実行と撮影を同時に行うと Story ファイルの書き換え中に撮影するため、順番に実行します。
