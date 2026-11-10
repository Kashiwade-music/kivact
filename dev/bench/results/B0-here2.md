# KPI B0-here2 (2026-11-10T14:06Z, MINGW64_NT-10.0-26200, commit 79bbd409)

release build of kivact-example-counter (cold for its own target dir): 144 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=484.8 samples=[ 488.2 486.1 478.1 469.7 484.8 ]
- K4 first_frame_raster renderer=default median_ms=30.0 samples=[ 30.0 29.4 30.8 29.8 31.5 ]
- K1 main_to_first_present renderer=cpu median_ms=454.5 samples=[ 447.3 454.5 448.7 475.8 475.2 ]
- K4 first_frame_raster renderer=cpu median_ms=29.7 samples=[ 29.7 30.1 29.4 30.3 29.2 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=696.6 samples=[ 1003.6 696.6 683.6 716.4 683.9 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=97688 private_kib=112984 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=624.0 configure_ms=15.6]
- counter renderer=cpu ws_kib=36580 private_kib=13968 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=97364 private_kib=112052 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=639.3 configure_ms=15.5]
- basic-catalog renderer=default ws_kib=212480 private_kib=218652 idle_cpu_ms_per_s=131.889 idle_frames=633 cpu_ticks_100ns=13906250 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=836.0 configure_ms=17.5]
- rows3000-still renderer=default ws_kib=569580 private_kib=680036 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=1282.6 configure_ms=21.3]
- basic-catalog renderer=cpu ws_kib=159904 private_kib=94288 idle_cpu_ms_per_s=120.299 idle_frames=631 cpu_ticks_100ns=12656250 
- rows3000-still renderer=cpu ws_kib=475568 private_kib=539076 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=37228 private_kib=17016 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=123140 private_kib=107540 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=28.64
- item-text-0 renderer=cpu ws_kib=37272 private_kib=17024 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=69004 private_kib=52748 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=10.58
- item-button-0 renderer=cpu ws_kib=37280 private_kib=16216 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=363788 private_kib=375500 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=108.84
- item-row-0 renderer=cpu ws_kib=37232 private_kib=16860 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=475416 private_kib=538508 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=146.06

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=29150
    [box] H site   30131.5 B/item   105633 allocs 
    [box] H site    2464.3 B/item       83 allocs CallWindowProcW
    [box] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
    [box] H site       0.2 B/item        1 allocs IsWindowEnabled
- heap mode=text live_bytes_per_item=11499
    [text] H site   12494.6 B/item    67594 allocs 
    [text] H site    2470.8 B/item      352 allocs CallWindowProcW
    [text] H site       0.2 B/item        1 allocs IsWindowEnabled
    [text] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
- heap mode=button live_bytes_per_item=114539
    [button] H site  115516.7 B/item   878210 allocs 
    [button] H site    2465.3 B/item      104 allocs CallWindowProcW
    [button] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
    [button] H site       0.2 B/item        1 allocs IsWindowEnabled
- heap mode=row live_bytes_per_item=166304
    [row] H site  166944.4 B/item  1024012 allocs 
    [row] H site    2804.8 B/item      341 allocs CallWindowProcW
    [row] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
    [row] H site       0.2 B/item        1 allocs IsWindowEnabled

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=333 fps=60.1 frame_period_ns=16666666 cpu_pct_of_core=14.37 ws_kib=140484 work_ms p50=2.1999 p95=3.1802 p99=3.7332 scene_p95=0.0404 raster_p95=2.8894 interval_p99=18.3969
- rows=0 renderer=cpu frames=331 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=10.77 ws_kib=39804 work_ms p50=1.6957 p95=2.1218 p99=2.4256 scene_p95=0.04 raster_p95=2.0167 interval_p99=17.4548
  WARNING rows=3000 renderer=default: 46.8 fps for a 60.0 Hz display (the OS throttled the window; work times stay valid, fps and interval do not)
- rows=3000 renderer=default frames=262 fps=46.8 frame_period_ns=16666666 cpu_pct_of_core=100.30 ws_kib=534380 work_ms p50=19.0327 p95=22.2231 p99=23.588 scene_p95=19.2585 raster_p95=3.1945 interval_p99=25.4367
- rows=3000 renderer=cpu frames=270 fps=48.8 frame_period_ns=16666666 cpu_pct_of_core=99.60 ws_kib=474892 work_ms p50=18.4337 p95=21.0706 p99=22.1514 scene_p95=18.7445 raster_p95=2.2524 interval_p99=24.0471

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=401.453 p50=401.453 p95=401.453 p99=401.453
- L counter_click n=200 mean=1.971 p50=1.961 p95=2.076 p99=2.132
- L mount_n100 n=1 mean=438.563 p50=438.563 p95=438.563 p99=438.563
- L unrelated_update_n100 n=50 mean=3.740 p50=3.671 p95=4.089 p99=6.429
- L row_update_n100 n=50 mean=13.928 p50=12.683 p95=33.334 p99=33.923
- L pointer_move_n100 n=120 mean=1.343 p50=0.578 p95=4.124 p99=4.642
- L wheel_n100 n=40 mean=0.460 p50=0.450 p95=0.535 p99=0.537
- S stats_n100 Stats { batches: 101, a11y_constructed: 2270, a11y_published: 1573, ops: 2853, last_batch_ops: 1, creates: 1223, set_props: 5609, structural: 1222, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 20512500, live_reactive: 5750, live_owners: 101, live_objects: 409, live_constants: 612, arena_slots: 6872, tree_nodes: 1223, timers: 0, frame_loops: 102, animations: 0 }
- L mount_n1000 n=1 mean=804.837 p50=804.837 p95=804.837 p99=804.837
- L unrelated_update_n1000 n=50 mean=20.369 p50=20.223 p95=21.919 p99=24.501
- L row_update_n1000 n=50 mean=34.557 p50=31.682 p95=55.456 p99=56.168
- L pointer_move_n1000 n=120 mean=12.353 p50=7.256 p95=29.391 p99=32.075
- L wheel_n1000 n=40 mean=6.303 p50=6.143 p95=8.634 p99=8.722
- S stats_n1000 Stats { batches: 101, a11y_constructed: 13070, a11y_published: 12373, ops: 27153, last_batch_ops: 1, creates: 12023, set_props: 54209, structural: 12022, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 164867000, live_reactive: 56150, live_owners: 1001, live_objects: 4009, live_constants: 6012, arena_slots: 67172, tree_nodes: 12023, timers: 0, frame_loops: 1002, animations: 0 }
- L mount_n3000 n=1 mean=1557.247 p50=1557.247 p95=1557.247 p99=1557.247
- L unrelated_update_n3000 n=50 mean=73.213 p50=72.198 p95=80.308 p99=83.144
- L row_update_n3000 n=50 mean=98.415 p50=95.584 p95=116.867 p99=126.231
- L pointer_move_n3000 n=120 mean=42.971 p50=24.099 p95=103.148 p99=106.701
- L wheel_n3000 n=40 mean=22.802 p50=22.689 p95=25.188 p99=28.575
- S stats_n3000 Stats { batches: 101, a11y_constructed: 37070, a11y_published: 36373, ops: 81153, last_batch_ops: 1, creates: 36023, set_props: 162209, structural: 36022, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 469883200, live_reactive: 168150, live_owners: 3001, live_objects: 12009, live_constants: 18012, arena_slots: 201172, tree_nodes: 36023, timers: 0, frame_loops: 3002, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 24594432
- K6 kv_startup default 24590848
- K6 kv_startup no-default-features 24590848
- K6 basic-catalog default 54251520
