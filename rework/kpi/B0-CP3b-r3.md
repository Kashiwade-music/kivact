# KPI B0-CP3b-r3 (2026-11-22T14:41Z, MINGW64_NT-10.0-26200, commit c9518946)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=52.2 samples=[ 52.2 51.1 56.4 113.4 45.4 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 3.1 2.7 2.7 2.7 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=52.0 samples=[ 44.9 65.0 60.9 46.0 52.0 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.7 2.7 2.6 2.7 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=186.6 samples=[ 530.6 186.6 207.0 182.2 173.8 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25696 private_kib=5896 idle_cpu_ms_per_s=2.964 idle_frames=2 cpu_ticks_100ns=312500 
- counter renderer=cpu ws_kib=25732 private_kib=5944 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=85544 private_kib=103600 idle_cpu_ms_per_s=1.482 idle_frames=0 cpu_ticks_100ns=156250 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=625.2 configure_ms=16.2]
- basic-catalog renderer=default ws_kib=51676 private_kib=27672 idle_cpu_ms_per_s=73.963 idle_frames=634 cpu_ticks_100ns=7812500 
- rows3000-still renderer=default ws_kib=100120 private_kib=84796 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=51468 private_kib=27492 idle_cpu_ms_per_s=88.687 idle_frames=634 cpu_ticks_100ns=9375000 
- rows3000-still renderer=cpu ws_kib=100536 private_kib=85100 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25888 private_kib=6624 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36608 private_kib=19036 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.57
- item-text-0 renderer=cpu ws_kib=25928 private_kib=6660 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34504 private_kib=16764 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.86
- item-button-0 renderer=cpu ws_kib=25912 private_kib=6656 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=81544 private_kib=73508 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.54
- item-row-0 renderer=cpu ws_kib=25876 private_kib=6608 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=100260 private_kib=84700 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.79

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
- rows=0 renderer=default frames=336 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=6.96 ws_kib=33392 work_ms p50=1.0296 p95=1.4671 p99=1.7578 scene_p95=0.0347 raster_p95=1.3944 interval_p99=17.3652
- rows=0 renderer=cpu frames=335 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=10.62 ws_kib=33292 work_ms p50=1.0534 p95=1.5024 p99=1.6997 scene_p95=0.0378 raster_p95=1.4523 interval_p99=17.5299
- rows=3000 renderer=default frames=334 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=11.21 ws_kib=105800 work_ms p50=1.4794 p95=2.0162 p99=2.2407 scene_p95=0.2079 raster_p95=1.7762 interval_p99=17.567
- rows=3000 renderer=cpu frames=334 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=7.86 ws_kib=105864 work_ms p50=1.464 p95=1.9541 p99=2.172 scene_p95=0.2107 raster_p95=1.7425 interval_p99=17.4164

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.373 p50=6.373 p95=6.373 p99=6.373
- L counter_click n=200 mean=0.869 p50=0.860 p95=0.898 p99=1.181
- L mount_n100 n=1 mean=23.767 p50=23.767 p95=23.767 p99=23.767
- L unrelated_update_n100 n=50 mean=1.218 p50=1.198 p95=1.258 p99=1.899
- L row_update_n100 n=50 mean=2.116 p50=2.099 p95=2.253 p99=2.890
- L pointer_move_n100 n=120 mean=0.312 p50=0.033 p95=1.287 p99=1.387
- L wheel_n100 n=40 mean=0.014 p50=0.014 p95=0.019 p99=0.039
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 14398100, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=171.326 p50=171.326 p95=171.326 p99=171.326
- L unrelated_update_n1000 n=50 mean=1.300 p50=1.233 p95=1.528 p99=3.108
- L row_update_n1000 n=50 mean=3.251 p50=3.217 p95=3.510 p99=5.377
- L pointer_move_n1000 n=120 mean=0.602 p50=0.092 p95=2.123 p99=2.282
- L wheel_n1000 n=40 mean=0.047 p50=0.046 p95=0.051 p99=0.068
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 43977100, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=501.532 p50=501.532 p95=501.532 p99=501.532
- L unrelated_update_n3000 n=50 mean=1.263 p50=1.183 p95=1.224 p99=5.096
- L row_update_n3000 n=50 mean=7.612 p50=7.485 p95=8.418 p99=11.173
- L pointer_move_n3000 n=120 mean=1.586 p50=0.266 p95=5.367 p99=5.746
- L wheel_n3000 n=40 mean=0.133 p50=0.131 p95=0.146 p99=0.164
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 148921500, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18945536
- K6 kv_startup default 18940928
- K6 kv_startup no-default-features 11298304
- K6 basic-catalog default 34042880
- K6 kv_startup no-gpu 14353408
- K6 basic-catalog no-gpu 29470208
