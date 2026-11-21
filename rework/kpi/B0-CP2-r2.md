# KPI B0-CP2-r2 (2026-11-21T14:20Z, MINGW64_NT-10.0-26200, commit c9518946)

release build of kivact-example-counter (cold for its own target dir): 0 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=44.3 samples=[ 48.8 42.2 44.3 44.5 43.6 ]
- K4 first_frame_raster renderer=default median_ms=2.6 samples=[ 2.8 2.6 2.8 2.6 2.6 ]
- K1 main_to_first_present renderer=cpu median_ms=45.9 samples=[ 42.6 45.9 47.1 47.0 45.2 ]
- K4 first_frame_raster renderer=cpu median_ms=2.6 samples=[ 2.6 2.6 2.6 2.7 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=170.8 samples=[ 538.8 170.1 170.8 174.3 170.0 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25660 private_kib=5896 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25676 private_kib=5912 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=85776 private_kib=103028 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=624.9 configure_ms=15.5]
- basic-catalog renderer=default ws_kib=51496 private_kib=27584 idle_cpu_ms_per_s=68.348 idle_frames=631 cpu_ticks_100ns=7187500 
- rows3000-still renderer=default ws_kib=100248 private_kib=84688 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=51580 private_kib=27512 idle_cpu_ms_per_s=63.679 idle_frames=633 cpu_ticks_100ns=6718750 
- rows3000-still renderer=cpu ws_kib=100032 private_kib=84548 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25864 private_kib=6648 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36468 private_kib=18932 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.53
- item-text-0 renderer=cpu ws_kib=25812 private_kib=6616 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34388 private_kib=16668 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.86
- item-button-0 renderer=cpu ws_kib=25836 private_kib=6620 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=81236 private_kib=73364 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.47
- item-row-0 renderer=cpu ws_kib=25848 private_kib=6612 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=100580 private_kib=85200 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.91

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
- rows=0 renderer=default frames=331 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.36 ws_kib=33220 work_ms p50=1.0421 p95=1.44 p99=1.5274 scene_p95=0.0321 raster_p95=1.3678 interval_p99=17.4428
- rows=0 renderer=cpu frames=332 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=9.89 ws_kib=33252 work_ms p50=1.08 p95=1.4832 p99=1.6862 scene_p95=0.0394 raster_p95=1.3913 interval_p99=17.3952
- rows=3000 renderer=default frames=331 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.76 ws_kib=106240 work_ms p50=1.4727 p95=1.9263 p99=2.0766 scene_p95=0.1869 raster_p95=1.718 interval_p99=17.2541
- rows=3000 renderer=cpu frames=331 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.48 ws_kib=105576 work_ms p50=1.544 p95=1.8087 p99=2.2257 scene_p95=0.2796 raster_p95=1.5631 interval_p99=17.3504

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=5.033 p50=5.033 p95=5.033 p99=5.033
- L counter_click n=200 mean=0.863 p50=0.858 p95=0.888 p99=0.908
- L mount_n100 n=1 mean=21.451 p50=21.451 p95=21.451 p99=21.451
- L unrelated_update_n100 n=50 mean=1.219 p50=1.200 p95=1.237 p99=1.910
- L row_update_n100 n=50 mean=2.142 p50=2.130 p95=2.296 p99=2.949
- L pointer_move_n100 n=120 mean=0.310 p50=0.033 p95=1.273 p99=1.379
- L wheel_n100 n=40 mean=0.014 p50=0.014 p95=0.020 p99=0.026
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 13405200, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=170.773 p50=170.773 p95=170.773 p99=170.773
- L unrelated_update_n1000 n=50 mean=1.241 p50=1.201 p95=1.255 p99=2.876
- L row_update_n1000 n=50 mean=3.291 p50=3.256 p95=3.408 p99=5.311
- L pointer_move_n1000 n=120 mean=0.592 p50=0.092 p95=2.073 p99=2.207
- L wheel_n1000 n=40 mean=0.046 p50=0.046 p95=0.051 p99=0.053
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 43365800, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=500.205 p50=500.205 p95=500.205 p99=500.205
- L unrelated_update_n3000 n=50 mean=1.263 p50=1.183 p95=1.236 p99=4.983
- L row_update_n3000 n=50 mean=7.137 p50=7.040 p95=7.452 p99=11.516
- L pointer_move_n3000 n=120 mean=1.524 p50=0.266 p95=4.986 p99=5.877
- L wheel_n3000 n=40 mean=0.155 p50=0.144 p95=0.192 p99=0.375
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 137016200, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18945536
- K6 kv_startup default 18940928
- K6 kv_startup no-default-features 11298304
- K6 basic-catalog default 34042880
- K6 kv_startup no-gpu 14353408
- K6 basic-catalog no-gpu 29470208
