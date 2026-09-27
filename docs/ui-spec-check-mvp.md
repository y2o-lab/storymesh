# Storymesh UI Spec Check — MVP 仕様

## 1. 目的

Storymesh に、Storybook の Story が事前定義した UI 仕様を満たしているか検証する機能を追加する。

MVP では「カバレッジ率を測る」ことよりも、

> YAML で定義した必要な Story が、Storybook 側にすべて存在するか

を機械的に検証することを目的とする。

Storymesh の中では、この機能を **UI Spec Check** として扱う。

---

## 2. Storymesh における位置づけ

Storymesh は UI の Story / Scenario / Spec を扱う基盤として発展させる。

その最初の機能として、MVP では次の範囲だけを実装する。

```text
UI Spec (YAML)
      ↓
必要な Story 一覧
      ↓
Storybook の Story 一覧
      ↓
差分比較
      ↓
PASS / FAIL
```

MVP では Scenario の実行や interaction test までは扱わない。

既存の Storymesh が持つ「コンポーネントに Story ファイルがあるか」という検査は維持する。
UI Spec Check はその上位レイヤーとして、「その Story ファイルの中に必要な UI 状態が定義されているか」を検証する。

---

## 3. UI Spec を Source of Truth にする

Storymesh 側で、必要な UI 状態を YAML として定義する。

例:

```yaml
component: Button

stories:
  - id: default
    required: true

  - id: loading
    required: true

  - id: error
    required: true

  - id: disabled
    required: true
```

Storybook 側では通常どおり Story を実装する。

```ts
export const Default = {};
export const Loading = {};
export const Disabled = {};
```

この場合、`error` が不足しているため Storymesh の UI Spec Check は失敗する。

---

## 4. Story と Scenario の違い

### Story

Story は UI の「状態」を表す。

例:

- Default
- Loading
- Error
- Disabled
- Empty

つまり、

> どの UI 状態を用意すべきか

を表現する。

### Scenario

Scenario は UI 上での「振る舞い」を表す。

例:

- ボタンを押したら保存処理が実行される
- Disabled 状態ではクリックできない
- Loading 中は Spinner が表示される

Scenario は操作と結果を含み、時間軸を持つ。

```text
初期状態
  ↓
ユーザー操作
  ↓
状態変化
  ↓
期待結果
```

MVP では Story の存在チェックだけを実装する。

将来的には Storymesh で、

- Story = UI 状態の定義
- Scenario = UI の振る舞いの定義

として両方を扱えるようにする。

---

## 5. MVP の責務

Storymesh の MVP では次のことを行う。

1. YAML の UI Spec を読み込む
2. 必要な Story を抽出する
3. Storybook の Story 一覧を取得する
4. 両者を照合する
5. Missing / Undeclared を検出する
6. PASS / FAIL を返す

### 判定

すべての required Story が存在する:

```text
PASS
```

1つでも required Story が存在しない:

```text
FAIL
```

内部的には、適合可否と検出した ERROR / WARNING を保持するシンプルな結果モデルとする。
具体的な Rust 型は「13. 内部モデル」で定義する。

---

## 6. MVP ではパーセントを採用しない

当初は UI Spec Coverage として `%` を表示する案もあった。

例:

```text
3 / 4 = 75%
```

しかし Storymesh の MVP で知りたいことは、

> UI Spec をすべて満たしているか

である。

そのため結果は、割合ではなく **適合 / 不適合** とする。

```text
全部満たしている
→ PASS

1つでも不足している
→ FAIL
```

### 理由

`95%` のような数値は、

> 95% なら十分

という解釈につながりやすい。

しかし `required: true` として定義した Story が不足しているのであれば、仕様としては未達である。

そのため Storymesh では MVP の時点では Coverage Model ではなく **Compliance / Validation Model** を採用する。

---

## 7. CLI

Storymesh にはすでに、コンポーネント単位で Story ファイルの存在を確認する `check` と、
件数・割合を表示する `coverage` が存在する。

そのため UI Spec Check は既存コマンドと責務を分離し、MVP では次のサブコマンドを想定する。

```bash
storymesh spec check
```

役割は次のとおり。

```text
storymesh check
  → コンポーネントに Story ファイルが存在するか

storymesh coverage
  → コンポーネント単位の Story coverage を表示する

storymesh spec check
  → YAML で宣言した必要 Story が実装されているか
```

これにより、既存機能を壊さずに UI Spec ベースの検証を追加できる。

---

## 8. CLI 表示

### 正常時

```text
$ storymesh spec check

Storymesh UI Spec

✓ UI spec satisfied

Components checked: 8
Stories checked:    24

No issues found.

PASS
```

### 異常時

```text
$ storymesh spec check

Storymesh UI Spec

✗ UI spec not satisfied

Missing stories
  ✗ Button/error
  ✗ Input/disabled

Undeclared stories
  ? Button/debug

Summary
  Errors:   2
  Warnings: 1

FAIL
```

重要なのは `%` ではなく、

- 通ったか
- 通らなかったか
- 何が不足しているか
- 何が Spec 外なのか

をすぐ確認できること。

---

## 9. Issue の分類

### Missing Story

UI Spec では required だが Storybook に存在しない。

```text
ERROR
```

例:

```text
Button/error
```

Missing が1つでも存在した場合:

```text
exit code 1
```

### Undeclared Story

Storybook には存在するが UI Spec には定義されていない。

```text
WARNING
```

例:

```text
Button/debug
```

MVP では Undeclared は CI failure にしない。

```text
Errors > 0
→ exit 1

Errors = 0
→ exit 0
```

---

## 10. Undeclared を Warning にする理由

Storybook には開発用や確認用として一時的な Story が存在する場合がある。

例:

```text
debug
playground
experimental
```

これらを常に ERROR とすると、開発時の自由度を下げる。

そのため MVP では、

```text
Missing    → ERROR
Undeclared → WARNING
```

とする。

将来的には Strict Mode を追加できる。

```bash
storymesh spec check --strict
```

Strict Mode では Undeclared も ERROR にする。

---

## 11. Story の紐付け

Story 名だけで照合すると、リネームに弱い。

そのため将来的には Storybook 側に Storymesh 用の安定した Spec ID を持たせる。

例:

```ts
export const Default = {
  parameters: {
    storymesh: {
      id: 'default'
    }
  }
};
```

UI Spec:

```yaml
stories:
  - id: default
    required: true
```

対応関係:

```text
Storymesh Spec ID
      ↓
   default
      ↓
Storybook Story
```

これにより表示名の変更と仕様上の ID を分離できる。

MVP 初期では Story 名の正規化による照合から開始してもよい。

---

## 12. 推奨 YAML

Storymesh の名前空間を意識して、将来拡張しやすい形にしておく。

```yaml
version: 1

component:
  id: button
  name: Button

stories:
  - id: default
    required: true

  - id: loading
    required: true

  - id: error
    required: true

  - id: disabled
    required: true
```

将来的には同じ Spec に Scenario を追加できる。

```yaml
version: 1

component:
  id: button
  name: Button

stories:
  - id: default
    required: true

  - id: disabled
    required: true

scenarios:
  - id: disabled-click
    story: disabled

    when:
      - click:
          role: button

    then:
      - notEmitted:
          event: onClick
```

MVP では `scenarios` は未実装とする。

---

## 13. 内部モデル

Storymesh は Rust CLI のため、ドメインモデルも Rust 側に置く。

### StoryRequirement

```rust
pub struct StoryRequirement {
    pub id: String,
    pub required: bool,
}
```

### Issue

```rust
pub enum SpecIssue {
    MissingStory {
        component: String,
        story: String,
    },
    UndeclaredStory {
        component: String,
        story: String,
    },
}
```

### CheckResult

```rust
pub struct SpecCheckResult {
    pub valid: bool,
    pub errors: Vec<SpecIssue>,
    pub warnings: Vec<SpecIssue>,
}
```

CLI の終了コード判定はドメインモデルとは分離し、既存の `src/cli` から扱う。

---

## 14. Storymesh 内部のモジュール構成案

現在の Storymesh は `src/cli`、`src/domain`、`src/scanner` を中心とした Rust CLI である。
UI Spec Check も既存の責務分離に合わせる。

MVP では例えば次の構成を想定する。

```text
src/
  cli/
    args.rs
    output.rs
    run.rs

  domain/
    coverage.rs
    framework.rs
    spec.rs

  spec/
    mod.rs
    parser.rs
    matching.rs

  scanner/
    ...
```

責務:

```text
domain/spec.rs
  → StoryRequirement / SpecIssue / SpecCheckResult

spec/parser.rs
  → YAML UI Spec の読み込みと検証

spec/matching.rs
  → Expected Story と Actual Story の照合

cli/
  → `storymesh spec check` の引数、表示、終了コード
```

既存のコンポーネント検出や Story ファイル検出で再利用できる部分は `scanner` 側を利用する。

MVP では過度に crate / package を分割せず、後から独立させられる境界だけを作る。

---

## 15. 処理フロー

```text
storymesh spec check
      │
      ▼
 UI Spec Loader
      │
      ▼
 YAML Parser
      │
      ▼
 Required Stories
      │
      ├──────────────────┐
      │                  │
      ▼                  ▼
Storybook Adapter    Spec Validator
      │
      ▼
 Actual Stories
      │
      └──────────┐
                 ▼
            Story Matcher
                 │
          ┌──────┴──────┐
          ▼             ▼
       Missing       Undeclared
          │             │
          └──────┬──────┘
                 ▼
             CheckResult
                 │
          ┌──────┴──────┐
          ▼             ▼
         CLI        exit code
```

---

## 16. CLI の責務

UI Spec Check は既存の `check` / `coverage` と分離する。

```bash
storymesh spec check
```

の意味は、

> Storybook の Story 実装が Storymesh UI Spec を満たしているか検証する

こと。

将来的には次のオプションを追加できる。

```bash
storymesh spec check --strict
storymesh spec check --json
storymesh spec check --verbose
```

### `spec check`

適合性確認。

```text
PASS / FAIL
```

MVP では割合レポートは追加しない。

進捗値が必要になった場合でも、既存 `coverage` と UI Spec の適合性を混同せず、
別のレポート機能として検討する。

---

## 17. CI での利用

GitHub Actions 等からそのまま利用できるようにする。

```yaml
- name: Check Storymesh UI Spec
  run: storymesh spec check
```

Storymesh 自身の開発中に動作確認する場合は、既存の開発フローに合わせて
`mise exec -- cargo run -- spec check` を使用する。

成功:

```text
exit 0
```

Missing Story が存在:

```text
exit 1
```

これにより Pull Request の時点で、

> UI Spec に必要な Story が実装されていない

状態を検出できる。

---

## 18. Storymesh の将来像

MVP:

```text
UI Spec
  ↓
Story existence
  ↓
PASS / FAIL
```

次の段階:

```text
UI Spec
  ↓
Story existence
  ↓
Story args / metadata
  ↓
PASS / FAIL
```

さらに:

```text
UI Spec
  │
  ├─ Story
  │    └─ UI State
  │
  └─ Scenario
       └─ Behavior
            ↓
      Test Runtime
```

最終的には、

```text
                 Storymesh Spec
                      │
          ┌───────────┼───────────┐
          ▼           ▼           ▼
      Storybook     Vitest     Playwright
          │           │           │
          └───────────┼───────────┘
                      ▼
              Spec Compliance
```

という UI Specification Validation 基盤へ発展させる。

---

## 19. MVP の最終方針

Storymesh の最初の UI Spec 機能では、以下に絞る。

- YAML で必要な Story を定義する
- YAML を UI Spec の Source of Truth とする
- Storybook の Story 一覧を取得する
- 必要 Story の存在を検証する
- Missing を ERROR として表示する
- Undeclared を WARNING として表示する
- required Story がすべて存在すれば PASS
- 1つでも Missing があれば FAIL
- `%` は表示しない
- CLI で「何が問題か」を具体的に表示する
- CI から利用できる exit code を返す
- Scenario は MVP の対象外とする

最小のユーザー体験は次の形とする。

```text
$ storymesh spec check

Storymesh UI Spec

✗ FAIL

Missing
  Button/error

Warnings
  Button/debug
```

Storymesh の MVP において重要なのは数値化ではなく、

> UI の仕様として宣言した Story が、実装側で満たされているか

をシンプルかつ機械的に保証することである。
