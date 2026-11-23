# KPI B0-CP3c (2026-11-22T23:37Z, MINGW64_NT-10.0-26200, commit c9518946)

release build of kivact-example-counter (cold for its own target dir): 6 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=43.4 samples=[ 44.8 43.4 42.2 43.0 44.1 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.8 2.9 2.6 2.6 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=43.4 samples=[ 43.4 46.8 44.3 43.3 43.3 ]
- K4 first_frame_raster renderer=cpu median_ms=2.6 samples=[ 2.6 2.7 2.6 2.6 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=174.9 samples=[ 501.8 171.5 205.1 174.9 173.9 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25704 private_kib=5900 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25720 private_kib=5932 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=85640 private_kib=103220 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=653.2 configure_ms=15.5]
- basic-catalog renderer=default ws_kib=51572 private_kib=27468 idle_cpu_ms_per_s=66.736 idle_frames=632 cpu_ticks_100ns=7031250 
- rows3000-still renderer=default ws_kib=100912 private_kib=84676 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=51676 private_kib=27516 idle_cpu_ms_per_s=75.665 idle_frames=632 cpu_ticks_100ns=7968750 
- rows3000-still renderer=cpu ws_kib=100372 private_kib=84968 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25888 private_kib=6616 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36664 private_kib=19100 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.59
- item-text-0 renderer=cpu ws_kib=25916 private_kib=6656 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34412 private_kib=16668 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.83
- item-button-0 renderer=cpu ws_kib=25948 private_kib=6700 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=81072 private_kib=73256 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.37
- item-row-0 renderer=cpu ws_kib=25880 private_kib=6608 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=100516 private_kib=84780 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.88

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
- heap mode=button live_bytes_per_item=20412
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
- rows=0 renderer=default frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=6.20 ws_kib=33344 work_ms p50=1.0592 p95=1.497 p99=1.6808 scene_p95=0.0362 raster_p95=1.4124 interval_p99=17.6056
- rows=0 renderer=cpu frames=333 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.88 ws_kib=33392 work_ms p50=1.058 p95=1.4419 p99=1.5607 scene_p95=0.0367 raster_p95=1.3445 interval_p99=17.2723
- rows=3000 renderer=default frames=335 fps=60.2 frame_period_ns=16666666 cpu_pct_of_core=10.67 ws_kib=105836 work_ms p50=1.5181 p95=1.9837 p99=2.3849 scene_p95=0.2158 raster_p95=1.7251 interval_p99=17.513
- rows=3000 renderer=cpu frames=332 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=8.47 ws_kib=106248 work_ms p50=1.4778 p95=2.0029 p99=2.2221 scene_p95=0.2121 raster_p95=1.7487 interval_p99=17.42

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.218 p50=6.218 p95=6.218 p99=6.218
- L counter_click n=200 mean=0.867 p50=0.861 p95=0.889 p99=1.011
- L mount_n100 n=1 mean=21.771 p50=21.771 p95=21.771 p99=21.771
- L unrelated_update_n100 n=50 mean=1.217 p50=1.198 p95=1.258 p99=1.929
- L row_update_n100 n=50 mean=2.112 p50=2.099 p95=2.228 p99=2.882
- L pointer_move_n100 n=120 mean=0.311 p50=0.034 p95=1.236 p99=1.395
- L wheel_n100 n=40 mean=0.015 p50=0.014 p95=0.015 p99=0.038
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 13517300, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=174.806 p50=174.806 p95=174.806 p99=174.806
- L unrelated_update_n1000 n=50 mean=1.246 p50=1.201 p95=1.274 p99=2.945
- L row_update_n1000 n=50 mean=3.225 p50=3.167 p95=3.341 p99=5.438
- L pointer_move_n1000 n=120 mean=0.603 p50=0.092 p95=2.112 p99=2.488
- L wheel_n1000 n=40 mean=0.070 p50=0.076 p95=0.093 p99=0.142
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 42800700, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=500.039 p50=500.039 p95=500.039 p99=500.039
- L unrelated_update_n3000 n=50 mean=1.301 p50=1.191 p95=1.764 p99=5.182
- L row_update_n3000 n=50 mean=7.898 p50=7.878 p95=8.478 p99=11.419
- L pointer_move_n3000 n=120 mean=1.442 p50=0.262 p95=4.638 p99=5.138
- L wheel_n3000 n=40 mean=0.135 p50=0.134 p95=0.145 p99=0.163
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 134412200, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18945536
- K6 kv_startup default 18940928
- K6 kv_startup no-default-features 11298304
- K6 basic-catalog default 34042880
- K6 kv_startup no-gpu 14353408
- K6 basic-catalog no-gpu 29470208
