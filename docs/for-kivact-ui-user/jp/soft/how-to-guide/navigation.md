# ナビゲーション

[English](../../../en/soft/how-to-guide/navigation.md)

現在のビューはアプリの状態に保持します。関連する少数のビューには、選択キーをセグメントコントロールにバインドし、同じキーで表示内容を切り替えます。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::ListItem;
use kivact::view::dynamic;
use kivact_ui_soft as soft;

fn project_view() -> impl IntoView {
    let view = signal(Some("summary"));
    column((
        radio_group(
            "Project view",
            vec![ListItem::new("summary", "Summary"), ListItem::new("activity", "Activity")],
            view,
        ).variant(kivact::ui::RadioGroupVariant::Segmented),
        dynamic(view, |view| match view {
            Some("activity") => kivact::ui::heading("Activity", 2),
            _ => kivact::ui::heading("Summary", 2),
        }),
    ))
    .style(|| style().gap(kivact::style::use_tokens::<soft::Tokens>().spacing.s4))
}
```

表示ラベルが変わっても選択を維持できるよう、`"summary"` などの安定したキーを使います。大きな画面の中で名前付きパネルを切り替える場合は、標準のタブが適しています。

サイズ変更できるパネルには、サイズ binding と `ScalarRange` を渡した `kivact::ui::splitter` を使います。
折りたたみは `.collapsible(割合)`、スナップ位置は `.snaps(割合の配列)`、上下の配置は `.orientation(..)`、
逆方向や比率のサイズ変更は `.pointer_scale(..)` で指定します。範囲を reactive にし、コンテナの
`NodeRef::rect()` から割合とピクセルの上下限を計算できます。独自のグリップは `.handle(..)` に渡し、
style で境界線、表示、位置、重ね合わせを指定します。同じサイズ binding を使う layout のパネルには
`LayoutRegion::width(..)` を使います。末尾の resize slot は呼び出し側で逆方向を指定します。
スクロールの端は `NodeRef::scroll_state()` で観測し、キー操作は中核の既定動作を使います。

開閉する節には標準の `disclosure(title, open).content(factory)` と `.animated(true)` を使います。Soft テーマは従来の spring transition を設定します。`disclosure_group(signal(None::<K>))` は単一選択、`Vec<K>` の binding は複数選択です。キー付きの節は `.panel(key, title, factory)` で追加します。区切り線には `.variant(soft::DisclosureVariant { dividers: true, ..Default::default() })` を指定します。`chevron` には `ChevronPosition::Start` または `End`、`density` には `Some(DisclosureDensity::Compact)`、`Balanced`、`Spacious` を指定できます。この装飾は単独の節とグループで使え、文字方向に従います。折りたたまない節には `.always_open()`、独自のトリガーには `.content(factory).attach(trigger)` を使います。初期状態は呼び出し側の `signal(default_open)` で指定します。

ツールバーには標準の `toolbar(label).start(...).center(...).end(...)` を使います。表面の装飾は `.variant(soft::ToolbarVariant(soft::SectionVariant::Muted))`、論理辺の区切り線は `.dividers(&[ToolbarEdge::Bottom])` で指定します。初回フォーカス時のヒントには content factory を `soft::with_key_hints(axis, || view)` で囲み、実際のキー操作対象に `soft::keyboard_hint(button)` を付けます。ヒントの方向はリアクティブな orientation に対応し、矢印操作、フォーカスが離れること、または 3 秒で終了します。segmented control は `radio_group(...).variant(RadioGroupVariant::Segmented)` と、必要に応じて `.variant(soft::SegmentedLook { layout: soft::SegmentedLayout::Fill, icon_only: false })` を使います。アイコンのみの節では `SegmentedLook { icon_only: true, ..Default::default() }` と、アイコンだけを返す `.item(...)` を組み合わせます。標準 radio はアクセシブルな名前を保持します。無効な理由は `tooltip(reason).enabled(disabled)` を付けて説明し、無効化には標準の `.disabled(...)` を使います。

segmented のヒントは `with_key_hints` の中で `.item_node(|target, _, _| soft::keyboard_hint(target))` を使い、実際の radio に付けます。内容だけを置き換える `.item` は、アイコンのみの表示にも引き続き使えます。

スクロールする carousel には標準の `carousel(index, slides).name(label).mode(CarouselMode::Scroll)` を使います。index は呼び出し側の `signal(0usize)` が管理します。スナップには `Snap::Mandatory`、自由なスクロールには `Snap::None` を指定します。`gap` と `padding` は論理ピクセル、`buttons`、`edge_fade`、`looping` は標準のオプションです。独自のナビゲーションには headless の `interaction::use_carousel(index, count)` を使います。Soft の recipe は端の丸いコントロール、影、間隔、4 ピクセルの fade を設定します。標準の操作名は Previous slide と Next slide、各グループの名前は Slide N / Count です。

パンくずはキー付き `ListItem` と標準 `breadcrumb(items)` を使用します。既定では末尾が現在ページです。`.current(value)` でキーを指定し、`.current(None::<K>)` で現在ページを解除します。`.on_activate` は現在以外のキーを処理します。`.item` でアイコン、項目固有の操作、標準メニューを合成でき、スロットが操作を持つ場合は全体のコールバックを省略します。`.separator` は区切りを置換します。`soft::BreadcrumbsVariant::Supporting` は補助文字、既定は本文文字を使用し、折り返しと論理方向に従います。

空の状態には標準 `empty_state(title).description(text).icon(view).actions(view)` を使用します。従来の自動変更通知には `.announce_changes()` を追加します。初回描画は読み上げず、実際のライブ領域の祖先では重複通知を抑制します。`.heading_level(level)` は1〜6に制限します。`.variant(soft::EmptyStateVariant::compact(compact))` はリアクティブな密度を受け取り、コンパクト時は操作を縦に並べます。中央揃えの文字と画像スロットも保持します。

カードは標準 `card(content)` を使用し、ボタンには `.name(label).on_press(action)`、チェックボックスには `.name(label).selected(binding)` を指定します。`soft::CardVariant` は Default/Transparent/Muted と全カテゴリ色を保持します。`soft::CardAppearance::new(color, elevation, padding)` はリアクティブな色・高さ表現と論理 `Space` 余白を受け取ります。インセットを利用する Soft 配置の子には `card(soft::with_card_insets(appearance, || content)).variant(appearance)` を合成します。幅と高さは標準 `.style(...)`、固定高さ内のスクロールは内容を標準 `scroll(...).axes(ScrollAxes::Vertical)` で囲みます。header/media/footer は標準スロットで、実際のポインター状態は装飾 overlay レシピに渡されます。

Avatar は標準 `avatar(name)` を使用し、リアクティブな `.src(Some(asset))`、`.fallback_src(Some(asset))`、標準 `AvatarShape`、`.on_press`/`.on_open`、`.status(Tone)` または `.status_content(view)` を指定します。`soft::AvatarSize` は Xsm20/Sm24/Md36/Lg48/Xl128 と段階に合わせる `Px` を保持し、`.variant(size)` または `.variant(soft::AvatarAppearance::new(size))` を使用します。Soft theme は既定で avatar の名前を tooltip に表示します（従来の Name）。`.name_tooltip(false)` で表示を止められ（従来の Off）、従来の Text は `.name_tooltip(false)` と標準 `tooltip(text).attach(avatar(...))` で独自の文字を表します。`.name(alt)` で代替文字を上書きします。独自の状態ラベルはリアクティブな `.name(...)` に合成でき、状態の内容には実際のラベルを付けます。ラベルのレジストリは不要です。`avatar_status_dot(...).avatar_size(size)` は独自の色・アイコンと直径に応じた寸法を保持します。匿名の場合は `.fallback_content(soft::avatar_silhouette)` を指定します。グループは標準 `avatar_group(children).name(label).total(total)` を使用します。各子にサイズと形状を指定し、共有枠は `.variant(soft::AvatarGroupAppearance::new(size, shape))`、サイズ変更時の重なりは `.overlap(move || side * 0.25)` で指定します。`.overflow_content(|count| button(...))` は実際の省略数から独自の文字と操作を構成します。キーボード移動は標準グループが所有します。

工程には標準 `stepper(vec![StepperItem::new(label)], current)` を使用します。項目の description/status/disabled/optional/indicator/content/end_content と、グループの orientation/on_step_press/min_step_width は標準 API です。リアクティブな状態は項目ベクトルを派生させて表します。`StepIndicator::custom(factory)` は独自のグリフを保持します。`soft::StepperAppearance` はリアクティブな密度、インジケーター位置、コンパクト表示、接続線の間隔、インデックスごとの密度を含みます。実際の leading/trailing/progress パーツを標準の index/count/orientation で描画し、最初・最後の線と到達済み区間を区別します。疎な従来のインデックスはベクトル位置に対応させ、独自の数字インジケーターと操作時の検索で保持します。WithLabelAndControls/WithLabel/HiddenLabel は従来の表示を保持します。標準 `CollapsedSteps::{Current,All}` は構築時に指定する別の構造方針です。コンパクト表示の前後移動では無効な工程を飛ばします。

Banner は標準 `alert(title).tone(Tone::Info).icon(soft::banner_icon(Tone::Info))` を使用します。Warning/Success はそのまま、Error は `Tone::Danger` に対応します。末尾の内容は `.actions(...)`、詳細は `.content(factory).collapsible(true).open(binding)` に置換します。従来の Banner の既定を保持するには collapsible を明示します。標準 `.dismissible(...)`、`.dismiss_label(...)`、`.on_dismiss(...)` は閉じる操作と外部フォーカス復帰を保持します。見出しを含む閉じる名前は `i18n::message_with` で翻訳して渡せます。`soft::BannerContainer::{Card,Section}` とリアクティブな `BannerAppearance::new(container,elevation)` は外観を保持します。テーマは実際の header/message/controls/body の枠を描画し、開閉対象には標準 tooltip が付きます。

ページ移動はゼロ始まりの標準 `pagination(count, page)` を使用します。従来の1始まり `u32` は `usize` のインデックス状態に置換するか、標準 `binding` の getter/setter で同じ呼び出し元の値を変換します。変更通知もキットのコールバックではなく受け入れられた書き込みで行います。`.window(PaginationWindow::Buffered)` は端を埋める従来のウィンドウと境界・区切りの4ケースを保持します。Pages/Count/Compact/Dots/Input は `PaginationMode` に対応します。従来の None（中央表示なし）は Count と非表示の `pagination_status` 要約で表し、標準 None は操作自体を消します。`PaginationAppearance::new(size)` はリアクティブな寸法を保持します。first_last/step/siblings/disabled/page_label/has_more/page_size/options は標準設定です。項目数から計算する場合、従来の暗黙値を保持するにはページサイズ10を明示します。従来の丁寧なページ通知には `.summary(soft::pagination_status)` を追加します。Count は `.summary(|facts| (soft::pagination_count(facts), soft::pagination_status(facts)))`、Compact は `pagination_page_summary` を使用します。これらは実際の要約元を読み、ページのモデルを所有しません。Input は要約スロットに独自の接頭辞・総数文字を表示でき、実際の編集欄は標準の下書きと名前を保持します。Dots は全ページを表示し、どちらの読み方向でも実際のフォーカスと選択を循環させます。

Soft の top/side/mobile navigation はドメイン配置として保持します。デスクトップのトップメニュー、ワークスペース見出しメニュー、メガメニュー、縮小レールのフライアウトは、ナビゲーションのルートごとに一つの `interaction::use_navigation_menu` を共有します。実際のパネル識別子は構築時の `panel_key` から取得し、ホバーと固定表示は `NavOpen` の状態です。Popup/Dialog/Menu は同じ受け入れ済み binding を参照します。ポインターの遅延はそのコントローラーが所有し、ホバー表示の複製は持ちません。メニューは標準のキーボード移動・文字検索を保持し、トリガー操作でホバー表示を固定します。レールの適用条件の変更は実際の終了イベントを送り、展開後の再縮小で古いパネルが復活しません。標準 `navigation_menu` と `navigation_menu_link` にも Neutral の全レシピを適用します。
