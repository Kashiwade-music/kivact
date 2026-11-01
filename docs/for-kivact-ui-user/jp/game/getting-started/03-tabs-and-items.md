# ステップ 3: タブとアイテム

[English](../../../en/game/getting-started/03-tabs-and-items.md)

このステップでは、武器と防具のタブを追加します。各タブにはアイテムの一覧があり、1 つ選んで詳細を読み、**装備**ボタンを押します。合計の威力は、装備しているものに追従します。最後に、これを通しで遊ぶテストを書きます。

## プログラム

`src/main.rs` を、テストを含む完全なプログラムに置き換えます。

```rust,test_harness
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::ui::Tone;
use kivact_ui_game::{self as game, Density, ThemeOptions as _};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Slot {
    Weapons,
    Armor,
}

#[derive(Clone, PartialEq)]
struct Item {
    id: u32,
    slot: Slot,
    name: &'static str,
    power: u32,
    equipped: bool,
}

fn item(id: u32, slot: Slot, name: &'static str, power: u32) -> Item {
    Item { id, slot, name, power, equipped: false }
}

fn loadout() -> impl IntoView {
    let items = signal(vec![
        item(1, Slot::Weapons, "レールライフル", 42),
        item(2, Slot::Weapons, "パルスブレード", 35),
        item(3, Slot::Armor, "プレートベスト", 30),
        item(4, Slot::Armor, "ゴーストクローク", 18),
    ]);
    let slot = signal(Slot::Weapons);
    let selected = signal(None::<u32>);

    // 見ているアイテム。開いているタブのものだけが対象です。
    let chosen = memo(move || {
        let id = selected.get()?;
        items.with(|all| all.iter().find(|i| i.id == id && i.slot == slot.get()).cloned())
    });
    let total = memo(move || {
        items.with(|all| all.iter().filter(|i| i.equipped).map(|i| i.power).sum::<u32>())
    });
    // 威力 10 ごとにゲージのバーが 1 本点灯します。
    let bars = memo(move || (total.get() / 10) as usize);

    let equip = move || {
        if let Some(chosen) = chosen.get() {
            items.update(|all| {
                for other in all.iter_mut().filter(|i| i.slot == chosen.slot) {
                    other.equipped = other.id == chosen.id;
                }
            });
        }
    };

    let panel = move |slot: Slot| {
        let rows = memo(move || {
            items.with(|all| {
                all.iter()
                    .filter(|i| i.slot == slot)
                    .map(|i| {
                        let mark = if i.equipped { " / 装備中" } else { "" };
                        ListItem::new(i.id, i.name)
                            .description(format!("威力 {}{mark}", i.power))
                    })
                    .collect::<Vec<_>>()
            })
        });
        list("アイテム", rows, selected)
    };

    game::hud(
        column((
            game::section_heading("01", "LOADOUT", "Squad 04"),
            game::badge("準備完了", Tone::Success),
            tabs(
                "スロット",
                vec![
                    ListItem::new(Slot::Weapons, "武器"),
                    ListItem::new(Slot::Armor, "防具"),
                ],
                slot,
            )
            .variant(game::Boxed)
            .content(panel),
            game::property_list(vec![
                (
                    "名前".to_owned(),
                    game::mono(move || chosen.get().map_or("-".to_owned(), |i| i.name.to_owned())),
                ),
                (
                    "威力".to_owned(),
                    game::mono(move || chosen.get().map_or("-".to_owned(), |i| i.power.to_string())),
                ),
            ]),
            row((
                button("装備")
                    .variant(game::Slant)
                    .disabled(move || chosen.get().is_none())
                    .on_press(equip),
                game::mono(move || format!("合計威力 {}", total.get())),
                game::gauge("合計威力", bars).total(10).tone(Tone::Info),
            ))
            .style(style().gap(16).align_items(Align::Center)),
        ))
        .style(style().gap(16).padding(20)),
    )
    .style(style().width(420).margin(24))
}

fn main() -> kivact::Result<()> {
    App::new("Loadout")
        .theme(game::theme(ThemeMode::Dark).density(Density::Comfortable))
        .run(loadout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::{RenderOptions, Screen, render_with};

    fn mount() -> Screen {
        let options = RenderOptions {
            theme: game::theme(ThemeMode::Dark).density(Density::Comfortable),
            height: 800.0,
            ..RenderOptions::default()
        };
        render_with(options, loadout)
    }

    #[test]
    fn equipping_a_weapon_adds_its_power() {
        let screen = mount();
        screen.get_by_text("合計威力 0");
        assert!(screen.get_by_role(Role::Button, "装備").is_disabled());
        screen.get_by_role(Role::Option, "レールライフル").click();
        screen.get_by_role(Role::Button, "装備").click();
        screen.get_by_text("合計威力 42");
        let gauge = screen.get_by_role(Role::Meter, "合計威力");
        assert_eq!(gauge.value_range().2, Some(4.0));
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }

    #[test]
    fn a_second_weapon_replaces_the_first_and_armor_adds_up() {
        let screen = mount();
        screen.get_by_role(Role::Option, "レールライフル").click();
        screen.get_by_role(Role::Button, "装備").click();
        screen.get_by_role(Role::Option, "パルスブレード").click();
        screen.get_by_role(Role::Button, "装備").click();
        screen.get_by_text("合計威力 35");

        screen.get_by_role(Role::Tab, "防具").click();
        screen.get_by_role(Role::Option, "プレートベスト").click();
        screen.get_by_role(Role::Button, "装備").click();
        screen.get_by_text("合計威力 65");
    }
}
```

`cargo run` で起動します。「武器」タブが開いていて、アイテムが 2 つあります。**レールライフル**をクリックすると、名前と威力がタブの下の行に表示され、**装備**ボタンが押せるようになります。押すと、アイテムの 2 行目が「威力 42 / 装備中」になり、合計が 42 になり、ゲージの 10 本のうち 4 本が点灯します。**パルスブレード**を装備すると、ライフルと入れ替わります。「防具」タブに切り替えると、専用のアイテムが並んでいます。ベストを装備すると合計が加算されます。一覧の中では矢印キーで移動と選択ができ、タブの行では左右の矢印キーでタブを切り替えられます。

`cargo test` を実行すると、同じ流れをテストが通しで遊びます。

## 仕組み

**アイテムはアプリが持ちます。** `items` は `Signal<Vec<Item>>`、`selected` は `Signal<Option<u32>>` です。一覧とタブはそれらを表示し、ユーザーの選択を渡されたシグナルに書き戻すだけで、コピーは持ちません。アイテムを装備するときは `items` を変更し、そこから導き出されるものがすべて更新されます。

**`list(label, items, selected)`** は標準コンポーネントで、キーボード移動、先頭文字検索、選択を備えた選択肢の一覧です。第 1 引数は支援技術向けに一覧の名前を付けるもので、画面には表示されません。各行は `ListItem::new(key, label)` に任意の `.description(..)` を付けたもので、キー（ここでは `u32`）が `selected` に入ります。行は `memo` から作るので、`items` が変わると一覧が作り直されます。Game テーマは、行を直線的な角と選択の縁取りで描きます。

**`tabs(label, tab_items, selected_key)`** も標準コンポーネントです。選択中のタブは必ずタブのどれかなので、シグナルは `Option` ではなくキーそのもの（`Slot`）を持ちます。`Clone + Eq + Hash` を満たす型ならキーに使え、`enum` は最適な選択です。`.variant(game::Boxed)` は Game キットの箱型の見た目で、指定しないと下線タブになります。`.content(|key| view)` は、タブが切り替わるたびに、開いたタブのパネルを作ります。

**ゲージ**はステップ 2 のものを使い、今度は合計の威力を、10 ポイントにつき 1 本で表します。`bars` は `total` から導き出した `memo` なので、ゲージは装備の変更に追従します。

**`game::property_list(entries)`** は、ラベルと値の行を表示します。各値はビューで、シグナルに追従させたいときは、2 つの `game::mono(move || ..)` のように、ビューの中でシグナルを読みます。

**テスト**は `RenderOptions` にアプリと同じテーマを渡し、ロールと名前で操作します。一覧の行は `Option`、タブは `Tab` です。これらはスクリーンリーダーが読み上げる言葉でもあるので、アプリがその方法で使えることも確かめられます。

## できあがったもの

見出し、メーター、タブ、アイテム一覧、そして合格するテストを備えたゲームのパネルができました。次は作業別ガイドで、テーマ、一覧、メーター、選択のコントロールをさらに詳しく見ていきます。

次へ: [Game 作業別ガイド](../how-to-guide/README.md)
