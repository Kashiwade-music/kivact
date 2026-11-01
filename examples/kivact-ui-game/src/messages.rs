//! Role: Japanese localization messages for the game catalog.
//!
//! The catalog's Japanese texts for the keys `kivact-ui-game` reads. The kit's
//! own texts are English; the catalog shows how an App replaces them.

use kivact::i18n::{Locale, Messages};

/// The catalog speaks Japanese.
pub fn locale() -> Locale {
    Locale::new("ja-JP")
}

/// Japanese texts for every key of the Game kit.
pub fn messages() -> Messages {
    [
        ("game.alert.close", "アラートを閉じる"),
        ("dialog.close", "ダイアログを閉じる"),
        (
            "game.dropzone.hint",
            "または クリックして選択 · CSV / JSON · 最大 20MB",
        ),
        ("game.dropzone.label", "ファイルをアップロード"),
        ("game.dropzone.picker", "ファイルを選択"),
        ("game.pagination.next", "次へ"),
        ("game.pagination.page", "ページ"),
        ("game.pagination.previous", "前へ"),
        ("palette.empty", "一致するコマンドはありません"),
        ("palette.hint", "↑ ↓ 移動     Enter 実行     Esc 閉じる"),
        ("palette.label", "コマンドパレット"),
        ("palette.placeholder", "コマンド・操作を検索..."),
        ("palette.search", "コマンド検索"),
        ("game.skeleton.label", "読み込み中"),
        ("switch.off", "OFF"),
        ("switch.on", "ON"),
        ("data_table.actions", "操作"),
        ("game.table.bulk", "選択中のレコード"),
        ("game.table.clear", "選択解除"),
        ("data_table.empty", "該当するレコードはありません"),
        ("game.table.progress", "進捗"),
        ("data_table.select_all", "表示行をすべて選択"),
        ("data_table.select_named", "{row} を選択"),
        ("game.table.selected", "{count} 件選択中"),
        ("game.table.shown", "{shown} 件表示 / 全 {total} 件"),
        ("tags.placeholder", "タグを追加..."),
        ("tags.remove", "{tag} を削除"),
        ("toast.close", "通知を閉じる"),
        ("game.window.branch", "main"),
        ("game.window.edit", "編集"),
        ("game.window.explorer", "EXPLORER"),
        ("game.window.explorer_width", "エクスプローラー幅"),
        ("game.window.inspector", "INSPECTOR"),
        ("game.window.menu.file", "ファイル"),
        ("game.window.menu.edit", "編集メニュー"),
        ("game.window.menu.view", "表示"),
        ("game.window.menu.tools", "ツール"),
        ("game.window.menu.help", "ヘルプ"),
        ("game.window.new", "新規"),
        ("game.window.pause", "一時停止"),
        ("game.window.preview", "プレビュー"),
        ("game.window.run", "実行"),
        ("game.window.save", "保存"),
        (
            "game.window.status",
            "UTF-8     LF     Markdown     Ln 12, Col 4",
        ),
    ]
    .into_iter()
    .fold(Messages::new(), |messages, (key, text)| {
        messages.with(key, text)
    })
}
