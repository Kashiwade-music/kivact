# KPI B0-CP3b-r4 (2026-11-22T15:20Z, MINGW64_NT-10.0-26200, commit c9518946)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=46.7 samples=[ 46.6 75.2 46.7 46.6 57.0 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.7 2.7 2.7 2.8 2.8 ]
- K1 main_to_first_present renderer=cpu median_ms=47.7 samples=[ 47.7 46.7 43.0 60.7 48.9 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.7 2.8 2.8 2.7 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=188.4 samples=[ 581.4 173.2 228.8 178.6 188.4 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25672 private_kib=5888 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25712 private_kib=5908 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=85444 private_kib=102848 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=698.4 configure_ms=19.9]
- basic-catalog renderer=default ws_kib=51228 private_kib=27240 idle_cpu_ms_per_s=113.327 idle_frames=637 cpu_ticks_100ns=12031250 
- rows3000-still renderer=default ws_kib=99788 private_kib=84244 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=51488 private_kib=27456 idle_cpu_ms_per_s=91.087 idle_frames=638 cpu_ticks_100ns=9687500 
- rows3000-still renderer=cpu ws_kib=100512 private_kib=85368 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25876 private_kib=6620 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36656 private_kib=19160 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.59
- item-text-0 renderer=cpu ws_kib=25920 private_kib=6660 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34256 private_kib=16648 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.78
- item-button-0 renderer=cpu ws_kib=25876 private_kib=6636 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=81260 private_kib=73336 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.46
- item-row-0 renderer=cpu ws_kib=25844 private_kib=6608 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=99944 private_kib=84628 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.70

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=3740
    [box] H site     989.9 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [box] H site     589.9 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     280.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     232.2 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     216.0 B/item     3000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [box] H site     207.5 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [box] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [box] H site     109.2 B/item        1 allocs kivact_engine::paint::state::ChildIndex::build
    [box] H site      99.7 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
- heap mode=text live_bytes_per_item=2971
    [text] H site     989.9 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [text] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [text] H site     360.6 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [text] H site     207.5 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [text] H site     192.1 B/item     6002 allocs kivact_layout::tree::LayoutPass::leaf<kivact_engine::tree::node_tree::NodeTree>
    [text] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [text] H site     109.2 B/item        1 allocs kivact_engine::paint::state::ChildIndex::build
    [text] H site      99.7 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
    [text] H site      65.5 B/item        2 allocs kivact_engine::measure::state::MeasureColumns::push
    [text] H site      54.6 B/item        1 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0<u32,kivact_core::view::descriptor::description::View,alloc::vec::Vec<u32,alloc::alloc::Global>,kv_heap::main::closure$0::closure_env$1>
- heap mode=button live_bytes_per_item=20377
    [button] H site    3959.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [button] H site    3921.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1504.0 B/item    72000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [button] H site     832.0 B/item    30000 allocs kivact_core::view::mount::mount_host_with
    [button] H site     830.1 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [button] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target
    [button] H site     696.2 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site     560.2 B/item    12004 allocs kivact_core::view::element_data::Common::push_style
    [button] H site     416.1 B/item     6002 allocs kivact_layout::tree::LayoutPass::leaf<kivact_engine::tree::node_tree::NodeTree>
- heap mode=row live_bytes_per_item=23453
    [row] H site    3959.5 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [row] H site    3921.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site     928.2 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site     928.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style
    [row] H site     832.0 B/item    30000 allocs kivact_core::view::mount::mount_host_with
    [row] H site     830.1 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [row] H site     797.4 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
    [row] H site     772.0 B/item    24000 allocs kivact_core::view::descriptor::description::impl$7::into_view
    [row] H site     732.0 B/item    48000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=332 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=8.19 ws_kib=33356 work_ms p50=1.0543 p95=1.4927 p99=1.6247 scene_p95=0.0341 raster_p95=1.3878 interval_p99=17.442
- rows=0 renderer=cpu frames=332 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=9.59 ws_kib=33280 work_ms p50=1.0517 p95=1.458 p99=1.8675 scene_p95=0.0383 raster_p95=1.362 interval_p99=17.4131
- rows=3000 renderer=default frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=11.26 ws_kib=106568 work_ms p50=1.4817 p95=1.9488 p99=2.2532 scene_p95=0.207 raster_p95=1.7611 interval_p99=17.642
- rows=3000 renderer=cpu frames=332 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=11.01 ws_kib=105812 work_ms p50=1.478 p95=1.96 p99=2.2179 scene_p95=0.2059 raster_p95=1.7743 interval_p99=17.4797

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=5.061 p50=5.061 p95=5.061 p99=5.061
- L counter_click n=200 mean=0.875 p50=0.860 p95=1.014 p99=1.056
- L mount_n100 n=1 mean=21.854 p50=21.854 p95=21.854 p99=21.854
- L unrelated_update_n100 n=50 mean=1.229 p50=1.199 p95=1.438 p99=1.919
- L row_update_n100 n=50 mean=2.154 p50=2.147 p95=2.381 p99=2.896
- L pointer_move_n100 n=120 mean=0.323 p50=0.034 p95=1.368 p99=1.452
- L wheel_n100 n=40 mean=0.014 p50=0.014 p95=0.014 p99=0.028
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 13666300, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=176.694 p50=176.694 p95=176.694 p99=176.694
- L unrelated_update_n1000 n=50 mean=1.258 p50=1.199 p95=1.387 p99=2.922
- L row_update_n1000 n=50 mean=3.291 p50=3.237 p95=3.453 p99=5.700
- L pointer_move_n1000 n=120 mean=0.592 p50=0.091 p95=2.099 p99=2.219
- L wheel_n1000 n=40 mean=0.047 p50=0.046 p95=0.051 p99=0.083
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 43426200, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=515.423 p50=515.423 p95=515.423 p99=515.423
- L unrelated_update_n3000 n=50 mean=1.298 p50=1.193 p95=1.372 p99=5.269
- L row_update_n3000 n=50 mean=7.630 p50=7.516 p95=8.261 p99=12.352
- L pointer_move_n3000 n=120 mean=1.599 p50=0.265 p95=5.768 p99=6.063
- L wheel_n3000 n=40 mean=0.135 p50=0.134 p95=0.141 p99=0.148
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 148561800, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18945536
- K6 kv_startup default 18940928
- K6 kv_startup no-default-features 11298304
- K6 basic-catalog default 34042880
- K6 kv_startup no-gpu 14353408
- K6 basic-catalog no-gpu 29470208
