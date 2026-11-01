# Pop で検証つきフォームを作る

Pop のテーマを App に渡し、標準の text input と button を使います。編集値と保存値を分け、検証結果を modifier へ渡します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn form() -> impl IntoView {
    let email = signal(String::new());
    let saved = signal(String::new());
    let invalid = memo(move || email.with(|s| !s.contains('@')));
    column((
        text_input("連絡先メール", email).invalid(invalid).max_length(120_usize),
        button("保存").disabled(invalid).on_press(move || saved.set(email.get())),
        text(move || format!("保存済み: {}", saved.get())),
    )).style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("連絡先").theme(pop::theme(kivact::style::ThemeMode::Light)).run(form)
}
```

ここでの検証は形式の例です。業務上の検証や server のエラーはアプリが持ちます。IME の未確定文字は binding に届かないので、入力途中の変換候補を保存した値として扱いません。

多くの行を扱う場合も、データは caller の signal/Binding に置き、[List/Table の契約](../../for-kivact-user/how-to-guide/lists-tables.md)を使います。保存、通信、一覧の並べ替えをテーマへ持たせません。外観を変える順序は token、recipe、個別 style です。

## 項目列の変更をアプリで反映する

標準の `kivact::ui::tag_input(label, tags, on_add, on_remove)`、`chip_list(tags)`、標準の `kivact::ui::data_table(rows, key, columns)` は項目列を `impl IntoValue<Vec<T>>` で読取専用に受けます。signal を渡しても kit はその列を書きません。タグの追加・削除は `on_add` と `on_remove` で受け、chip は `.on_add(..)` と `.on_remove(..)` で受け、アプリが重複や上限を検査して反映します。削除の引数は一覧内の index です。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::tag_input;

fn tags() -> impl IntoView {
    let tags = signal(vec![String::from("Rust")]);
    tag_input(
        "タグ",
        tags,
        move |tag| tags.update(|items| {
            if !items.contains(&tag) { items.push(tag); }
        }),
        move |index| tags.update(|items| {
            if index < items.len() { items.remove(index); }
        }),
    )
}
```

`chip_list` の `on_add` は引数なしの要求 `()` です。入力用の dialog などをアプリで開きます。table の行はアプリの型 `R` のままで、`key` が行の識別子を返します。表の機能は、その状態を binding で渡したときだけ付きます。選択の check box と選択中の件数は `.selection(selected)`（選択した行の key、`Vec<K>`）、検索欄は `.filter(query)`（欄の hint は `.filter_placeholder(..)`）、ページは `.page(page, size)`（ゼロ始まりの page と1ページの行数）です。並べ替えの見出しは binding がなくても働き、並べ替えの列（`columns` での位置）と向き `Option<(usize, kivact::view::SortDirection)>` をアプリが持つときは `.sort(sort)` で渡します。並べ替えや検索をすると先頭の page に戻ります。範囲外の page は表示上だけ制限し、binding は書き換えません。列はすべての kit で共通の `kivact::ui::DataColumn::new(title, text)` で作り、`text` が cell の名前と既定の表示になります。幅は `.width(..)`（既定は160）、並べ替えるには `.compare(..)`（text の順でよければ `.sortable()`）、検索の対象にするには `.filter(..)`、cell を描き替えるには `.cell(..)`、末尾に揃えるには `.numeric(true)` を指定します。どれも渡さない列は、並べ替えも照合もしません。行ごとの操作（詳細を開く button など）の列は、表の `.row_actions(width, |row| view)` で足します。見出しは題を出さず、「Actions」という名前を持ちます。

書き出しは `.actions(|| ..)` にアプリの button を置き、列・行・TSV などの形式を選んで `kivact::platform::copy_text` 等へ渡します。kit は自動で書き出しません。catalog の table はアプリ側で TSV を作る例です。

## segmented control と field の hint

segmented control は、標準の `radio_group` に Pop の型 `Segmented` を `.variant(..)` で渡して表します。排他的な選択なので tabs ではなく radio group で、矢印キーで移動と選択を行います。各 segment の icon と label は既定の slot で描きます（tabs は `.item(pop::tab_label)`）。`kivact::ui::field(label, control)` は control の上に label を置き、`.hint(text)` で下に説明を足し、`.error(value)` でエラーを関連付けます。豊かな説明は呼出元の `column` で合成します。部品の slot（部品の中の決まった場所に置く内容）は、このように view を作る closure で渡します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn period() -> impl IntoView {
    let period = signal(Some("week"));
    kivact::ui::field(
        "期間",
        radio_group(
            "期間",
            vec![ListItem::new("day", "日"), ListItem::new("week", "週").description("7")],
            period,
        )
        .orientation(Orientation::Horizontal)
        .variant(pop::Segmented),
    )
    .hint(|| "集計の単位です。")
}
```

## 内容と表示の意味を渡す

`dropzone(label).on_files(..)` の説明は `.hint(..)`、native picker の filter は `.filters(impl IntoValue<Vec<(String, Vec<String>)>>)` で渡します。既定はどちらも空です。filter や説明はファイルの大きさの検証にはならないため、受け取ったファイルの検証はアプリで行います。

`stat_card` の傾向の色は `.tone(Hue)` で明示します。Hue（Blue・Mint・Yellow・Pink・Red・Gray）は Pop の色相の名前で、`.variant(pop::Hue::Pink)` のように外観の variant にも使います。通知や alert の意味は標準の `kivact::ui::Tone`（Neutral・Info・Success・Warning・Danger・Accent）で表します。`agenda_item` の時間帯は `.period(..)` で渡し、`.on_details(..)` があるときだけ詳細 button を描きます。`calendar(...).weekend(|weekday| ..)` は `Weekday` から `Option<Hue>` を返す関数で、既定では週末を着色しません。`Mission` と mission card、画面上部の hero は catalog のアプリ固有の例です。`thread(..)` の avatar は `.portrait(|| view)`、`stepper(..)` の各段階の注記は `.notes(..)`、`timeline_item(..)` の title は `.title_content(|| view)` で差し替えます。`kivact::ui::toast_region(toaster)` を mount しても全体の設定は変わりません。通知は core の `Toast::new(title).tone(Tone::Success).body(text)` で作り、tone の icon は Pop の theme が描きます（`Toast::new("保存しました").tone(Tone::Success).body("下書きを送信しました")`）。icon は `kivact::control::icon("name")` で描き、Pop の画像は theme の `IconSet` に入っています。通知の表示上限はアプリの setup で `kivact::ui::Toaster` に設定します。

`cargo run -p kivact-ui-pop-catalog` でフォーム・一覧・dialog を確認できます。試験では accessible role/name から入力と保存を操作し、値・検証表示・disabled・errors を確認します。OS IME の手動確認は自動試験と分けます。

`calendar(selected, today).year_month_menus(true)` は月の題名を年・月の select に替えます。年は表示年の前後50年で、年や月を選んでも選択済みの日付は変わりません。月ラベルは `month_title` と同じ formatter を使い、各月の1日を渡します。

`date_range_input` の popup は開始日・終了日の選択段階を polite な案内文で示します。「Close」で閉じると開始日の入力欄へ focus が戻ります。文言は [i18n のキー一覧](../../for-kivact-user/how-to-guide/i18n.md)から差し替えます。

必須値だけをコンストラクタへ渡します。`stat_card(title, value)` の `.trend`・`.icon`・`.points`、`list_item(title)` の `.subtitle`・`.icon`・`.trailing` は任意です。文言とサンプルの既定は空、icon はなし、tone は Gray です。`thread(initials, title)` の `.preview`・`.unread`・`.selected`、`message(content)` と `message_group(content)` の `.timestamp`・`.mine`・`.initials` も任意で、件数は 0、bool は false が既定です。`date_range_input((start_label, end_label), (start, end), today)` は必須の開始日と終了日を組で受けます。`stepper(labels, current).notes(notes)`、sparkline、file filter は反応する一覧を読みます。
