# AC-3 の人間の確認（V2）

作成：W8（2026-11-11）。V2 作業指示書 12.1 の (b)。**確認の欄は Orchestrator が書いていない（DEC-16）。人間が書く。**

## 機械の検査（(a)）

- `cargo xtask check --no-write --only state-fields --report-audit`：exit 0。`dev/xtask/state-audit.md` の OPEN は 0 件（CP2 11.2 で確認し、commit `1f000b93`）。
- V2 の前（`fac80712`）との比較：監査の行は 1239 → 1150。OPEN は 27 → 0。
- 閉じ方ごとの件数（V2 の後 / 前）：
  - option：617 / 558
  - independent：317 / 342
  - cache：110 / 117
  - projection：17 / 101
  - type：38 / 40
  - identity：31 / 32
  - statistic：20 / 22
  - OPEN：0 / 27
- 監査の行の差分：追加 686 行、削除 775 行。

## 全ての projection の行（17 件）

書き手（`written only by …`）を名指す。下の表は `dev/xtask/state-audit.md` から機械的に抜き出した。

| file:line | field | 書き手の記述 |
|---|---|---|
| crates/kivact-core/src/gpu_interop/share/app_side.rs:32 | SharedImage.current | projection: the import of the link's current request, written only by SharedImage::replace_current |
| crates/kivact-core/src/view/semantics_data.rs:25 | SemanticsPatch.reactive | projection: written only by Common::semantic from whether the patch's source is fixed; the source itself is erased into `apply` |
| crates/kivact-engine/src/host/pump.rs:103 | Host.app_motion | projection: the App motion last reported to the client, written only by Host::report_app_motion |
| crates/kivact-engine/src/host/window.rs:58 | HostWindow.environment | projection: the environment last reported to the client, written only by Host::record_environment |
| crates/kivact-engine/src/host/window.rs:64 | HostWindow.soft_keyboard | projection: the software keyboard state last sent to the platform, written only by Host::synchronize_ime |
| crates/kivact-engine/src/host/window.rs:68 | HostWindow.ime_allowed | projection: the IME permission last sent to the platform, written only by Host::synchronize_ime |
| crates/kivact-engine/src/host/window.rs:70 | HostWindow.reported_adapter | projection: the adapter last reported to the client, written only by Host::adapter_changed |
| crates/kivact-engine/src/measure/editor.rs:25 | EditorNode.reported_session | projection: the session last sent to the selection listener, written only by EditorNode::report_session |
| crates/kivact-engine/src/model/aggregate.rs:45 | Model.modifiers | projection: the modifiers last reported to the client, written only by Model::report_modifiers |
| crates/kivact-engine/src/model/pointer/state.rs:18 | Pointer.hover_sent | projection: the hover last reported to the client, written only by Model::report_hover |
| crates/kivact-engine/src/model/pointer/state.rs:20 | Pointer.position_sent | projection: the mouse position last reported to the client, written only by Model::report_mouse_position |
| crates/kivact-interaction/src/select/machine.rs:24 | State::Open.previewed | projection: the option last reported to the preview callback, written only by Machine::transition on Event::Preview, which only track_preview sends (the hovered key, else the list's active key); Open starts it at None and Close reports None |
| crates/kivact-runtime/src/task/action.rs:250 | ActionCell.last | projection: written only by ActionCell::begin_run (the input of the latest started run; None before the first run) |
| crates/kivact-runtime/src/task/async_state.rs:67 | AsyncCell.latest | projection: written only by AsyncCell::publish (Ready(v) implies latest == Some(v)) |
| crates/kivact-test/src/facts.rs:10 | Node.node_type | projection: written only by Facts::new, which copies the engine reader's node type for each id |
| crates/kivact-test/src/facts.rs:13 | Node.parent | projection: written only by Facts::new, which copies the engine reader's parent for each id |
| crates/kivact-ui-basic/src/sidebar/menu.rs:189 | ButtonCfg.has_action | projection: whether the button has an action view, written only by build_button |

## 全ての independent の行（317 件）

監査の行（file:line、field、理由）をそのまま載せる。理由は先頭 240 字で切る（全文は `dev/xtask/state-audit.md`）。

| file:line | field | 理由（先頭） |
|---|---|---|
| crates/arvane/src/presentation.rs:64 | RendererChange::Gpu.interop | independent: the adapter when the device can import shared textures (feature gpu-interop and an adapter that supports it), absent otherwise; it depends on the device alone, while configure depends on where the swapchain was configured, so a |
| crates/arvane-native/src/mirror.rs:120 | MirrorBatch.accessibility | independent: set only by the accessibility tree's change callback, while frame, title, resize and gpu come from other MirrorWindow calls on the test thread; flush applies each on its own, so any mix in one batch is valid |
| crates/arvane-native/src/mirror.rs:127 | MirrorBatch.gpu | independent: set only by `MirrorWindow::gpu_presents`, while the other batch values come from other MirrorWindow calls; flush switches the window to GPU use on its own (a frame in the same batch is then not kept), so any mix is valid |
| crates/arvane-native/src/module.rs:22 | Retained.reported | independent: set by the first `retain_module` call before it logs, whatever the loader answered; a reentrant call from the log subscriber sees it set and returns, so both answers occur with it unset and set |
| crates/arvane-native/src/native_input/state.rs:16 | NativeInput.text | independent: written only by the IME port's `surrounding` when the application focuses or edits a text input, while reduced_motion and locale come from the system's settings; neither source reads the other, so every pair occurs |
| crates/arvane-native/src/native_input/state.rs:64 | ContactNote.canceled | independent: the platform's CANCELED flag for the contact's pointer message; tilt comes from the pen info of the same message, and a touch (no tilt) or a pen (with or without tilt) can each be canceled, so every pair occurs |
| crates/arvane-native/src/port/winit/runtime.rs:124 | Runner.wake_reasons | independent: bits OR-ed in by the ShellWaker on any thread and swapped to zero by `user_event`; wakes arrive in every LoopPhase (before the main window, while running, after a stop) and are drained in each, so every pair with phase occurs |
| crates/arvane-native/src/port/winit/windows.rs:25 | WindowTable.pending_show | independent: filled when the handler opens a window during a callback and emptied by the runner once it returns; placement is fixed when the table is made, so windows wait to be shown on platforms with and without placement alike |
| crates/arvane-native/src/port/x11/dnd.rs:49 | Drag::Hovering.accepts | independent: decided once at XdndEnter from the protocol version and the offered types; point is filled by every XdndPosition whether or not we accept (a refusing status is still sent), so all four pairs occur |
| crates/arvane-native/src/port/x11/dnd.rs:51 | Drag::Hovering.point | independent: the last XdndPosition, absent until the first one arrives (no status sent yet); it is filled whatever accepts says, and drop takes the URIs only for accepts with a point, so all four pairs occur |
| crates/arvane-shell/src/clicks.rs:41 | ClickCounter.last | independent: written only by a mouse `Down`, while tap is written only by a touch or pen `Down`; a window may see either device first or only one of them, so each of None and Some occurs with any value of the other |
| crates/arvane-shell/src/clicks.rs:47 | ClickCounter.tap | independent: written only by a touch or pen `Down`, while last is written only by a mouse `Down`; a window may see either device first or only one of them, so each of None and Some occurs with any value of the other |
| crates/arvane-shell/src/window.rs:275 | StepOutcome::Idle.next_deadline | independent: the loop control's earliest timer deadline when the turn ends; timers come from the handler, apart from window redraw flags and wakes, so it is present or absent with any mix of them |
| crates/arvane-shell/src/window.rs:277 | StepOutcome::Idle.redraw_requested | independent: whether any window's redraw flag is still set when the turn ends; windows set it from their own requests, apart from the timer deadline and pending wakes, so a turn may end with any mix |
| crates/arvane-shell/src/window.rs:280 | StepOutcome::Idle.woken | independent: whether a wake is still pending when the turn ends (`WakePending::pending`); wakers set it from any thread, apart from the timer deadline and redraw flags, so a turn may end with any mix |
| crates/arvane-virtual/src/window.rs:147 | VirtualWindow.redraw | independent: the script's redraw request is recorded separately; every combination is a valid script state |
| crates/arvane-virtual/src/window.rs:151 | VirtualWindow.min_size | independent: the size the application sets is recorded separately; every combination is a valid script state |
| crates/arvane-virtual/src/window.rs:156 | VirtualWindow.ime | independent: the script's IME caret area is recorded separately; every combination is a valid script state |
| crates/arvane-virtual/src/window.rs:159 | VirtualWindow.ime_text | independent: the script's surrounding text is recorded separately; every combination is a valid script state |
| crates/arvane-virtual/src/window.rs:164 | VirtualWindow.soft_keyboard_shown | independent: the script's soft keyboard request is recorded separately; every combination is a valid script state |
| crates/arvane-virtual/src/window.rs:172 | VirtualWindow.gpu | independent: the renderer the script requires is recorded separately; every combination is a valid script state |
| crates/arvane-virtual/src/window.rs:177 | VirtualWindow.occluded | independent: the script's occlusion is recorded separately; every combination is a valid script state |
| crates/kivact-core/src/custom/measurement.rs:24 | Constraints.known_width | independent: the width the layout already fixed; either dimension may be fixed on its own |
| crates/kivact-core/src/custom/measurement.rs:27 | Constraints.known_height | independent: the height the layout already fixed; either dimension may be fixed on its own |
| crates/kivact-core/src/gpu_interop/import/image.rs:32 | ExternalImage.latest | independent: set only by the import's task when the engine calls FramePublisher::publish; the window sets the status from the import result, and a frame kept after Lost or published before Ready simply draws nothing, so every pair is valid |
| crates/kivact-core/src/gpu_interop/share/app_side.rs:35 | SharedImage.pixels | independent: the last pixels delivered, kept for every phase |
| crates/kivact-core/src/view/devtools_state.rs:96 | PropWrite.changed | independent: whether this one write changed the node's value (the engine's comparison); a write from an effect and a direct write can each change the value or leave it equal, and DevTools shows all four |
| crates/kivact-core/src/view/drawable_share.rs:128 | ExportRequest::Pixels.latest | independent: set only by ExportOpen::on_frame with a frame of this request's generation; None until the first such frame, and any generation may still lack one |
| crates/kivact-core/src/view/drawable_share.rs:151 | ExportRequest::GpuProvided.latest | independent: set only by ExportOpen::on_frame with a GPU frame of this request's generation; None until the first such frame, and any generation may still lack one |
| crates/kivact-core/src/view/part_data.rs:66 | Availability::Enabled.pressed | independent: a held activation key presses the target while the pointer is elsewhere, and the pointer rests over it unpressed, so every pair occurs |
| crates/kivact-core/src/view/part_data.rs:69 | Availability::Enabled.hovered | independent: a held activation key presses the target while the pointer is elsewhere, and the pointer rests over it unpressed, so every pair occurs |
| crates/kivact-core/src/view/state.rs:51 | FocusRequest.visible | independent: given once by the caller of request_focus and never changed; the stage moves from Queued to Sent on the host write whatever the indicator, so both values occur in both stages |
| crates/kivact-core/src/view/state.rs:198 | Requests.pending_activations | independent: pushed by request_activation (NodeRef::activate) and taken whole by the client after each event; the id counter grows only on host requests, so any queue length goes with any id |
| crates/kivact-core/src/view/state.rs:239 | ViewState.initial_render | independent: one-way latch: true until the first frame is drawn (launch::runtime sets it false once) |
| crates/kivact-core/src/view/tree.rs:273 | NodeObservation.measured | independent: set from each layout report, None while the report carries no measured layout box; text_truncated is set from the same report on its own, so every pair occurs |
| crates/kivact-core/src/view/tree.rs:275 | NodeObservation.text_truncated | independent: set from every layout report, including before any measurement is published |
| crates/kivact-core/src/view/tree.rs:313 | CoreNode.handlers | independent: the declared handlers recorded when the node is created, or an empty record ViewNodes::handlers_mut and ViewNodes::observe_node create for a later observer; a node with or without a location may have either |
| crates/kivact-core/src/view/tree.rs:319 | CoreNode.rare | independent: created by the first rare resource the node gets (a reference, an exit callback, an observation, a presence source, semantic origins or DevTools style layers) and kept until removal; a node with or without a location or handler |
| crates/kivact-core/src/view/tree.rs:335 | CoreRare.observation | independent: created by ViewNodes::observe_node when a reference first needs the node's geometry, text truncation, scroll or image status; the presence source beside it is created separately by ViewNodes::presence, so either exists without  |
| crates/kivact-core/src/view/tree.rs:338 | CoreRare.presence | independent: created by ViewNodes::presence when a reference first observes the node's lifetime; the observation beside it is created separately, so either exists without the other |
| crates/kivact-devtools/src/panels/elements/view.rs:22 | view::owners_on | independent: set only by the Owners checkbox; it swaps the element tree for the owner tree while the details pane and its layout overlay checkbox stay shown, so it pairs with either overlay value |
| crates/kivact-devtools/src/panels/elements/view.rs:24 | view::layout | independent: set only through layout_toggle by the details pane's layout overlay checkbox, which is shown under either tree, so it pairs with either value of owners_on |
| crates/kivact-devtools/src/session.rs:33 | new::status | independent: becomes Session.status, empty at setup; the target var beside it is fixed for the session whether or not a failure is shown |
| crates/kivact-devtools/src/state.rs:75 | Session.status | independent: the status bar's last RPC failure, set by Session::send when a call fails and by Session::report; it stays while connection moves to Lost, and a connected session may have none |
| crates/kivact-editing/src/behavior.rs:50 | EditContext.layout | independent: the editor's laid-out lines for this request, which the engine always supplies and a caller without layout leaves out; the flags are authored props and do not decide whether lines exist |
| crates/kivact-editing/src/edit_state.rs:54 | EditState.log | independent: the edits kivact has not taken yet (take_changes, start_log, revert clear it); the line-cache damage is cleared only by take_damage, and set_controlled leaves damage without a log, so all four combinations occur |
| crates/kivact-editing/src/edit_state.rs:58 | EditState.damage | independent: line-cache damage is taken by the editor, separately from the edit log |
| crates/kivact-engine/src/check/boxes.rs:70 | Axes.x | independent: Axes::held_by clears x when the node scrolls horizontally or is a text input, y when it scrolls vertically; overflow-x and overflow-y are authored apart, so all four pairs occur |
| crates/kivact-engine/src/check/boxes.rs:72 | Axes.y | independent: Axes::held_by clears y when the node scrolls vertically, x for horizontal scrolling or a text input; overflow-x and overflow-y are authored apart, so all four pairs occur |
| crates/kivact-engine/src/host/window.rs:74 | HostWindow.last_stats | independent: the stats of the last shown frame stay readable while later presents are skipped |
| crates/kivact-engine/src/lower/assets.rs:64 | ArvaneAssets.pending | independent: missing-family reports check_family queues during a layout pass and sync takes each commit; the font and store generations version other data and never gate the queue |
| crates/kivact-engine/src/lower/context.rs:58 | Context.dropped | independent: drop_command counts each undrawable command; succeeded_since compares it to a checkpoint to keep a failed scene out of the cache and offscreen fails on it; canvas hosting or not, any count occurs |
| crates/kivact-engine/src/model/aggregate.rs:58 | Model.window_request | independent: the title and minimum size Writer::set_window queues and finish_window takes after the turn; the modifiers projection follows shell key input, so either is set without the other |
| crates/kivact-engine/src/model/changes.rs:137 | Changes.paint | independent: raised by Model::invalidate_paint (host environment and asset events) and by image and layout-motion steps; restack and fonts are raised by other inputs (paint-order writes, a font registration), so any set can be pending at a  |
| crates/kivact-engine/src/model/changes.rs:139 | Changes.restack | independent: raised by writes and motions that change paint order (Model::apply, state and layout motion, presence); paint and fonts are raised by other inputs (host repaints, a font registration), so any set can be pending at a sync |
| crates/kivact-engine/src/model/changes.rs:141 | Changes.fonts | independent: raised only by Model::fonts_added when the app registers a font; paint and restack are raised by other inputs (host repaints, paint-order writes), so any set can be pending at a sync |
| crates/kivact-engine/src/model/changes.rs:149 | Dispatched.paint | independent: Changes::dispatch takes Changes::paint as raised by its own inputs (host repaints, images, layout motion); a font registration no longer raises it, so it pairs with any fonts and restack |
| crates/kivact-engine/src/model/changes.rs:151 | Dispatched.restack | independent: Changes::dispatch takes Changes::restack (paint-order writes and motions) as is; paint and fonts come from other inputs, so any set occurs |
| crates/kivact-engine/src/model/changes.rs:153 | Dispatched.fonts | independent: Changes::dispatch takes Changes::fonts (Model::fonts_added) as is; sync no longer raises paint with it, so it pairs with any paint and restack |
| crates/kivact-engine/src/model/drawables/place.rs:65 | LayerFrame.in_3d | independent: push_layer reads it from the nearest open layer's preserves (inside_3d); preserves is this layer's own transform_style, so a flat or preserving layer sits in a flat or 3D parent |
| crates/kivact-engine/src/model/drawables/place.rs:70 | LayerFrame.preserves | independent: push_layer takes it from this layer's transform_style; in_3d is the parent layer's, so a flat or preserving layer sits in a flat or 3D parent |
| crates/kivact-engine/src/model/pointer/state.rs:16 | Pointer.file_target | independent: the file-listening node under an OS file drag, set by the file drag events and cleared at Drop, Leave or its removal; the hover and position projections follow mouse events, so any pair occurs |
| crates/kivact-engine/src/model/pointer/state.rs:187 | Route.capture | independent: where the gesture's listener events go: set by any button's Down over a listener or by set_pointer_capture, cleared when no button is held; a secondary-button drag has it without a press |
| crates/kivact-engine/src/model/pointer/state.rs:189 | Route.press | independent: the primary button's pressed chain and activation target, set only by primary_press and taken by that button's Up; a press over a node without a pointer listener has no capture, and a listener may release the capture mid-press |
| crates/kivact-engine/src/model/pointer/state.rs:440 | PointerSource.pressure | independent: the pressure of the pointer's last shell sample (shell.rs); platforms report pressure and tilt apart (a touch may have pressure and no tilt, a mouse neither), so every pair occurs |
| crates/kivact-engine/src/model/pointer/state.rs:443 | PointerSource.tilt | independent: the pen tilt of the pointer's last shell sample (shell.rs); platforms report tilt and pressure apart (a pen may lack either, a mouse has neither), so every pair occurs |
| crates/kivact-engine/src/mutation.rs:98 | Mutation.style | independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is |
| crates/kivact-engine/src/mutation.rs:101 | Mutation.measure | independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is |
| crates/kivact-engine/src/mutation.rs:104 | Mutation.text | independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is |
| crates/kivact-engine/src/mutation.rs:108 | Mutation.placed | independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is |
| crates/kivact-engine/src/mutation.rs:111 | Mutation.restack | independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is |
| crates/kivact-engine/src/mutation.rs:114 | Mutation.paint | independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is |
| crates/kivact-engine/src/mutation.rs:117 | Mutation.states | independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is |
| crates/kivact-engine/src/mutation.rs:120 | Mutation.read_only | independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is |
| crates/kivact-engine/src/mutation.rs:123 | Mutation.subtree | independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is |
| crates/kivact-engine/src/mutation.rs:126 | Mutation.payload_only | independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is |
| crates/kivact-engine/src/paint/state.rs:250 | Step.own | independent: whether the node draws itself; any grouping can hold either |
| crates/kivact-engine/src/text_input/source.rs:13 | Source.damage | independent: the damage refresh_lines takes from the visible lines (none for a width probe or before any edit); password is the node's purpose at this refresh, so edits of a masked or plain input give every pair |
| crates/kivact-engine/src/text_input/source.rs:18 | Source.password | independent: the node's password purpose (HostNode::password) at this refresh; damage comes from edits of any input, masked or not, so both pairs occur |
| crates/kivact-engine/src/tree/child_order.rs:9 | Link.previous | independent: the neighbor links of one child; each side is absent at an end of the order |
| crates/kivact-engine/src/tree/child_order.rs:11 | Link.next | independent: the neighbor links of one child; each side is absent at an end of the order |
| crates/kivact-engine/src/tree/node_tree.rs:25 | NodeTree.pending_child_order | independent: parents whose child order insert or detach edited, drained by pack_child_orders before layout; detached subtrees edit orders too, so pending work occurs with or without a root |
| crates/kivact-engine/src/tree/node_tree.rs:29 | NodeTree.root | independent: the Root-kind node, set when the client installs it and cleared when it is removed; detached subtrees queue child-order packing too, so pending work occurs with or without a root |
| crates/kivact-i18n/src/numbers.rs:152 | SymbolCapture.symbols | independent: the decimal symbols ICU loads through this provider while it builds the formatter; ICU loads the digits through the other slot in its own order, and NumberSymbols::new reads both once after the build |
| crates/kivact-i18n/src/numbers.rs:154 | SymbolCapture.digits | independent: the decimal digits ICU loads through this provider while it builds the formatter; ICU loads the symbols through the other slot in its own order, and NumberSymbols::new reads both once after the build |
| crates/kivact-interaction/src/combobox/handle.rs:114 | ComboboxApi.input_state | independent: the observation of the attached input part, absent until it attaches |
| crates/kivact-interaction/src/combobox/handle.rs:116 | ComboboxApi.content_state | independent: set by ComboboxApi::content when the list surface attaches; the input part attaches separately through ComboboxApi::input, before the surface first mounts |
| crates/kivact-interaction/src/disclosure.rs:55 | DisclosureApi.trigger_state | independent: set by DisclosureApi::trigger when the kit attaches a trigger; the content part attaches through its own call, so a disclosure may have either part without the other |
| crates/kivact-interaction/src/disclosure.rs:57 | DisclosureApi.content_state | independent: set by DisclosureApi::content while the content part is mounted and emptied when that mount ends; a closed disclosure has its trigger and no content |
| crates/kivact-interaction/src/drag/handle.rs:112 | Step::Stop.owns_edit | independent: copied from the ended drag's EditOwnership (Owned when the drag began the edit); a drag with or without an edit can end with or without an update |
| crates/kivact-interaction/src/drag/handle.rs:114 | Step::Stop.update | independent: what on_end receives, absent when the delta from the drag's start is not finite; that depends on the positions alone, so it comes with either edit ownership |
| crates/kivact-interaction/src/drag/handle.rs:147 | DragApi.observed | independent: the observation of the attached source part, absent until it attaches |
| crates/kivact-interaction/src/list/data.rs:306 | ListItemState.selected | independent: whether the selection holds the row; a multiple selection or a caller's binding write keeps rows selected while another row is active |
| crates/kivact-interaction/src/list/data.rs:308 | ListItemState.active | independent: whether the row is the roving or highlighted candidate; arrows under Manual activation, a combobox highlight and Ctrl+arrow move it without selecting |
| crates/kivact-interaction/src/list/state.rs:39 | State.container_state | independent: the observation of the attached container part, absent until it attaches |
| crates/kivact-interaction/src/menu/level.rs:26 | Level.content_state | independent: set by MenuApi::content when this level's surface mounts; a submenu level exists in its session before its surface first mounts |
| crates/kivact-interaction/src/menubar.rs:97 | use_menubar::focus | independent: the trigger list's selection, written by use_list as the arrows and activation move along the bar; menus is fixed at setup, so any index or none pairs with it |
| crates/kivact-interaction/src/popup/api.rs:104 | PopupApi.content_state | independent: set by PopupApi::content_with while the surface is mounted and emptied when it unmounts; the trigger has attached by then or is absent |
| crates/kivact-interaction/src/popup/api.rs:106 | PopupApi.trigger_state | independent: set by PopupApi::base_trigger at the trigger's mount; while the popup is closed there is a trigger and no content |
| crates/kivact-interaction/src/scalar/edit_state.rs:148 | Edit.observed | independent: the state of the part the kit attached, absent until input() or thumb() attaches it; single and range thumbs (bounds None or Some) exist both before and after attaching |
| crates/kivact-interaction/src/select/handle.rs:132 | SelectApi.content_state | independent: set by SelectApi::content_in when the option list or sheet mounts; the preview callback beside it is set by the caller at setup, before or without any surface |
| crates/kivact-interaction/src/select/machine.rs:20 | State::Open.highlight | independent: the key the keyboard highlights in the open list |
| crates/kivact-interaction/src/select/machine.rs:22 | State::Open.hovered | independent: the key the pointer is over in the open list |
| crates/kivact-interaction/src/space_pan/handle.rs:19 | SpacePanApi.observed | independent: the observation of the attached surface part, absent until it attaches |
| crates/kivact-interaction/src/splitter/handle.rs:117 | SplitterApi.observed | independent: the observation of the attached handle part, absent until it attaches |
| crates/kivact-interaction/src/table/data.rs:110 | TableRowState.selected | independent: whether the table selection holds the row; Space, a pointer press and the caller's binding change it, and a multiple selection keeps rows selected while the tab stop is elsewhere |
| crates/kivact-interaction/src/table/data.rs:112 | TableRowState.active | independent: whether the row holds the tab stop (itself under TableFocus::Rows, one of its cells under TableFocus::Cells); arrows between cells and under Manual activation move it without changing the selection |
| crates/kivact-interaction/src/table/data.rs:122 | TableCellState.selected | independent: copied from the cell's row selection; Space selects the row of the focused cell, and arrows leave the selection where it is |
| crates/kivact-interaction/src/table/data.rs:125 | TableCellState.active | independent: whether the cell holds the tab stop, only under TableFocus::Cells; arrows move it between cells of selected and unselected rows alike |
| crates/kivact-interaction/src/text_edit/handle.rs:500 | Data.observed | independent: the observation of the attached control part, absent until it attaches |
| crates/kivact-interaction/src/toaster/column.rs:42 | Effect::Dismiss.previous | independent: the return target when the focus was inside the dismissed toast; a toast with or without an on_dismiss handler can hold the focus, so it pairs with either callback |
| crates/kivact-interaction/src/toggle.rs:33 | ButtonApi.observed | independent: the observation of the attached part, absent until it attaches |
| crates/kivact-interaction/src/tooltip/api.rs:131 | TooltipApi.trigger_state | independent: set by TooltipApi::trigger at mount; the hint content is mounted only while the tooltip shows |
| crates/kivact-interaction/src/tooltip/api.rs:133 | TooltipApi.content_state | independent: set by TooltipApi::content when the hint mounts after the show delay, after the trigger attached |
| crates/kivact-interaction/src/tree.rs:332 | TreeItemState.selected | independent: copied from the row's ListItemState; a multiple selection or a caller's binding write keeps rows selected while another row is active |
| crates/kivact-interaction/src/tree.rs:334 | TreeItemState.active | independent: copied from the row's ListItemState; Ctrl+arrow moves the active row without selecting it |
| crates/kivact-interaction/src/zoom_pan/handle.rs:22 | ZoomPanApi.observed | independent: the observation of the attached viewport part, absent until it attaches |
| crates/kivact-runtime/src/motion/state.rs:110 | AnimInfo.decorative | independent: a fixed property of the animation; any value is valid beside its status |
| crates/kivact-runtime/src/reactive/data.rs:76 | Runtime.ext | independent: ADR-17: filled by the constructor before it returns; never observed empty |
| crates/kivact-runtime/src/reactive/data.rs:89 | Runtime.clock | independent: the App's monotonic time, written only by advance_clock from each turn's host time; timers and gestures read it, and every reading is valid in any turn or exit state |
| crates/kivact-runtime/src/reactive/data.rs:333 | OwnerEntry.parent | independent: the creating owner, set by create_owner and cleared only by detach_owner when this owner's disposal starts; the root is live without one, a detached disposal root is disposing without one, and children keep theirs |
| crates/kivact-runtime/src/reactive/data.rs:338 | OwnerEntry.extra | independent: the box of the rare members, created by the first object, constant, cleanup, context or error handler registered on the owner and kept until the entry is removed; either phase holds it or not (disposal empties it, never drops i |
| crates/kivact-runtime/src/reactive/data.rs:388 | Frame.owner | independent: callback_frame and handler_frame replace the owner under any frame kind |
| crates/kivact-runtime/src/reactive/data.rs:391 | Frame.pure | independent: pure_section sets it under any frame kind and owner |
| crates/kivact-runtime/src/reactive/data.rs:393 | Frame.writer | independent: inherited from the enclosing frame except in a non-user effect's own frame (tracking_frame) |
| crates/kivact-runtime/src/reactive/data.rs:395 | Frame.tracking | independent: untracked_frame and callback_frame clear it under any kind |
| crates/kivact-runtime/src/reactive/detail.rs:60 | DetailWrite.cause | independent: the turn's last cause when the write happened (None before the turn's first cause or after the cap dropped one); a write with or without a cause may or may not change the value |
| crates/kivact-runtime/src/reactive/detail.rs:71 | DetailWrite.changed | independent: a read-only DevTools record of one write; whether it changed the value is a fact of its own |
| crates/kivact-runtime/src/reactive/detail.rs:82 | DetailRun.cause | independent: the turn's last cause when the run happened (None before the turn's first cause or after the cap dropped one); a run with or without a cause may or may not change its value |
| crates/kivact-runtime/src/reactive/detail.rs:90 | DetailRun.changed | independent: a read-only DevTools record of one run; whether it changed the value is a fact of its own |
| crates/kivact-runtime/src/reactive/diag.rs:36 | Reported.handled | independent: a read-only error report; whether a handler handled it is a separate fact from the sequence number |
| crates/kivact-runtime/src/reactive/host_types.rs:7 | Pending.user | independent: a snapshot of the user-effect queue taken by Runtime::pending; user effects queue whether or not render effects, deferred reports or a sent wake exist |
| crates/kivact-runtime/src/reactive/host_types.rs:10 | Pending.render | independent: a snapshot of the render and internal effect queue taken by Runtime::pending; a render effect queues with or without user effects, deferred reports or a sent wake |
| crates/kivact-runtime/src/reactive/host_types.rs:13 | Pending.later | independent: a snapshot of the deferred error reports taken by Runtime::pending; layout can report an error whatever effects are queued and whether a wake was sent |
| crates/kivact-runtime/src/reactive/host_types.rs:16 | Pending.turn_requested | independent: a snapshot of TurnState::WakeSent taken by Runtime::pending; a wake is also sent for tasks and timers, and a running turn has queued work without one, so it pairs with any queue contents |
| crates/kivact-test/src/facts.rs:15 | Node.at | independent: a per-query creation location; any combination is valid |
| crates/kivact-test/src/screen/disposal_tests.rs:32 | a_cleanup_writes_signals_and_registers_on_its_disposing_owner::slots | independent: a test slot holding the branch signals; written once |
| crates/kivact-test/src/screen/disposal_tests.rs:36 | a_cleanup_writes_signals_and_registers_on_its_disposing_owner::open | independent: the show() condition the test flips to dispose the branch; the closed counter and the slot are written by other steps |
| crates/kivact-test/src/screen/disposal_tests.rs:38 | a_cleanup_writes_signals_and_registers_on_its_disposing_owner::closed | independent: the counter a late cleanup writes after the branch is gone; the test reads it whether open is true or false |
| crates/kivact-test/src/screen/editing_drop_probe.rs:8 | Probe.armed | independent: the reader and the other live node the test arms this probe with; an unarmed probe skips the drop check, and the shared counters count drops whatever it holds |
| crates/kivact-test/src/screen/host_sync_tests.rs:31 | rejected_pointer_down_cannot_retain_an_ended_region_gesture::disabled | independent: a test slot holding the disabled signal; written once |
| crates/kivact-test/src/screen/host_sync_tests.rs:35 | rejected_pointer_down_cannot_retain_an_ended_region_gesture::disabled | independent: the canvas's disabled input the test sets to reject a pointer down; the press and down counters move only on accepted input |
| crates/kivact-test/src/screen/host_sync_tests.rs:116 | a_primary_down_of_the_gesture_pointer_releases_a_gesture_whose_up_was_lost::downs | independent: counts the Down events the canvas handler sees and lets only the first start a gesture; the press counter counts activations separately, so any pair of counts is valid |
| crates/kivact-test/src/screen/real_window_tests.rs:88 | virtual_time_drives_sleep_and_frame_loops_exactly::slot | independent: a test slot holding the frame loop; written once |
| crates/kivact-test/src/screen/real_window_tests.rs:119 | virtual_time_drives_sleep_and_frame_loops_exactly::shown | independent: set by the spawned sleep after 300 ms of virtual time; the frame-loop slot of the same test is filled at mount, before and after it flips |
| crates/kivact-test/src/screen/real_window_tests.rs:140 | a_display_rate_loop_follows_the_window_to_a_faster_display::slot | independent: a test slot holding the frame loop; written once |
| crates/kivact-test/src/screen/real_window_tests.rs:229 | ticks::slot | independent: a test slot holding the frame loop; written once |
| crates/kivact-ui/src/calendar/navigation.rs:74 | SwipeSnapshot.rtl | independent: computed per event in swipe_navigation from the inherited reading direction; it only flips the drag sign, so it pairs with any at_min/at_max taken from the caller's bounds |
| crates/kivact-ui/src/calendar/navigation.rs:77 | SwipeSnapshot.at_min | independent: computed per event in swipe_navigation as "shown month <= the caller's min month"; it is true together with at_max when min and max fall in the shown month, which resists drags both ways |
| crates/kivact-ui/src/calendar/navigation.rs:80 | SwipeSnapshot.at_max | independent: computed per event in swipe_navigation as "shown month >= the caller's max month"; it is set apart from the reading direction and is true with at_min only when both bounds fall in the shown month |
| crates/kivact-ui/src/calendar_values.rs:31 | CalendarDayVisual.today | independent: date == the clock's today; the grid shows today inside or outside the displayed month, with or without a tone |
| crates/kivact-ui/src/calendar_values.rs:33 | CalendarDayVisual.in_month | independent: date lies in the displayed month; the grid's leading and trailing days are outside it whether or not they are today or toned |
| crates/kivact-ui/src/calendar_values.rs:35 | CalendarDayVisual.mark | independent: the tone from Calendar::marks, else Calendar::weekend, for this date; the caller's callbacks may tone any date, today or not, in the month or not |
| crates/kivact-ui/src/carousel_geometry.rs:7 | PageRequest.vertical | independent: the carousel orientation read when a page step is requested; the caller sets it apart from reading direction, step sign and looping, and page() handles every pair |
| crates/kivact-ui/src/carousel_geometry.rs:9 | PageRequest.rtl | independent: the inherited reading direction at the request; page() reads it only on the horizontal axis, so a vertical request with rtl set is valid and ignores it |
| crates/kivact-ui/src/carousel_geometry.rs:11 | PageRequest.next | independent: the sign of the key or button step (delta > 0); either direction can be asked on any axis, in either reading direction, with or without looping |
| crates/kivact-ui/src/chat_values.rs:83 | ChatMessageState.has_reactions | independent: ChatMessage::reactions fills the reaction slot of a message with or without header, footer, avatar or press handler |
| crates/kivact-ui/src/chat_values.rs:85 | ChatMessageState.actionable | independent: ChatMessage::on_press makes the bubble pressable, whichever of its slots are filled |
| crates/kivact-ui/src/chat_values.rs:87 | ChatMessageState.has_header | independent: ChatMessage::header fills the slot above the bubble; header, footer and avatar are separate builder calls and each renders alone |
| crates/kivact-ui/src/chat_values.rs:89 | ChatMessageState.has_footer | independent: ChatMessage::footer fills the slot below the bubble, with or without a header |
| crates/kivact-ui/src/chat_values.rs:91 | ChatMessageState.has_avatar | independent: ChatMessage::avatar fills the avatar slot beside a bubble of any group position, with or without header or footer |
| crates/kivact-ui/src/chat_values.rs:102 | ChatComposerState.stopped | independent: copied from the caller's stop_shown and status |
| crates/kivact-ui/src/chat_values.rs:104 | ChatComposerState.status | independent: copied from the caller's stop_shown and status |
| crates/kivact-ui/src/code_values.rs:17 | CodeBlockState.numbered | independent: CodeBlock::line_numbers shows the gutter in every collapse state, wrapped or not |
| crates/kivact-ui/src/code_values.rs:19 | CodeBlockState.wrapped | independent: CodeBlock::wrap wraps long lines whether or not the gutter shows |
| crates/kivact-ui/src/code_values.rs:22 | CodeBlockState.copied | independent: use_copy_feedback holds it for copy_feedback_ms after Copy; the feedback shows in any collapse state and Expand works while it shows |
| crates/kivact-ui/src/code_values.rs:24 | CodeBlockState.has_header_content | independent: CodeBlock::title, language, copy_button or end_content fill the header; a collapsible block shows its header even when this is false |
| crates/kivact-ui/src/code_values.rs:28 | CodeBlockState.has_footer | independent: CodeBlock::footer fills the footer slot, with or without header content |
| crates/kivact-ui/src/collection_values.rs:44 | ToolbarState.centered | independent: Toolbar::center gave a centre region; it is laid out in either reading direction |
| crates/kivact-ui/src/collection_values.rs:46 | ToolbarState.rtl | independent: the reading direction from use_direction at mount; it mirrors the start and end regions with or without a centre region |
| crates/kivact-ui/src/collection_values.rs:83 | ItemRowState.has_description | independent: ItemRow::description (text or content) was given; pressable and passive rows both show one |
| crates/kivact-ui/src/collection_values.rs:85 | ItemRowState.actionable | independent: ItemRow::on_press or a delegated target makes the row pressable, with or without a description or a list position |
| crates/kivact-ui/src/component_values.rs:11 | AlertState.content | independent: Alert::content fills the content slot (Some) and Alert::collapsible with the open binding choose its Expansion; it combines with any icon, actions, description or dismiss button |
| crates/kivact-ui/src/component_values.rs:15 | AlertState.dismissible | independent: Alert::dismissible shows the dismiss button, with or without content, icon or actions |
| crates/kivact-ui/src/component_values.rs:17 | AlertState.has_icon | independent: Alert::icon fills the icon slot of a dismissible or fixed alert alike |
| crates/kivact-ui/src/component_values.rs:19 | AlertState.has_actions | independent: Alert::actions fills the action slot, beside or without a dismiss button |
| crates/kivact-ui/src/component_values.rs:21 | AlertState.has_description | independent: Alert::description adds the description line, with or without content below it |
| crates/kivact-ui/src/component_values.rs:46 | CardState.disabled | independent: the caller's Card::disabled value applies to any action |
| crates/kivact-ui/src/component_values.rs:48 | CardState.has_media | independent: media and footer are separate optional slots; all four combinations render |
| crates/kivact-ui/src/component_values.rs:50 | CardState.has_footer | independent: media and footer are separate optional slots; all four combinations render |
| crates/kivact-ui/src/component_values.rs:149 | InputGroupState.focused | independent: focus is within the group (focus_within of its root); invalid and any adornment combine with it |
| crates/kivact-ui/src/component_values.rs:151 | InputGroupState.invalid | independent: the group's .invalid(..) (accessible_invalid); a group is invalid whether focused or not |
| crates/kivact-ui/src/component_values.rs:153 | InputGroupState.has_start | independent: InputGroup::start fills the inline start adornment, with or without an end one |
| crates/kivact-ui/src/component_values.rs:155 | InputGroupState.has_end | independent: InputGroup::end fills the inline end adornment, with or without a start one |
| crates/kivact-ui/src/component_values.rs:157 | InputGroupState.has_block_start | independent: InputGroup::block_start adds a row above the controls, beside any inline adornments |
| crates/kivact-ui/src/component_values.rs:159 | InputGroupState.has_block_end | independent: InputGroup::block_end adds a row below the controls, beside any inline adornments |
| crates/kivact-ui/src/component_values.rs:169 | InputGroupMemberState.first | independent: index == 0 in GroupRender::member; a single member is both first and last |
| crates/kivact-ui/src/component_values.rs:171 | InputGroupMemberState.last | independent: index + 1 == count in GroupRender::member; the middle member of three is neither |
| crates/kivact-ui/src/component_values.rs:173 | InputGroupMemberState.focused | independent: focus is within this member (focus_within of its node), at any position in the group |
| crates/kivact-ui/src/date_range_draft.rs:19 | Draft::Editing.start | independent: the last date the start DateInput published, None once it is cleared; the two editors publish separately, so any pair with end is a draft, and start after end shows order_invalid |
| crates/kivact-ui/src/date_range_draft.rs:21 | Draft::Editing.end | independent: the last date the end DateInput published, None once it is cleared; it is set apart from start, and an end before start is kept and reported by order_invalid |
| crates/kivact-ui/src/disclosure_tokens.rs:10 | DisclosureMotion.animate | independent: the theme's default for Disclosure::animated; a disclosure that opts in with .animated(true) uses transition even when this is false |
| crates/kivact-ui/src/disclosure_values.rs:15 | DisclosureRecipeState.open | independent: copied from the attached part's DisclosureState::open by DisclosureRecipeState::from_part; standalone panels and accordion members (group Some) both open and close |
| crates/kivact-ui/src/group_types.rs:44 | ButtonGroupMemberState.first | independent: index == 0 in ButtonGroup's member memo; a lone child is both first and last and a middle child neither, so all four pairs occur |
| crates/kivact-ui/src/group_types.rs:46 | ButtonGroupMemberState.last | independent: index + 1 == count in ButtonGroup's member memo; with first it marks a lone child (both), the two ends (one each) and middle children (neither) |
| crates/kivact-ui/src/group_types.rs:68 | ToggleGroupState.rtl | independent: the inherited reading direction at render; the caller's spacing is set apart from it, so either direction pairs with a set or default spacing |
| crates/kivact-ui/src/group_types.rs:77 | ToggleGroupItemState.first | independent: whether this toggle's key is the first of the group's items; a lone item is both first and last and a middle one neither, so all four pairs occur |
| crates/kivact-ui/src/group_types.rs:79 | ToggleGroupItemState.last | independent: whether this toggle's key is the last of the group's items; with first it marks a lone item (both), the two ends (one each) and middle ones (neither) |
| crates/kivact-ui/src/indicator_values.rs:26 | BadgeState.actionable | independent: Badge::on_press makes the badge pressable; its icons are added by separate calls |
| crates/kivact-ui/src/indicator_values.rs:29 | BadgeState.leading_icon | independent: Badge::icon fills the leading slot, with or without a trailing icon or press handler |
| crates/kivact-ui/src/indicator_values.rs:31 | BadgeState.trailing_icon | independent: Badge::trailing_icon fills the trailing slot, with or without a leading icon or press handler |
| crates/kivact-ui/src/navigation_values.rs:24 | StepperItemState.interactive | independent: whether Stepper::on_step_press was given, the same for every step; a disabled, optional or collapsed step keeps it, and disabled alone refuses the press |
| crates/kivact-ui/src/navigation_values.rs:26 | StepperItemState.collapsed | independent: copied from StepperState::collapsed, the whole stepper's responsive layout; disabled and optional steps appear in either layout |
| crates/kivact-ui/src/navigation_values.rs:28 | StepperItemState.disabled | independent: StepperItem::disabled; a disabled step may also be optional and may have an activation handler on the stepper |
| crates/kivact-ui/src/navigation_values.rs:30 | StepperItemState.optional | independent: StepperItem::optional only marks the label; it combines with disabled and either layout |
| crates/kivact-ui/src/navigation_values.rs:135 | PaginationSummary.has_more | independent: Pagination::has_more for remote paging; the caller reports more pages with or without a known total |
| crates/kivact-ui/src/progress_tokens.rs:7 | ProgressMotion.animate_initial | independent: whether the first determinate fill animates from zero; the moving-fill clock that reduced_period_ms drives is unaffected |
| crates/kivact-ui/src/selection_field.rs:52 | FieldMachine.bounds | independent: the trigger rect read from layout for each event, None until it is first laid out; a disabled trigger still has bounds and an unmeasured one may be enabled |
| crates/kivact-ui/src/selection_field.rs:54 | FieldMachine.disabled | independent: the caller's disabled flag read for each event; it is set apart from layout, so it is true or false whether or not the trigger has been measured |
| crates/kivact-ui/src/tree_values.rs:9 | TreeRowRecipeState.is_last | independent: the row is the last visible sibling at its level (TreeRow::is_last); last rows may be branches or leaves |
| crates/kivact-ui/src/tree_values.rs:11 | TreeRowRecipeState.has_branches | independent: some visible row of the whole tree has children; every row, last or not, reads the same value |
| crates/kivact-ui/src/tree_values.rs:13 | TreeRowRecipeState.custom_marker | independent: Tree::marker_content was given; it replaces the glyph on last and inner rows alike |
| crates/kivact-ui/src/values.rs:118 | ButtonState.busy | independent: busy comes from the caller's busy value and is valid with any icon or label layout |
| crates/kivact-ui/src/values.rs:128 | ButtonState.leading_icon | independent: the leading icon, the trailing icon and the hidden label are set by separate builder calls and all eight combinations are drawn |
| crates/kivact-ui/src/values.rs:131 | ButtonState.trailing_icon | independent: the leading icon, the trailing icon and the hidden label are set by separate builder calls and all eight combinations are drawn |
| crates/kivact-ui/src/values.rs:135 | ButtonState.label_hidden | independent: the leading icon, the trailing icon and the hidden label are set by separate builder calls and all eight combinations are drawn |
| crates/kivact-ui/src/values.rs:171 | ToggleButtonState.pressed | independent: the pressed state follows the toggle and is valid with any icon or label layout |
| crates/kivact-ui/src/values.rs:176 | ToggleButtonState.leading_icon | independent: the leading icon and the hidden label are set separately; all four combinations are drawn |
| crates/kivact-ui/src/values.rs:179 | ToggleButtonState.label_hidden | independent: the leading icon and the hidden label are set separately; all four combinations are drawn |
| crates/kivact-ui/src/values.rs:312 | CheckControl::Switch.on | independent: the switch's bound value; the opted-in drag starts from either value and moves the thumb whichever one the binding holds |
| crates/kivact-ui/src/values.rs:320 | CheckControl::Switch.drag | independent: the live visual of the opted-in drag gesture, None when the switch has no drag or none is running; it runs while the switch is on or off |
| crates/kivact-ui/src/values.rs:418 | DialogState.open | independent: the popup's open state (Shape::state) or the drawer's open binding; a dialog with or without media opens and closes |
| crates/kivact-ui/src/values.rs:421 | DialogState.media | independent: Dialog::media gave a header view (the drawer's drag handle reports false); it is fixed while the dialog opens and closes |
| crates/kivact-ui/src/values.rs:446 | SelectTriggerState.open | independent: the select's popup or the selection field's FieldState::Open; the list opens with or without a selected option |
| crates/kivact-ui/src/values.rs:449 | SelectTriggerState.placeholder | independent: no listed option is selected (trigger_projection; a selection field reports false); the trigger shows it while the list is open or closed |
| crates/kivact-ui-creative/src/audio/transport.rs:16 | TransportState.playing | independent: Play toggles playing and Stop clears it; Record and Cycle leave it |
| crates/kivact-ui-creative/src/audio/transport.rs:18 | TransportState.recording | independent: Record toggles recording; Play, Stop and Cycle leave it |
| crates/kivact-ui-creative/src/audio/transport.rs:20 | TransportState.cycling | independent: Cycle toggles cycling; Play, Stop and Record leave it |
| crates/kivact-ui-creative/src/editors/arrangement/model.rs:59 | TimelineTrack.muted | independent: the application's track mute, changed only on a TrackFlag::Mute request from its header toggle; a muted track may also be soloed, armed or reading automation, as on a mixer |
| crates/kivact-ui-creative/src/editors/arrangement/model.rs:61 | TimelineTrack.solo | independent: the application's track solo, changed only on a TrackFlag::Solo request from its header toggle; soloing never clears mute, arm or automation read, so it pairs with any of them |
| crates/kivact-ui-creative/src/editors/arrangement/model.rs:63 | TimelineTrack.armed | independent: the application's record arm, changed only on a TrackFlag::Arm request from its header toggle; an armed track may be muted, soloed or reading automation |
| crates/kivact-ui-creative/src/editors/arrangement/model.rs:65 | TimelineTrack.automation_read | independent: the application's automation read, set by TimelineTrack::automation and changed on a TrackFlag::AutomationRead request; the widget only shows it as a header toggle, so it pairs with any mute, solo or arm |
| crates/kivact-ui-creative/src/editors/node/model.rs:50 | NodeData.muted | independent: the application's node mute, flipped by the application on NodeRequest::ToggleMute (M key); it only dims the node in paint, so a muted node may have a preview image or not |
| crates/kivact-ui-creative/src/outliner_data.rs:13 | TreeItem.parent | independent: the application's hierarchy, given in TreeItem::new and changed by the application after a TreeDrop; a root or a child may carry any visibility, selectability, render or placeholder flag |
| crates/kivact-ui-creative/src/outliner_data.rs:18 | TreeItem.visible | independent: the application's visibility, changed on a Restriction{flag: Visible} proposal (eye toggle or H); it only dims the label, so a hidden item can still be selectable, rendered or a placeholder |
| crates/kivact-ui-creative/src/outliner_data.rs:20 | TreeItem.selectable | independent: the application's selectability, changed on a Restriction{flag: Selectable} proposal; it only disables the tree node, so it pairs with any visibility, render or placeholder state |
| crates/kivact-ui-creative/src/outliner_data.rs:22 | TreeItem.rendered | independent: the application's render toggle, changed on a Restriction{flag: Rendered} proposal; the outliner only shows it in its toggle column, so it pairs with any visibility, selectability or placeholder state |
| crates/kivact-ui-creative/src/outliner_data.rs:26 | TreeItem.placeholder | independent: the application marks an unresolved item with TreeItem::placeholder; it only draws the label italic and muted, so the item keeps its parent and all restriction toggles |
| crates/kivact-ui-creative/src/outliner_data.rs:73 | TreeDrop.target | independent: the new parent drop_plan computes from the row under the pointer; None makes a root, and both a root and a child drop may land first (after None) or after a sibling |
| crates/kivact-ui-creative/src/outliner_data.rs:76 | TreeDrop.after | independent: the sibling under target that drop_plan puts the item after; None puts it first, which a drop into a root list and into a parent both allow |
| crates/kivact-ui-soft/src/markdown.rs:808 | Marks.strong | independent: set by the caller's strong flag or a *** / ** / __ delimiter; nesting reaches it with any italic, deleted or code subset, and styled_span paints each mark separately. |
| crates/kivact-ui-soft/src/markdown.rs:810 | Marks.italic | independent: set only by a *** / * / _ delimiter; delimiters nest in any order, so every combination with strong, deleted and code is parsed and painted. |
| crates/kivact-ui-soft/src/markdown.rs:812 | Marks.deleted | independent: set only by a ~~ delimiter; it nests inside or around strong and italic and may wrap code, and styled_span adds the strike-through regardless of the other marks. |
| crates/kivact-ui-soft/src/markdown.rs:814 | Marks.code | independent: set only by a backtick delimiter; code stops further parsing but keeps the enclosing strong, italic and deleted marks, so all those combinations are painted. |
| crates/kivact-ui-soft/src/power_search.rs:144 | FilterDraft.field | independent: set only by the Field select or seeded from the edited filter; operator is chosen separately and Apply requires both, so a draft missing either is a valid in-progress form. |
| crates/kivact-ui-soft/src/power_search.rs:146 | FilterDraft.operator | independent: the chosen comparison; every field offers every operator |
| crates/kivact-ui-soft/src/rich_block_doc.rs:25 | Kind::List.number | independent: parsed from the `N.` marker and incremented on split; a task checkbox may sit on ordered or bullet items, so every number and checked pair comes from valid source. |
| crates/kivact-ui-soft/src/rich_block_doc.rs:27 | Kind::List.checked | independent: parsed from the `[ ]` / `[x]` prefix and toggled only by Document::task, which rewrites the head text in the same step; numbering never constrains it. |
| crates/kivact-ui-soft/src/side_navigation/items.rs:432 | ItemBranch::Parent.expansion | independent: whether the item has its own collapse owner (AlwaysOpen has none); independent of the shell's flyout |
| crates/kivact-ui-soft/src/side_navigation/items.rs:435 | ItemBranch::Parent.flyout | independent: whether the shell is collapsible (a fixed shell has no rail flyout); independent of the item's collapse owner |
| dev/tests/gpu-interop-producer/src/producer.rs:28 | Producer.shared | independent: the pool's handles until shared_textures or signal_prepared takes them, once per engine; draw numbers frames before or after that, since the engine's first frames wait for the preparation signal |
| dev/xtask/src/public_surface.rs:20 | Binding.public | independent: whether a binding is public is a fixed fact, separate from whether it names a value |
| dev/xtask/src/public_surface.rs:23 | Binding.value | independent: whether a binding names a value is a fixed fact, separate from its visibility |
| dev/xtask/src/state_fields/mode.rs:7 | Mode.report | independent: all four combinations are valid because console notes and the audit file are separate requests |
| dev/xtask/src/state_fields/mode.rs:9 | Mode.audit | independent: the audit file can be requested without console notes |
| examples/kivact/component/src/main.rs:34 | app::first | independent: whether the First card is shown, cleared only by its own close button and never set back; the Second card has its own signal, so either card or both can be closed |
| examples/kivact/component/src/main.rs:36 | app::second | independent: whether the Second card is shown, cleared only by its own close button and never set back; the First card has its own signal, so either card or both can be closed |
| examples/kivact-ui-basic/src/docs/alert_dialog.rs:78 | destructive::open | independent: "Delete Chat" opens the alert and Cancel or Delete closes it; it opens again after a deletion, so it is true or false with either value of deleted |
| examples/kivact-ui-basic/src/docs/alert_dialog.rs:80 | destructive::deleted | independent: set by the alert's Delete button and never cleared; the muted line shows it while the alert is open or closed |
| examples/kivact-ui-basic/src/docs/data_table.rs:65 | payments::sort | independent: the table's sort binding, written by the data_table column headers; any sort can be shown on any page, since sorting only reorders the filtered rows |
| examples/kivact-ui-basic/src/docs/data_table.rs:67 | payments::page | independent: the table's page binding, moved by the pager and reset to 0 by a new sort or filter text; every page is reachable afterwards under any sort |
| examples/kivact-ui-basic/src/docs/dialog.rs:123 | confirm::open | independent: "Delete account" opens the dialog and Cancel or Delete closes it; reopening after the deletion keeps deleted true |
| examples/kivact-ui-basic/src/docs/dialog.rs:125 | confirm::deleted | independent: set by the dialog's Delete button and never cleared; the account line reads it with the dialog open or closed |
| examples/kivact-ui-basic/src/docs/drawer.rs:36 | default::open | independent: the drawer's open binding, set by Open Drawer and cleared by Submit or Cancel; the goal keeps its value while the drawer is closed, so either open state pairs with any goal |
| examples/kivact-ui-basic/src/docs/drawer.rs:38 | default::goal | independent: the calorie goal, stepped by the minus and plus buttons in the drawer; nothing resets it on close, so every goal pairs with either drawer state |
| examples/kivact-ui-basic/src/docs/dropdown_menu.rs:76 | checkboxes::status | independent: the Status Bar checkbox, flipped only by its own menu entry; the Activity Bar entry never touches it, so both, either or neither bar can be on |
| examples/kivact-ui-basic/src/docs/dropdown_menu.rs:78 | checkboxes::activity | independent: the Activity Bar checkbox, flipped only by its own menu entry; the Status Bar entry never touches it, so both, either or neither bar can be on |
| examples/kivact-ui-basic/src/docs/trace.rs:65 | steps::first | independent: the open binding of the first trace collapsible, toggled only by its header; the second collapsible has its own binding, so both, either or neither can be open |
| examples/kivact-ui-basic/src/docs/trace.rs:67 | steps::second | independent: the open binding of the second trace collapsible, toggled only by its header; the first collapsible has its own binding, so both, either or neither can be open |
| examples/kivact-ui-creative/src/creative/audio_values.rs:15 | TransportState.playing | independent: Play and Stop toggle playing; Record and Cycle leave it |
| examples/kivact-ui-creative/src/creative/audio_values.rs:17 | TransportState.recording | independent: the catalog's record toggle (audio_samples.rs) and the kit transport (controls) flip it; the stop button clears only playing and the loop switches only cycling |
| examples/kivact-ui-creative/src/creative/audio_values.rs:19 | TransportState.cycling | independent: the loop switches in arrange.rs and audio_samples.rs and the kit transport set it; play, stop and record leave it |
| examples/kivact-ui-creative/src/creative/layout.rs:21 | properties::tab | independent: the inspector tab strip's selection, set only by inspector_tabs; it never touches the viewport display checkboxes, so any tab pairs with any display setting |
| examples/kivact-ui-creative/src/creative/layout.rs:24 | properties::display_name | independent: bound only to the Name checkbox of Viewport Display; Axes and Wireframe have their own signals, so every on/off mix is a valid display |
| examples/kivact-ui-creative/src/creative/layout.rs:26 | properties::display_axes | independent: bound only to the Axes checkbox of Viewport Display; Name and Wireframe have their own signals, so every on/off mix is a valid display |
| examples/kivact-ui-creative/src/creative/layout.rs:28 | properties::display_wireframe | independent: bound only to the Wireframe checkbox of Viewport Display; Name and Axes have their own signals, so every on/off mix is a valid display |
| examples/kivact-ui-creative/src/creative/shell.rs:811 | CatalogControls.compact | independent: flipped by the density button and Ctrl+Alt+D only; either row density shows with any chapter, sidebar and search overlay state |
| examples/kivact-ui-creative/src/creative/shell.rs:815 | CatalogControls.current | independent: the chapter chosen in the sidebar list; every chapter is a reachable view in either density and with the sidebar or search overlay in any state |
| examples/kivact-ui-creative/src/creative/shell.rs:817 | CatalogControls.sidebar_open | independent: the top bar's sidebar toggle; the sidebar hides or shows over any chapter and density, with the search overlay up or down |
| examples/kivact-ui-creative/src/creative/shell.rs:821 | CatalogControls.search | independent: opened by the search button, F3 and the feedback chapter's button, and closed by the overlay; it covers any chapter, density and sidebar state |
| examples/kivact-ui-creative/src/creative/shell.rs:827 | CatalogControls.shading | independent: the viewport shading mode chosen in the viewport toolbar or the pie menu; every mode draws with the viewport grid on or off |
| examples/kivact-ui-creative/src/creative/shell.rs:829 | CatalogControls.viewport_grid | independent: the viewport grid flipped from the viewport toolbar or the pie menu; the grid draws over every shading mode |
| examples/kivact-ui-creative/src/creative/shell.rs:837 | new::compact | independent: flipped by the density button and Ctrl+Alt+D only; either row density shows with any chapter, sidebar and search overlay state |
| examples/kivact-ui-creative/src/creative/shell.rs:841 | new::current | independent: the chapter chosen in the sidebar list; every chapter is a reachable view in either density and with the sidebar or search overlay in any state |
| examples/kivact-ui-creative/src/creative/shell.rs:843 | new::sidebar_open | independent: the top bar's sidebar toggle; the sidebar hides or shows over any chapter and density, with the search overlay up or down |
| examples/kivact-ui-creative/src/creative/shell.rs:848 | new::search | independent: opened by the search button, F3 and the feedback chapter's button, and closed by the overlay; it covers any chapter, density and sidebar state |
| examples/kivact-ui-creative/src/creative/shell.rs:854 | new::shading | independent: the viewport shading mode chosen in the viewport toolbar or the pie menu; every mode draws with the viewport grid on or off |
| examples/kivact-ui-creative/src/creative/shell.rs:856 | new::viewport_grid | independent: the viewport grid flipped from the viewport toolbar or the pie menu; the grid draws over every shading mode |
| examples/kivact-ui-game/src/game/controls.rs:66 | buttons::grid_on | independent: the grid toggle and the filter toggle are pressed separately and all four combinations are valid views |
| examples/kivact-ui-game/src/game/controls.rs:68 | buttons::filter_on | independent: the grid toggle and the filter toggle are pressed separately and all four combinations are valid views |
| examples/kivact-ui-game/src/game/controls.rs:165 | navigation::step | independent: the wizard step, moved only by the stepper's controls in wizard_steps; the pagination specimen has its own page signal, so any step pairs with any page |
| examples/kivact-ui-game/src/game/controls.rs:167 | navigation::page | independent: the pagination specimen's page, moved only by its pagination control; the wizard has its own step signal, so any page pairs with any step |
| examples/kivact-ui-game/src/game/system.rs:73 | feedback::info | independent: the info alert's open binding, cleared only by its own close button and never set back; the other alerts have their own bindings, so any subset of the four can be dismissed |
| examples/kivact-ui-game/src/game/system.rs:75 | feedback::success | independent: the success alert's open binding, cleared only by its own close button and never set back; the other alerts have their own bindings, so any subset of the four can be dismissed |
| examples/kivact-ui-game/src/game/system.rs:77 | feedback::warning | independent: the warning alert's open binding, cleared only by its own close button and never set back; the other alerts have their own bindings, so any subset of the four can be dismissed |
| examples/kivact-ui-game/src/game/system.rs:79 | feedback::danger | independent: the danger alert's open binding, cleared only by its own close button and never set back; the other alerts have their own bindings, so any subset of the four can be dismissed |
| examples/kivact-ui-game/src/game/system.rs:93 | overlays::pinned | independent: the detail-column menu checkbox, flipped only by its own entry; archived and the user popover have their own signals, so it pairs with either of their states |
| examples/kivact-ui-game/src/game/system.rs:95 | overlays::archived | independent: the include-archived menu checkbox, flipped only by its own entry; pinned and the user popover have their own signals, so it pairs with either of their states |
| examples/kivact-ui-game/src/game/system.rs:97 | overlays::user_open | independent: the user popover's open binding, set by its own trigger and cleared by the popover; it never reads or writes the menu checkboxes, so it opens with any checkbox pair |
| examples/kivact-ui-game/src/game/system.rs:260 | layout::notify | independent: bound only to the inspector's notify switch; the opacity slider has its own signal, so notifications on or off pair with any opacity |
| examples/kivact-ui-game/src/game/system.rs:262 | layout::opacity | independent: bound only to the inspector's opacity slider; the notify switch has its own signal, so any opacity pairs with notifications on or off |
| examples/kivact-ui-game/src/game/window.rs:37 | frame::width | independent: the explorer width, written only by the native_window divider within explorer_range; it is layout only, so any width pairs with any editor mode, run state or About dialog |
| examples/kivact-ui-game/src/game/window.rs:40 | frame::help | independent: the About dialog's open binding, set by the Help menu's help command and cleared by the dialog; it never touches mode, run or the explorer width, so it opens over any editor state |
| examples/kivact-ui-pop/src/pop/components.rs:458 | switch_controls::volume | independent: bound only to the BGM volume slider; the difficulty slider has its own signal, so any volume pairs with any difficulty |
| examples/kivact-ui-pop/src/pop/components.rs:460 | switch_controls::difficulty | independent: bound only to the difficulty slider; the volume slider has its own signal, so any difficulty pairs with any volume |
| examples/kivact-ui-pop/src/pop/components.rs:516 | check_controls::notifications | independent: bound only to the notifications checkbox; the weekly report checkbox has its own signal, so both, either or neither can be checked |
| examples/kivact-ui-pop/src/pop/components.rs:518 | check_controls::reports | independent: bound only to the weekly report checkbox; the notifications checkbox has its own signal, so both, either or neither can be checked |
| examples/kivact-ui-pop/src/pop/data.rs:42 | display::next_chip | independent: the index of the next suggested chip tag, advanced only by the badge specimen's add button; the progress loop never reads it, so any index pairs with any progress |
| examples/kivact-ui-pop/src/pop/data.rs:44 | display::live | independent: the demo progress value, advanced only by the 900 ms spawn loop; the chip suggestion never reads it, so any progress pairs with any chip index |
| examples/kivact-ui-pop/src/pop/data.rs:74 | table::page | independent: the student table's page, moved by the pager and reset to 0 by a new sort, status filter or search; every page is reachable afterwards under any sort |
| examples/kivact-ui-pop/src/pop/data.rs:76 | table::sort | independent: the student table's sort binding, written by the column headers; sorting only reorders the filtered rows, so any sort can be shown on any page |
| examples/kivact-ui-pop/src/pop/patterns.rs:20 | feedback::dismissed_info | independent: set only by the maintenance notice's close button and never cleared; the success notice has its own signal, so both, either or neither notice can be dismissed |
| examples/kivact-ui-pop/src/pop/patterns.rs:22 | feedback::dismissed_success | independent: set only by the success notice's close button and never cleared; the maintenance notice has its own signal, so both, either or neither notice can be dismissed |
| examples/kivact-ui-pop/src/pop/shell.rs:638 | student_drawer::pin_student | independent: bound only to the pin-to-main-team switch in the student drawer; the auto-skill switch has its own signal, so both, either or neither can be on |
| examples/kivact-ui-pop/src/pop/shell.rs:640 | student_drawer::auto_skill | independent: bound only to the auto-skill switch in the student drawer; the pin switch has its own signal, so both, either or neither can be on |
| examples/kivact-ui-soft/src/model.rs:9 | CatalogState.dark | independent: the appearance, seeded from the caller's ThemeMode and flipped by the Toggle appearance button; it only picks the theme, so every page and query can be shown in light or dark |
| examples/kivact-ui-soft/src/model.rs:11 | CatalogState.selected | independent: the shown page, seeded from the caller's slug and set by the navigation buttons; the theme toggle never writes it, so any page shows in light or dark |

## FINAL-2 の検査器が「未検証」と報告した項目

0 件（`dev/xtask/state-audit.md` に「未検証」の行は無い）。

## V2 で新しく作った状態（作業指示書 12.1、設計書 6.1）

| 状態 | 持ち主 | 書き手（単一の入口） | 破棄 | unit |
|---|---|---|---|---|
| frame 源と layout motion の索引（`MotionSources`） | engine の Model | prop の書き手（`index_sources`）と削除（`forget_sources`） | node の削除・window の close | F1 |
| flat 子の slot の cache | engine の `NodeTree` | `flat_at`、`forget_flat_around`・`remove_with` | node の削除・子の変化 | F3 |
| ChildIndex の部分更新の印 | engine の paint の state | layout pass の geometry の変化 | container の再構築 | F4 |
| damage の判定の `Reach` | arvane-compose の cache | `plan/damage.rs` の `indexed` | frame の cache の再構築 | F5-fix |
| a11y の dirt（`A11yPublish::Published` の中） | engine の a11y の state | `A11yState` の method | AT の停止で捨てる | AH1 |
| ink の `beyond` の bit | engine の paint の ink の cache | ink の計算 | ink の cache の世代 | AH3 |
| `LastPresent::Shown { at_ns }` と CPU の frame state の解放 | engine の host の window | `redraw`（Shown）、`wake`（解放） | window の close | S2・S3 |
| `Recent<T>`（直近値の共有） | engine の Model と layout | prop の書込み | Model と一緒 | M3 |
| `RefObservation` | core の NodeRef | `require_observation` | owner の破棄 | M8 |
| `CheckFold`（semantics の plan） | core の mount | `fold`（mount 時） | plan の破棄 | M7-fix |
| font の bytes の共有（`FontSource`） | glyph run・登録 font・SVG の DB | 登録・shaping | 最後の参照 | S45 |
| content の Part の状態の slot（`attach_part_state`） | interaction の各 Api | `attach_part_state` | content の owner の終わり | IN-3 |

## 人間の確認（(b)）

- 確認者：
- 日付：
- 結果（承認 / 差戻し）：
- 確認した内容・コメント：

（この欄は人間が書く。Orchestrator は書いていない。）
