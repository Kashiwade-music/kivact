# KPI CP1-r2 (2026-11-19T12:18Z, MINGW64_NT-10.0-26200, commit d615568c)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=52.5 samples=[ 67.3 63.0 52.5 50.1 49.2 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.7 2.8 2.7 2.8 2.6 ]
- K1 main_to_first_present renderer=cpu median_ms=51.3 samples=[ 51.3 49.5 50.0 54.9 57.3 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.7 2.7 2.6 2.7 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=183.7 samples=[ 777.7 181.2 189.8 183.7 178.6 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=26472 private_kib=5900 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26520 private_kib=5972 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=84524 private_kib=102040 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=669.6 configure_ms=18.8]
- basic-catalog renderer=default ws_kib=56496 private_kib=31688 idle_cpu_ms_per_s=118.996 idle_frames=638 cpu_ticks_100ns=12656250 
- rows3000-still renderer=default ws_kib=160052 private_kib=153184 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=56524 private_kib=31660 idle_cpu_ms_per_s=120.647 idle_frames=637 cpu_ticks_100ns=12812500 
- rows3000-still renderer=cpu ws_kib=159876 private_kib=152964 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=26660 private_kib=6604 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=44300 private_kib=26776 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=5.88
- item-text-0 renderer=cpu ws_kib=26672 private_kib=6624 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=40940 private_kib=22520 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.76
- item-button-0 renderer=cpu ws_kib=26668 private_kib=6608 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=123052 private_kib=112984 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=32.13
- item-row-0 renderer=cpu ws_kib=26676 private_kib=6636 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=160420 private_kib=153732 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=44.58

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=6222
    [box] H site    1048.7 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [box] H site    1022.7 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [box] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [box] H site     571.9 B/item     3025 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [box] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [box] H site     336.3 B/item     9009 allocs kivact_core::view::mount::style::bind_node_style
    [box] H site     280.2 B/item     6004 allocs kivact_core::view::element_data::Common::push_style
    [box] H site     216.1 B/item     3001 allocs kivact_core::view::mount::style::merge_layers::closure$0
    [box] H site     216.0 B/item     3000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [box] H site     207.5 B/item        3 allocs kivact_engine::model::aggregate::Slots::push
- heap mode=text live_bytes_per_item=4719
    [text] H site    1022.7 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [text] H site     778.3 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [text] H site     667.2 B/item     9025 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [text] H site     491.5 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [text] H site     360.6 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [text] H site     207.5 B/item        3 allocs kivact_engine::model::aggregate::Slots::push
    [text] H site     183.6 B/item     9013 allocs arvane_native::a11y::conversion::update
    [text] H site     160.1 B/item     6002 allocs kivact_layout::tree::pass::impl$0::leaf::closure$1::closure$2
    [text] H site     114.2 B/item       25 allocs arvane_native::native_input::windows::subclass
    [text] H site     112.3 B/item        3 allocs kivact_engine::paint::list::walk::Walk::node
- heap mode=button live_bytes_per_item=32646
    [button] H site    4090.6 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [button] H site    3921.2 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [button] H site    3113.0 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [button] H site    2320.9 B/item    24025 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [button] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [button] H site    1656.1 B/item    21001 allocs kivact_core::view::mount::style::merge_layers::closure$0
    [button] H site    1600.1 B/item     9001 allocs kivact_style::paint::background::impl$5::clone
    [button] H site    1536.0 B/item    75000 allocs kivact_core::view::flow::list::for_index::closure$0::closure$0::closure$2
    [button] H site    1280.0 B/item     6000 allocs kivact_style::sheet::fields::impl$72::clone
    [button] H site    1008.3 B/item    27009 allocs kivact_core::view::mount::style::bind_node_style
- heap mode=row live_bytes_per_item=45753
    [row] H site    7591.3 B/item        3 allocs kivact_runtime::reactive::arena::Arena::insert
    [row] H site    6225.9 B/item        2 allocs arvane_shell::a11y::state::AcceptedTree::new
    [row] H site    4525.5 B/item    39025 allocs arvane_native::a11y::native::NativeAccessibility::hand_over
    [row] H site    4090.6 B/item       13 allocs kivact_layout::columns::NodeLayoutColumns::push_slot
    [row] H site    1992.1 B/item    24001 allocs kivact_core::view::mount::style::merge_layers::closure$0
    [row] H site    1966.1 B/item        2 allocs kivact_engine::tree::store::NodeStore::reserve
    [row] H site    1600.1 B/item     9001 allocs kivact_style::paint::background::impl$5::clone
    [row] H site    1344.3 B/item    36009 allocs kivact_core::view::mount::style::bind_node_style
    [row] H site    1280.0 B/item     6000 allocs kivact_style::sheet::fields::impl$72::clone
    [row] H site     936.2 B/item    18004 allocs kivact_style::sheet::fields::Style::merge

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=339 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.22 ws_kib=34160 work_ms p50=1.3678 p95=1.6619 p99=1.925 scene_p95=0.0521 raster_p95=1.5432 interval_p99=17.6468
- rows=0 renderer=cpu frames=340 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.20 ws_kib=34128 work_ms p50=1.4117 p95=1.6502 p99=1.8497 scene_p95=0.054 raster_p95=1.5453 interval_p99=17.5329
- rows=3000 renderer=default frames=340 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=12.96 ws_kib=165540 work_ms p50=1.8244 p95=2.2705 p99=2.4146 scene_p95=0.2816 raster_p95=1.9412 interval_p99=17.8363
- rows=3000 renderer=cpu frames=339 fps=59.9 frame_period_ns=16666666 cpu_pct_of_core=12.71 ws_kib=165404 work_ms p50=1.8541 p95=2.3945 p99=2.5641 scene_p95=0.3091 raster_p95=1.9926 interval_p99=17.9279

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=5.338 p50=5.338 p95=5.338 p99=5.338
- L counter_click n=200 mean=1.034 p50=1.105 p95=1.151 p99=1.264
- L mount_n100 n=1 mean=22.922 p50=22.922 p95=22.922 p99=22.922
- L unrelated_update_n100 n=50 mean=1.077 p50=1.048 p95=1.237 p99=1.694
- L row_update_n100 n=50 mean=1.294 p50=1.284 p95=1.401 p99=2.060
- L pointer_move_n100 n=120 mean=0.293 p50=0.032 p95=1.165 p99=1.247
- L wheel_n100 n=40 mean=0.013 p50=0.013 p95=0.013 p99=0.021
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 11299100, live_reactive: 2474, live_owners: 101, live_objects: 205, live_constants: 409, arena_slots: 3189, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=171.180 p50=171.180 p95=171.180 p99=171.180
- L unrelated_update_n1000 n=50 mean=1.129 p50=1.041 p95=1.302 p99=2.643
- L row_update_n1000 n=50 mean=2.054 p50=2.012 p95=2.284 p99=4.191
- L pointer_move_n1000 n=120 mean=0.590 p50=0.099 p95=2.089 p99=2.409
- L wheel_n1000 n=40 mean=0.048 p50=0.048 p95=0.053 p99=0.055
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 41597800, live_reactive: 24074, live_owners: 1001, live_objects: 2005, live_constants: 4009, arena_slots: 31089, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=512.201 p50=512.201 p95=512.201 p99=512.201
- L unrelated_update_n3000 n=50 mean=1.184 p50=1.067 p95=1.300 p99=4.816
- L row_update_n3000 n=50 mean=4.151 p50=3.995 p95=4.626 p99=8.516
- L pointer_move_n3000 n=120 mean=1.716 p50=0.291 p95=5.239 p99=5.524
- L wheel_n3000 n=40 mean=0.151 p50=0.140 p95=0.241 p99=0.283
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 158616200, live_reactive: 72074, live_owners: 3001, live_objects: 6005, live_constants: 12009, arena_slots: 93089, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 19049984
- K6 kv_startup default 19045888
- K6 kv_startup no-default-features 11397120
- K6 basic-catalog default 34378240
- K6 kv_startup no-gpu 14451712
- K6 basic-catalog no-gpu 29795840
