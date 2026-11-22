# KPI B0-CP3-r3 (2026-11-22T11:23Z, MINGW64_NT-10.0-26200, commit c9518946)

release build of kivact-example-counter (cold for its own target dir): 5 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=44.3 samples=[ 48.9 44.3 43.7 43.4 47.9 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.7 2.8 2.7 2.6 2.6 ]
- K1 main_to_first_present renderer=cpu median_ms=45.3 samples=[ 44.7 44.5 47.1 45.3 49.0 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.7 2.6 2.8 2.6 2.9 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=182.3 samples=[ 611.1 193.0 177.2 176.5 182.3 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25708 private_kib=5912 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25680 private_kib=5880 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=85472 private_kib=104632 idle_cpu_ms_per_s=2.956 idle_frames=0 cpu_ticks_100ns=312500 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=636.8 configure_ms=19.0]
- basic-catalog renderer=default ws_kib=51572 private_kib=27592 idle_cpu_ms_per_s=91.518 idle_frames=635 cpu_ticks_100ns=9687500 
- rows3000-still renderer=default ws_kib=100292 private_kib=84832 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=51536 private_kib=27556 idle_cpu_ms_per_s=73.657 idle_frames=636 cpu_ticks_100ns=7812500 
- rows3000-still renderer=cpu ws_kib=100212 private_kib=84632 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25884 private_kib=6632 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36732 private_kib=19080 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.62
- item-text-0 renderer=cpu ws_kib=25872 private_kib=6628 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34288 private_kib=16452 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.81
- item-button-0 renderer=cpu ws_kib=25860 private_kib=6624 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=81108 private_kib=73508 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.42
- item-row-0 renderer=cpu ws_kib=25876 private_kib=6632 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=100404 private_kib=85188 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.84

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
- rows=0 renderer=default frames=338 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.27 ws_kib=33372 work_ms p50=1.2243 p95=1.7851 p99=1.829 scene_p95=0.0436 raster_p95=1.687 interval_p99=17.5629
- rows=0 renderer=cpu frames=336 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=8.93 ws_kib=33328 work_ms p50=1.2254 p95=1.798 p99=1.8359 scene_p95=0.0423 raster_p95=1.7004 interval_p99=17.5621
- rows=3000 renderer=default frames=336 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=11.98 ws_kib=105752 work_ms p50=1.636 p95=2.0932 p99=2.3238 scene_p95=0.2188 raster_p95=1.8134 interval_p99=17.47
- rows=3000 renderer=cpu frames=338 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=12.21 ws_kib=106376 work_ms p50=1.6604 p95=2.1761 p99=2.3397 scene_p95=0.2215 raster_p95=1.8863 interval_p99=17.441

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=5.490 p50=5.490 p95=5.490 p99=5.490
- L counter_click n=200 mean=0.930 p50=0.877 p95=1.170 p99=1.287
- L mount_n100 n=1 mean=23.983 p50=23.983 p95=23.983 p99=23.983
- L unrelated_update_n100 n=50 mean=1.221 p50=1.200 p95=1.296 p99=1.969
- L row_update_n100 n=50 mean=2.192 p50=2.173 p95=2.390 p99=2.900
- L pointer_move_n100 n=120 mean=0.312 p50=0.033 p95=1.241 p99=1.447
- L wheel_n100 n=40 mean=0.014 p50=0.014 p95=0.015 p99=0.026
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 15153200, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=180.942 p50=180.942 p95=180.942 p99=180.942
- L unrelated_update_n1000 n=50 mean=1.295 p50=1.241 p95=1.498 p99=3.107
- L row_update_n1000 n=50 mean=3.595 p50=3.539 p95=3.822 p99=5.707
- L pointer_move_n1000 n=120 mean=0.682 p50=0.101 p95=2.518 p99=2.657
- L wheel_n1000 n=40 mean=0.047 p50=0.047 p95=0.048 p99=0.060
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 54309100, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=567.270 p50=567.270 p95=567.270 p99=567.270
- L unrelated_update_n3000 n=50 mean=1.395 p50=1.313 p95=1.374 p99=5.212
- L row_update_n3000 n=50 mean=8.361 p50=8.279 p95=8.574 p99=12.040
- L pointer_move_n3000 n=120 mean=2.347 p50=0.696 p95=7.088 p99=7.406
- L wheel_n3000 n=40 mean=0.302 p50=0.295 p95=0.335 p99=0.424
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 192190200, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18945536
- K6 kv_startup default 18940928
- K6 kv_startup no-default-features 11298304
- K6 basic-catalog default 34042880
- K6 kv_startup no-gpu 14353408
- K6 basic-catalog no-gpu 29470208
