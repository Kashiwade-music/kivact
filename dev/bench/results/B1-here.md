# KPI B1-here (2026-11-11T09:49Z, MINGW64_NT-10.0-26200, commit 1f000b93)

release build of kivact-example-counter (cold for its own target dir): 2 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=42.3 samples=[ 44.5 41.1 42.1 42.4 42.3 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.9 2.8 2.7 2.7 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=42.1 samples=[ 42.1 42.9 42.6 41.2 41.8 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.7 2.7 2.6 2.7 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=174.0 samples=[ 495.3 174.0 172.7 175.0 171.5 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25716 private_kib=5892 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25720 private_kib=5912 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=85272 private_kib=104124 idle_cpu_ms_per_s=1.487 idle_frames=0 cpu_ticks_100ns=156250 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=616.3 configure_ms=15.9]
- basic-catalog renderer=default ws_kib=51468 private_kib=27676 idle_cpu_ms_per_s=62.319 idle_frames=632 cpu_ticks_100ns=6562500 
- rows3000-still renderer=default ws_kib=99164 private_kib=82668 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=51368 private_kib=27600 idle_cpu_ms_per_s=71.325 idle_frames=631 cpu_ticks_100ns=7500000 
- rows3000-still renderer=cpu ws_kib=98536 private_kib=83132 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25892 private_kib=6608 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36696 private_kib=19216 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.60
- item-text-0 renderer=cpu ws_kib=25896 private_kib=6608 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34596 private_kib=16564 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.90
- item-button-0 renderer=cpu ws_kib=25876 private_kib=6596 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=81620 private_kib=73768 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.58
- item-row-0 renderer=cpu ws_kib=25944 private_kib=6668 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=98584 private_kib=83092 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.21

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=3740
    [box] H site     984.4 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [box] H site     589.9 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     280.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     232.2 B/item     3003 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     216.0 B/item     3000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [box] H site     207.5 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [box] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [box] H site     109.2 B/item        1 allocs kivact_engine::paint::state::ChildIndex::build
    [box] H site      99.7 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
- heap mode=text live_bytes_per_item=2936
    [text] H site     984.4 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [text] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [text] H site     360.6 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [text] H site     207.5 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [text] H site     192.1 B/item     6002 allocs kivact_layout::tree::LayoutPass::leaf<kivact_engine::tree::node_tree::NodeTree>
    [text] H site     113.8 B/item       21 allocs arvane_native::native_input::windows::subclass
    [text] H site     109.2 B/item        1 allocs kivact_engine::paint::state::ChildIndex::build
    [text] H site      99.7 B/item        1 allocs kivact_core::view::tree::ViewNodes::create
    [text] H site      65.5 B/item        2 allocs kivact_engine::measure::state::MeasureColumns::push
    [text] H site      54.6 B/item        1 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0<u32,kivact_core::view::descriptor::description::View,alloc::vec::Vec<u32,alloc::alloc::Global>,kv_heap::main::closure$0::closure_env$1>
- heap mode=button live_bytes_per_item=20342
    [button] H site    3937.7 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
    [button] H site    3921.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1504.0 B/item    72000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [button] H site     832.0 B/item    30000 allocs kivact_core::view::mount::mount_host_with
    [button] H site     830.1 B/item        2 allocs kivact_engine::paint::state::PaintState::push_slot
    [button] H site     728.0 B/item    12000 allocs kivact_core::view::mount::part_input::root_target
    [button] H site     696.2 B/item     9003 allocs kivact_core::view::mount::style::bind_node_style
    [button] H site     560.2 B/item    12004 allocs kivact_core::view::element_data::Common::push_style
    [button] H site     416.1 B/item     6002 allocs kivact_layout::tree::LayoutPass::leaf<kivact_engine::tree::node_tree::NodeTree>
- heap mode=row live_bytes_per_item=23034
    [row] H site    3937.7 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push
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
- rows=0 renderer=default frames=332 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=7.33 ws_kib=32336 work_ms p50=1.0157 p95=1.2405 p99=1.4213 scene_p95=0.0317 raster_p95=1.1557 interval_p99=17.4239
- rows=0 renderer=cpu frames=332 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=6.49 ws_kib=32328 work_ms p50=1.0379 p95=1.1944 p99=1.3975 scene_p95=0.03 raster_p95=1.12 interval_p99=17.2163
- rows=3000 renderer=default frames=333 fps=60.1 frame_period_ns=16666666 cpu_pct_of_core=12.12 ws_kib=104348 work_ms p50=1.4708 p95=1.7832 p99=2.0806 scene_p95=0.2174 raster_p95=1.542 interval_p99=17.6238
- rows=3000 renderer=cpu frames=332 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=8.46 ws_kib=104160 work_ms p50=1.4842 p95=1.8523 p99=2.1152 scene_p95=0.1876 raster_p95=1.5962 interval_p99=17.4083

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.525 p50=6.525 p95=6.525 p99=6.525
- L counter_click n=200 mean=0.878 p50=0.861 p95=0.908 p99=1.367
- L mount_n100 n=1 mean=21.971 p50=21.971 p95=21.971 p99=21.971
- L unrelated_update_n100 n=50 mean=1.195 p50=1.179 p95=1.215 p99=1.839
- L row_update_n100 n=50 mean=2.345 p50=2.351 p95=2.538 p99=2.793
- L pointer_move_n100 n=120 mean=0.306 p50=0.031 p95=1.218 p99=1.368
- L wheel_n100 n=40 mean=0.013 p50=0.013 p95=0.013 p99=0.039
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 13682400, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=174.938 p50=174.938 p95=174.938 p99=174.938
- L unrelated_update_n1000 n=50 mean=1.206 p50=1.167 p95=1.196 p99=3.002
- L row_update_n1000 n=50 mean=3.516 p50=3.515 p95=3.728 p99=5.345
- L pointer_move_n1000 n=120 mean=0.584 p50=0.092 p95=2.062 p99=2.218
- L wheel_n1000 n=40 mean=0.047 p50=0.045 p95=0.050 p99=0.086
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 43384700, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=499.897 p50=499.897 p95=499.897 p99=499.897
- L unrelated_update_n3000 n=50 mean=1.237 p50=1.152 p95=1.222 p99=4.899
- L row_update_n3000 n=50 mean=7.408 p50=7.303 p95=7.752 p99=11.256
- L pointer_move_n3000 n=120 mean=1.382 p50=0.257 p95=4.537 p99=4.932
- L wheel_n3000 n=40 mean=0.137 p50=0.137 p95=0.142 p99=0.148
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 127169300, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18900480
- K6 kv_startup default 18897920
- K6 kv_startup no-default-features 11250176
- K6 basic-catalog default 33744384
- K6 counter_nogpu 14304768
- K6 basic-catalog_nogpu 29166080
