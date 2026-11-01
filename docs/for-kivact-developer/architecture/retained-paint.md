# 保持した scene の描画契約

`ShapedText` は immutable な `Arc<GlyphRun>` を持ちます。draw command は run と
origin・色 override を渡し、renderer が描画時に配置します。shaping snapshot の構築時
以外に glyph 座標や variation の配列を書き換えません。render は合成座標の overflow
を含め、glyph id・variation・色を毎回検証します。

## lower の所有と再利用

Model の PaintState と engine Offscreen はそれぞれ `RefCell<SceneCache>` を所有し、
短命な Context は resources・raster・cache の3つを借ります。新しい output を作る前に
`begin_frame` を呼び、その frame の output は append-only とします。cache の entry
は128件までの LRU で、Weak scene 同一性と長さ、継承 text 全体、raster bits、Tint を
比較します。成功した origin zero の payload にだけ配置を付けて再利用します。

asset poll の changed/failed、store 交換、font 登録成功は lower-private revision を
進めます。hit でも natural/pixels を再解決し、使用履歴を更新して結果の寸法と Weak
pixel identity を確認します。未 ready・異なる pixel は通常 lowering に戻します。
pending・error・drop を含む結果と、revision を表現できない external frame は保存しません。
直接 Arvane Engine へ font を登録する lower caller は SceneCache を clear します。

配置の1 group が renderer の単一定義 `MAX_SCENE_DEPTH` を超える場合は、cache hit
でも origin 付き lowering に戻します。output cursor は前回以後だけを走査して親 group
の深さを追うため、frame 全体に対して線形です。local trial の診断は fallback 前に戻し、
最終的な error/drop だけを通常の owner が報告します。構造の検証は render に残します。

## memory pressure

MemoryWarning は Model の scene cache、text と未使用 asset cache を解放し、通常の
Window → NativeWindow/Presenter → CPU/GPU renderer 経路で font validation metadata
を解放します。非同期 GPU の受け取り待ちでは clear を予約して受け取り後に適用します。
live import・lease・使用中の resource と登録 font は保ちます。

engine Offscreen の embedder は通常の `release_caches` を明示して scene・text・unused
asset・Rasterizer validation metadata を解放できます。core draw Offscreen の public
API は増やさず、独立出力の facade では従来どおり drop を解放境界とします。

font metadata は成功した face validity/glyph count だけを Weak bytes identity と face
index で256件まで保持します。metadata の hit は run geometry の検証を省略しません。
