# Storymesh UI Spec Check 実装設計書（MVP）

作成日: 2026-09-27  
対象: `y2o-lab/storymesh` の `main`、`docs/ui-spec-check-mvp.md`  
状態: 実装着手用の設計案

## 1. 目的と完了条件

`storymesh spec check` を追加し、YAML で宣言した UI 状態に対応する Story が Storybook に登録されているかを検査する。必要な Story が欠ければ `FAIL` と不足項目を表示し、CI で終了コード `1` を返す。宣言外の Story は警告にとどめる。割合は表示しない。導入を助ける `spec init`（コンポーネント起点のスケルトン）と `spec import`（既存 Story 起点の逆生成）も追加する。

この検査が保証するのは **Storybook に登録された Story の存在** までである。描画の正しさ、args の値、操作結果、Scenario は検査しない。既存の `check`、`coverage`、`report` と `--generate` の振る舞いを維持する。

受け入れ条件:

1. 有効な Spec と Storybook index を読み、required Story がすべてあれば終了コード `0` と `PASS`。
2. required Story が一つでもなければ、`Component/story` を列挙して終了コード `1` と `FAIL`。
3. Spec にない Story は `WARNING`。警告のみなら `PASS`、終了コード `0`。
4. YAML・index・入出力・照合の曖昧さなど検査不能な状態は終了コード `2`。`PASS` と表示しない。
5. 出力順序は入力順に依存せず決定的。既存 CLI のテストは引き続き通る。
6. コンポーネントから YAML の下書きを生成でき、`--with-stories` では不足する story ファイルを同時に生成できる。
7. Storybook の index から実在する Story を YAML に取り込める。既存の手書き内容を黙って上書き・削除しない。

## 2. 確認した現状と設計判断

| 対象 | 現状 | MVP での扱い |
| --- | --- | --- |
| `src/cli/args.rs` | clap のトップレベル `check` / `coverage` / `report` | `spec check` / `spec init` / `spec import` をネストしたサブコマンドで追加 |
| `src/cli/run.rs` / `output.rs` | 実行、表示、終了コードを分離 | `spec check` のオーケストレーションと表示を追加 |
| `src/scanner` | ファイル名からコンポーネントと story **ファイル** を対応付ける | `init` のコンポーネント列挙と不足 story 検出に再利用。Story 数や export は得られないので `check` / `import` の Story 一覧には使わない |
| `src/domain` / `src/lib.rs` | coverage 用モデルと公開 API | Spec のモデルと入口関数を追加 |
| `Cargo.toml` | `clap` と `ignore` のみ、Rust 2024 / 1.85 | YAML・JSON のデシリアライズ用に `serde`、`serde_yaml`、`serde_json` を追加する案。採用前に保守状況と lockfile を確認 |
| `tests/cli.rs` | 実バイナリによる終了コードと出力の検証 | 同じ手法で `spec check` の実動作を検証 |

**Story 一覧の入力源はローカルの `index.json` とする。** Storybook の `storybook index -o ...` で生成でき、Storybook が認識する Story 一覧を使える。Story ファイルを Rust で独自に解析する案は、CSF、タイトル自動生成、カスタム indexer と差が出やすいため MVP では採らない。HTTP 取得や Storybook の起動も CLI の責務に入れない。Storybook 公式は `index.json` を Story 一覧のインデックスとして説明し、`storybook index` コマンドを提供している。`init` だけはコンポーネント列挙に既存 scanner を使い、index を要求しない。

## 3. CLI とファイル規約

```text
storymesh spec check [--spec-dir PATH] [--index PATH]
storymesh spec init [PATH] [--framework react|vue|angular] [--spec-dir PATH] [--with-stories]
storymesh spec import [--index PATH] [--spec-dir PATH] [--merge]
```

| 引数 | 既定値 | 内容 |
| --- | --- | --- |
| `--spec-dir` | `.storymesh/specs` | `*.yaml` / `*.yml` を再帰読み込みするディレクトリ。カレントディレクトリ基準 |
| `--index` | `storybook-static/index.json` | Storybook が生成した index。カレントディレクトリ基準 |
| `init PATH` | `.` | 既存 scanner のコンポーネント走査ルート。`init` のみ |
| `init --framework` | `react` | 既存 scanner と同じフレームワーク。`init` のみ |
| `init --with-stories` | 無効 | story ファイルがないコンポーネントに story skeleton を同時生成 |
| `import --merge` | 無効 | 既存 Spec に index の新規 Story を追記。明示指定した場合のみ |

`check` で Spec が 0 件、index が存在しない、または Story エントリが 0 件の場合は検査不能として終了コード `2` にする。誤設定による空の `PASS` を防ぐ。Spec ディレクトリ内の隠しディレクトリや `node_modules` は探索しない。symlink は辿らず、読み取り失敗はパスつきで報告する。初回実装では `--strict`、`--json`、`--verbose`、ネットワーク入力、自動 index 生成を追加しない。

利用例:

```sh
pnpm exec storybook index -o storybook-static/index.json
storymesh spec check --spec-dir .storymesh/specs --index storybook-static/index.json
storymesh spec init src/components --framework react
storymesh spec init src/components --framework react --with-stories
storymesh spec import --index storybook-static/index.json
```

Storybook 側の `index` コマンドや出力先は使用中の Storybook 版と package scripts に合わせて CI で固定する。生成した `index.json` はその checkout と同じソースから作る。古い成果物による誤判定を避けるため、CI は毎回 index を生成する。

## 4. YAML v1 の契約

MVP で受け入れる形式を一つに固定する。元仕様の簡略例 `component: Button` は説明用とし、実装入力は同仕様 §12 の `version: 1` 形式を採用する。

```yaml
version: 1
component:
  id: button
  name: Button
  title: Components/Button # 省略時は name で候補解決
stories:
  - id: default
    required: true
  - id: loading
    required: true
  - id: debug
    required: false
```

- `version` は整数 `1` のみ、`component.id`、`component.name`、`stories[].id` は空でない文字列。`stories` は通常 1 件以上。`init` が出力する `stories: []` は未記入の下書きとして許可するが、`check` はそのファイルとパスを示して終了コード `2` とし、空の Spec を適合とみなさない。`required` の省略時は `true` とする。`component.title` は任意で、Storybook index の `title` に **完全一致** させる。タイトルに階層があるプロジェクトでは明示を推奨する。
- `component.id` は Spec 内で一意な識別子であり、Storybook の自動生成 ID ではない。同一ファイルの story ID 重複、ファイル間の component ID 重複、同一 `title` を指す複数 Spec は入力エラー。
- `component.title` が省略された場合、index の `title` 全体が `name` と一致する候補、または最後の `/` 区切りセグメントが `name` と一致する候補を集める。候補が一つなら採用し、複数なら **曖昧な入力エラー** として `title` の追記を促す。候補ゼロなら `name` を期待タイトルとして扱い、required Story を Missing とする。これによりコンポーネント自体が未登録でも不足を表示できる。
- v1 は未知フィールドを拒否する。`scenarios` を書いた場合も `unsupported field` として終了コード `2`。未実装の意味が黙って無視されることを避ける。
- YAML の型違い、不正値、重複、構文エラーはファイル名と可能なら行・列を表示する。Unicode は保持し、ID に空白や制御文字を含めない。ID は英小文字・数字・`-` の kebab-case を推奨し、MVP の検証もこれに固定する。必要なら後続版で緩和する。

`required: false` は存在を要求しないが、実在する場合に Undeclared として警告しない。比較集合は全宣言 Story、Missing の対象は required Story のみ。

### 4.1 コンポーネントからのスケルトン生成: `spec init`

`spec init` は既存の `scan_with_options` による検出結果を使い、**Spec がまだないコンポーネント**について YAML を生成する。コンポーネントを検知しただけで必要な UI 状態は推測できないため、通常モードは次の下書きとする。

```yaml
version: 1
component:
  id: button
  name: Button
stories: [] # TODO: 必要な UI 状態を列挙する
```

出力は既定で `.storymesh/specs/<コンポーネントの走査ルートからの相対パスの stem>.yaml` とする。例: `src/components/forms/Button.tsx` を `src/components` から走査した場合は `.storymesh/specs/forms/Button.yaml`。`index.tsx` は親ディレクトリ名をコンポーネント名に使い、出力先は `.storymesh/specs/forms/Button/index.yaml` のように元パスを区別する。`component.id` は相対パスの拡張子を除いて小文字・kebab-case にした値（例: `forms-button`）とし、同名や正規化衝突は書き込み前のエラーとする。表示名 `name` には既存 scanner のコンポーネント名を使う。既存 story の `title` はファイル名から推測しない。`title` は省略し、ユーザーが YAML を編集するときに指定できるようにする。

`--with-stories` では story ファイルが欠けているコンポーネントに限り、既存 `generate_story_skeletons` と同じ CSF skeleton を作る。現在の generator は `Components/<name>` という title と `Default` export を出力するため、対応する YAML は `title: Components/<name>` と `stories: [{ id: default, required: true }]` にする。story ファイルが**すでに存在する**場合は変更せず、通常モードと同じ `stories: []` の下書きを出す。既存 story の中身を知るには `spec import` を使う。`--with-stories` で生成した story は Storybook への登録や表示の妥当性まで保証しない。index を生成して `spec check` する手順を案内する。

`init` は Spec の既存ファイルをスキップして件数を表示し、内容を上書きしない。走査対象が 0 コンポーネントなら終了コード `2`。1 件以上あり、全件作成または既存のためスキップなら `0`。出力先の通常ファイル以外の衝突、読取・作成エラーは `2`。書き込み前に **全対象の YAML / story 出力パス、ID 衝突、既存ファイル、story 生成可能性を検証**し、新規ファイルを `create_new` で作成する。`--with-stories` で二つのコンポーネントが同じ `Components/<name>` を生成する場合も曖昧な title としてエラーにする。書き込み途中に失敗した場合は今回新規作成したファイルだけを片付け、既存ファイルには触れない。ロールバックにも失敗したら作成済みの全パスを明示する。現行 `generate_story_skeletons` は一括事前確認は行うが途中の書き込み失敗をロールバックしないので、`init --with-stories` では同じ生成ロジックを内部 API に切り出して計画・書き込みを一元化する（既存 `check --generate` の外部動作は変えない）。

空の下書きは `spec check` で失敗し、編集すべきファイルを示す。これにより自動作成だけで仕様が満たされたという誤認を防ぐ。

### 4.2 既存 Story からの逆生成: `spec import`

`spec import` は `index.json` の Story entry を `title` ごとにまとめ、**現在 Storybook に登録されている状態の YAML 初期案**を作る。コンポーネントファイルを走査する必要はない。例:

```yaml
version: 1
component:
  id: components-button
  name: Button
  title: Components/Button
stories:
  - id: default
    required: true
  - id: loading
    required: true
```

`component.id` は title の各階層を kebab-case 化して結合し、`name` は最後の階層名、出力は `.storymesh/specs/<id>.yaml` とする。`stories[].id` は `normalize_story_name(name)` による。title / name がこの規則で表現できない場合、正規化後の ID が空・重複・v1 の ID 規則外になる場合、または別 title と同じ ID / パスに衝突する場合は、**推測して出力せず**対象の Storybook ID と title / name を示して終了コード `2`。すべての story entry は `required: true` として初期化する。開発用 Story は生成後に `required: false` に変えるか、その title の Spec を管理対象から外す。これは一度の bootstrap 操作であり、`check` 実行時に YAML を更新しない。

既定は **create-only**: 対応する `component.title` の既存 Spec があればスキップする。新規 title の Spec のみ作成する。既存ファイル名が異なる場合も全 YAML を読み込んで title / id の重複を調べ、同じ title の二重作成を防ぐ。既存 Spec に title がなく `name` がその title の末尾と一致する場合は、安全に同一視できないため title の明記を促すエラーとする。新規の出力パスを別の Spec が占めている場合もエラーとする。

明示的な `--merge` は既存 Spec の `component.title` と正確に一致するグループだけを対象に、未宣言の Story を `required: true` で**追記**する。既存の `required`、順序、コメント、追加の手書き値は保持する。index から消えた宣言も削除しない。title が未指定の既存 Spec に暗黙で結び付けず、対象 title を示して指定を促す。`--merge` は YAML の行を安全に追記できる単純な v1 リスト構造だけを受け入れ、anchor、alias、複数ドキュメント、フロー形式など書式を壊しうる構文は拒否する。文書全体を serde で再シリアライズしてコメントを消す実装は採らない。コメント保存を伴う更新がこの制約で実現できなければ `--merge` を公開せず、create-only 版の完了後に別タスクで設計を改める。

`import` も全グループの入力検証と書き込み計画を先に行い、新規ファイルは `create_new`、既存ファイルは上書きしない。`--merge` の既存ファイル更新は一時ファイルを同一ディレクトリに書いて原子的に置換し、変更前のバックアップを保持して失敗を報告する。新規ファイルが 0 件でスキップのみでも `0`、index がない・Story entry が 0 件・衝突・書き込み失敗は `2`。成功時は `Created N spec(s) / Updated M spec(s) / Skipped K` とパスを表示する。元の Storybook ソースを編集しない。

## 5. Storybook index の入力契約

JSON のトップレベル `entries` を読み、各 entry の `type == "story"` に限る。`type == "docs"` は除外する。使うフィールドは `id`、`title`、`name`、`type`。`id` は診断や重複検査用、照合キーは `title` と正規化した `name` とする。index 全体を独自の厳密な構造体で拒否せず、未知フィールドは読み飛ばす。必要フィールドが欠けた story entry、未知のトップレベル形状、重複した Storybook ID は終了コード `2`。

**正規化関数 `normalize_story_name`**: 前後の空白を除き、ASCII の大文字を小文字に変換し、空白・`_`・`-` の連続を単一の `-` にする。例: `Default` → `default`、`Loading State` → `loading-state`。非 ASCII 文字は保持し、文字変換のルールを勝手に拡張しない。`Button/error` の `Button` は `title` で比較し、名前だけで別コンポーネントの Story と結び付けない。同一タイトル内で二つの Story が同じ正規化キーになる場合は曖昧な入力エラーとする。表示名変更が照合に影響する制約は README に記載する。

Storybook の `index.json` は Story の情報を持つが、元仕様にある `parameters.storymesh.id` を必ず出力するという契約ではない。安定 ID による照合は index 作成側の拡張契約を設計する段階まで延期する。今回の実装で `parameters.storymesh.id` を直接読むと約束しない。

## 6. データモデルと責務

```rust
pub struct ComponentSpec {
    pub id: String,
    pub name: String,
    pub title: Option<String>,
    pub stories: Vec<StoryRequirement>,
    // 内部診断用の source_path は必要に応じ保持
}

pub struct StoryRequirement {
    pub id: String,
    pub required: bool,
}

pub struct IndexedStory {
    pub id: String,
    pub title: String,
    pub name: String,
}

pub enum SpecIssue {
    MissingStory { component: String, story: String },
    UndeclaredStory { component: String, story: String },
}

pub struct SpecCheckResult {
    pub valid: bool,
    pub components_checked: usize,
    pub stories_checked: usize,
    pub errors: Vec<SpecIssue>,
    pub warnings: Vec<SpecIssue>,
}
```

`valid` は `errors.is_empty()` と常に一致するよう生成関数内で計算する。構文エラーなどは `SpecIssue` に混ぜず `SpecCheckError` で返し、CLI が終了コード `2` に変換する。件数は `components_checked = 読み込んだ Spec 数`、`stories_checked = 対応解決したタイトル内の index の story entry 数` と定義する。Spec と無関係な Storybook 全体の Story は Undeclared の対象にしない。

```text
src/domain/spec.rs       検査結果と issue のモデル
src/spec/parser.rs       YAML 読み込み、スキーマ検証、重複検査
src/spec/index.rs        index.json 読み込み、story entry 抽出と入力検証
src/spec/matching.rs     タイトル解決、集合比較、安定したソート
src/spec/generation.rs   コンポーネント→YAML、index→YAML の生成計画と安全な書き込み
src/spec/mod.rs          check_spec(spec_dir, index_path) の公開入口とエラー
src/cli/args.rs          Spec(Check / Init / Import) の clap 引数
src/cli/run.rs           検査と生成コマンドのオーケストレーション、終了コード
src/cli/output.rs        人間向け表示
src/lib.rs               必要な公開 API を re-export
```

入口の例: `pub fn check_spec(spec_dir: &Path, index_path: &Path) -> Result<SpecCheckResult, SpecCheckError>`。検査本体は既存 scanner の内部 API に依存させない。生成側は公開済みの `scan_with_options` を利用し、story skeleton の計画処理は既存 generator と共用する。`src/spec` は単一 crate 内に置き、パーサ、index reader、照合器、生成計画を個別にテストできるようにする。実装する crate の具体的なバージョンは作業時点の lockfile と互換性を確認して確定する。

## 7. 照合アルゴリズム

1. Spec ディレクトリ内の YAML を相対パス順に集め、すべて読み込み、重複・形式を検証する。
2. index を読み、story entry だけ抽出する。`title` ごとに `normalize_story_name(name)` の集合を作る。Storybook ID の重複と正規化キーの衝突を検査する。
3. 各 Spec の対応タイトルを §4 の規則で決める。対応がない場合、Spec の全 required Story を Missing にする。
4. 各 required ID が集合にないなら Missing。対応タイトル内の実在 Story のうち、Spec に **宣言されていない** 正規化 ID を Undeclared にする。
5. issue を `(component 表示名, story ID)` の昇順に並べる。Missing の表示名は Spec の `component.name`、Undeclared は対応する Spec の `component.name` を使う。表示上の同名コンポーネントが複数ある場合は `component.id` またはタイトルも併記し、診断を一意にする。
6. エラーがゼロなら `valid = true`。WARNING の数は判定に影響させない。

例: `Button` の Spec が `default`、`loading`、`error` を required とし、index に `Default`、`Loading`、`Debug` がある場合、`Button/error` が ERROR、`Button/debug` が WARNING、終了コードは `1`。

## 8. CLI 出力と終了コード

```text
Storymesh UI Spec

Missing stories
  ✗ Button/error

Undeclared stories
  ? Button/debug

Summary
  Components checked: 1
  Stories checked:    3
  Errors:             1
  Warnings:           1

FAIL
```

| 状態 | stdout | stderr | code |
| --- | --- | --- | ---: |
| 不足なし、警告なし | 件数、`No issues found.`、`PASS` | 空 | 0 |
| 不足なし、宣言外あり | WARNING の詳細、`PASS` | 空 | 0 |
| Missing あり | ERROR / WARNING の詳細、`FAIL` | 空 | 1 |
| YAML / index / 入出力 / 曖昧な照合 | 成功を示す出力なし | `error: <path>: <原因>` | 2 |

出力に `%` を追加しない。テキストの色や記号は端末依存なので、テストは内容と順序を検証し、装飾自体には依存しない。stdout 書き込みエラーも従来どおり `2`。

`init` / `import` は検査結果の `PASS` / `FAIL` を出さず、作成・更新・スキップしたパスと件数を表示する。通常終了 `0`、生成不能・入力不正・書き込み失敗 `2`。生成後の適合判定には別途 `storymesh spec check` を実行する。

## 9. 実装順と検証

1. **入力契約と検査**: YAML・index の fixture を用意し、正常系とパースエラーのテストを作る。required、optional、Missing、Undeclared、title 階層、曖昧な title、正規化衝突を照合する。
2. **コンポーネント起点生成**: 既存 scanner と story generator の計画処理を再利用し、`spec init` と `--with-stories` を実装する。生成した下書きが `check` で誤って PASS しないことを確認する。
3. **Story 起点生成**: index reader と正規化関数を共用して `spec import` の create-only を実装する。次に `--merge` をコメント保存・原子的な更新のテストが満たせる条件で実装する。満たせなければ CLI から `--merge` を公開せず別タスクに切り出す。
4. **CLI**: `spec check` / `init` / `import` の引数、表示、終了コードを実バイナリで確認する。既存 `check --generate` などの回帰を検証する。
5. **文書と CI 例**: README に `index.json` の生成、YAML 例、三つのコマンド、下書きの編集、保証範囲を追加する。Storymesh 自身は Storybook プロジェクトでないため、既存 CI に `spec check` を無条件追加しない。fixture を使った Rust テストを CI に通す。
6. **品質ゲート**: `mise exec -- cargo test spec` など狭い確認、`mise exec -- cargo run -- spec --help`、`mise run handoff` を実行。差分と `git diff --check` を確認する。

必須の振る舞いテスト:

| ケース | 期待 |
| --- | --- |
| required がすべて存在 | PASS / 0 |
| required が不足、optional だけ不足 | 前者は FAIL / 1、後者は PASS / 0 |
| 宣言外 Story のみ | 警告を表示して PASS / 0 |
| docs entry が混在 | docs は検査対象外 |
| Storybook 全体に Spec のない別コンポーネントあり | その Story は警告しない |
| `Components/Button` と `Forms/Button` が存在し title 省略 | title 指定を促すエラー / 2 |
| index なし、YAML なし、破損 JSON / YAML、重複 ID | パスつきエラー / 2 |
| 同じ正規化名に二つの Story | 曖昧さを報告 / 2 |
| file / entry の順序を入れ替え | 同じ出力順 |
| story のないコンポーネントに `spec init` | `stories: []` の YAML を作成。`spec check` は下書きとして 2 |
| 同じコンポーネントに `spec init --with-stories` | `Default` story skeleton と `default` を required とする YAML を作成 |
| 既存 story ファイルのあるコンポーネントに `--with-stories` | story を変更せず YAML は下書き。`spec import` を案内 |
| `spec import` に title が異なる複数 Story | title ごとに YAML。docs は出力しない |
| 既存 YAML がある状態で再度 `init` / `import` | 内容を維持してスキップ。二重 Spec を作らない |
| `import --merge` と optional / 手書きコメント | 既存値とコメントを維持し、新しい Story だけ追記 |
| 出力先・ID が衝突、生成中に書き込み失敗 | 2。既存ファイルは不変、部分生成は片付けるかパスを報告 |

## 10. 既知の制約と次段階

- index が古いと判定も古くなる。CLI 自身は鮮度を検証できないため CI で同じコミットから生成する。
- 表示名 `name` の変更、Storybook の自動命名規則、ASCII 以外の命名は YAML ID との対応に影響する。将来の安定 Spec ID 対応では、Storybook から ID を index に載せる方法と後方互換を別途設計する。
- Story が存在しても UI が仕様どおりとは限らない。args / metadata、描画、interaction / Scenario の実行は次の段階とする。
- `--strict` や機械向け JSON 出力は実利用から必要性を確認して追加する。
- `spec init` は仕様を推定しない。下書きに必要な UI 状態を書いてから検査する。`--with-stories` の `Default` だけを仕様の完成とみなさず、追加すべき状態をレビューする。
- `spec import` は既存 Story を出発点にするため、既存 Story 自体の不足を発見できない。逆生成後に要求を見直して YAML を Source of Truth とし、以後は `spec check` で差分を検出する。

## 参考

- [Storymesh MVP 仕様](https://github.com/y2o-lab/storymesh/blob/main/docs/ui-spec-check-mvp.md)
- [Storymesh README](https://github.com/y2o-lab/storymesh/blob/main/README.md)
- [Storybook CLI: `index`](https://storybook.js.org/docs/api/cli-options)
- [Storybook: Story index と sidebar](https://storybook.js.org/docs/configure/user-interface/sidebar-and-urls)
- [Storybook: Story の命名](https://storybook.js.org/docs/writing-stories)
