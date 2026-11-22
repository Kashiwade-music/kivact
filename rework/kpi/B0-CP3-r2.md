# KPI B0-CP3-r2 (2026-11-22T10:42Z, MINGW64_NT-10.0-26200, commit c9518946)

release build of kivact-example-counter (cold for its own target dir): 6 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=47.7 samples=[ 48.0 45.9 44.7 47.7 49.3 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.8 2.6 2.7 2.6 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=46.1 samples=[ 46.1 47.5 45.0 43.7 51.5 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.8 2.7 2.6 2.7 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=177.7 samples=[ 562.3 172.7 183.0 175.3 177.7 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=25704 private_kib=5908 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=25680 private_kib=5900 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=85272 private_kib=103708 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=776.6 configure_ms=16.2]
- basic-catalog renderer=default ws_kib=51524 private_kib=27488 idle_cpu_ms_per_s=68.008 idle_frames=634 cpu_ticks_100ns=7187500 
- rows3000-still renderer=default ws_kib=100076 private_kib=84680 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=51644 private_kib=27644 idle_cpu_ms_per_s=81.245 idle_frames=635 cpu_ticks_100ns=8593750 
- rows3000-still renderer=cpu ws_kib=100364 private_kib=84896 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=25900 private_kib=6640 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=36716 private_kib=19160 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=3.61
- item-text-0 renderer=cpu ws_kib=25888 private_kib=6620 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=34472 private_kib=16692 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=2.86
- item-button-0 renderer=cpu ws_kib=25944 private_kib=6728 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=81528 private_kib=73560 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=18.53
- item-row-0 renderer=cpu ws_kib=25884 private_kib=6620 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=100032 private_kib=84384 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=24.72

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
- rows=0 renderer=default frames=335 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=7.54 ws_kib=33420 work_ms p50=1.0763 p95=1.5402 p99=1.645 scene_p95=0.0397 raster_p95=1.4274 interval_p99=17.5119
- rows=0 renderer=cpu frames=334 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=5.62 ws_kib=33376 work_ms p50=1.043 p95=1.4261 p99=1.6416 scene_p95=0.0438 raster_p95=1.3309 interval_p99=17.5959
- rows=3000 renderer=default frames=334 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.66 ws_kib=105920 work_ms p50=1.5165 p95=1.9214 p99=2.241 scene_p95=0.2076 raster_p95=1.7126 interval_p99=17.5311
- rows=3000 renderer=cpu frames=335 fps=60.1 frame_period_ns=16666666 cpu_pct_of_core=10.66 ws_kib=105828 work_ms p50=1.4853 p95=2.0578 p99=2.2234 scene_p95=0.213 raster_p95=1.7826 interval_p99=17.5073

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=6.310 p50=6.310 p95=6.310 p99=6.310
- L counter_click n=200 mean=0.928 p50=0.870 p95=1.078 p99=1.115
- L mount_n100 n=1 mean=21.705 p50=21.705 p95=21.705 p99=21.705
- L unrelated_update_n100 n=50 mean=1.210 p50=1.194 p95=1.219 p99=1.877
- L row_update_n100 n=50 mean=2.132 p50=2.098 p95=2.278 p99=3.057
- L pointer_move_n100 n=120 mean=0.311 p50=0.033 p95=1.316 p99=1.512
- L wheel_n100 n=40 mean=0.014 p50=0.013 p95=0.019 p99=0.037
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 13494900, live_reactive: 1973, live_owners: 101, live_objects: 205, live_constants: 408, arena_slots: 2687, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=167.223 p50=167.223 p95=167.223 p99=167.223
- L unrelated_update_n1000 n=50 mean=1.214 p50=1.168 p95=1.344 p99=2.876
- L row_update_n1000 n=50 mean=3.268 p50=3.196 p95=3.654 p99=5.386
- L pointer_move_n1000 n=120 mean=0.602 p50=0.095 p95=2.081 p99=2.393
- L wheel_n1000 n=40 mean=0.047 p50=0.047 p95=0.051 p99=0.052
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 42945800, live_reactive: 19073, live_owners: 1001, live_objects: 2005, live_constants: 4008, arena_slots: 26087, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=533.836 p50=533.836 p95=533.836 p99=533.836
- L unrelated_update_n3000 n=50 mean=1.318 p50=1.221 p95=1.326 p99=5.487
- L row_update_n3000 n=50 mean=7.847 p50=7.854 p95=8.794 p99=11.555
- L pointer_move_n3000 n=120 mean=1.482 p50=0.274 p95=4.832 p99=5.032
- L wheel_n3000 n=40 mean=0.141 p50=0.138 p95=0.146 p99=0.214
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 138150900, live_reactive: 57073, live_owners: 3001, live_objects: 6005, live_constants: 12008, arena_slots: 78087, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 18945536
- K6 kv_startup default 18940928
- K6 kv_startup no-default-features 11298304
- K6 basic-catalog default 34042880
- K6 kv_startup no-gpu 14353408
- K6 basic-catalog no-gpu 29470208
