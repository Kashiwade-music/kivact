//! Role: Japanese texts of the Pop kit catalog for the keys it reads.
//!
//! The catalog's Japanese texts for the keys `kivact-ui-pop` reads. The kit's
//! own texts are English; the catalog shows how an App replaces them.

use kivact::i18n::{Locale, Messages};

/// The catalog speaks Japanese.
pub fn locale() -> Locale {
    Locale::new("ja-JP")
}

/// Japanese texts for every key of the Pop kit.
pub fn messages() -> Messages {
    [
        ("pop.agenda.details", "予定の詳細"),
        ("calendar.next", "翌月"),
        ("calendar.next_year", "翌年"),
        ("calendar.previous", "前月"),
        ("calendar.previous_year", "前年"),
        ("pop.chip.add", "追加"),
        ("pop.chip.remove", "{label} を削除"),
        (
            "date.error.format",
            "YYYY-MM-DD の形式で入力してください",
        ),
        ("date.error.invalid", "存在しない日付です"),
        ("date.error.order", "終了日が開始日より前です"),
        ("date.open", "日付を選択"),
        ("date.range_open", "期間を選択"),
        ("calendar.range.pick_start", "開始日を選択"),
        ("calendar.range.pick_end", "終了日を選択"),
        ("dialog.close", "閉じる"),
        (
            "pop.dropzone.hint",
            "またはクリックして選択 · PNG、PDF 最大 10MB",
        ),
        ("pop.dropzone.picker", "ファイルを選択"),
        ("spinner.loading", "読み込み中"),
        ("mission.open", "開く"),
        ("mission.progress", "進捗"),
        ("pop.notifications.mark_read", "すべて既読"),
        ("pop.notifications.title", "お知らせ"),
        ("pop.notifications.unread", "未読のお知らせ"),
        ("palette.empty", "見つかりませんでした"),
        ("palette.hint", "↑↓ 移動　 Enter 実行　 Esc 閉じる"),
        ("palette.label", "コマンドパレット"),
        ("palette.placeholder", "コンポーネントやアクションを検索…"),
        ("palette.search", "コマンドを検索"),
        (
            "pop.resize.description",
            "縦にドラッグするか、上下キーで高さを変えます。",
        ),
        ("pop.resize.name", "テキスト欄の高さを変更"),
        ("select.placeholder", "選択してください"),
        ("stepper.decrease", "減らす"),
        ("stepper.increase", "増やす"),
        ("data_table.empty", "該当する生徒が見つかりません"),
        ("data_table.label", "データテーブル"),
        ("pop.table.range", "全 {total} 件中 {from}–{to} 件を表示"),
        ("pop.table.search", "テーブルを検索"),
        ("pop.table.search_placeholder", "生徒・学園で絞り込み…"),
        ("data_table.select_all", "表示中の行を選択"),
        ("data_table.select_named", "{row} を選択"),
        ("pop.table.selected", "{count} 件選択中"),
        ("tags.placeholder", "タグを追加…"),
        ("tags.remove", "{tag} を削除"),
        ("toast.close", "通知を閉じる"),
        ("pop.typing.label", "入力中"),
        ("pop.upload.done", "アップロード完了"),
        ("pop.upload.unknown_size", "サイズ不明"),
        ("pop.upload.uploading", "アップロード中"),
    ]
    .into_iter()
    .fold(Messages::new(), |messages, (key, text)| {
        messages.with(key, text)
    })
}
