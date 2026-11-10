# KPI B1-cp1 (2026-11-10T14:19Z, MINGW64_NT-10.0-26200, commit 443ed1c3)

release build of kivact-example-counter (cold for its own target dir): 291 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=41.5 samples=[ 44.9 40.8 42.9 41.5 40.7 ]
- K4 first_frame_raster renderer=default median_ms=2.8 samples=[ 2.8 2.6 2.8 2.6 2.8 ]
- K1 main_to_first_present renderer=cpu median_ms=42.4 samples=[ 41.6 42.4 43.1 40.8 43.3 ]
- K4 first_frame_raster renderer=cpu median_ms=2.8 samples=[ 2.7 2.6 3.1 3.0 2.8 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=165.8 samples=[ 482.0 165.8 164.2 166.5 165.5 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25708 private_kib=5916 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25688 private_kib=5896 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=85356 private_kib=103800 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=649.7 configure_ms=15.6]
- basic-catalog renderer=default ws_kib=51516 private_kib=27536 idle_cpu_ms_per_s=72.969 idle_frames=630 cpu_ticks_100ns=7656250 
- rows3000-still renderer=default ws_kib=99436 private_kib=82808 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=51432 private_kib=27448 idle_cpu_ms_per_s=75.969 idle_frames=629 cpu_ticks_100ns=7968750 
- rows3000-still renderer=cpu ws_kib=98748 private_kib=83004 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25956 private_kib=6640 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36604 private_kib=19020 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.55
- item-text-0 renderer=cpu ws_kib=25916 private_kib=6604 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34688 private_kib=16752 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.92
- item-button-0 renderer=cpu ws_kib=25960 private_kib=6632 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=80908 private_kib=73056 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.32
- item-row-0 renderer=cpu ws_kib=25940 private_kib=6612 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=98824 private_kib=83244 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.29

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=3775
    [box] H site     995.4 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
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
    [text] H site     995.4 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [text] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [text] H site     360.6 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [text] H site     207.5 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [text] H site     192.1 B/item     6002 allocs kivact_layout::tree::LayoutPass::leaf<kivact_engine::tree::node_tree::NodeTree>
    [text] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [text] H site     109.2 B/item        1 allocs kivact_engine::paint::state::ChildIndex::build
    [text] H site      99.7 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
    [text] H site      65.5 B/item        2 allocs kivact_engine::measure::state::MeasureColumns::push
    [text] H site      54.6 B/item        1 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0<u32,kivact_core::view::descriptor::description::View,alloc::vec::Vec<u32,alloc::alloc::Global>,kv_heap::main::closure$0::closure_env$1>
- heap mode=button live_bytes_per_item=20412
    [button] H site    3981.3 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [button] H site    3921.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1476.0 B/item    69000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [button] H site     832.0 B/item    30000 allocs kivact_core::view::mount::mount_host_with
    [button] H site     830.1 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [button] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target
    [button] H site     696.2 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site     560.2 B/item    12004 allocs kivact_core::view::element_data::Common::push_style
    [button] H site     416.1 B/item     6002 allocs kivact_layout::tree::LayoutPass::leaf<kivact_engine::tree::node_tree::NodeTree>
- heap mode=row live_bytes_per_item=23104
    [row] H site    3981.3 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [row] H site    3921.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site     928.2 B/item    12003 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site     928.2 B/item    21004 allocs kivact_core::view::element_data::Common::push_style
    [row] H site     832.0 B/item    30000 allocs kivact_core::view::mount::mount_host_with
    [row] H site     830.1 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [row] H site     797.4 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
    [row] H site     772.0 B/item    24000 allocs kivact_core::view::descriptor::description::impl$7::into_view
    [row] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=332 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=6.50 ws_kib=32416 work_ms p50=1.0616 p95=1.3655 p99=1.6304 scene_p95=0.0303 raster_p95=1.2809 interval_p99=17.3281
- rows=0 renderer=cpu frames=331 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=7.35 ws_kib=32372 work_ms p50=1.0516 p95=1.1912 p99=1.4261 scene_p95=0.0319 raster_p95=1.1164 interval_p99=17.3069
- rows=3000 renderer=default frames=330 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.51 ws_kib=104044 work_ms p50=1.4749 p95=1.7638 p99=2.2041 scene_p95=0.2002 raster_p95=1.5234 interval_p99=17.5537
- rows=3000 renderer=cpu frames=329 fps=59.8 frame_period_ns=16666666 cpu_pct_of_core=8.80 ws_kib=103624 work_ms p50=1.4571 p95=1.7077 p99=2.0787 scene_p95=0.1844 raster_p95=1.4605 interval_p99=17.4853

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.291 p50=6.291 p95=6.291 p99=6.291
- L counter_click n=200 mean=0.808 p50=0.767 p95=1.156 p99=1.190
- L mount_n100 n=1 mean=21.834 p50=21.834 p95=21.834 p99=21.834
- L unrelated_update_n100 n=50 mean=1.100 p50=1.082 p95=1.113 p99=1.784
- L row_update_n100 n=50 mean=2.047 p50=2.032 p95=2.194 p99=2.789
- L pointer_move_n100 n=120 mean=0.300 p50=0.033 p95=1.177 p99=1.371
- L wheel_n100 n=40 mean=0.015 p50=0.014 p95=0.015 p99=0.047
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 13596700, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=172.706 p50=172.706 p95=172.706 p99=172.706
- L unrelated_update_n1000 n=50 mean=1.100 p50=1.060 p95=1.106 p99=2.853
- L row_update_n1000 n=50 mean=3.049 p50=3.021 p95=3.164 p99=5.228
- L pointer_move_n1000 n=120 mean=0.551 p50=0.090 p95=1.873 p99=2.003
- L wheel_n1000 n=40 mean=0.045 p50=0.045 p95=0.050 p99=0.050
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 40925700, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=562.805 p50=562.805 p95=562.805 p99=562.805
- L unrelated_update_n3000 n=50 mean=1.210 p50=1.111 p95=1.341 p99=4.991
- L row_update_n3000 n=50 mean=7.320 p50=7.114 p95=8.059 p99=11.611
- L pointer_move_n3000 n=120 mean=1.363 p50=0.261 p95=4.401 p99=4.614
- L wheel_n3000 n=40 mean=0.131 p50=0.130 p95=0.144 p99=0.145
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 134536000, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18883072
- K6 kv_startup default 18881536
- K6 kv_startup no-default-features 11246592
- K6 basic-catalog default 33698304
- K6 counter_nogpu 14301184
- K6 basic-catalog_nogpu 29133312
