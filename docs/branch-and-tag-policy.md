# ブランチ・タグ運用

Storymesh は、`main` と短命な作業ブランチによる GitHub Flow を採用する。
リリースは、検証済みのコミットに SemVer のタグを付けて行う。
小さくレビューできる変更と、公開物を追跡できる履歴を維持する。

## ブランチ

| ブランチ | 用途 |
| --- | --- |
| `main` | CI を通過した変更の集約先。通常のリリース元 |
| `feat/<内容>` | 機能追加。例: `feat/ignore-files` |
| `fix/<内容>` | 不具合修正 |
| `docs/<内容>` | ドキュメント変更 |
| `chore/<内容>` | 依存更新、CI、リリース準備 |
| `maintenance/1.x` | 将来、旧メジャー版の継続保守が必要な場合のみ作成 |

- 作業ブランチは `main` から作成し、1 PR に1つの目的をまとめる。
- 大きな機能は小さな PR に分割する。準備中の変更は Draft PR で共有する。
- ローカルの必要な検証と PR の CI を通してからマージする。
- マージ方法は Squash merge に統一し、PR 単位で変更と取り消しを扱う。
- マージ後は作業ブランチを削除する。
- 恒常的な `develop` や `release/*` は設けない。リリース準備も
  `chore/release-0.2.0` のような短命ブランチで行う。

### `main` の保護設定

GitHub Rulesets で以下を設定する。

- PR 経由の変更を必須にする。
- [CI workflow](../.github/workflows/ci.yml) の `verify` を必須チェックにする。
- force push とブランチ削除を禁止する。
- Require linear history を有効にし、Squash merge を許可する。
- 複数人で開発する場合は承認1件を必須にする。単独開発では承認件数を0件とし、
  PR と CI は必須にする。

## バージョン

互換性は、文書化した CLI オプション、終了コード、JSON 出力、spec の形式など、
利用者が依存するインターフェースを基準に判断する。

| 段階 | PATCH | MINOR | MAJOR |
| --- | --- | --- | --- |
| `0.x` | 修正 | 機能追加、意図的な互換性変更 | 安定したインターフェースを定義するときに `1.0.0` |
| `1.0.0` 以降 | 互換性を保つ修正 | 互換性を保つ機能追加 | 互換性を壊す変更 |

`0.x` は安定性を保証しない。互換性変更はリリースノートに明記する。
Rust の `Cargo.toml`、`Cargo.lock` 内の Storymesh、および npm 全6パッケージと
プラットフォームパッケージへの依存バージョンを一致させる。
更新には `node scripts/set-version.mjs VERSION` を使う。

## タグ

Git タグは公開したソースのコミットを固定する。npm dist-tag は利用者が選ぶ
配布チャネルであり、Git タグとは別のものとして扱う。

| Git タグ例 | 用途 | npm dist-tag |
| --- | --- | --- |
| `v0.1.1` | 修正リリース | `latest` |
| `v0.2.0` | 機能追加リリース | `latest` |
| `v0.2.0-alpha.1` | 初期の試験版 | `alpha` |
| `v0.2.0-beta.1` | 利用者による試験版 | `beta` |
| `v0.2.0-rc.1` | 最終候補 | `rc` |
| `v0.3.0-canary.1` | 開発版の試験配布 | `canary` |
| `skills-v0.1.0` | スキルだけのリリース | npm 公開なし |

- `v*` は npm 公開専用に予約する。現行 workflow はタグの push で公開を起動する。
- npm リリースの形式は `vX.Y.Z` または `vX.Y.Z-{alpha|beta|rc|canary}.N` とする。
  `N` は1から始め、同じプレリリース系列の変更ごとに増やす。
- タグ名のバージョンとパッケージのバージョンを一致させる。タグを付けるだけでは
  パッケージのバージョンは更新されない。
- 注釈付きタグを使う。署名環境が整っている場合は署名付きタグを使う。
- そのリリースで CI が成功した正確なコミット SHA にタグを付ける。
- 公開済みタグは移動、再利用、削除しない。内容の修正には新バージョンを発行する。
- プレリリースを `latest` に公開しない。現行スクリプトは全6パッケージに
  `alpha`、`beta`、`rc`、`canary` または `latest` を明示的に設定する。

### タグの保護設定

`v*` と `skills-v*` を対象とするタグ用 Ruleset を設定する。

1. 作成制限の Ruleset で、例外権限を持つリリース担当のみ作成できるようにする。
2. 更新・削除禁止は別 Ruleset とし、作成担当への例外権限でタグの変更まで
   許可しないようにする。

## リリース手順

通常の npm リリースは次の順で行う。初回公開の staging、Trusted Publishing の
設定、具体的な復旧手順は [npm リリース用スキル](../.agents/skills/storymesh-npm-release/SKILL.md)
を参照する。

1. `main` から `chore/release-X.Y.Z` を作成する。
2. `node scripts/set-version.mjs X.Y.Z` を実行し、差分を確認する。
3. リリースノートに変更内容、互換性変更、必要な移行手順をまとめる。
4. `mise run handoff` を通して PR を `main` にマージする。
5. マージ後のリリース対象 SHA の CI 成功を確認する。
6. その SHA のクリーンなチェックアウトで
   `node scripts/check-npm-version.mjs X.Y.Z` を実行する。
7. 公開を行うときに、その SHA に注釈付きの `vX.Y.Z` タグを作成し、
   そのタグだけを push する。例: `git tag -a vX.Y.Z <SHA> -m "Release X.Y.Z"`、
   `git push origin refs/tags/vX.Y.Z`。署名する場合は `-a` の代わりに `-s` を使う。
8. Release npm workflow、全6パッケージの公開バージョン、dist-tag、provenance を確認する。
9. 新しい一時ディレクトリで
   `npm exec --yes --package=storymesh@X.Y.Z -- storymesh --version` を実行し、
   インストールと表示バージョンを確認する。

一部のパッケージだけ公開に失敗した場合は、コミットとパッケージ内容が変わらない
限り同じタグの workflow を再実行する。既に公開済みのバージョンはスクリプトが
スキップする。内容を変える必要がある場合は、新しいコミット、バージョン、タグで
リリースする。

スキルだけのリリースは README のスキル公開手順に従い、`main` に取り込んだ
検証済みのコミットへ `skills-vX.Y.Z` を付ける。npm 公開用の `v*` は使わない。

## 現在の実装と今後の対応

この文書は運用方針を定める。GitHub の保護設定が適用済みであることを意味しない。

[リリース workflow](../.github/workflows/release.yml) はタグとバージョンの整合性を
検証するが、タグのコミットが `main` に含まれることと、その SHA の CI 成功を
自動検証していない。自動化するまではリリース担当が確認する。

旧メジャー版の保守を始める場合は、`maintenance/1.x` などからの公開を許可する
検証・Environment 設定と、旧版の公開で `latest` を戻さない配布チャネルを先に
整備する。現行スクリプトは正式版を常に `latest` に公開するため、そのままでは
旧版の並行公開に使わない。旧版への修正は対応する保守ブランチで行い、必要な修正を
`main` にも反映する。

## 参考

- [GitHub Flow](https://docs.github.com/en/get-started/using-github/github-flow)
- [GitHub Rulesets のルール](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets)
- [Semantic Versioning](https://semver.org/)
- [Git のタグ](https://git-scm.com/docs/git-tag)
- [npm dist-tag](https://docs.npmjs.com/cli/v11/commands/npm-dist-tag/)
