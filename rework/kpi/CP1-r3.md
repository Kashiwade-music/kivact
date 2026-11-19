# KPI CP1-r3 (2026-11-19T12:42Z, MINGW64_NT-10.0-26200, commit d615568c)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=55.7 samples=[ 58.4 49.8 55.7 59.5 55.7 ]
- K4 first_frame_raster renderer=default median_ms=2.7 samples=[ 2.6 2.7 2.7 2.7 2.7 ]
- K1 main_to_first_present renderer=cpu median_ms=53.2 samples=[ 53.9 47.9 49.4 54.4 53.2 ]
- K4 first_frame_raster renderer=cpu median_ms=2.7 samples=[ 2.7 2.7 2.7 2.6 2.7 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=183.1 samples=[ 783.2 181.6 182.6 184.1 183.1 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=26504 private_kib=5928 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=cpu ws_kib=26488 private_kib=5928 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=84344 private_kib=102040 idle_cpu_ms_per_s=1.468 idle_frames=0 cpu_ticks_100ns=156250 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=631.2 configure_ms=18.7]
- basic-catalog renderer=default ws_kib=56572 private_kib=31828 idle_cpu_ms_per_s=108.730 idle_frames=638 cpu_ticks_100ns=11562500 
- rows3000-still renderer=default ws_kib=160164 private_kib=153260 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- basic-catalog renderer=cpu ws_kib=56592 private_kib=31708 idle_cpu_ms_per_s=123.584 idle_frames=637 cpu_ticks_100ns=13125000 
- rows3000-still renderer=cpu ws_kib=159424 private_kib=152492 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=26668 private_kib=6612 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=44256 private_kib=26600 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=5.86
- item-text-0 renderer=cpu ws_kib=26652 private_kib=6596 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=40872 private_kib=22292 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=4.74
- item-button-0 renderer=cpu ws_kib=26688 private_kib=6636 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=122780 private_kib=112688 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=32.03
- item-row-0 renderer=cpu ws_kib=26680 private_kib=6648 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=157036 private_kib=153160 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=43.45

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
- rows=0 renderer=default frames=338 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=8.59 ws_kib=34328 work_ms p50=1.4048 p95=1.6486 p99=1.778 scene_p95=0.0474 raster_p95=1.5046 interval_p99=17.4533
- rows=0 renderer=cpu frames=338 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.82 ws_kib=34224 work_ms p50=1.4057 p95=1.5916 p99=1.7467 scene_p95=0.0482 raster_p95=1.4696 interval_p99=17.4326
- rows=3000 renderer=default frames=338 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=12.48 ws_kib=165484 work_ms p50=1.8623 p95=2.0269 p99=2.2843 scene_p95=0.2455 raster_p95=1.7275 interval_p99=17.6215
- rows=3000 renderer=cpu frames=338 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=13.31 ws_kib=165204 work_ms p50=1.8704 p95=2.2085 p99=2.3731 scene_p95=0.3081 raster_p95=1.8295 interval_p99=17.6745

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=5.794 p50=5.794 p95=5.794 p99=5.794
- L counter_click n=200 mean=1.075 p50=1.109 p95=1.376 p99=1.403
- L mount_n100 n=1 mean=21.961 p50=21.961 p95=21.961 p99=21.961
- L unrelated_update_n100 n=50 mean=1.103 p50=1.055 p95=1.369 p99=1.662
- L row_update_n100 n=50 mean=1.315 p50=1.274 p95=1.565 p99=1.826
- L pointer_move_n100 n=120 mean=0.310 p50=0.030 p95=1.305 p99=1.390
- L wheel_n100 n=40 mean=0.013 p50=0.013 p95=0.019 p99=0.021
- S stats_n100 Stats { batches: 101, a11y_constructed: 1021, a11y_published: 762, a11y_enumerated: 512, ops: 1124, last_batch_ops: 1, creates: 512, set_props: 1741, structural: 511, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 10838300, live_reactive: 2474, live_owners: 101, live_objects: 205, live_constants: 409, arena_slots: 3189, tree_nodes: 512, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n1000 n=1 mean=168.778 p50=168.778 p95=168.778 p99=168.778
- L unrelated_update_n1000 n=50 mean=1.128 p50=1.028 p95=1.496 p99=2.865
- L row_update_n1000 n=50 mean=2.016 p50=1.944 p95=2.240 p99=4.284
- L pointer_move_n1000 n=120 mean=0.558 p50=0.093 p95=2.014 p99=2.135
- L wheel_n1000 n=40 mean=0.052 p50=0.047 p95=0.053 p99=0.222
- S stats_n1000 Stats { batches: 101, a11y_constructed: 5521, a11y_published: 5262, a11y_enumerated: 5012, ops: 10124, last_batch_ops: 1, creates: 5012, set_props: 16141, structural: 5011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 38364000, live_reactive: 24074, live_owners: 1001, live_objects: 2005, live_constants: 4009, arena_slots: 31089, tree_nodes: 5012, timers: 0, frame_loops: 0, animations: 0 }
- L mount_n3000 n=1 mean=503.355 p50=503.355 p95=503.355 p99=503.355
- L unrelated_update_n3000 n=50 mean=1.204 p50=1.078 p95=1.402 p99=4.944
- L row_update_n3000 n=50 mean=3.961 p50=3.821 p95=4.542 p99=8.047
- L pointer_move_n3000 n=120 mean=1.579 p50=0.279 p95=5.053 p99=5.255
- L wheel_n3000 n=40 mean=0.147 p50=0.142 p95=0.172 p99=0.321
- S stats_n3000 Stats { batches: 101, a11y_constructed: 15521, a11y_published: 15262, a11y_enumerated: 15012, ops: 30124, last_batch_ops: 1, creates: 15012, set_props: 48141, structural: 15011, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 144121100, live_reactive: 72074, live_owners: 3001, live_objects: 6005, live_constants: 12009, arena_slots: 93089, tree_nodes: 15012, timers: 0, frame_loops: 0, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 19049984
- K6 kv_startup default 19045888
- K6 kv_startup no-default-features 11397120
- K6 basic-catalog default 34378240
- K6 kv_startup no-gpu 14451712
- K6 basic-catalog no-gpu 29795840
