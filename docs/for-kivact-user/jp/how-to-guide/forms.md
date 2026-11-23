# フォームを検証してから保存する

[English](../../en/how-to-guide/forms.md)

編集中の下書きを検証し、`memo` で誤りを計算し、`field` で入力欄の隣に表示し、ユーザーが保存したときにだけ確定した値を書き込みます。このガイドでは、まず 1 つの入力でこのパターンを作り、次に標準の入力部品をすべて使ったフォームを作ります。

## 最短で動くフォーム

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    let name = signal(String::new()); // the draft
    let saved = signal(None::<String>); // the committed value

    let error = memo(move || {
        name.with(|n| n.trim().is_empty()).then(|| "Enter a name".to_owned())
    });
    let save = move || {
        if error.get().is_none() {
            saved.set(Some(name.get().trim().to_owned()));
        }
    };

    column((
        field("Name", text_input(name).on_submit(save))
            .hint("Shown on your profile")
            .error(error),
        button("Save").disabled(move || error.get().is_some()).on_press(save),
        show_some(saved, |name| text(move || format!("Saved: {}", name.get()))),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Profile").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_is_disabled_until_the_name_is_valid() {
        let screen = kivact::test::render(app);
        screen.get_by_text("Enter a name");
        assert!(screen.get_by_role(Role::Button, "Save").is_disabled());

        screen.get_by_role(Role::TextBox, "Name").type_text("Ann");
        assert!(screen.query_by_text("Enter a name").is_none());
        screen.get_by_role(Role::Button, "Save").click();
        screen.get_by_text("Saved: Ann");
    }
}
```

この例は 3 つの部品で成り立っています。

1. **入力に渡したシグナルが下書きです。** `text_input(name)` は、確定した編集のたびに `name` へ即座に書き込みます。この時点では何も「保存」されていません。`saved` が変わるのは `save` が実行されたときだけです。
2. **誤りは派生値です。** `Option<String>` を返す `memo` は、入力が正しい間は `None`、正しくない間は `Some(メッセージ)` になります。`name` が変わると再計算され、結果が変わったときだけ読み手に通知します。
3. **`field` が部品をつなぎます。** ラベルは入力のアクセシブルな名前になり、hint は入力の説明になります。error が `Some` の間は、入力の下にメッセージを表示し、入力を invalid にし、メッセージを入力の説明に加えます。`invalid` や `described_by` を自分で結ぶ必要はありません。

クロージャに `move` が付いている理由: ボタンのハンドラーは `app` が終わった後も生き続けるので、使う値を自分で所有する必要があります。`name`、`saved`、`error` は小さな `Copy` ハンドルなので、クロージャも `Copy` になり、同じ `save` を `on_submit` と `on_press` の両方に渡せます。ハンドルを clone する必要はありません。

## Enter で送信する

`text_input(..).on_submit(f)` は、入力欄で Enter が押されたときに `f` を実行します。上の例のように保存ボタンと同じクロージャを共有すれば、どちらの経路でも同じ検証が走ります。

- `on_submit` は 1 行の入力用です。`text_area` では Enter は改行になるので、送信にはボタンを使います。
- IME が変換中の文字はバインディングに書かれず、確定した文字だけが書かれます。そのため、下書きに変換途中の文字が入ることはありません。
- 変換中かどうかを知りたいとき(たとえば変換中だけ hint を隠したいとき)は、`.on_composition_change(|composing| ..)` を使います。

## 下書きと保存済みモデルを分ける

値が複数あるときは、構造体全体を下書きとして持ち、もう 1 つのコピーを保存済みの値にします。`bind!(draft.name)` は、構造体の 1 つのフィールドへの `Signal` を、入力ごとに作ります。

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Plan {
    Free,
    Pro,
}

#[derive(Clone, PartialEq)]
struct Profile {
    name: String,
    bio: String,
    plan: Plan,
    volume: f64,
    public: bool,
    terms: bool,
}

fn app() -> impl IntoView {
    let saved = signal(Profile {
        name: String::new(),
        bio: String::new(),
        plan: Plan::Free,
        volume: 40.0,
        public: true,
        terms: false,
    });
    let draft = signal(saved.get());

    let name = bind!(draft.name);
    let bio = bind!(draft.bio);
    let plan = bind!(draft.plan).optional();
    let volume = bind!(draft.volume);
    let public = bind!(draft.public);
    let terms = bind!(draft.terms);

    let name_error = memo(move || {
        draft.with(|p| p.name.trim().is_empty()).then(|| "Enter a name".to_owned())
    });
    let terms_error = memo(move || {
        (!draft.with(|p| p.terms)).then(|| "Accept the terms to continue".to_owned())
    });
    let valid = memo(move || name_error.get().is_none() && terms_error.get().is_none());
    let changed = memo(move || draft.with(|d| saved.with(|s| d != s)));

    column((
        field("Name", text_input(name)).error(name_error),
        field("Bio", text_area(bio).wrap(true).rows(2..=5)),
        field(
            "Plan",
            select(
                vec![ListItem::new(Plan::Free, "Free"), ListItem::new(Plan::Pro, "Pro")],
                plan,
            ),
        ),
        field("Volume", slider(volume).step(5.0)),
        switch("Public profile", public),
        field("Terms", checkbox("I accept the terms", terms)).error(terms_error),
        row((
            button("Save")
                .disabled(move || !valid.get() || !changed.get())
                .on_press(move || saved.set(draft.get())),
            button("Reset")
                .disabled(move || !changed.get())
                .on_press(move || draft.set(saved.get())),
        ))
        .style(style().gap(8)),
    ))
    .style(style().gap(12).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Profile").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_needs_a_valid_changed_draft_and_reset_restores_the_saved_one() {
        let screen = kivact::test::render(app);
        let save = || screen.get_by_role(Role::Button, "Save");
        assert!(save().is_disabled());

        screen.get_by_role(Role::TextBox, "Name").type_text("Ann");
        assert!(save().is_disabled(), "the terms are not accepted yet");

        // `field` made its label the checkbox's accessible name.
        screen.get_by_role(Role::CheckBox, "Terms").click();
        assert!(!save().is_disabled());

        save().click();
        assert!(save().is_disabled(), "nothing differs from the saved profile");

        screen.get_by_role(Role::TextBox, "Name").type_text(" B");
        assert!(!save().is_disabled());
        screen.get_by_role(Role::Button, "Reset").click();
        assert_eq!(
            screen.get_by_role(Role::TextBox, "Name").value().as_deref(),
            Some("Ann")
        );
    }
}
```

各部分の役割です。

- `bind!(draft.field)` は、下書きの 1 つのフィールドへの `Signal<U>` を返します。書き込むと下書きに書かれます。
- `select` と `radio_group` は `Option<K>` を編集します(何も選ばれていない状態が `None` です)。モデルが素の値を持つときは `.optional()` を付けます。読むと `Some(現在の値)` になり、`None` の書き込みは無視されます。`K` には `Clone + Eq + Hash` が必要で、上のような小さな enum なら derive できます。
- `changed` は `with` で 2 つのコピーを、clone せずに借用して比較します。構造体が `PartialEq` を derive しているのはこのためです。
- `field` は、入力のアクセシブルな名前をそのラベルで置き換えます。そのため上のチェックボックスは、支援技術からもテストからも「規約」として見つかります。
- 保存は下書きを保存済みの値へコピーし、元に戻すは逆向きにコピーします。保存済みの値は、サーバーへ送ったりディスクへ書いたりする値です。

`Clone + PartialEq` は、`get` で読んで `set` で比較するシグナルに必要な境界です。フォームのモデルには両方を derive してください。`String` や `f64` のフィールドは、すでに両方を満たしています。

## 入力部品を選ぶ

| 値 | 入力部品 | 値の型 |
| --- | --- | --- |
| 短い文字列 | `text_input(Signal<String>)` | `Signal<String>` または `bind!(form.field)` |
| 長い文字列 | `text_area(Signal<String>)` | 同上。Enter は改行になる |
| 少数の選択肢から 1 つ | `radio_group(name, items, Signal<Option<K>>)` | 選択肢がすべて見える |
| 多数の選択肢から 1 つ | `select(items, Signal<Option<K>>)` | ドロップダウン |
| 範囲内の数 | `slider(Signal<f64>)` | `.range(..)`、`.step(..)` |
| すぐ反映するオンとオフ | `switch(label, Signal<bool>)` | |
| はい / いいえの選択 | `checkbox(label, Signal<bool>)` | 中間状態は `.indeterminate(..)` |
| 入力する数値 | `number_input(..)` | `kivact::ui` の rustdoc を参照 |

ラベルの見えない部品(テキスト欄、選択、スライダー、日付と時刻の欄)は、`.label(..)` か、外側の `field(..)` から名前を得ます。名前のない操作可能な部品は、マウントされたときに報告されます。

`checkbox` と `switch` の最初の引数は、見えるラベルです。

## 誤りを見せるタイミング

上の例は、下書きが正しくなくなった時点で、まだ何も入力していなくても誤りを表示します。ユーザーが保存を試みるまで待つには、メッセージをフラグの後ろに置きます。

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    let email = signal(String::new());
    let tried = signal(false);

    let problem = memo(move || {
        email.with(|e| !e.contains('@')).then(|| "Enter an email address".to_owned())
    });
    // Visible only after the first attempt.
    let shown = memo(move || if tried.get() { problem.get() } else { None });

    column((
        field("Email", text_input(email)).error(shown),
        button("Subscribe").on_press(move || {
            tried.set(true);
            if problem.get().is_none() {
                println!("subscribed {}", email.get());
            }
        }),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Subscribe").run(app)
}
```

そのほかの決まりです。

- 入力が独自の理由で invalid(`text_input(..).invalid(..)`)のときは、`field` に error がなくても invalid のままです。入力は、どちらか一方でも invalid と言えば invalid になります。
- バインディングの書き込み側(`binding(read, write)` の `write`)が値を拒否すると、入力欄はバインディングの値へ戻ります。これは「数字だけ」のような入力フィルター向けで、「必須」の表現には使いません。
- `.max_length(n)` は文字数(Unicode スカラー値)で入力を制限します。変換中の文字が途中で切られることはありません。
- `.read_only(true)` は選択とコピーを許し、`.disabled(..)` は入力を受け付けません。`.purpose(InputPurpose::Password)` は文字を隠し、コピーと切り取りを防ぎます。
- ダブルクリックするとポインターの下の単語を、トリプルクリックするとその行を選択します(1 行の欄では欄の全体)。
- テキストエリアでは、上下矢印でキャレットを同じ横位置のまま上下の行へ動かします(Shift 付きなら選択を広げます)。最初の行での上矢印は先頭へ、最後の行での下矢印は末尾へ動きます。キャレットがすでにそこにあるときは、キーは欄を囲むハンドラーへ渡ります。

## 1 つの欄を Enter かフォーカス移動で確定する

その場での名前変更のように、ユーザーが確定したときだけ編集を反映したいことがあります。`text_input(..).on_commit(on_commit)` は、欄を下書きにします。Enter か欄からフォーカスが外れると、テキストを渡して `on_commit` が呼ばれ、Escape は編集前の文字列に戻します。

その場での名前変更では、どの項目を変更中かも分かっている必要があります。欄を文字列を選択した状態で開き、確定後に閉じるためです。`use_inline_edit` がそれを持ちます。`edit(id)` で変更を始め、`editing()` で変更中の項目のキーを得て、`field(..)` で欄を作ります。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::use_inline_edit;

fn names() -> impl IntoView {
    let items = signal(vec![(1u32, "Intro".to_owned()), (2, "Outro".to_owned())]);
    let rename = use_inline_edit(signal(None::<u32>));
    column(for_each(
        items,
        |(id, _)| *id,
        move |item| {
            let id = item.get().0;
            row((
                text(move || item.get().1),
                show(
                    move || rename.editing().get() != Some(id),
                    move || button("Rename").on_press(move || rename.edit(id)),
                ),
                show(
                    move || rename.editing().get() == Some(id),
                    move || {
                        let api = rename.field(id, "Name", signal(item.get().1)).on_commit(move |name| {
                            items.update(|all| {
                                if let Some(row) = all.iter_mut().find(|row| row.0 == id) {
                                    row.1 = name;
                                }
                            });
                        });
                        node().part(api.control())
                    },
                ),
            ))
        },
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Rename").run(names)
}
```

欄は現れたときにフォーカスを取り、文字列を選択した状態になります。Enter か、変更したまま欄を離れると、文字列を渡して確定の関数が呼ばれます。Escape は編集を捨てます。どちらの場合も欄は閉じます。行が取り除かれたときも欄は閉じるので、名前変更が項目より長く残ることはありません。`invalid` の入力は確定されません。画面に残る欄には、`text_input(..).on_commit(on_commit)` だけを使います。Enter は下書きのものになるため `on_commit` は `on_submit` の代わりになり、`.on_edit(..)` は `Begin`、`Commit`、`Cancel` を報告します。

## 関連項目

- [一覧と表](lists-and-tables.md): 一覧の行の編集と、行のチェックボックスから一覧への書き戻し。
- [オーバーレイ](overlays.md): 未保存の変更をダイアログで確認する。
- [翻訳とロケール](i18n.md): ラベル、hint、エラーメッセージを翻訳する。
- [アプリをテストする](testing.md): 上のテストのようにフォームを操作する。
- リファレンス: 各入力のオプションは `cargo doc -p kivact-ui --open`。

### モバイル入力と状態復元

`.purpose(InputPurpose::Email)` で期待する入力を指定します。
ほかに Text、Number、Decimal、Url、Phone、Password、Search があります。
Password は値を隠し、クリップボードやアクセシビリティへの文字列公開を防ぎます。
デスクトップの winit は Password をパスワードのヒントに、それ以外を Normal に写します。
`platform::keyboard_inset()` でソフトウェアキーボードが覆う範囲を論理ピクセルで監視できます。

`platform::on_lifecycle` で `platform::LifecycleEvent::SaveState` を受け、
`platform::save_state("draft", bytes)` を呼びます。前回の起動で保存した値は
`platform::restored_state("draft")` で読み、キーがなければ `None` が返ります。
保存しても起動時のスナップショットは変わりません。対応状況は
`platform::capabilities().get().state_restoration` で確認します。デスクトップは false で保存を無視します。
仮想テストでは `screen.script().save_state()` で保存を要求し、
`screen.script().saved_state()` を次の起動の `RenderOptions::restored_state` に渡します。

## カレンダーと日付・時刻の入力

```rust,no_run
use kivact::prelude::*;
use kivact::i18n::{CalendarDate, DateRange, DateStyle, TimeOfDay, DateTime, TimeStyle};
use kivact::ui::{self, DatePresentation};
fn dates() -> impl IntoView {
    let day = signal(None::<CalendarDate>);
    let range = signal(None::<DateRange>);
    let time = signal(None::<TimeOfDay>);
    let appointment = signal(None::<DateTime>);
    column((
        ui::calendar(day),
        ui::date_input(day).label("Date").format(DateStyle::Short),
        ui::date_range_input(range).label("Period"),
        ui::time_input(time).label("Time").format(TimeStyle { seconds: false }),
        ui::date_time_input(appointment)
            .label("Appointment")
            .time_format(TimeStyle { seconds: true })
            .presentation(DatePresentation::Adaptive),
    ))
}
```

`calendar` は単日または順序付き範囲の Signal を受け取ります。
表示月は `month` の Signal で管理します。`today`、`min`、`max`、`week_start`
はリアクティブな値を受け取れます。`disabled_dates` は選択を禁止し、`marks`
は日付を装飾します。`range_span` は両端を含む日数を制限します。
月・年のメニューには `caption(CalendarCaption::Dropdown)` を使います。

文字列は `Locale::parse_date` で ISO またはロケールの日付として解釈します。
Enter は確定し、下矢印はカレンダーを開きます。カレンダーの矢印、Home/End、
Page Up/Down は移動し、Enter / Space は選択します。月名・曜日名はロケールに
従います。`DatePresentation::Adaptive` は `Capabilities.touch` に従って
下端のシートを選び、`Desktop` / `BottomSheet` は表示方法を明示します。
wheel は spinbutton のキーと支援技術の値操作に対応します。

外観は部品のテーマ変数と、`kivact::ui::theme` の `<Name>Theme` の表
(`CalendarTheme`、`DateInputTheme`、`TimeInputTheme`)で変更します。
## 操作とトグルをまとめる

関連する三つ以上の操作は `toolbar` でまとめます。start・center・end の領域全体が
一つの Tab 停止位置になります。向きに沿った矢印キーでフォーカスを移し、Home と
End で利用可能な先頭と末尾へ移動します。見出しを表示しない場合も名前を付けます。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{button_group, toggle_group, toolbar};

fn editor_actions() -> impl IntoView {
    let formatting = signal(Vec::<&'static str>::new());
    column((
        toolbar("Document actions")
            .start((button("Undo"), button("Redo")))
            .end((button("Save"),)),
        toggle_group("Formatting", vec![
            ListItem::new("bold", "Bold").with_icon("bold"),
            ListItem::new("italic", "Italic").with_icon("italic"),
        ], formatting),
        button_group("Clipboard", (button("Copy"), button("Paste"))),
    ))
}
```

トグルには最大一つを押下状態にする `Signal<Option<K>>`、または複数を選ぶ
`Signal<Vec<K>>` を渡します。binding と正準の `Selection` も使えます。
矢印キーは選択を変えず、フォーカスだけを移動します。Space・Enter・クリック・
アクセシビリティの起動操作で項目を切り替えます。`ListItem::with_pressed_icon`
は押下中のアイコンを指定し、`icon_only(true)` でも名前は残ります。
移動と同時に一つを選ぶ排他的な選択には
`radio_group(..).variant(kivact::ui::RadioGroupVariant::Segmented)` を使います。

`button_group` は `IntoChildren` を通して任意の操作部品を受け取ります。
`button_group_separator()` と `button_group_text(..)` はフォーカスを取りません。
どのグループでも `.orientation(Orientation::Vertical)` で配置と操作の向きを
変えられます。無効化には共通の `Element` の `.disabled(source)` を使います。
テーマは対応する部品のテーマ変数と `kivact::ui::theme` の `<Name>Theme` の表で見た目を変えます。
呼び出し側のスタイルは、それらより優先します。

`radio_group(...).item_node(|target, item, state| target.with_node(...))` は、キットのキーボードヒントなどを付けるために完成済みの標準ターゲットを渡します。受け取った target に modifier を付けて返し、別のネイティブなフォーカスや選択の owner を作らないでください。`.item(...)` は内容だけを置き換え、両方の slot を組み合わせられます。

### Picker の操作と touch sheet

日付・時刻の field は TextBox と独立した picker button を持ちます。
`picker_label(..)` で button の名前を別に指定できます。下矢印または
Alt+下矢印で picker を開き、内部へ focus を移します。選択・dismiss 後は
実際に開いた入力または button へ戻り、外側をクリックした場合は移動先の
focus を保ちます。

範囲の最初の端点は一時的な候補で、部分的な range は公開しません。
2 点目で順序付き range を公開し、Escape は候補を破棄します。bottom sheet
では日付選択と time wheel の各操作を即座に反映し、Save で閉じます。
日時の sheet は Save date で Time に進み、その後 Save で閉じます。
月の見出しは Month/Year wheel を開き、Done で calendar に戻ります。
Spinbutton の上矢印 / Increment は増加、下矢印 / Decrement は減少で、
Page Up/Down は大きな step を使います。月の swipe は接触した日を選択しません。

独自の interaction は `CalendarOptions::activation_guard` で、disabled 表示を
変えずに単日・範囲の遷移前に activation を拒否できます。
`DialogApi::return_focus` は定数または reactive な NodeRef を受け取り、
開くたびに復帰先を snapshot します。
