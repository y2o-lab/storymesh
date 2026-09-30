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

バージョンはタグから自動生成しません。公開する人が `MAJOR.MINOR.PATCH` を決め、`node scripts/set-version.mjs X.Y.Z` で Cargo と6つの npm package を一括更新します。判断の目安は [Semantic Versioning](https://semver.org/) に従い、互換性を保つ不具合修正なら patch（`0.1.0` → `0.1.1`）、機能追加なら minor（`0.1.0` → `0.2.0`）、`1.0.0` 以降の互換性を壊す変更なら major を上げます。`0.x` は開発段階で、安定した互換性は保証しません。安定した利用者向けの仕様を定める際に `1.0.0` を選びます。

Git タグは `vX.Y.Z` とします。例えば `v0.2.0` の push で公開するには、タグが指す commit の `Cargo.toml`、`Cargo.lock`、6つの `package.json` と optional dependencies がすべて `0.2.0` である必要があります。一致しない場合、workflow は公開前に失敗します。公開済みのバージョンは上書きできないため、内容を変更するリリースには新しいバージョンを使います。

現在の公開スクリプトは npm dist-tag を指定しないため、`npm publish` は `latest` を付けます。[npm の dist-tag 説明](https://docs.npmjs.com/adding-dist-tags-to-packages/) に従い、`0.2.0-beta.1` などの先行版を公開したい場合は、先に6 package すべてに適切な dist-tag を設定するよう公開スクリプトを変更してください。

## 初回公開前の準備

1. [npm](https://www.npmjs.com/) のアカウントで2要素認証を有効にし、ローカルで `npm login` を実行します。
2. GitHub のリポジトリ設定で `npm` Environment を作成します。公開前に確認を挟む場合は required reviewer も設定します。Deployment branches and tags を制限する場合は `v*` タグを許可してください。
3. npm 名がまだ空いていることを `npm view storymesh` および表の各 package 名で確認します。`E404` なら未公開です。名前は先着順なので、別の所有者が取得済みなら manifest とランチャー内の全 package 名を変更する必要があります。
4. 公開対象のコミットが `main` に入り、CI が成功していることを確認します。

初回は package がまだ存在せず npm 側で trusted publisher を設定できないため、GitHub Actions で全環境のバイナリをビルドし、公開だけを認証済みのローカル端末から行います。長期間有効な npm token を GitHub に保存する必要はありません。手動起動した Release npm workflow はビルドと artifact 作成だけを行い、npm には公開しません。

## 初回公開

現在の `0.1.0` を初回公開する場合はバージョン更新を省略できます。別のバージョンにする場合は次を実行します。

```sh
node scripts/set-version.mjs X.Y.Z
mise run verify
```

変更を commit して `main` に取り込んだ後、GitHub の Actions から **Release npm** を `main` で Run workflow します。成功した run の ID を確認し、5つのバイナリアーティファクトを取得します。

```sh
gh run list --workflow release.yml --limit 5
gh run download RUN_ID --dir artifacts
node scripts/npm-packages.mjs artifacts release-npm
```

公開前に各ディレクトリの tarball 内容を確認します。少なくとも `package.json`、`README.md`、`LICENSE` と、platform package の `bin/storymesh`（Windows は `bin/storymesh.exe`）以外の不要なファイルが入っていないことを確認してください。

```sh
for package in release-npm/storymesh*; do npm pack --dry-run "$package"; done
```

問題がなければ、platform package を先に、ランチャーを最後に公開します。スクリプトはこの順序で処理し、npm の2要素認証が必要な場合は npm CLI の案内に従います。

```sh
node scripts/publish-npm-packages.mjs release-npm
```

公開後、npm の6つの package すべてで Settings → Trusted publishing を開き、同じ GitHub Actions publisher を設定します。

- Organization or user: `y2o-lab`
- Repository: `storymesh`
- Workflow filename: `release.yml`
- Environment name: `npm`
- Allowed actions: `npm publish`

設定後は各 package の Publishing access を「2要素認証を必須にし、token を許可しない」設定にします。公開 workflow は GitHub OIDC を使い、公開リポジトリでは npm provenance が自動生成されます。GitHub 側の npm token や `NODE_AUTH_TOKEN` は不要です。npm の trusted publishing には Node.js 22.14.0 以上と npm CLI 11.5.1 以上が必要で、workflow は Node.js 24 と npm CLI の版チェックを使用します。

最後に初回バージョンのタグを作ります。package は既に公開済みなので、このタグで起動する workflow は同じバージョンを検出して安全に skip します。

```sh
git tag v0.1.0
git push origin v0.1.0
```

## 2回目以降の公開

1. `node scripts/set-version.mjs X.Y.Z` で Cargo、lockfile、全 npm manifest のバージョンを同時に更新します。
2. `mise run verify` を実行し、変更を commit、レビューして `main` に取り込みます。
3. GitHub 上の `main` の commit とローカルの対象 commit が同一であることを確認します。
4. `main` の対象 commit で `git tag vX.Y.Z` と `git push origin vX.Y.Z` を実行します。タグ push だけが npm 公開 job を起動します。
5. **Release npm** workflow の build と publish が成功したことを確認します。
6. npm 上の version、provenance、platform package の依存関係を確認し、クリーンな一時ディレクトリで実行確認します。

```sh
mkdir /tmp/storymesh-npm-smoke
cd /tmp/storymesh-npm-smoke
npm exec --yes --package=storymesh@X.Y.Z -- storymesh --version
```

タグの `v` を除いた値、`Cargo.toml`、`Cargo.lock`、6つの `package.json`、main package の optional dependencies はすべて同一バージョンでなければなりません。workflow は公開前にこの条件と tarball 内容を検査します。公開済みの同じ package/version は上書きできないため、失敗後に内容を修正する場合は新しい patch version を使用してください。

## 失敗時の確認

build が失敗した場合は、修正して新しい commit とバージョンでリリースしてください。publish が途中で失敗した場合は、同じタグの workflow を再実行できます。スクリプトは npm 上で公開済みの package/version を skip し、残りの package を順に公開します。公開済み package の内容を変更した場合は同じ version に再公開できないため、新しい patch version に進めてください。手動起動の workflow では公開できません。

Trusted publishing の設定項目と要件は [npm 公式ドキュメント](https://docs.npmjs.com/trusted-publishers/) を参照してください。GitHub Environment のタグ制限は [GitHub 公式ドキュメント](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments) を参照してください。
