# npm 公開手順

このリポジトリは、JavaScript のランチャー `storymesh` と、OS/CPU ごとの Rust バイナリを含む5つの optional package を公開します。インストール時にコンパイルや外部ダウンロードは行いません。

| npm package | 対象 |
| --- | --- |
| `storymesh` | 利用者がインストールする CLI ランチャー |
| `storymesh-darwin-arm64` | macOS ARM64 |
| `storymesh-darwin-x64` | macOS x64 |
| `storymesh-linux-arm64` | glibc Linux ARM64 |
| `storymesh-linux-x64` | glibc Linux x64 |
| `storymesh-win32-x64` | Windows x64 |

## バージョンの決め方

バージョンはタグから自動生成しません。公開する人が `VERSION` を決め、`node scripts/set-version.mjs VERSION` で Cargo と6つの npm package を一括更新します。通常版の `VERSION` は `MAJOR.MINOR.PATCH` です。判断の目安は [Semantic Versioning](https://semver.org/) に従い、互換性を保つ不具合修正なら patch（`0.1.0` → `0.1.1`）、機能追加なら minor（`0.1.0` → `0.2.0`）、`1.0.0` 以降の互換性を壊す変更なら major を上げます。`0.x` は開発段階で、安定した互換性は保証しません。安定した利用者向けの仕様を定める際に `1.0.0` を選びます。

先行版の `VERSION` は `MAJOR.MINOR.PATCH-{alpha|beta|rc|canary}.N`（`N` は1以上の整数）とします。用途と npm dist-tag は次のとおりです。

| `VERSION` の例 | 用途 | npm dist-tag |
| --- | --- | --- |
| `0.2.0-canary.1` | 頻繁な開発版の確認 | `canary` |
| `0.2.0-alpha.1` | 初期の先行版 | `alpha` |
| `0.2.0-beta.1` | 利用者による試用 | `beta` |
| `0.2.0-rc.1` | 正式版の候補 | `rc` |
| `0.2.0` | 通常版 | `latest` |

先行版も手動で番号を選び、同じ段階で内容を変える場合は `N` を増やします。例えば `0.2.0-beta.1` の次は `0.2.0-beta.2` です。公開スクリプトは6 package に同じ npm dist-tag を明示し、`npm install storymesh@beta` のように指定して試せます。通常の `npm install storymesh` は `latest` を選びます。[npm の dist-tag 説明](https://docs.npmjs.com/adding-dist-tags-to-packages/)。

Git タグは `vVERSION` とします。例えば `v0.2.0-beta.1` の push で公開するには、タグが指す commit の `Cargo.toml`、`Cargo.lock`、6つの `package.json` と optional dependencies がすべて `0.2.0-beta.1` である必要があります。一致しない場合、workflow は公開前に失敗します。公開済みのバージョンは上書きできないため、内容を変更するリリースには新しいバージョンを使います。

## 初回公開前の準備

公開元は `y2o-lab/storymesh`、公開先は npm 公式レジストリ（`https://registry.npmjs.org`）です。全6 package の `repository.url`、GitHub の公開元、Trusted Publisher の設定を一致させます。`git remote -v` で push 先も確認してください。

1. [npm](https://www.npmjs.com/) のアカウントで2要素認証を有効にします。ローカルの `npm login` は、この初回手順では不要です。
2. GitHub の `y2o-lab/storymesh` の Settings → Environments に `npm` を作成します。`npm` は環境変数ではなく Environment の名前です。Deployment branches and tags を制限する場合は、初回用の `main` と通常公開用の `v*` タグを許可してください。公開前に確認を挟む場合は required reviewer も設定できます。
3. npm 名がまだ空いていることを `npm view storymesh` および表の各 package 名で確認します。`E404` なら未公開です。他のエラーは未公開の証拠ではありません。別の所有者が取得済みなら、manifest とランチャー内の全 package 名を変更する必要があります。
4. npm のプロフィール → Access Tokens → Generate New Token で、初回用の Granular Access Token を作成します。
   - Packages and scopes の Permissions: **Read and write (stage only)**
   - Bypass two-factor authentication: **無効**
   - Select Packages: 新規の unscoped package を作成するため **All Packages**
   - 有効期限: 初回作業に必要な短期間
5. GitHub の `npm` Environment の Secret に **`NPM_TOKEN`** として保存します。トークンをリポジトリやログに書かないでください。
6. 公開対象のコミットが `main` に入り、CI が成功していることを確認します。

Trusted Publisher を設定する package は npm 上に存在する必要があります。初回は stage-only token で package を staging し、npm のWeb画面で2FA承認します。直接公開する token や Bypass 2FA は不要です。

## 初回公開

現在の `0.1.0` を初回公開する場合はバージョン更新を省略できます。別のバージョンにする場合は次を実行し、変更を commit して `main` に取り込みます。

```sh
node scripts/set-version.mjs VERSION
mise run handoff
node scripts/check-npm-version.mjs VERSION
```

GitHub の Actions → **Release npm** → Run workflow で、ブランチを **main** にし、**bootstrap_stage** を有効にして実行します。

- 通常の手動実行（`bootstrap_stage` 無効）はビルドと artifact 作成のみです。
- `bootstrap_stage` 有効時は `main` でのみ初回用 staging job を実行します。
- staging job は Node.js 24、npm CLI 11.15.0 以上を使用し、`NPM_TOKEN` を `NODE_AUTH_TOKEN` としてこのステップだけに渡します。OIDC の権限は付与しません。
- 5つの platform package を先に、ランチャーを最後に `npm stage publish` します。既に公開済み、または同じバージョンが承認待ちなら skip します。
- 新規 package の staging は公開される `0.0.0-stage` placeholder を作成します。実際のリリース内容は承認まで非公開です。

workflow が成功したら、npmjs.com の **Staged Packages** で各 package の version、dist-tag、tarball 内容を確認します。platform package 5つを先に2FA承認し、すべてが公開された後に **storymesh** を最後に承認してください。workflow の成功だけでは公開は完了しません。

公開後、npm の全6 package の Settings → Trusted publishing に GitHub Actions publisher を設定します。

| 項目 | 値 |
| --- | --- |
| Organization or user | `y2o-lab` |
| Repository | `storymesh` |
| Workflow filename | `release.yml` |
| Environment name | `npm` |
| Allowed actions | `npm publish` を許可 |

全6 package の Publishing access を **Require two-factor authentication and disallow tokens** にします。Trusted Publishing はこの設定でも OIDC 認証で動作します。初回用トークンを npm 側で失効させ、GitHub の `NPM_TOKEN` Secret も削除してください。

通常公開 job は GitHub OIDC を使い、npm token や `NODE_AUTH_TOKEN` は不要です。公開リポジトリ・公開 package では provenance が自動生成されます。Trusted Publishing には Node.js 22.14.0 以上と npm CLI 11.5.1 以上が必要で、workflow は Node.js 24 と npm CLI の版チェックを使用します。

最後に、初回公開に使った正確な commit にタグを作ります。package は既に公開済みなので、このタグで起動する workflow は同じバージョンを検出して skip します。

```sh
git tag v0.1.0 RELEASE_COMMIT_SHA
git push origin v0.1.0
```

設定仕様は [npm の token 発行手順](https://docs.npmjs.com/creating-and-viewing-access-tokens/)、[staged publishing](https://docs.npmjs.com/staged-publishing/)、[Trusted Publishing](https://docs.npmjs.com/trusted-publishers/) を参照してください。

## 2回目以降の公開

1. `node scripts/set-version.mjs VERSION` で Cargo、lockfile、全 npm manifest のバージョンを同時に更新します。
2. `mise run handoff` を実行し、変更を commit、レビューして `main` に取り込みます。
3. GitHub 上の `main` の commit とローカルの対象 commit が同一であることを確認します。
4. `main` の対象 commit で `git tag vVERSION` と `git push origin vVERSION` を実行します。タグ push だけが npm 公開 job を起動します。
5. **Release npm** workflow の build と publish が成功したことを確認します。
6. npm 上の version、provenance、platform package の依存関係を確認し、クリーンな一時ディレクトリで実行確認します。

```sh
mkdir /tmp/storymesh-npm-smoke
cd /tmp/storymesh-npm-smoke
npm exec --yes --package=storymesh@VERSION -- storymesh --version
```

タグの `v` を除いた値、`Cargo.toml`、`Cargo.lock`、6つの `package.json`、main package の optional dependencies はすべて同一バージョンでなければなりません。workflow は公開前にこの条件と tarball 内容を検査します。公開済みの同じ package/version は上書きできません。内容を修正する場合は、通常版なら次のバージョン、先行版なら `N` を増やした新しいバージョンを使用してください。

## 失敗時の確認

build が失敗した場合は、修正して新しい commit とバージョンでリリースしてください。publish が途中で失敗した場合は、同じタグの workflow を再実行できます。スクリプトは npm 上で公開済みの package/version を skip し、残りの package を順に公開します。公開済み package の内容を変更した場合は同じ version に再公開できないため、新しいバージョンに進めてください。通常の手動起動では公開しません。初回 staging の部分失敗は、同じ commit の `main` で `bootstrap_stage` を有効にして再実行できます。承認待ちの内容を変更する場合は新しいバージョンを使い、不要な staging は npm のWeb画面で reject してください。

Trusted publishing の設定項目と要件は [npm 公式ドキュメント](https://docs.npmjs.com/trusted-publishers/) を参照してください。GitHub Environment のタグ制限は [GitHub 公式ドキュメント](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments) を参照してください。
