# Primitive と操作対象の契約

`kivact::primitive` は部品作者が新しい基本部品を作る口です。実験的な API を含みます。通常の合成で足りる場合は Node と interaction の Part を使います。

> **自分の値の型を prop にするときは、型を定義した crate で `kivact::view::constant_value!(MyType);` を1行書きます。** これで `impl IntoValue<MyType>` の引数に定数の `MyType` をそのまま渡せます（signal・`Value`・closure は書かなくても渡せます）。`impl<T: Clone + PartialEq> IntoValue<T> for T` の blanket impl で済ませないのは、closure を受ける `impl<T, F: Fn() -> T> IntoValue<T> for F` と coherence で衝突するためです（`F` が自分自身を返す closure 型である可能性を Rust は除外できず、E0119 になります）。closure を受けられることを優先し、定数を受ける型は一つずつ登録します。登録していない型を呼出し側が定数で渡すときは `kivact::view::constant(value)` で包みます（`badge(constant(Lane::Done))`）。登録を忘れた型をそのまま渡すと「`MyType` cannot be used as a `MyType` property」と出ます（この note はコンパイラの診断には出ません。診断の note はすべての型に共通で、型ごとに出し分けられないためです）。

## Part と target

`part(|input: PartState| ...)` は状態・focus・入力・semantics を持つ部位です。`part`・`Part`・`PartState`・`PartMount` はどれも `kivact::primitive` にあります。一度だけ node の `.part(part)`、または canvas の `.region(id, part)` へ渡します。`state()` と `node_ref()` は消費する前に取り出します。複数の Part を同じ対象へ重ねる場合も、focus の正本は一つです。

disabled/hidden は caller と OR、focusable は AND で合わさります。canvas は宣言に加え、描画時に同じ ID の region の幾何を与える必要があります。Tab は宣言順、hit は描画の重なり順を使います。role/name と AT の操作も同じ target に結びます。数値の内部 ID や独自の focus manager は不要です。

Canvas 全体の pan や context menu のように、region が処理しなかった入力も受ける部位は `.region_events()` を付けます。caller の handler、region の既定処理、その部位の順で pointer/key/wheel を処理します。合成した別の部位、activation、AT の操作は転送しません。capture 中は開始した region の識別を保ち、region の退役・無効化や終端イベントの消費でも、開始済みの振る舞いへ Cancel を届けます。`use_zoom_pan` と標準 menu はこの口を内部で使うため、kit で入力を転送する必要はありません。

opt-in した部位が主ボタンの押下を消費した場合は、その gesture の release を region のクリックとして重ねて扱いません。支援技術・キーボードの activation と、次の通常のクリックは独立して動きます。

## 独自の大きさを測る

```rust,no_run
use kivact::prelude::*;
use kivact::draw::Size;
use kivact::primitive::{Constraints, primitive};

fn strip(width: impl IntoValue<f32>) -> impl IntoView {
    primitive().measure(width, |width: &f32, _: Constraints| {
        Size::new(*width, 24.0)
    }).style(style().background(ColorToken::Accent))
}
```

measure は渡した content と constraints だけで答える純粋な関数です。engine が model を借用して呼ぶため、signal、theme、engine の木をその中から読みません。必要な値を content に含めます。`Constraints::measure_text` は同じ文字 style の計測を借りて使えます。返す baseline は content box 上端からの logical px です。独自 measure は子を持つ layout container を作る口ではありません。

[page_dots](../../dev/tests/fixtures/external-component/src/page_dots.rs) は、ページ数で測り、描画と region の幾何を同じ値から作る実例です。

## Native 編集の下位契約

通常のキー編集には [TextEdit](interaction.md#native-編集を付ける)を使います。さらに下の `TextEditing` は Part に編集のテキスト、shaping、IME session、選択、caret を付けます。作者が role/name/focus と controlled な通知処理を決めます。`TextEditing::new` のテキストは `Rc<str>` の source で、editor は自分の写しを持ちます。アプリが文書を差し替えるときだけ書けば、1文字の入力で source は動きません。

キーと paste の規則は `TextEditing::behavior(f)` に渡す closure です。未指定なら標準のキー policy は付きません。`f(state, request, context)` は与えられた `EditState` を変更し、捕まえた状態（構文、入力の制限、undo の履歴など）を `Cell`・`RefCell` で読み書きできます。engine が木を借りている間に同期で呼ぶので、signal・node の照会・待つ処理は使えません。Tab と Shift+Tab も `f` に届き、`EditOutcome::NotEditing` を返したときだけ focus が移ります。自分で扱わない key は `kivact::interaction::text_edit_policy(state, request, context)` に委ねます。IME の transport と pointer の hit/選択は native に残ります。`f` が panic したときは、その request を処理しなかったことにし（panic の前の変更も戻ります）、一度だけ error として報告します。`f` がテキストを変えたら必ず `changed: true` を返します。報告しない変更は戻されます。

`EditContext` は editor の制約（`multiline`・`read_only`・`password`）と、request が届いたときのテキストの配置を読む口です。`line_count(state)`・`line_of(state, at)`（byte offset の行。改行はその行に含む）・`line_range(state, line)`（改行を除く byte 範囲）・`x_of(state, at)`（caret の x）・`offset_at(state, line, x)`（その行で x に最も近い grapheme 境界）で、上下の行への移動のような配置に依る key を書けます。`x` は editor の text の箱の先頭からの logical px で、横の scroll の前です。引数の `state` は behavior に渡されたものを渡します。behavior がテキストを変えた後、配置されていない間、placeholder を表示している間は `None` です。

`TextEditing::highlight(content, f)` は行ごとの色を付けます。`f(&content, line, state)` は改行を除いた1行と、前の行が残した状態（最初の行は `S::default()`）を受け取り、その行の byte 範囲と色（`TextHighlight`）と、次の行へ渡す状態を返します。block comment や複数行の文字列のような行をまたぐ token は状態で受け渡します。色は glyph の描き方だけを変え、shaping・caret・選択は変えません。engine は描く行と、その前でまだ色を付けていない行についてだけ `f` を呼びます。編集の後は変わった行を呼び直し、その後の行は始まりの状態が変わった間だけ（描く範囲まで。残りは描くときに）呼び直します。content か theme が変わると色を付け直します。`f` は引数だけの関数にし、signal を読み書きしません。状態を持たない色付けは `S = ()` にします。

`TextEditEvent` は全体の値ではなく変更を渡します。`changes` は順に適用する範囲の置換（`TextChange`。各範囲は一つ前の変更の後のテキストの byte 範囲）、`session` は変更後の選択と composition です。`event.apply(&mut copy)` で写しを editor と同じに保てます（preedit も含む）。範囲が写しに文字境界で収まらない変更は適用せず `false` を返すので、そのときは `edit_state` で写しを読み直します。composition 中にアプリが書いた値は composition の終わりに当てられ、その変更も `TextEditEvent` で届きます。全体の値が要る場合は `TextEditing::report_text(true)` を付けると `event.text` に入りますが、編集ごとの複製になります。`EditState`・`TextSession`・`TextChange`・`TextEditEvent` の範囲は UTF-8 byte offset、platform IME との境界は UTF-16 unit です。`TextLayout` の line range は byte、caret offset は値の先頭からの Unicode scalar offset です。矩形と baseline は scroll・transform 適用後の logical window 座標で、倍率を手で掛けません。

`TextEditing::on_selection(f)` は選択と composition（`TextSession`）が変わるたびに `f` を呼びます。caret の移動、選択、編集（その `on_edit` の後）、`set_edit_session` の書き込みのどれでも、最初の配置のときにも一度呼びます。行・列の表示はここで signal に書きます。

`NodeRefExt` の `edit_session`・`edit_state`・`text_layout` は購読せず snapshot を読みます。`edit_session` はテキストを複製しません。`text_layout(lines)` は求めた行の範囲だけを shaping して、その行と caret、caret の位置、行の数（`line_count`）を返します。caret だけなら `0..0`、表示している行や caret の周りの行だけを求めてください（全行を求めると文書の長さに比例します）。`edit_state` の複製は以後の変更を記録し、editor のテキストが変わる前に `set_edit_state` で返せば、その範囲の変更だけが適用されます。`set_edit_state` は pure computation の外で使い、disabled・read-only・長さ制限を守ります。値または composition の変化だけが編集通知になり、caret の移動だけなら通知しません。選択だけを変えるなら `set_edit_session(session)` を使います。テキストを読まず複製もせず、session はテキストの文字境界に丸めます。disabled では拒否し、read-only では選択だけを受け付けます。`set_edit_value` は controlled な値の同期用で、編集通知を発行しません。同じ `Rc` を編集の前にもう一度渡しても何もしません。

editor は hard line ごとに shaping し、編集は変わった行だけを shaping し直します。計測・描画・行の色・accessibility の text run・IME に渡すテキストも行ごとで、描くのは見えている行だけです。1文字の入力の費用は文書の長さにほとんど比例しません。

[code editor の fixture](../../dev/tests/fixtures/external-design-system/src/code_editor.rs) は、範囲の変更で写しと行数と undo を保ち、Tab の字下げ・Enter の字下げの継続・行の Home/End を behavior で、上下の移動を `EditContext` の配置で、キーワードと block comment の色を状態を受け渡す highlight で、caret の行と列を `on_selection` で作る実例です。未 mount の参照と region には作用しません。

編集を付けた node は native leaf です。Box/Text/Image に付けられますが、子・独自 draw・canvas region の編集は `Unsupported` です。a11y の text snapshot は描画と同じ shaping に由来し、password は snapshot を省きます。AT からの範囲選択要求、混在 bidi の run 分割、語境界 metadata は未対応です。[手動確認](../for-kivact-developer/manual-verification.md)も別に残しています。

## Owner と寿命

`Owner` と `Owned<T>` は owner に属する状態と cleanup を扱い、`NodeToken` は mount の不透明な識別を扱います。外部部品は Runtime、wire の NodeId、木の借用を保持しません。timer/task と bind 後の資源は実際に mount された owner の cleanup に結び、再表示時に前の control が残らないようにします。`NodeRefExt` の詳細な条件は rustdoc を参照してください。
owner は signal を作らず setup も走らせません。部品が必要とする signal は setup で作り、owner が生きているかは setup で `store` した `Owned<()>` の `try_get` で調べます。App 全体で一つの状態（toaster のようなもの）は `primitive::app_state` が root owner の下で一度だけ作ります。handler や effect から行ごとに `NodeRef` を作るときは `Owner::node_ref` を使います。

## 描画の外で段落を shaping する

`primitive::shape_paragraph(text, &style, max_width)` は setup、handler、effect、value closure で使えます。現在の theme の family と line height が未指定の値を補います。node 固有の継承 style があるときは、その family と line height を明示してください。同じ実効入力は draw・measure と shaping cache を共有します。戻り値は engine の借用を保持しません。

`TextParagraph::next_boundary`・`prev_boundary` は Unicode grapheme と shaping cluster の両方の境界に移ります。byte offset は文字境界へ切り下げ、範囲外はテキストの端へ丸めます。空のテキストでは両方 0、終端での next は終端、先頭での prev は 0 です。IME と accessibility の値はその場で shaping でき、最後の draw の段落を cell に保存する必要はありません。native editor と段落は `kivact-editing` の同じ幾何実装を使います。

native editor のキー編集は従来の Unicode grapheme 単位の規則を保ちます。shaping cluster も越えて移動する `next_boundary`・`prev_boundary` は、段落の API とそれを使う独自 editor の契約です。caret・hit・選択帯の幾何は native editor と共有します。

`IntoValue::into_prop` が作る `Prop` と、保存できる `Reactive` は ownerless な入力の記述です。App のない場所で項目データを作れます。部品の setup で `into_value` にすると、定数はその owner の constant arena、closure は memo に束縛されます。定数の `Value<T>` も `T: Copy` / `Clone` によらず Copy ですが、owner の寿命を延ばしません。定数の読出しは購読や effect を作りません。分岐の `IntoReactive` は Signal・Value・Binding・Animated・closure に限る marker で、raw scalar は受けません。
