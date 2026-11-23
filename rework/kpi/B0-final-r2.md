# KPI remote (2026-11-23T06:42Z, MINGW64_NT-10.0-22631, commit e99e222d)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=41.5 samples=[ 44.7 41.5 41.4 40.8 42.3 ]
- K4 first_frame_raster renderer=default median_ms=2.8 samples=[ 2.8 2.8 2.7 2.6 2.8 ]
- K1 main_to_first_present renderer=cpu median_ms=40.6 samples=[ 42.3 39.8 39.9 40.6 42.1 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.7 2.7 2.7 2.7 2.8 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=202.6 samples=[ 348.0 198.6 201.3 207.5 202.6 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=22588 private_kib=7048 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=22556 private_kib=7024 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=162956 private_kib=125536 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=Intel(R) UHD Graphics 770 (Vulkan) init_ms=118.0 configure_ms=38.7]
- basic-catalog renderer=default ws_kib=39944 private_kib=20444 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=default ws_kib=97288 private_kib=86148 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=40124 private_kib=20752 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=cpu ws_kib=97448 private_kib=86308 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=23180 private_kib=8212 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=33720 private_kib=20656 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.51
- item-text-0 renderer=cpu ws_kib=23188 private_kib=8208 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=31392 private_kib=18124 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.73
- item-button-0 renderer=cpu ws_kib=23192 private_kib=8220 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=79060 private_kib=75836 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.62
- item-row-0 renderer=cpu ws_kib=23252 private_kib=8288 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=96960 private_kib=85848 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.57

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=3775
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
- heap mode=button live_bytes_per_item=20482
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
- heap mode=row live_bytes_per_item=23418
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
- rows=0 renderer=default frames=173 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=33080 work_ms p50=1.7141 p95=3.2251 p99=3.5981 scene_p95=0.0633 raster_p95=3.0494 interval_p99=33.6658
- rows=0 renderer=cpu frames=173 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=4.63 ws_kib=33108 work_ms p50=2.1183 p95=3.2359 p99=3.763 scene_p95=0.0557 raster_p95=3.0568 interval_p99=33.6207
- rows=3000 renderer=default frames=173 fps=32.1 frame_period_ns=31250000 cpu_pct_of_core=0.29 ws_kib=105732 work_ms p50=2.019 p95=4.0253 p99=4.2833 scene_p95=0.3002 raster_p95=3.6318 interval_p99=34.1212
- rows=3000 renderer=cpu frames=173 fps=32.0 frame_period_ns=31250000 cpu_pct_of_core=0.00 ws_kib=106268 work_ms p50=2.3202 p95=4.1921 p99=5.6833 scene_p95=0.3165 raster_p95=3.7683 interval_p99=36.3899

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=4.004 p50=4.004 p95=4.004 p99=4.004
- L counter_click n=200 mean=0.467 p50=0.463 p95=0.491 p99=0.509
- L mount_n100 n=1 mean=13.341 p50=13.341 p95=13.341 p99=13.341
- L unrelated_update_n100 n=50 mean=0.707 p50=0.695 p95=0.731 p99=1.126
- L row_update_n100 n=50 mean=1.318 p50=1.318 p95=1.462 p99=1.797
- L pointer_move_n100 n=120 mean=0.158 p50=0.020 p95=0.627 p99=0.736
- L wheel_n100 n=40 mean=0.008 p50=0.008 p95=0.008 p99=0.016
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 8246600, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=98.514 p50=98.514 p95=98.514 p99=98.514
- L unrelated_update_n1000 n=50 mean=0.712 p50=0.687 p95=0.771 p99=1.682
- L row_update_n1000 n=50 mean=2.037 p50=2.010 p95=2.175 p99=2.959
- L pointer_move_n1000 n=120 mean=0.339 p50=0.057 p95=1.216 p99=1.261
- L wheel_n1000 n=40 mean=0.028 p50=0.027 p95=0.030 p99=0.040
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 24954200, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=290.217 p50=290.217 p95=290.217 p99=290.217
- L unrelated_update_n3000 n=50 mean=0.760 p50=0.701 p95=0.843 p99=3.115
- L row_update_n3000 n=50 mean=4.727 p50=4.511 p95=5.435 p99=10.091
- L pointer_move_n3000 n=120 mean=0.825 p50=0.180 p95=2.606 p99=2.957
- L wheel_n3000 n=40 mean=0.093 p50=0.092 p95=0.096 p99=0.109
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 71165100, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18949120
- K6 kv_startup default 18944512
- K6 kv_startup no-default-features 11301888
- K6 basic-catalog default 34037248
- K6 kv_startup no-gpu 14356480
- K6 basic-catalog no-gpu 29464064
